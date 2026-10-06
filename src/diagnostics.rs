use crate::{
    constants::WASM_PAGE_SIZE_BYTES,
    declaration::AllocationDeclaration,
    ledger::{AllocationLedger, AllocationRecord},
    physical::CommitStoreDiagnostic,
    policy::PolicyIdentity,
    registry::SealedDeclarationFingerprint,
    slot::{MemoryManagerAuthorityRecord, MemoryManagerRangeAuthority, MemoryManagerSlot},
};
use serde::{Deserialize, Serialize};

///
/// DiagnosticExport
///
/// Read-only machine-readable allocation ledger export.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticExport {
    /// Current committed generation.
    pub current_generation: u64,
    /// Checked ledger anchor slot.
    pub ledger_anchor: MemoryManagerSlot,
    /// Allocation records.
    pub records: Vec<DiagnosticRecord>,
    /// Optional protected commit recovery diagnostic.
    #[serde(deserialize_with = "crate::cbor::deserialize_present_option")]
    pub commit_recovery: Option<CommitStoreDiagnostic>,
}

///
/// DiagnosticRuntimeBinding
///
/// Operator-facing view of the policy identity and declaration snapshot bound
/// to one successful memory-runtime bootstrap.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticRuntimeBinding {
    /// Bounded semantic identity of the bootstrap policy.
    pub policy_identity: PolicyIdentity,
    /// Deterministic fingerprint of the sealed declaration snapshot.
    pub declaration_fingerprint: SealedDeclarationFingerprint,
}

impl DiagnosticRuntimeBinding {
    /// Build one runtime binding diagnostic.
    #[must_use]
    pub const fn new(
        policy_identity: PolicyIdentity,
        declaration_fingerprint: SealedDeclarationFingerprint,
    ) -> Self {
        Self {
            policy_identity,
            declaration_fingerprint,
        }
    }
}

///
/// MemoryRuntimeDoctorReport
///
/// Preflight and runtime diagnostic report for one concrete
/// [`crate::MemoryRuntime`].
///
/// This report is intended for operator-facing diagnostics. Recoverable
/// runtime problems, such as corrupt stable-cell bytes or commit recovery
/// failure, are represented as fields instead of aborting report construction.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MemoryRuntimeDoctorReport {
    /// Whether this runtime has completed bootstrap validation.
    pub bootstrapped: bool,
    /// Policy identity supplied for this diagnostic evaluation.
    pub tested_policy_identity: Result<PolicyIdentity, DiagnosticFailure>,
    /// Sealed declaration fingerprint supplied for this diagnostic evaluation.
    pub tested_declaration_fingerprint: SealedDeclarationFingerprint,
    /// Binding published by this runtime's successful bootstrap, when present.
    #[serde(deserialize_with = "crate::cbor::deserialize_present_option")]
    pub established_bootstrap_binding: Option<DiagnosticRuntimeBinding>,
    /// Whether the tested identity and declarations match the established
    /// bootstrap binding.
    pub bootstrap_binding: DiagnosticCheck,
    /// Checked ledger anchor slot used by this runtime.
    pub ledger_anchor: MemoryManagerSlot,
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
    /// authority table supplied to this runtime.
    pub range_authority: DiagnosticRangeAuthority,
    /// Declaration validation result under the tested caller-supplied policy.
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
/// DiagnosticCode
///
/// Stable machine-readable category for an operator diagnostic failure.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum DiagnosticCode {
    /// Stable-cell storage could not be decoded.
    #[serde(rename = "stable_cell")]
    StableCell,
    /// Persisted bytes use a recognized but unsupported durable format.
    #[serde(rename = "unsupported_format")]
    UnsupportedFormat,
    /// Protected ledger recovery failed.
    #[serde(rename = "ledger_recovery")]
    LedgerRecovery,
    /// Current declarations failed allocation validation.
    #[serde(rename = "allocation_validation")]
    AllocationValidation,
    /// Runtime bootstrap policy identity was invalid.
    #[serde(rename = "policy_identity")]
    PolicyIdentity,
    /// Tested bootstrap identity or declarations differ from runtime state.
    #[serde(rename = "runtime_binding")]
    RuntimeBinding,
}

///
/// DiagnosticFailure
///
/// Machine-readable diagnostic code paired with an operator-facing message.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticFailure {
    /// Stable diagnostic category.
    pub code: DiagnosticCode,
    /// Human-readable failure detail.
    pub message: String,
}

