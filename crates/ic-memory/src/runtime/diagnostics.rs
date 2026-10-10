use super::{MemoryRuntime, RuntimeDiagnosticError, RuntimeLifecycle};
use crate::{
    AllocationLedger, AllocationPolicy, DiagnosticCheck, DiagnosticCode, DiagnosticExport,
    DiagnosticFailure, DiagnosticMemorySize, DiagnosticRuntimeBinding, DiagnosticStableCell,
    DiagnosticStableCellStatus, LedgerCommitError, LedgerPayloadEnvelopeError,
    MemoryRuntimeDoctorReport, PolicyIdentity, RecoveredLedger, RuntimeBootstrapPolicy,
    StableCellLedgerRecord,
    physical::CommitStoreDiagnostic,
    registry::{SealedDeclarationFingerprint, SealedDeclarationSnapshot},
    slot::MEMORY_MANAGER_LEDGER_ID,
    stable_cell::decode_stable_cell_ledger_record_from_memory,
};
use ic_stable_structures::{Memory, memory_manager::MemoryId};
use std::{borrow::Cow, fmt::Display};

impl<M: Memory> MemoryRuntime<M> {
    /// Export this runtime's recovered ledger and live virtual-memory sizes.
    pub fn diagnostic_export(&self) -> Result<DiagnosticExport, RuntimeDiagnosticError> {
        if !self.is_bootstrapped() {
            return Err(RuntimeDiagnosticError::NotBootstrapped);
        }
        let (recovered, commit_recovery) = self
            .ledger_record_from_memory()?
            .store()
            .recover_with_diagnostic();
        let recovered = recovered?;
        Ok(self.recovered_diagnostic_export(Cow::Owned(recovered), commit_recovery))
    }

    /// Diagnose protected commit recovery from this runtime's ledger memory.
    ///
    /// This operation is available before bootstrap when the stable-cell
    /// envelope is readable or the ledger memory is empty.
    pub fn commit_recovery_diagnostic(
        &self,
    ) -> Result<CommitStoreDiagnostic, RuntimeDiagnosticError> {
        let record = self.ledger_record_from_memory()?;
        Ok(record.store().physical().diagnostic())
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
        // physical slots before projecting the report or invoking custom policy.
        let recovery = stable_cell
            .record
            .map(|record| record.store().recover_with_diagnostic());
        let ledger = recovery.as_ref().and_then(|(recovered, diagnostic)| {
            recovered.as_ref().ok().map(|recovered| {
                self.recovered_diagnostic_export(Cow::Borrowed(recovered), *diagnostic)
            })
        });
        let tested_policy_identity = policy
            .runtime_bootstrap_identity()
            .map_err(|err| DiagnosticFailure::new(DiagnosticCode::PolicyIdentity, err.to_string()));
        let tested_declaration_fingerprint = declarations.fingerprint();
        let established_bootstrap_binding = self.established_bootstrap_binding();
        let bootstrap_binding = diagnostic_bootstrap_binding(
            &tested_policy_identity,
            tested_declaration_fingerprint,
            pool,
            established_bootstrap_binding.as_ref(),
        );
        let validation = match &tested_policy_identity {
            Ok(_) => diagnostic_validation(
                self,
                declarations,
                pool,
                policy,
                recovery.as_ref().map(|(recovered, _)| recovered),
            ),
            Err(failure) => DiagnosticCheck::not_run(failure.code, failure.message.clone()),
        };

        MemoryRuntimeDoctorReport {
            bootstrapped: self.is_bootstrapped(),
            tested_policy_identity,
            tested_declaration_fingerprint,
            established_bootstrap_binding,
            bootstrap_binding,
            ledger_anchor: crate::slot::LEDGER_SLOT,
            stable_cell: stable_cell.diagnostic,
            commit_recovery: recovery.as_ref().map(|(_, diagnostic)| *diagnostic),
            ledger,
            requests: declarations.requests().to_vec(),
            allocation_pool: pool.clone(),
            validation,
        }
    }

    fn recovered_diagnostic_export(
        &self,
        recovered: Cow<'_, RecoveredLedger>,
        commit_recovery: CommitStoreDiagnostic,
    ) -> DiagnosticExport {
        let anchor = crate::slot::LEDGER_SLOT;
        let mut export = match recovered {
            Cow::Borrowed(recovered) => DiagnosticExport::from_ledger(recovered.ledger(), anchor),
            Cow::Owned(recovered) => {
                DiagnosticExport::from_owned_ledger(recovered.into_ledger(), anchor)
            }
        };
        export.commit_recovery = Some(commit_recovery);
        for record in &mut export.records {
            let id = record.allocation.slot().id();
            record.memory_size = Some(DiagnosticMemorySize::from_wasm_pages(
                self.memory_size_pages(id),
            ));
        }
        export
    }

