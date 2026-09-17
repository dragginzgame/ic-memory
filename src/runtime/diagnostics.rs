use super::{MemoryRuntime, RuntimeDiagnosticError, RuntimeLifecycle};
use crate::{
    AllocationHistory, AllocationLedger, AllocationPolicy, AllocationSlotDescriptor,
    DiagnosticCheck, DiagnosticCode, DiagnosticDeclaration, DiagnosticExport, DiagnosticFailure,
    DiagnosticMemorySize, DiagnosticMemorySizeOutcome, DiagnosticRangeAuthority,
    DiagnosticRuntimeBinding, DiagnosticStableCell, DiagnosticStableCellStatus, LedgerCommitError,
    LedgerPayloadEnvelopeError, MemoryRuntimeDoctorReport, PolicyIdentity, RuntimeBootstrapPolicy,
    StableCellLedgerRecord,
    physical::CommitStoreDiagnostic,
    registry::{SealedDeclarationFingerprint, SealedDeclarationSnapshot},
    slot::MEMORY_MANAGER_LEDGER_ID,
    stable_cell::decode_stable_cell_ledger_record_from_memory,
};
use ic_stable_structures::Memory;
use std::fmt::Display;

impl<M: Memory> MemoryRuntime<M> {
    /// Export this runtime's recovered ledger and live virtual-memory sizes.
    pub fn diagnostic_export(&self) -> Result<DiagnosticExport, RuntimeDiagnosticError> {
        if !self.is_bootstrapped() {
            return Err(RuntimeDiagnosticError::NotBootstrapped);
        }
        let record = self.ledger_record_from_memory()?;
        let recovered = record.store().recover()?;
        let ledger = recovered.ledger();
        Ok(
            DiagnosticExport::from_ledger_with_commit_recovery_and_memory_sizes(
                ledger,
                ledger_anchor_descriptor(),
                Some(record.store().physical().diagnostic()),
                self.memory_sizes(ledger)?,
            ),
        )
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
        let commit_recovery = stable_cell
            .record
            .as_ref()
            .map(|record| record.store().physical().diagnostic());
        let recovered = stable_cell
            .record
            .as_ref()
            .map(|record| record.store().recover());
        let recovered_for_export = recovered.as_ref().and_then(|result| result.as_ref().ok());
        let ledger = recovered_for_export.map(|recovered| {
            DiagnosticExport::from_ledger_with_commit_recovery_and_memory_size_outcomes(
                recovered.ledger(),
                ledger_anchor_descriptor(),
                commit_recovery,
                self.memory_size_outcomes(recovered.ledger()),
            )
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
            Ok(declarations.range_authority().clone()),
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
                stable_cell.record.as_ref(),
                recovered.as_ref(),
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
            commit_recovery,
            ledger,
            registered_declarations: diagnostic_declarations,
            range_authority,
            validation,
        }
    }

    fn memory_sizes(
        &self,
        ledger: &AllocationLedger,
    ) -> Result<Vec<(AllocationSlotDescriptor, DiagnosticMemorySize)>, RuntimeDiagnosticError> {
        ledger
            .allocation_history()
            .records()
            .iter()
            .map(|record| {
                let id = record.slot().memory_manager_id()?;
                Ok((
                    record.slot().clone(),
                    DiagnosticMemorySize::from_wasm_pages(self.memory(id).size()),
                ))
            })
            .collect()
    }

    pub(super) fn memory_size_outcomes(
        &self,
        ledger: &AllocationLedger,
    ) -> Vec<(AllocationSlotDescriptor, DiagnosticMemorySizeOutcome)> {
        ledger
            .allocation_history()
            .records()
            .iter()
            .map(|record| {
                let outcome = match record.slot().memory_manager_id() {
                    Ok(id) => DiagnosticMemorySizeOutcome::Measured(
                        DiagnosticMemorySize::from_wasm_pages(self.memory(id).size()),
                    ),
                    Err(err) => DiagnosticMemorySizeOutcome::Failed(DiagnosticFailure::new(
                        DiagnosticCode::MemorySize,
                        err.to_string(),
                    )),
                };
                (record.slot().clone(), outcome)
            })
            .collect()
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
        if memory.size() == 0 {
            return StableCellDiagnostic {
                diagnostic: DiagnosticStableCell::new(
                    DiagnosticStableCellStatus::Empty,
                    memory_size,
                ),
                record: Some(StableCellLedgerRecord::default()),
            };
        }

        match decode_stable_cell_ledger_record_from_memory(&memory) {
            Ok(record) => StableCellDiagnostic {
                diagnostic: DiagnosticStableCell::new(
                    DiagnosticStableCellStatus::Readable,
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
    stable_cell_record: Option<&StableCellLedgerRecord>,
    recovered: Option<&Result<crate::RecoveredLedger, LedgerCommitError>>,
) -> DiagnosticCheck
where
    P::Error: Display,
{
    let recovered = match diagnostic_validation_ledger(stable_cell_record, recovered) {
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
    match crate::validate_allocations(&recovered, resolved.allocation_snapshot().clone(), &policy) {
        Ok(_) => DiagnosticCheck::passed(),
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
    stable_cell_record: Option<&StableCellLedgerRecord>,
    recovered: Option<&Result<crate::RecoveredLedger, LedgerCommitError>>,
) -> Result<crate::RecoveredLedger, DiagnosticFailure> {
    if let Some(Ok(recovered)) = recovered {
        return Ok(recovered.clone());
    }
    if let Some(Err(err)) = recovered {
        if stable_cell_record.is_some_and(|record| record.store().physical().is_uninitialized()) {
            return diagnostic_genesis_recovered_ledger();
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
    if stable_cell_record.is_some() {
        return diagnostic_genesis_recovered_ledger();
    }
    Err(DiagnosticFailure::new(
        DiagnosticCode::StableCell,
        "stable-cell ledger record is not readable",
    ))
}

fn diagnostic_genesis_recovered_ledger() -> Result<crate::RecoveredLedger, DiagnosticFailure> {
    AllocationLedger::new(0, AllocationHistory::default())
        .map(|ledger| crate::RecoveredLedger::from_trusted_parts(ledger, 0))
        .map_err(|err| {
            DiagnosticFailure::new(
                DiagnosticCode::GenesisLedger,
                format!("genesis ledger: {err}"),
            )
        })
}