impl DiagnosticFailure {
    /// Build a coded diagnostic failure.
    #[must_use]
    pub fn new(code: DiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
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
    /// Validated effective range authority from the sealed declarations.
    pub effective_authority: MemoryManagerRangeAuthority,
}

impl DiagnosticRangeAuthority {
    /// Build a range-authority diagnostic.
    #[must_use]
    pub const fn new(
        registered_records: Vec<MemoryManagerAuthorityRecord>,
        effective_authority: MemoryManagerRangeAuthority,
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
        /// Stable-cell envelope or ledger-record decode failure.
        failure: DiagnosticFailure,
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
        /// Stable diagnostic category.
        code: DiagnosticCode,
        /// Reason the check could not run.
        message: String,
    },
    /// The check completed successfully.
    Passed,
    /// The check ran and found a problem.
    Failed {
        /// Stable diagnostic category.
        code: DiagnosticCode,
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
    pub fn failed(code: DiagnosticCode, message: impl Into<String>) -> Self {
        Self::Failed {
            code,
            message: message.into(),
        }
    }

    /// Build a skipped diagnostic check.
    #[must_use]
    pub fn not_run(code: DiagnosticCode, message: impl Into<String>) -> Self {
        Self::NotRun {
            code,
            message: message.into(),
        }
    }
}

impl DiagnosticExport {
    /// Build a read-only diagnostic export from an allocation ledger.
    ///
    /// Records start unmeasured and without commit recovery observations. The
    /// exporter can fill those public fields from its own backing and recovery
    /// evidence; this DTO constructor neither recovers nor measures memory.
    #[must_use]
    pub fn from_ledger(ledger: &AllocationLedger, ledger_anchor: MemoryManagerSlot) -> Self {
        Self::from_records(
            ledger.current_generation,
            ledger_anchor,
            ledger.records.iter().cloned(),
        )
    }

    // Normal exports consume their decoded records. Borrowed exports copy
    // records directly into the same projection without an intermediate ledger.
    pub(crate) fn from_owned_ledger(
        ledger: AllocationLedger,
        ledger_anchor: MemoryManagerSlot,
    ) -> Self {
        Self::from_records(
            ledger.current_generation,
            ledger_anchor,
            ledger.records.into_iter(),
        )
    }