    fn established_bootstrap_binding(&self) -> Option<DiagnosticRuntimeBinding> {
        match &self.lifecycle {
            RuntimeLifecycle::Unbootstrapped => None,
            RuntimeLifecycle::Bootstrapped { binding, .. } => Some(DiagnosticRuntimeBinding::new(
                binding.policy_identity.clone(),
                binding.source.fingerprint(),
                binding.pool.clone(),
            )),
        }
    }

    fn stable_cell_diagnostic(&self) -> StableCellDiagnostic {
        let memory = self
            .memory_manager
            .get(MemoryId::new(MEMORY_MANAGER_LEDGER_ID));
        let memory_size = DiagnosticMemorySize::from_wasm_pages(memory.size());
        match decode_stable_cell_ledger_record_from_memory(&memory) {
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
    record: Option<StableCellLedgerRecord>,
}

fn diagnostic_validation<M: Memory, P: AllocationPolicy>(
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
    if let Err(error) = runtime.validate_pool_custody(recovered.ledger(), pool) {
        return DiagnosticCheck::failed(DiagnosticCode::AllocationValidation, error.to_string());
    }
    let resolved = match declarations.resolve(recovered.ledger(), Vec::new(), pool) {
        Ok(resolved) => resolved,
        Err(err) => {
            return DiagnosticCheck::failed(DiagnosticCode::AllocationValidation, err.to_string());
        }
    };
    let policy = super::policy::RuntimeMemoryManagerPolicy { custom_policy };
    match crate::validation::check_allocations(&recovered, &resolved, &policy) {
        Ok(()) => DiagnosticCheck::passed(),
        Err(err) => DiagnosticCheck::failed(DiagnosticCode::AllocationValidation, err.to_string()),
    }
}

fn diagnostic_bootstrap_binding(
    tested_policy_identity: &Result<PolicyIdentity, DiagnosticFailure>,
    tested_declaration_fingerprint: SealedDeclarationFingerprint,
    pool: &crate::MemoryAllocationPool,
    established: Option<&DiagnosticRuntimeBinding>,
) -> DiagnosticCheck {
    let tested_policy_identity = match tested_policy_identity {
        Ok(identity) => identity,
        Err(failure) => {
            return DiagnosticCheck::not_run(failure.code, failure.message.clone());
        }
    };
    let Some(established) = established else {
        return DiagnosticCheck::not_run(
            DiagnosticCode::RuntimeBinding,
            "runtime has not completed bootstrap",
        );
    };
    if &established.policy_identity == tested_policy_identity
        && established.declaration_fingerprint == tested_declaration_fingerprint
        && &established.allocation_pool == pool
    {
        return DiagnosticCheck::passed();
    }
    DiagnosticCheck::failed(
        DiagnosticCode::RuntimeBinding,
        format!(
            "tested policy/declaration/pool binding differs from established runtime binding: \
             tested_policy={tested_policy_identity:?}, \
             tested_declarations={tested_declaration_fingerprint:?}, \
             tested_pool={pool:?}, \
             established={established:?}"
        ),
    )
}

pub(super) fn diagnostic_validation_ledger(
    recovered: Option<&Result<crate::RecoveredLedger, LedgerCommitError>>,
) -> Result<Cow<'_, crate::RecoveredLedger>, DiagnosticFailure> {
    if let Some(Ok(recovered)) = recovered {
        return Ok(Cow::Borrowed(recovered));
    }
    if let Some(Err(err)) = recovered {
        // Protected recovery returns NoValidGeneration only for two absent slots.
        if matches!(
            err,
            LedgerCommitError::Recovery(crate::CommitRecoveryError::NoValidGeneration)
        ) {
            return Ok(Cow::Owned(RecoveredLedger::from_trusted_ledger(
                AllocationLedger::empty_genesis(),
            )));
        }
        let code = if matches!(
            err,
            LedgerCommitError::PayloadEnvelope(
                LedgerPayloadEnvelopeError::UnsupportedFormat { .. }
            )
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
