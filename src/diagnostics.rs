use crate::{
    constants::WASM_PAGE_SIZE_BYTES,
    ledger::{AllocationLedger, AllocationRecord, GenerationRecord},
    physical::CommitStoreDiagnostic,
    slot::AllocationSlotDescriptor,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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
        Self::from_ledger_with_commit_recovery_and_memory_sizes(
            ledger,
            ledger_anchor,
            commit_recovery,
            std::iter::empty(),
        )
    }

    /// Build a read-only diagnostic export with live memory sizes.
    #[must_use]
    pub fn from_ledger_with_memory_sizes(
        ledger: &AllocationLedger,
        ledger_anchor: AllocationSlotDescriptor,
        memory_sizes: impl IntoIterator<Item = (AllocationSlotDescriptor, DiagnosticMemorySize)>,
    ) -> Self {
        Self::from_ledger_with_commit_recovery_and_memory_sizes(
            ledger,
            ledger_anchor,
            None,
            memory_sizes,
        )
    }

    /// Build a read-only diagnostic export with protected recovery state and live memory sizes.
    #[must_use]
    pub fn from_ledger_with_commit_recovery_and_memory_sizes(
        ledger: &AllocationLedger,
        ledger_anchor: AllocationSlotDescriptor,
        commit_recovery: Option<CommitStoreDiagnostic>,
        memory_sizes: impl IntoIterator<Item = (AllocationSlotDescriptor, DiagnosticMemorySize)>,
    ) -> Self {
        let memory_sizes: BTreeMap<_, _> = memory_sizes.into_iter().collect();
        Self {
            current_generation: ledger.current_generation,
            ledger_anchor,
            records: ledger
                .allocation_history()
                .records()
                .iter()
                .cloned()
                .map(|allocation| {
                    let memory_size = memory_sizes.get(allocation.slot()).copied();
                    DiagnosticRecord {
                        allocation,
                        memory_size,
                    }
                })
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
    /// Live backing memory size, when the exporter measured one.
    ///
    /// This is allocation size reported by the backing memory, not logical user
    /// payload size inside the stable structure.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_size: Option<DiagnosticMemorySize>,
}

///
/// DiagnosticMemorySize
///
/// Live size reported by a backing stable memory.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticMemorySize {
    /// WebAssembly pages reported by the memory.
    pub wasm_pages: u64,
    /// Bytes represented by the page count.
    pub bytes: u64,
}

impl DiagnosticMemorySize {
    /// Build a size from a WebAssembly page count.
    #[must_use]
    pub const fn from_wasm_pages(wasm_pages: u64) -> Self {
        Self {
            wasm_pages,
            bytes: wasm_pages.saturating_mul(WASM_PAGE_SIZE_BYTES),
        }
    }
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
        assert_eq!(export.records[0].memory_size, None);
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
    fn diagnostic_export_can_include_memory_sizes() {
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
                Vec::new(),
            ),
        };

        let export = DiagnosticExport::from_ledger_with_memory_sizes(
            &ledger,
            AllocationSlotDescriptor::memory_manager(0).expect("usable slot"),
            [(
                AllocationSlotDescriptor::memory_manager(100).expect("usable slot"),
                DiagnosticMemorySize::from_wasm_pages(2),
            )],
        );

        assert_eq!(
            export.records[0].memory_size,
            Some(DiagnosticMemorySize {
                wasm_pages: 2,
                bytes: 131_072,
            })
        );
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