    fn from_records(
        current_generation: u64,
        ledger_anchor: MemoryManagerSlot,
        records: impl Iterator<Item = AllocationRecord>,
    ) -> Self {
        Self {
            current_generation,
            ledger_anchor,
            records: records
                .map(|allocation| DiagnosticRecord {
                    allocation,
                    memory_size: None,
                })
                .collect(),
            commit_recovery: None,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        declaration::AllocationDeclaration,
        ledger::AllocationRecord,
        physical::{CommitRecoveryError, CommitSlotDiagnostic, CommitStoreDiagnostic},
        schema::SchemaMetadata,
    };

    #[test]
    fn diagnostic_export_copies_ledger_records() {
        let declaration = AllocationDeclaration::new(
            "app.users.v1",
            MemoryManagerSlot::new(100).expect("usable slot"),
            None,
            SchemaMetadata::default(),
        )
        .expect("declaration");
        let ledger = AllocationLedger {
            current_generation: 3,
            records: vec![AllocationRecord::active(&declaration)],
        };

        let export =
            DiagnosticExport::from_ledger(&ledger, MemoryManagerSlot::new(0).expect("usable slot"));

        assert_eq!(export.current_generation, 3);
        assert_eq!(export.records.len(), 1);
        assert_eq!(export.records[0].memory_size, None);

        assert_eq!(
            export.ledger_anchor,
            MemoryManagerSlot::new(0).expect("usable slot")
        );
        assert_eq!(export.commit_recovery, None);

        assert_eq!(
            export,
            DiagnosticExport::from_owned_ledger(ledger, export.ledger_anchor.clone())
        );
        let wire = serde_json::to_value(&export).expect("diagnostic JSON");
        assert_eq!(
            wire["records"][0]["allocation"]["schema"],
            serde_json::json!({"schema_version": null})
        );

        let decoded: DiagnosticExport = serde_json::from_value(wire).expect("current JSON shape");
        assert_eq!(decoded, export);
    }

    #[test]
    fn diagnostic_export_rejects_unknown_top_level_fields() {
        use crate::test_cbor::Value;

        let export = DiagnosticExport {
            current_generation: 0,
            ledger_anchor: MemoryManagerSlot::new(0).expect("usable slot"),
            records: Vec::new(),
            commit_recovery: None,
        };
        let Value::Map(mut map) = crate::test_cbor::to_value(export).expect("diagnostic value")
        else {
            panic!("diagnostic export encodes as a map");
        };
        map.push((Value::Text("future_field".to_string()), Value::Bool(true)));
        let bytes = crate::test_cbor::to_vec(&Value::Map(map)).expect("diagnostic bytes");

        let err = crate::test_cbor::from_slice::<DiagnosticExport>(&bytes)
            .expect_err("unknown diagnostic field must fail closed");

        assert!(err.to_string().contains("future_field"));
    }

    #[test]
    fn diagnostic_outcome_states_round_trip() {
        let stable_cell = DiagnosticStableCell::new(
            DiagnosticStableCellStatus::Corrupt {
                failure: DiagnosticFailure::new(
                    DiagnosticCode::StableCell,
                    "bad stable-cell record",
                ),
            },
            DiagnosticMemorySize::from_wasm_pages(1),
        );
        let range_authority =
            DiagnosticRangeAuthority::new(Vec::new(), MemoryManagerRangeAuthority::default());
        let check = DiagnosticCheck::failed(
            DiagnosticCode::AllocationValidation,
            "duplicate declaration",
        );

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
    fn diagnostic_codes_have_stable_wire_names() {
        let cases = [
            (DiagnosticCode::StableCell, "stable_cell"),
            (DiagnosticCode::UnsupportedFormat, "unsupported_format"),
            (DiagnosticCode::LedgerRecovery, "ledger_recovery"),
            (
                DiagnosticCode::AllocationValidation,
                "allocation_validation",
            ),
            (DiagnosticCode::PolicyIdentity, "policy_identity"),
            (DiagnosticCode::RuntimeBinding, "runtime_binding"),
        ];

        for (code, expected) in cases {
            assert_eq!(
                crate::test_cbor::to_value(code).expect("diagnostic code value"),
                crate::test_cbor::Value::Text(expected.to_string())
            );
        }
    }

    #[test]
    fn diagnostic_export_can_include_commit_recovery_state() {
        let ledger = AllocationLedger {
            current_generation: 3,
            records: Vec::new(),
        };
        let commit_recovery = CommitStoreDiagnostic {
            slot0: CommitSlotDiagnostic::Valid { generation: 3 },
            slot1: CommitSlotDiagnostic::Empty,
            recovery: Ok(3),
        };

        let mut export =
            DiagnosticExport::from_ledger(&ledger, MemoryManagerSlot::new(0).expect("usable slot"));
        export.commit_recovery = Some(commit_recovery);

        assert_eq!(export.commit_recovery, Some(commit_recovery));
    }

    #[test]
    fn diagnostic_export_can_include_memory_sizes() {
        let declaration = AllocationDeclaration::new(
            "app.users.v1",
            MemoryManagerSlot::new(100).expect("usable slot"),
            None,
            SchemaMetadata::default(),
        )
        .expect("declaration");
        let ledger = AllocationLedger {
            current_generation: 3,
            records: vec![AllocationRecord::active(&declaration)],
        };

        let mut export =
            DiagnosticExport::from_ledger(&ledger, MemoryManagerSlot::new(0).expect("usable slot"));
        export.records[0].memory_size = Some(DiagnosticMemorySize::from_wasm_pages(2));

        assert_eq!(
            export.records[0].memory_size,
            Some(DiagnosticMemorySize {
                wasm_pages: 2,
                bytes: 131_072,
            })
        );
        let wire = serde_json::to_value(&export).expect("diagnostic JSON");
        assert_eq!(
            wire["records"][0]["memory_size"],
            serde_json::json!({"wasm_pages": 2, "bytes": 131_072})
        );
    }

    #[test]
    fn diagnostic_export_can_report_recovery_failure() {
        let ledger = AllocationLedger {
            current_generation: 0,
            records: Vec::new(),
        };
        let commit_recovery = CommitStoreDiagnostic {
            slot0: CommitSlotDiagnostic::Empty,
            slot1: CommitSlotDiagnostic::Empty,
            recovery: Err(CommitRecoveryError::NoValidGeneration),
        };

        let mut export =
            DiagnosticExport::from_ledger(&ledger, MemoryManagerSlot::new(0).expect("usable slot"));
        export.commit_recovery = Some(commit_recovery);

        assert_eq!(
            export.commit_recovery.expect("commit recovery").recovery,
            Err(CommitRecoveryError::NoValidGeneration)
        );
    }
}
