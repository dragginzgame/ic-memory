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
    pub commit_recovery: Option<CommitStoreDiagnostic>,
    /// Recovered allocation ledger export when protected recovery succeeded.
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
    pub declaring_crate: String,
    /// Allocation declaration registered by that authority.
    pub declaration: AllocationDeclaration,
}

impl DiagnosticDeclaration {
    /// Build a diagnostic declaration record.
    #[must_use]
    pub fn new(declaring_crate: impl Into<String>, declaration: AllocationDeclaration) -> Self {
        Self {
            declaring_crate: declaring_crate.into(),
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
    /// Effective range authority table, including runtime-owned internal
    /// records, when the table validated successfully.
    pub effective_authority: Option<MemoryManagerRangeAuthority>,
    /// Validation error when the effective authority table could not be built.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl DiagnosticRangeAuthority {
    /// Build a range-authority diagnostic.
    #[must_use]
    pub const fn new(
        registered_records: Vec<MemoryManagerAuthorityRecord>,
        effective_authority: Option<MemoryManagerRangeAuthority>,
        error: Option<String>,
    ) -> Self {
        Self {
            registered_records,
            effective_authority,
            error,
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
    /// Decode error when the stable cell was not readable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl DiagnosticStableCell {
    /// Build a stable-cell diagnostic.
    #[must_use]
    pub const fn new(
        status: DiagnosticStableCellStatus,
        memory_size: DiagnosticMemorySize,
        error: Option<String>,
    ) -> Self {
        Self {
            status,
            memory_size,
            error,
        }
    }
}

///
/// DiagnosticStableCellStatus
///
/// Stable-cell ledger storage status.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum DiagnosticStableCellStatus {
    /// The ledger memory is empty and can be initialized.
    Empty,
    /// The stable-cell envelope and ledger record decoded successfully.
    Readable,
    /// The ledger memory is present but could not be decoded as the expected
    /// stable-cell ledger record.
    Corrupt,
}

///
/// DiagnosticCheck
///
/// Read-only diagnostic status for a preflight check.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticCheck {
    /// Check status.
    pub status: DiagnosticCheckStatus,
    /// Failure or skip reason.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl DiagnosticCheck {
    /// Build a passed diagnostic check.
    #[must_use]
    pub const fn passed() -> Self {
        Self {
            status: DiagnosticCheckStatus::Passed,
            message: None,
        }
    }

    /// Build a failed diagnostic check.
    #[must_use]
    pub fn failed(message: impl Into<String>) -> Self {
        Self {
            status: DiagnosticCheckStatus::Failed,
            message: Some(message.into()),
        }
    }

    /// Build a skipped diagnostic check.
    #[must_use]
    pub fn not_run(message: impl Into<String>) -> Self {
        Self {
            status: DiagnosticCheckStatus::NotRun,
            message: Some(message.into()),
        }
    }
}

///
/// DiagnosticCheckStatus
///
/// Status for one diagnostic preflight check.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum DiagnosticCheckStatus {
    /// The check could not run because prerequisite state was unavailable.
    NotRun,
    /// The check completed successfully.
    Passed,
    /// The check ran and found a problem.
    Failed,
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
                vec![
                    AllocationRecord::from_declaration(3, declaration, AllocationState::Active)
                        .expect("valid schema metadata"),
                ],
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
        use serde_cbor::Value;

        let export = DiagnosticExport {
            current_generation: 0,
            ledger_anchor: AllocationSlotDescriptor::memory_manager(0).expect("usable slot"),
            records: Vec::new(),
            generations: Vec::new(),
            commit_recovery: None,
        };
        let Value::Map(mut map) = serde_cbor::value::to_value(export).expect("diagnostic value")
        else {
            panic!("diagnostic export encodes as a map");
        };
        map.insert(Value::Text("future_field".to_string()), Value::Bool(true));
        let bytes = serde_cbor::to_vec(&Value::Map(map)).expect("diagnostic bytes");

        let err = serde_cbor::from_slice::<DiagnosticExport>(&bytes)
            .expect_err("unknown diagnostic field must fail closed");

        assert!(err.to_string().contains("future_field"));
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
                vec![
                    AllocationRecord::from_declaration(3, declaration, AllocationState::Active)
                        .expect("valid schema metadata"),
                ],
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
