use crate::{
    constants::WASM_PAGE_SIZE_BYTES,
    declaration::AllocationDeclaration,
    ledger::{AllocationLedger, AllocationRecord, GenerationRecord},
    physical::CommitStoreDiagnostic,
    slot::{AllocationSlotDescriptor, MemoryManagerAuthorityRecord, MemoryManagerRangeAuthority},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

///
/// DiagnosticExport
///
/// Read-only machine-readable allocation ledger export.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
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
    #[serde(deserialize_with = "crate::cbor::deserialize_present_option")]
    pub commit_recovery: Option<CommitStoreDiagnostic>,
}

///
/// DefaultMemoryManagerDoctorReport
///
/// Preflight and runtime diagnostic report for the default `MemoryManager`
/// integration.
///
/// This report is intended for operator-facing diagnostics. Recoverable
/// runtime problems, such as corrupt stable-cell bytes or commit recovery
/// failure, are represented as fields instead of aborting report construction.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DefaultMemoryManagerDoctorReport {
    /// Whether the default runtime has completed bootstrap validation.
    pub bootstrapped: bool,
    /// Ledger anchor descriptor used by the default runtime.
    pub ledger_anchor: AllocationSlotDescriptor,
    /// Stable-cell ledger storage status.
    pub stable_cell: DiagnosticStableCell,
    /// Protected commit recovery status when a ledger record was readable.
    #[serde(deserialize_with = "crate::cbor::deserialize_present_option")]
    pub commit_recovery: Option<CommitStoreDiagnostic>,
    /// Recovered allocation ledger export when protected recovery succeeded.
    #[serde(deserialize_with = "crate::cbor::deserialize_present_option")]
    pub ledger: Option<DiagnosticExport>,
    /// Static declarations registered by linked crates.
    pub registered_declarations: Vec<DiagnosticDeclaration>,
    /// Static range authority registered by linked crates and the effective
    /// authority table used by the default runtime.
    pub range_authority: DiagnosticRangeAuthority,
    /// Current generic default-runtime declaration validation preflight result.
    ///
    /// Caller-supplied policies passed to
    /// [`crate::bootstrap_default_memory_manager_with_policy`] are not
    /// represented in this check.
    pub validation: DiagnosticCheck,
}

///
/// DiagnosticDeclaration
///
/// Read-only diagnostic view of one static allocation declaration.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticDeclaration {
    /// Crate or integration authority that registered the declaration.
    pub authority: String,
    /// Allocation declaration registered by that authority.
    pub declaration: AllocationDeclaration,
}

impl DiagnosticDeclaration {
    /// Build a diagnostic declaration record.
    #[must_use]
    pub fn new(authority: impl Into<String>, declaration: AllocationDeclaration) -> Self {
        Self {
            authority: authority.into(),
            declaration,
        }
    }
}

///
/// DiagnosticRangeAuthority
///
/// Read-only diagnostic view of registered and effective range authority.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticRangeAuthority {
    /// Range records registered directly by linked crates.
    pub registered_records: Vec<MemoryManagerAuthorityRecord>,
    /// Effective range authority table or its validation error.
    pub effective_authority: Result<MemoryManagerRangeAuthority, String>,
}

impl DiagnosticRangeAuthority {
    /// Build a range-authority diagnostic.
    #[must_use]
    pub const fn new(
        registered_records: Vec<MemoryManagerAuthorityRecord>,
        effective_authority: Result<MemoryManagerRangeAuthority, String>,
    ) -> Self {
        Self {
            registered_records,
            effective_authority,
        }
    }
}

///
/// DiagnosticStableCell
///
/// Read-only diagnostic view of the stable-cell ledger storage envelope.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticStableCell {
    /// Stable-cell status.
    pub status: DiagnosticStableCellStatus,
    /// Backing memory size for the ledger cell.
    pub memory_size: DiagnosticMemorySize,
}

impl DiagnosticStableCell {
    /// Build a stable-cell diagnostic.
    #[must_use]
    pub const fn new(
        status: DiagnosticStableCellStatus,
        memory_size: DiagnosticMemorySize,
    ) -> Self {
        Self {
            status,
            memory_size,
        }
    }
}

///
/// DiagnosticStableCellStatus
///
/// Stable-cell ledger storage status.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub enum DiagnosticStableCellStatus {
    /// The ledger memory is empty and can be initialized.
    Empty,
    /// The stable-cell envelope and ledger record decoded successfully.
    Readable,
    /// The ledger memory is present but could not be decoded as the expected
    /// stable-cell ledger record.
    Corrupt {
        /// Stable-cell envelope or ledger-record decode error.
        error: String,
    },
}

///
/// DiagnosticCheck
///
/// Read-only diagnostic status for a preflight check.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub enum DiagnosticCheck {
    /// The check could not run because prerequisite state was unavailable.
    NotRun {
        /// Reason the check could not run.
        message: String,
    },
    /// The check completed successfully.
    Passed,
    /// The check ran and found a problem.
    Failed {
        /// Validation failure.
        message: String,
    },
}

