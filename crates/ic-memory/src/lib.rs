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
//! [`AllocationBootstrap`] is the golden path for whichever layer owns a given
//! ledger store. Canic may own bootstrap for a framework canister and compose
//! IcyDB/application declarations through its registry; IcyDB may own bootstrap
//! directly for generated database stores; or a standalone application canister
//! may own bootstrap itself. Exactly one owner should bootstrap one ledger
//! store. Multiple layers in the same canister must either compose declarations
//! into that owner or use distinct ledger stores and allocation domains.
//!
//! `ic-stable-structures` `MemoryManager` IDs are the first-class supported
//! physical slot substrate. That ID domain is `u8`: IDs `0..=254` are usable,
//! and ID `255` is always the `ic-stable-structures` unallocated sentinel.
//! The crate still keeps narrow internal abstractions for storage adapters and
//! diagnostics, but the native IC path is
//! `MemoryManager` ID 0 -> `ic-stable-structures::Cell<StableCellLedgerRecord,
//! _>` -> [`LedgerCommitStore`] -> [`CommittedGenerationBytes`] ->
//! [`LedgerPayloadEnvelope`] -> [`RecoveredLedger`] -> [`ValidatedAllocations`]
//! -> [`CommittedAllocations`].
//!
//! [`ic_stable_structures`] re-exports the exact substrate version used by this
//! crate. Use its collections and traits with [`RuntimeMemory`] handles;
//! `ic-memory` owns allocation governance without wrapping typed collections.

mod allocation_pool;
mod bootstrap;
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
mod registry;
mod runtime;
mod schema;
mod slot;
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
pub use bootstrap::{
    AllocationBootstrap, BootstrapError, BootstrapReservationError, BootstrapRetirementError,
    PendingBootstrapCommit,
};
pub use capability::{CommittedAllocations, ValidatedAllocations};
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
pub use ledger::{
    AllocationLedger, AllocationRecord, AllocationReservationError, AllocationRetirement,
    AllocationRetirementError, AllocationStageError, AllocationState,
    LEDGER_PAYLOAD_FORMAT_VERSION, LedgerCommitError, LedgerCommitStore, LedgerIntegrityError,
    LedgerPayloadEnvelope, LedgerPayloadEnvelopeError, RecoveredLedger,
};
pub use physical::{
    CommitRecoveryError, CommitSlotDiagnostic, CommitStoreDiagnostic, CommittedGenerationBytes,
    DualCommitStore,
};
pub use policy::{AllocationPolicy, PolicyIdentity, PolicyIdentityError, RuntimeBootstrapPolicy};
pub use registry::{
    MemoryRequest, SealedDeclarationFingerprint, SealedDeclarationSnapshot,
    StaticMemoryDeclarationError, register_memory_request, sealed_declaration_snapshot,
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
    IC_MEMORY_AUTHORITY_OWNER, IC_MEMORY_LEDGER_LABEL, IC_MEMORY_LEDGER_STABLE_KEY,
    IC_MEMORY_STABLE_KEY_PREFIX, MEMORY_MANAGER_GOVERNANCE_MAX_ID, MEMORY_MANAGER_INVALID_ID,
    MEMORY_MANAGER_LEDGER_ID, MEMORY_MANAGER_MAX_ID, MEMORY_MANAGER_MIN_ID, MemoryManagerIdRange,
    MemoryManagerRangeError, MemoryManagerSlot, MemoryManagerSlotError, is_ic_memory_stable_key,
    memory_manager_governance_range, validate_memory_manager_id,
};
pub use stable_cell::{
    STABLE_CELL_HEADER_SIZE, STABLE_CELL_LAYOUT_VERSION, STABLE_CELL_MAGIC,
    STABLE_CELL_VALUE_OFFSET, StableCellLedgerError, StableCellLedgerRecord,
    StableCellPayloadError, decode_stable_cell_ledger_record,
    decode_stable_cell_ledger_record_from_memory, decode_stable_cell_payload,
};
pub use validation::{AllocationValidationError, validate_allocations};

#[doc(hidden)]
pub use registry::{defer_eager_init, defer_static_memory_registration};

#[doc(hidden)]
pub mod __reexports {
    pub use ctor;
}

/// Register a `MemoryManager` allocation declaration during static initialization.
///
/// The authority names the owner admitted by the host namespace grant.
/// A string literal or shared compile-time string constant
/// may be used. Internal `ic-memory` authority is unavailable to callers.
///
/// This macro only registers declaration metadata. It does not open stable
/// memory. The bootstrap owner still has to collect/seal declarations, validate
/// them against the ledger, commit the generation, and then open memory handles.
#[macro_export]
macro_rules! ic_memory_declaration {
    (authority = $authority:expr, key = $stable_key:literal $(,)?) => {
        const _: () = {
            fn __ic_memory_register_request() -> Result<(), $crate::StaticMemoryDeclarationError> {
                $crate::register_memory_request($crate::MemoryRequest::new(
                    $authority, $stable_key, $crate::SchemaMetadata::default(),
                )?)
            }
            #[ $crate::__reexports::ctor::ctor(unsafe, anonymous, crate_path = $crate::__reexports::ctor) ]
            fn __ic_memory_defer_request() {
                $crate::defer_static_memory_registration(__ic_memory_register_request);
            }
        };
    };
}

/// Declare a key-only request and open it after the host has committed bootstrap.
#[macro_export]
macro_rules! ic_memory_key {
    (authority = $authority:expr, key = $stable_key:literal $(,)?) => {{
        $crate::ic_memory_declaration!(authority = $authority, key = $stable_key);
        $crate::open_default_memory_manager_memory($stable_key)
    }};
}

/// Register one pre-bootstrap hook.
#[macro_export]
macro_rules! eager_init {
    ($body:block) => {
        const _: () = {
            fn __ic_memory_registered_eager_init_body() {
                $body
            }

            #[ $crate::__reexports::ctor::ctor(unsafe, anonymous, crate_path = $crate::__reexports::ctor) ]
            fn __ic_memory_register_eager_init() {
                $crate::defer_eager_init(__ic_memory_registered_eager_init_body);
            }
        };
    };
}
