use crate::{
    ledger::{AllocationLedger, AllocationRecord, GenerationRecord},
    physical::CommitStoreDiagnostic,
    slot::AllocationSlotDescriptor,
};
use serde::{Deserialize, Serialize};

///
/// DiagnosticExport
///
/// Read-only machine-readable allocation ledger export.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DiagnosticExport {
    /// Current committed generation.
    pub current_generation: u64,
    /// Ledger anchor descriptor.
    pub ledger_anchor: AllocationSlotDescriptor,
    /// Allocation records.
    pub records: Vec<DiagnosticRecord>,
    /// Generation records.
    pub generations: Vec<DiagnosticGeneration>,
    /// Optional protected commit recovery diagnostic.
    pub commit_recovery: Option<CommitStoreDiagnostic>,
}

impl DiagnosticExport {
    /// Build a read-only diagnostic export from an allocation ledger.
    #[must_use]
    pub fn from_ledger(ledger: &AllocationLedger, ledger_anchor: AllocationSlotDescriptor) -> Self {
        Self::from_ledger_with_commit_recovery(ledger, ledger_anchor, None)
    }

    /// Build a read-only diagnostic export with protected commit recovery state.
    #[must_use]
    pub fn from_ledger_with_commit_recovery(
        ledger: &AllocationLedger,
        ledger_anchor: AllocationSlotDescriptor,
        commit_recovery: Option<CommitStoreDiagnostic>,
    ) -> Self {
        Self {
            current_generation: ledger.current_generation,
            ledger_anchor,
            records: ledger
                .allocation_history()
                .records()
                .iter()
                .cloned()
                .map(|allocation| DiagnosticRecord { allocation })
                .collect(),
            generations: ledger
                .allocation_history()
                .generations()
                .iter()
                .cloned()
                .map(|generation| DiagnosticGeneration { generation })
                .collect(),
            commit_recovery,
        }
    }
}

///
/// DiagnosticRecord
///
/// Read-only diagnostic allocation record.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DiagnosticRecord {
    /// Allocation record.
    pub allocation: AllocationRecord,
}

///
/// DiagnosticGeneration
///
/// Read-only diagnostic generation record.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DiagnosticGeneration {
    /// Generation record.
    pub generation: GenerationRecord,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        declaration::AllocationDeclaration,
        ledger::{AllocationHistory, AllocationRecord, AllocationState},
        physical::{CommitRecoveryError, CommitSlotDiagnostic, CommitStoreDiagnostic},
        schema::SchemaMetadata,
    };

    #[test]
    fn diagnostic_export_copies_ledger_records() {
        let declaration = AllocationDeclaration::new(
            "app.users.v1",
            AllocationSlotDescriptor::memory_manager(100).expect("usable slot"),
            None,
            SchemaMetadata::default(),
        )
        .expect("declaration");
        let ledger = AllocationLedger {
            current_generation: 3,
            allocation_history: AllocationHistory::from_parts(
                vec![AllocationRecord::from_declaration(
                    3,
                    declaration,
                    AllocationState::Active,
                )],
                vec![GenerationRecord {
                    generation: 3,
                    parent_generation: 2,
                    runtime_fingerprint: Some("wasm:abc123".to_string()),
                    declaration_count: 1,
                    committed_at: None,
                }],
            ),
        };

        let export = DiagnosticExport::from_ledger(
            &ledger,
            AllocationSlotDescriptor::memory_manager(0).expect("usable slot"),
        );

        assert_eq!(export.current_generation, 3);
        assert_eq!(export.records.len(), 1);
        assert_eq!(export.generations.len(), 1);
        assert_eq!(
            export.ledger_anchor,
            AllocationSlotDescriptor::memory_manager(0).expect("usable slot")
        );
        assert_eq!(export.commit_recovery, None);
    }

    #[test]
    fn diagnostic_export_can_include_commit_recovery_state() {
        let ledger = AllocationLedger {
            current_generation: 3,
            allocation_history: AllocationHistory::default(),
        };
        let commit_recovery = CommitStoreDiagnostic {
            slot0: CommitSlotDiagnostic {
                present: true,
                generation: Some(3),
                valid: true,
            },
            slot1: CommitSlotDiagnostic {
                present: false,
                generation: None,
                valid: false,
            },
            authoritative_generation: Some(3),
            recovery_error: None,
        };

        let export = DiagnosticExport::from_ledger_with_commit_recovery(
            &ledger,
            AllocationSlotDescriptor::memory_manager(0).expect("usable slot"),
            Some(commit_recovery),
        );

        assert_eq!(export.commit_recovery, Some(commit_recovery));
    }

    #[test]
    fn diagnostic_export_can_report_recovery_failure() {
        let ledger = AllocationLedger {
            current_generation: 0,
            allocation_history: AllocationHistory::default(),
        };
        let commit_recovery = CommitStoreDiagnostic {
            slot0: CommitSlotDiagnostic {
                present: false,
                generation: None,
                valid: false,
            },
            slot1: CommitSlotDiagnostic {
                present: false,
                generation: None,
                valid: false,
            },
            authoritative_generation: None,
            recovery_error: Some(CommitRecoveryError::NoValidGeneration),
        };

        let export = DiagnosticExport::from_ledger_with_commit_recovery(
            &ledger,
            AllocationSlotDescriptor::memory_manager(0).expect("usable slot"),
            Some(commit_recovery),
        );

        assert_eq!(
            export
                .commit_recovery
                .expect("commit recovery")
                .recovery_error,
            Some(CommitRecoveryError::NoValidGeneration)
        );
    }
}