impl DiagnosticCheck {
    /// Build a passed diagnostic check.
    #[must_use]
    pub const fn passed() -> Self {
        Self::Passed
    }

    /// Build a failed diagnostic check.
    #[must_use]
    pub fn failed(message: impl Into<String>) -> Self {
        Self::Failed {
            message: message.into(),
        }
    }

    /// Build a skipped diagnostic check.
    #[must_use]
    pub fn not_run(message: impl Into<String>) -> Self {
        Self::NotRun {
            message: message.into(),
        }
    }
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
#[serde(deny_unknown_fields)]
pub struct DiagnosticRecord {
    /// Allocation record.
    pub allocation: AllocationRecord,
    /// Live backing memory size, when the exporter measured one.
    ///
    /// This is allocation size reported by the backing memory, not logical user
    /// payload size inside the stable structure.
    #[serde(skip_serializing_if = "Option::is_none")]
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
#[serde(deny_unknown_fields)]
pub struct DiagnosticGeneration {
    /// Generation record.
    pub generation: GenerationRecord,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        declaration::AllocationDeclaration,
        ledger::{AllocationHistory, AllocationRecord},
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
                vec![AllocationRecord::active(3, declaration).expect("valid schema metadata")],
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
    fn diagnostic_export_rejects_unknown_top_level_fields() {
        use crate::test_cbor::Value;

        let export = DiagnosticExport {
            current_generation: 0,
            ledger_anchor: AllocationSlotDescriptor::memory_manager(0).expect("usable slot"),
            records: Vec::new(),
            generations: Vec::new(),
            commit_recovery: None,
        };
        let Value::Map(mut map) = crate::test_cbor::to_value(export).expect("diagnostic value")
        else {
            panic!("diagnostic export encodes as a map");
        };
        crate::test_cbor::map_insert(
            &mut map,
            Value::Text("future_field".to_string()),
            Value::Bool(true),
        );
        let bytes = crate::test_cbor::to_vec(&Value::Map(map)).expect("diagnostic bytes");

        let err = crate::test_cbor::from_slice::<DiagnosticExport>(&bytes)
            .expect_err("unknown diagnostic field must fail closed");

        assert!(err.to_string().contains("future_field"));
    }

    #[test]
    fn diagnostic_outcome_states_round_trip() {
        let stable_cell = DiagnosticStableCell::new(
            DiagnosticStableCellStatus::Corrupt {
                error: "bad stable-cell record".to_string(),
            },
            DiagnosticMemorySize::from_wasm_pages(1),
        );
        let range_authority = DiagnosticRangeAuthority::new(
            Vec::new(),
            Err("overlapping authority ranges".to_string()),
        );
        let check = DiagnosticCheck::failed("duplicate declaration");

        for value in [DiagnosticCheck::passed(), check] {
            let bytes = crate::test_cbor::to_vec(&value).expect("check bytes");
            let decoded: DiagnosticCheck =
                crate::test_cbor::from_slice(&bytes).expect("check round trip");
            assert_eq!(decoded, value);
        }

        let bytes = crate::test_cbor::to_vec(&stable_cell).expect("stable-cell diagnostic bytes");
        let decoded: DiagnosticStableCell =
            crate::test_cbor::from_slice(&bytes).expect("stable-cell round trip");
        assert_eq!(decoded, stable_cell);

        let bytes = crate::test_cbor::to_vec(&range_authority).expect("range diagnostic bytes");
        let decoded: DiagnosticRangeAuthority =
            crate::test_cbor::from_slice(&bytes).expect("range round trip");
        assert_eq!(decoded, range_authority);
    }

    #[test]
    fn diagnostic_export_can_include_commit_recovery_state() {
        let ledger = AllocationLedger {
            current_generation: 3,
            allocation_history: AllocationHistory::default(),
        };
        let commit_recovery = CommitStoreDiagnostic {
            slot0: CommitSlotDiagnostic::Valid { generation: 3 },
            slot1: CommitSlotDiagnostic::Empty,
            recovery: Ok(3),
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
                vec![AllocationRecord::active(3, declaration).expect("valid schema metadata")],
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
            slot0: CommitSlotDiagnostic::Empty,
            slot1: CommitSlotDiagnostic::Empty,
            recovery: Err(CommitRecoveryError::NoValidGeneration),
        };

        let export = DiagnosticExport::from_ledger_with_commit_recovery(
            &ledger,
            AllocationSlotDescriptor::memory_manager(0).expect("usable slot"),
            Some(commit_recovery),
        );

        assert_eq!(
            export.commit_recovery.expect("commit recovery").recovery,
            Err(CommitRecoveryError::NoValidGeneration)
        );
    }
}
