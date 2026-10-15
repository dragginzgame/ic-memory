#![deny(unsafe_code, unsafe_op_in_unsafe_fn)]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc = include_str!("../README.md")]

//! Stable-memory allocation-governance primitives for Internet Computer
//! canister upgrades.
//!
//! `ic-memory` prevents stable-memory slot drift.
//!
//! Once a stable key is committed to a physical allocation slot, future binaries
//! must either reopen that same stable key on that same slot or declare a new
//! stable key.
//!
//! The crate records and validates durable ownership in both directions: an
//! active stable key cannot move to a different physical slot, and an active
//! physical slot cannot be reused by a different stable key.
//!
//! The intended runtime integration flow is:
//!
//! 1. Recover the persisted allocation ledger.
//! 2. Admit consumer identity and authorized historical selections from bounded
//!    metadata under the host's `RuntimeBootstrapPolicy`.
//! 3. Resolve key requests in the host-wide pool under current namespace grants,
//!    then validate retained key-to-ID bindings and application policy.
//! 4. Stage and durably persist the next generation.
//! 5. Only then open stable-memory handles through committed allocation
//!    authority.
//!
//! This crate owns allocation invariants, not framework policy. Namespace
//! rules, controller authorization, endpoint lifecycle, schema migrations, and
//! application validation belong to the framework or application.
//!
//! The host supplies one [`MemoryAllocationPool`] with explicit namespace grants
//! and physical exclusions. Linked components request permanent keys and name
//! their owner; they do not select numeric IDs or reserve component subranges.
//! Namespace grants are current host policy, not caller authentication or
//! persisted ownership labels. Application admission remains host-owned.
//!
//! Use these primitives before opening stable-memory handles. Integrations
//! should recover the historical ledger, declare the stores expected by the
//! current binary, admit recovered identity, resolve requests, validate against
//! history and policy, persist a new generation, and only then publish authority
//! before opening slots through the storage owner.
//!
//! Bounded physical attribution is available through
//! [`MemoryRuntime::memory_allocations`] and
//! [`default_memory_manager_memory_allocations`]. It reports actual persisted
//! buckets and explicit residuals without decoding retained ownership. Virtual
//! extent is not payload occupancy. Opens return [`RuntimeMemory`]; explicit
//! [`MemoryManagerConfig`] selects fresh-state buckets or checks a persisted
//! setting without migration. The default remains 128 pages.
//!
//! [`MemoryRuntime`] is the canonical owner for one backing memory instance. It
//! owns that memory's manager, ledger persistence, bootstrap lifecycle, committed
//! capability, opens, and diagnostics. Each bootstrap attempt fallibly decodes
//! its ledger record and uses a temporary cell for capacity-checked writes.
//! Linked code contributes declarations to
//! one immutable [`SealedDeclarationSnapshot`], which is supplied to each
//! runtime independently.
//!
//! The host supplies the complete checked request snapshot, allocation pool and
//! admission policy directly to the runtime. Component factories and generators
//! contribute request lists; opening a store never registers a declaration.
//! Exactly one runtime owns bootstrap and persistence for each backing memory.
//!
//! `ic-stable-structures` `MemoryManager` IDs are the first-class supported
//! physical slot substrate. That ID domain is `u8`: IDs `0..=254` are usable,
//! and ID `255` is always the `ic-stable-structures` unallocated sentinel.
//! Protected current-snapshot commits and stable-cell encoding remain internal to the
//! runtime. Public diagnostics describe their state without granting a manual
//! commit or persistence-confirmation route.
//!
//! [`ic_stable_structures`] re-exports the exact substrate version used by this
//! crate. Use its collections and traits with [`RuntimeMemory`] handles;
//! `ic-memory` owns allocation governance without wrapping typed collections.

mod allocation_pool;
mod bounded_writer;
mod capability;
mod cbor;
mod constants;
mod declaration;
mod diagnostics;
mod hash;
mod key;
mod ledger;
mod physical;
mod policy;
mod requests;
mod runtime;
mod schema;
mod slot;
mod sort;
mod stable_cell;
mod text;
mod validation;

#[cfg(test)]
mod test_cbor {
    use serde::{Serialize, de::DeserializeOwned};

    pub use ciborium::Value;

    pub fn to_vec<T: Serialize>(
        value: &T,
    ) -> Result<Vec<u8>, ciborium::ser::Error<std::io::Error>> {
        let mut bytes = Vec::new();
        ciborium::into_writer(value, &mut bytes)?;
        Ok(bytes)
    }

    pub fn from_slice<T: DeserializeOwned>(
        bytes: &[u8],
    ) -> Result<T, ciborium::de::Error<std::io::Error>> {
        crate::cbor::from_slice_exact(bytes)
    }

    pub fn to_value<T: Serialize>(value: T) -> Result<Value, ciborium::value::Error> {
        Value::serialized(&value)
    }

