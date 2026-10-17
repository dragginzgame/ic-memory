use super::{MemoryRuntime, RuntimeDiagnosticError, RuntimeLifecycle};
use crate::{
    DiagnosticCheck, DiagnosticCode, DiagnosticExport, DiagnosticFailure, DiagnosticMemorySize,
    DiagnosticRuntimeBinding, DiagnosticStableCell, DiagnosticStableCellStatus, LedgerCommitError,
    LedgerCommitStore, MemoryRuntimeDoctorReport, PolicyIdentity, RecoveredLedger,
    RuntimeBootstrapPolicy, physical::CommitSnapshotDiagnostic,
    requests::SealedDeclarationSnapshot, slot::MEMORY_MANAGER_LEDGER_ID,
    stable_cell::decode_stable_cell_commit_store_from_memory,
};
use ic_stable_structures::{Memory, memory_manager::MemoryId};
use std::{borrow::Cow, fmt::Display};

impl<M: Memory> MemoryRuntime<M> {
    /// Export this runtime's recovered ledger and live virtual-memory sizes.
    pub fn diagnostic_export(&self) -> Result<DiagnosticExport, RuntimeDiagnosticError> {
        if !self.is_bootstrapped() {
            return Err(RuntimeDiagnosticError::NotBootstrapped);
        }
        let recovered = self.ledger_record_from_memory()?.recover()?;
        Ok(self.recovered_diagnostic_export(recovered))
    }

    /// Diagnose protected commit recovery from this runtime's ledger memory.
    ///
    /// This operation is available before bootstrap when the stable-cell
    /// envelope is readable or the ledger memory is empty.
    pub fn commit_recovery_diagnostic(
        &self,
    ) -> Result<CommitSnapshotDiagnostic, RuntimeDiagnosticError> {
        let record = self.ledger_record_from_memory()?;
        Ok(record.diagnostic())
    }

    /// Build preflight and lifecycle diagnostics for this runtime.
    ///
    /// Validation checks the supplied declarations and allocation policy only.
    /// It does not execute `prepare_bootstrap`, predict its completed set, or
    /// certify consumer admission. Diagnostics never replay preparation.
    #[must_use]
    pub fn doctor_report<P>(
        &self,
        declarations: &SealedDeclarationSnapshot,
        pool: &crate::MemoryAllocationPool,
        policy: &P,
    ) -> MemoryRuntimeDoctorReport
    where
        P: RuntimeBootstrapPolicy,
        P::Error: Display,
    {
        let stable_cell = self.stable_cell_diagnostic();
        // Recovery owns its ledger and diagnostic evidence. Release the decoded
        // physical snapshot before observing memory or invoking custom policy.
        let recovery = stable_cell
            .record
            .map(|record| record.recover_with_diagnostic());
        // Keep the original observation boundary before policy callbacks, without
        // copying the recovered keys/records merely to retain those observations.
        let mut observed_pages = [0; crate::constants::MAX_ALLOCATIONS];
        if let Some((Ok(recovered), _)) = &recovery {
            for (record, pages) in recovered.ledger().records().iter().zip(&mut observed_pages) {
                *pages = self.memory_size_pages(record.slot().id());
            }
        }
        let tested_policy_identity = policy.runtime_bootstrap_identity();
        let established_bootstrap_binding = self.established_bootstrap_binding();
        let bootstrap_binding = diagnostic_bootstrap_binding(
            &tested_policy_identity,
            declarations,
            pool,
            &self.lifecycle,
        );
        let validation = diagnostic_validation(
            self,
            declarations,
            pool,
            policy,
            recovery.as_ref().map(|(recovered, _)| recovered),
        );
        let commit_recovery = recovery.as_ref().map(|(_, diagnostic)| *diagnostic);
        let ledger = recovery.and_then(|(recovered, _)| {
            recovered.ok().map(|recovered| {
                let mut export = DiagnosticExport::from_owned_ledger(recovered.into_ledger());
                for (record, pages) in export.records.iter_mut().zip(&observed_pages) {
                    record.memory_size = Some(DiagnosticMemorySize::from_wasm_pages(*pages));
                }
                export
            })
        });

        MemoryRuntimeDoctorReport {
            bootstrapped: self.is_bootstrapped(),
            tested_policy_identity,
            established_bootstrap_binding,
            bootstrap_binding,
            stable_cell: stable_cell.diagnostic,
            commit_recovery,
            ledger,
            requests: declarations.requests().to_vec(),
            allocation_pool: pool.clone(),
            validation,
        }
    }

    fn recovered_diagnostic_export(&self, recovered: RecoveredLedger) -> DiagnosticExport {
        let mut export = DiagnosticExport::from_owned_ledger(recovered.into_ledger());
        for record in &mut export.records {
            record.memory_size = Some(DiagnosticMemorySize::from_wasm_pages(
                self.memory_size_pages(record.allocation.slot().id()),
            ));
        }
        export
    }

