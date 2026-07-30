use super::{MemoryRuntime, RuntimeDiagnosticError, policy::NoopPolicy};
use crate::{
    AllocationHistory, AllocationLedger, AllocationSlotDescriptor, DiagnosticCheck, DiagnosticCode,
    DiagnosticDeclaration, DiagnosticExport, DiagnosticFailure, DiagnosticMemorySize,
    DiagnosticRangeAuthority, DiagnosticStableCell, DiagnosticStableCellStatus, LedgerCommitError,
    LedgerPayloadEnvelopeError, MemoryRuntimeDoctorReport, StableCellLedgerRecord,
    physical::CommitStoreDiagnostic, registry::SealedDeclarationSnapshot,
    slot::MEMORY_MANAGER_LEDGER_ID, stable_cell::decode_stable_cell_ledger_record_from_memory,
};
use ic_stable_structures::Memory;

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
    #[must_use]
    pub fn doctor_report(
        &self,
        declarations: &SealedDeclarationSnapshot,
    ) -> MemoryRuntimeDoctorReport {
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
            DiagnosticExport::from_ledger_with_commit_recovery_and_memory_sizes(
                recovered.ledger(),
                ledger_anchor_descriptor(),
                commit_recovery,
                self.memory_sizes_lossy(recovered.ledger()),
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
        let validation = diagnostic_validation(
            declarations,
            stable_cell.record.as_ref(),
            recovered.as_ref(),
        );

        MemoryRuntimeDoctorReport {
            bootstrapped: self.is_bootstrapped(),
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

    fn memory_sizes_lossy(
        &self,
        ledger: &AllocationLedger,
    ) -> Vec<(AllocationSlotDescriptor, DiagnosticMemorySize)> {
        self.memory_sizes(ledger).unwrap_or_default()
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

fn diagnostic_validation(
    declarations: &SealedDeclarationSnapshot,
    stable_cell_record: Option<&StableCellLedgerRecord>,
    recovered: Option<&Result<crate::RecoveredLedger, LedgerCommitError>>,
) -> DiagnosticCheck {
    let recovered = match diagnostic_validation_ledger(stable_cell_record, recovered) {
        Ok(recovered) => recovered,
        Err(failure) => return DiagnosticCheck::not_run(failure.code, failure.message),
    };
    let policy = super::policy::RuntimeMemoryManagerPolicy {
        declarations,
        custom_policy: &NoopPolicy,
    };
    match crate::validate_allocations(
        &recovered,
        declarations.allocation_snapshot().clone(),
        &policy,
    ) {
        Ok(_) => DiagnosticCheck::passed(),
        Err(err) => DiagnosticCheck::failed(DiagnosticCode::AllocationValidation, err.to_string()),
    }
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
