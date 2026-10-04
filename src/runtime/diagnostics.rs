use super::{MemoryRuntime, RuntimeDiagnosticError, RuntimeLifecycle};
use crate::{
    AllocationLedger, AllocationPolicy, AllocationSlotDescriptor, DiagnosticCheck, DiagnosticCode,
    DiagnosticDeclaration, DiagnosticExport, DiagnosticFailure, DiagnosticMemorySize,
    DiagnosticRangeAuthority, DiagnosticRuntimeBinding, DiagnosticStableCell,
    DiagnosticStableCellStatus, LedgerCommitError, LedgerPayloadEnvelopeError,
    MemoryRuntimeDoctorReport, PolicyIdentity, RecoveredLedger, RuntimeBootstrapPolicy,
    StableCellLedgerRecord,
    physical::CommitStoreDiagnostic,
    registry::{SealedDeclarationFingerprint, SealedDeclarationSnapshot},
    slot::MEMORY_MANAGER_LEDGER_ID,
    stable_cell::decode_stable_cell_ledger_record_from_memory,
};
use ic_stable_structures::Memory;
use std::{borrow::Cow, fmt::Display};

impl<M: Memory> MemoryRuntime<M> {
    /// Export this runtime's recovered ledger and live virtual-memory sizes.
    pub fn diagnostic_export(&self) -> Result<DiagnosticExport, RuntimeDiagnosticError> {
        if !self.is_bootstrapped() {
            return Err(RuntimeDiagnosticError::NotBootstrapped);
        }
        let record = self.ledger_record_from_memory()?;
        let (recovered, commit_recovery) = record.store().recover_with_diagnostic();
        let recovered = recovered?;
        Ok(self.recovered_diagnostic_export(&recovered, commit_recovery))
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
        policy: &P,
    ) -> MemoryRuntimeDoctorReport
    where
        P: RuntimeBootstrapPolicy,
        P::Error: Display,
    {
        let stable_cell = self.stable_cell_diagnostic();
        let recovery = stable_cell
            .record
            .as_ref()
            .map(|record| record.store().recover_with_diagnostic());
        let ledger = recovery.as_ref().and_then(|(recovered, diagnostic)| {
            recovered
                .as_ref()
                .ok()
                .map(|recovered| self.recovered_diagnostic_export(recovered, *diagnostic))
        });
        let diagnostic_declarations = declarations
            .registered_declarations()
            .iter()
            .map(|registration| {
                DiagnosticDeclaration::new(
                    registration.authority(),
                    registration.declaration().clone(),
                )
            })
            .collect();
        let registered_records = declarations
            .registered_ranges()
            .iter()
            .map(|registration| registration.record().clone())
            .collect();
        let range_authority = DiagnosticRangeAuthority::new(
            registered_records,
            declarations.range_authority().clone(),
        );
        let tested_policy_identity = policy
            .runtime_bootstrap_identity()
            .map_err(|err| DiagnosticFailure::new(DiagnosticCode::PolicyIdentity, err.to_string()));
        let tested_declaration_fingerprint = declarations.fingerprint();
        let established_bootstrap_binding = self.established_bootstrap_binding();
        let bootstrap_binding = diagnostic_bootstrap_binding(
            &tested_policy_identity,
            tested_declaration_fingerprint,
            established_bootstrap_binding.as_ref(),
        );
        let validation = match &tested_policy_identity {
            Ok(_) => diagnostic_validation(
                declarations,
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
            ledger_anchor: ledger_anchor_descriptor(),
            stable_cell: stable_cell.diagnostic,
            commit_recovery: recovery.as_ref().map(|(_, diagnostic)| *diagnostic),
            ledger,
            registered_declarations: diagnostic_declarations,
            range_authority,
            validation,
        }
    }

    fn recovered_diagnostic_export(
        &self,
        recovered: &RecoveredLedger,
        commit_recovery: CommitStoreDiagnostic,
    ) -> DiagnosticExport {
        let mut export =
            DiagnosticExport::from_ledger(recovered.ledger(), ledger_anchor_descriptor());
        export.commit_recovery = Some(commit_recovery);
        for record in &mut export.records {
            let id = record
                .allocation
                .slot()
                .memory_manager_id()
                .expect("recovered ledger slot");
            record.memory_size = Some(DiagnosticMemorySize::from_wasm_pages(
                self.memory(id).size(),
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
            )),
        }
    }

    fn stable_cell_diagnostic(&self) -> StableCellDiagnostic {
        let memory = self.memory(MEMORY_MANAGER_LEDGER_ID);
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

const fn ledger_anchor_descriptor() -> AllocationSlotDescriptor {
    AllocationSlotDescriptor::memory_manager_unchecked(MEMORY_MANAGER_LEDGER_ID)
}

fn diagnostic_validation<P: AllocationPolicy>(
    declarations: &SealedDeclarationSnapshot,
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
    let resolved = match declarations.resolve(recovered.ledger(), Vec::new()) {
        Ok(resolved) => resolved,
        Err(err) => {
            return DiagnosticCheck::failed(DiagnosticCode::AllocationValidation, err.to_string());
        }
    };
    let policy = super::policy::RuntimeMemoryManagerPolicy {
        declarations: &resolved,
        custom_policy,
    };
    match crate::validation::check_allocations(&recovered, resolved.allocation_snapshot(), &policy)
    {
        Ok(()) => DiagnosticCheck::passed(),
        Err(err) => DiagnosticCheck::failed(DiagnosticCode::AllocationValidation, err.to_string()),
    }
}

fn diagnostic_bootstrap_binding(
    tested_policy_identity: &Result<PolicyIdentity, DiagnosticFailure>,
    tested_declaration_fingerprint: SealedDeclarationFingerprint,
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
    {
        return DiagnosticCheck::passed();
    }
    DiagnosticCheck::failed(
        DiagnosticCode::RuntimeBinding,
        format!(
            "tested policy/declaration binding differs from established runtime binding: \
             tested_policy={tested_policy_identity:?}, \
             tested_declarations={tested_declaration_fingerprint:?}, \
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