    pub fn hex_fixture(contents: &str) -> Vec<u8> {
        let hex = contents
            .chars()
            .filter(|char| !char.is_whitespace())
            .collect::<String>();
        assert_eq!(hex.len() % 2, 0, "fixture hex must have byte pairs");
        hex.as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| {
                let pair = std::str::from_utf8(pair).expect("fixture hex is utf8");
                u8::from_str_radix(pair, 16).expect("fixture hex byte")
            })
            .collect()
    }
}

/// Stable collections and traits from this crate's exact substrate dependency.
///
/// Use the upstream collections with [`RuntimeMemory`] handles obtained through
/// the owned runtime. This re-export preserves upstream type identity.
pub use ic_stable_structures;

pub use allocation_pool::{MemoryAllocationPool, MemoryAllocationPoolError, MemoryAuthority};
pub use capability::CommittedAllocations;
pub use constants::{
    MAX_LEDGER_BYTES, MAX_LEDGER_NESTING, MAX_LEDGER_RECORD_BYTES, WASM_PAGE_SIZE_BYTES,
};
pub use declaration::{AllocationDeclaration, DeclarationSnapshot, DeclarationSnapshotError};
pub use diagnostics::{
    DiagnosticCheck, DiagnosticCode, DiagnosticExport, DiagnosticFailure, DiagnosticMemorySize,
    DiagnosticRecord, DiagnosticRuntimeBinding, DiagnosticStableCell, DiagnosticStableCellStatus,
    MemoryRuntimeDoctorReport,
};
pub use key::{StableKey, StableKeyError};
pub(crate) use ledger::LedgerCommitStore;
#[cfg(test)]
pub(crate) use ledger::LedgerPayloadEnvelope;
pub use ledger::{
    AllocationLedger, AllocationRecord, AllocationReservationError, AllocationRetirement,
    AllocationRetirementError, AllocationStageError, AllocationState,
    LEDGER_PAYLOAD_FORMAT_VERSION, LedgerCommitError, LedgerIntegrityError,
    LedgerPayloadEnvelopeError, RecoveredLedger,
};
#[cfg(test)]
pub(crate) use physical::CommittedGenerationBytes;
pub use physical::{CommitRecoveryError, CommitSnapshotDiagnostic};
pub use policy::{AllocationPolicy, PolicyIdentity, PolicyIdentityError, RuntimeBootstrapPolicy};
pub use requests::{
    MemoryDeclarationError, MemoryRequest, SealedDeclarationFingerprint, SealedDeclarationSnapshot,
};
pub use runtime::{
    AllocationBinding, BootstrapAdmission, BootstrapAdmissionError, GenericAllocationPolicy,
    MemoryAllocation, MemoryAllocationSummary, MemoryAllocations, MemoryBindingSummary,
    MemoryManagerConfig, MemoryManagerLayoutError, MemoryResolutionError, MemoryRuntime,
    RecoveredAllocationMetadata, RuntimeAdoptionError, RuntimeBootstrapError,
    RuntimeConstructionError, RuntimeDiagnosticError, RuntimeGrowError, RuntimeMemory,
    RuntimeOpenError, RuntimeStateError, bootstrap_default_memory_manager,
    bootstrap_default_memory_manager_with_config, bootstrap_default_memory_manager_with_policy,
    committed_allocations, default_memory_manager_commit_recovery_diagnostic,
    default_memory_manager_diagnostic_export, default_memory_manager_doctor_report,
    default_memory_manager_doctor_report_with_policy,
    default_memory_manager_memory_allocation_summary, default_memory_manager_memory_allocations,
    default_memory_manager_memory_id, is_default_memory_manager_bootstrapped,
    open_default_memory_manager_memory, verify_default_memory_manager_authority,
};
pub use schema::{SchemaMetadata, SchemaMetadataError};
pub use slot::{
    IC_MEMORY_AUTHORITY_OWNER, IC_MEMORY_LEDGER_STABLE_KEY, IC_MEMORY_STABLE_KEY_PREFIX,
    MEMORY_MANAGER_GOVERNANCE_MAX_ID, MEMORY_MANAGER_INVALID_ID, MEMORY_MANAGER_LEDGER_ID,
    MEMORY_MANAGER_MAX_ID, MEMORY_MANAGER_MIN_ID, MemoryManagerIdRange, MemoryManagerRangeError,
    MemoryManagerSlot, MemoryManagerSlotError, is_ic_memory_stable_key,
    memory_manager_governance_range, validate_memory_manager_id,
};
pub use stable_cell::{
    STABLE_CELL_HEADER_SIZE, STABLE_CELL_LAYOUT_VERSION, STABLE_CELL_MAGIC,
    STABLE_CELL_VALUE_OFFSET, StableCellLedgerError, StableCellLedgerRecord,
    StableCellPayloadError, decode_stable_cell_ledger_record,
    decode_stable_cell_ledger_record_from_memory, decode_stable_cell_payload,
};
pub use validation::AllocationValidationError;
#[cfg(test)]
pub(crate) use validation::validate_allocations;

#[cfg(test)]
mod allocation_measurements_tests;

#[cfg(feature = "test-support")]
pub mod testing;