    fn established_bootstrap_binding(&self) -> Option<DiagnosticRuntimeBinding> {
        match &self.lifecycle {
            RuntimeLifecycle::Unbootstrapped => None,
            RuntimeLifecycle::Bootstrapped { binding, .. } => Some(DiagnosticRuntimeBinding::new(
                binding.policy_identity,
                binding.pool.clone(),
            )),
        }
    }

    fn stable_cell_diagnostic(&self) -> StableCellDiagnostic {
        let memory = self
            .memory_manager
            .get(MemoryId::new(MEMORY_MANAGER_LEDGER_ID));
        let memory_size = DiagnosticMemorySize::from_wasm_pages(memory.size());
        match decode_stable_cell_commit_store_from_memory(&memory) {
            Ok(record) => StableCellDiagnostic {
                diagnostic: DiagnosticStableCell::new(
                    if memory_size.wasm_pages == 0 {
                        DiagnosticStableCellStatus::Empty
                    } else {
                        DiagnosticStableCellStatus::Readable
                    },
                    memory_size,
                ),
                record: Some(record),
            },
            Err(err) => StableCellDiagnostic {
                diagnostic: DiagnosticStableCell::new(
                    DiagnosticStableCellStatus::Corrupt {
                        failure: DiagnosticFailure::new(
                            DiagnosticCode::StableCell,
                            err.to_string(),
                        ),
                    },
                    memory_size,
                ),
                record: None,
            },
        }
    }
}

struct StableCellDiagnostic {
    diagnostic: DiagnosticStableCell,
    record: Option<LedgerCommitStore>,
}

fn diagnostic_validation<M: Memory, P: RuntimeBootstrapPolicy>(
    runtime: &MemoryRuntime<M>,
    declarations: &SealedDeclarationSnapshot,
    pool: &crate::MemoryAllocationPool,
    custom_policy: &P,
    recovered: Option<&Result<crate::RecoveredLedger, LedgerCommitError>>,
) -> DiagnosticCheck
where
    P::Error: Display,
{
    let recovered = match diagnostic_validation_ledger(recovered) {
        Ok(recovered) => recovered,
        Err(failure) => return DiagnosticCheck::not_run(failure.code, failure.message),
    };
    let occupied = recovered.ledger().occupied_slots();
    if let Err(error) = runtime.validate_pool_custody(&occupied, pool) {
        return DiagnosticCheck::failed(DiagnosticCode::AllocationValidation, error.to_string());
    }
    let resolved = match declarations.resolve(&recovered, &occupied, Vec::new(), pool) {
        Ok(resolved) => resolved,
        Err(err) => {
            return DiagnosticCheck::failed(DiagnosticCode::AllocationValidation, err.to_string());
        }
    };
    match crate::validation::check_allocations(&recovered, &resolved, custom_policy) {
        Ok(()) => DiagnosticCheck::passed(),
        Err(err) => DiagnosticCheck::failed(DiagnosticCode::AllocationValidation, err.to_string()),
    }
}

fn diagnostic_bootstrap_binding(
    tested_policy_identity: &PolicyIdentity,
    declarations: &SealedDeclarationSnapshot,
    pool: &crate::MemoryAllocationPool,
    lifecycle: &RuntimeLifecycle,
) -> DiagnosticCheck {
    let RuntimeLifecycle::Bootstrapped { binding, .. } = lifecycle else {
        return DiagnosticCheck::not_run(
            DiagnosticCode::RuntimeBinding,
            "runtime has not completed bootstrap",
        );
    };
    match binding.validate::<std::convert::Infallible>(declarations, pool, tested_policy_identity) {
        Ok(()) => DiagnosticCheck::passed(),
        Err(_) => DiagnosticCheck::failed(
            DiagnosticCode::RuntimeBinding,
            "tested policy/declaration/pool binding differs from established runtime binding",
        ),
    }
}

pub(super) fn diagnostic_validation_ledger(
    recovered: Option<&Result<crate::RecoveredLedger, LedgerCommitError>>,
) -> Result<Cow<'_, crate::RecoveredLedger>, DiagnosticFailure> {
    if let Some(Ok(recovered)) = recovered {
        return Ok(Cow::Borrowed(recovered));
    }
    if let Some(Err(err)) = recovered {
        if let Some(genesis) = RecoveredLedger::genesis_for_absent_snapshot(err) {
            return Ok(Cow::Owned(genesis));
        }
        let code = if matches!(
            err,
            LedgerCommitError::Recovery(crate::CommitRecoveryError::UnsupportedFormat { .. })
        ) {
            DiagnosticCode::UnsupportedFormat
        } else {
            DiagnosticCode::LedgerRecovery
        };
        return Err(DiagnosticFailure::new(
            code,
            format!("protected ledger recovery: {err}"),
        ));
    }
    Err(DiagnosticFailure::new(
        DiagnosticCode::StableCell,
        "stable-cell ledger record is not readable",
    ))
}
