#![forbid(unsafe_code)]
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
//! The intended integration flow is:
//!
//! 1. Recover the persisted allocation ledger.
//! 2. Declare the stable stores expected by the current binary.
//! 3. Validate those declarations against ledger history and any framework
//!    policy.
//! 4. Commit the next generation.
//! 5. Only then open stable-memory handles through a committed allocation
//!    session.
//!
//! This crate owns allocation invariants, not framework policy. Namespace
//! rules, controller authorization, endpoint lifecycle, schema migrations, and
//! application validation belong to the framework or application.
//!
//! For the default `MemoryManager` runtime, registered `ic-memory` range claims
//! are generic allocation policy and are enforced before caller-supplied
//! policy. A framework such as Canic that wants higher-level range semantics
//! should adapt to this contract deliberately: either register the ranges it
//! wants `ic-memory` to enforce, or omit user ranges and enforce application
//! space through its own [`AllocationPolicy`].
//!
//! Use these primitives before opening stable-memory handles. Integrations
//! should recover the historical ledger, declare the stores expected by the
//! current binary, validate declarations against history and policy, commit a
//! new generation, and only then publish a committed allocation session that can
//! open slots through a storage substrate.
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
//! `ic-memory` is not a replacement for `ic-stable-structures` collections and
//! does not wrap typed stores such as `StableBTreeMap`.

mod bootstrap;
mod constants;
mod declaration;
mod diagnostics;
mod key;
mod ledger;
mod physical;
mod policy;
mod registry;
mod runtime;
mod schema;
mod session;
mod slot;
mod stable_cell;
mod substrate;
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
        ciborium::from_reader(bytes)
    }

    pub fn to_value<T: Serialize>(value: T) -> Result<Value, ciborium::value::Error> {
        Value::serialized(&value)
    }

    pub fn map_insert(map: &mut Vec<(Value, Value)>, key: Value, value: Value) {
        map.push((key, value));
    }
}

pub use bootstrap::{
    AllocationBootstrap, BootstrapError, BootstrapReservationError, BootstrapRetirementError,
    PendingBootstrapCommit,
};
pub use constants::WASM_PAGE_SIZE_BYTES;
pub use declaration::{
    AllocationDeclaration, DeclarationCollector, DeclarationSnapshot, DeclarationSnapshotError,
};
pub use diagnostics::{
    DefaultMemoryManagerDoctorReport, DiagnosticCheck, DiagnosticCheckStatus,
    DiagnosticDeclaration, DiagnosticExport, DiagnosticGeneration, DiagnosticMemorySize,
    DiagnosticRangeAuthority, DiagnosticRecord, DiagnosticStableCell, DiagnosticStableCellStatus,
};
pub use key::{StableKey, StableKeyError};
pub use ledger::{
    AllocationHistory, AllocationLedger, AllocationRecord, AllocationReservationError,
    AllocationRetirement, AllocationRetirementError, AllocationStageError, AllocationState,
    GenerationRecord, LedgerCommitError, LedgerCommitStore, LedgerIntegrityError,
    LedgerPayloadEnvelope, LedgerPayloadEnvelopeError, RecoveredLedger, SchemaMetadataRecord,
};
pub use physical::{
    AuthoritativeSlot, CommitRecoveryError, CommitSlotDiagnostic, CommitSlotIndex,
    CommitStoreDiagnostic, CommittedGenerationBytes, DualCommitStore, DualProtectedCommitStore,
    ProtectedGenerationSlot, select_authoritative_slot,
};
pub use policy::AllocationPolicy;
pub use registry::{
    StaticMemoryDeclaration, StaticMemoryDeclarationError, StaticMemoryRangeDeclaration,
    collect_static_memory_declarations, register_static_memory_declaration,
    register_static_memory_manager_declaration,
    register_static_memory_manager_declaration_with_schema, register_static_memory_manager_range,
    register_static_memory_range_declaration, static_memory_declaration_snapshot,
    static_memory_declarations, static_memory_range_authority, static_memory_range_declarations,
};
pub use runtime::{
    RuntimeBootstrapError, RuntimeDiagnosticError, RuntimeOpenError, RuntimePolicyError,
    bootstrap_default_memory_manager, bootstrap_default_memory_manager_with_policy,
    committed_allocations, default_memory_manager_commit_recovery_diagnostic,
    default_memory_manager_diagnostic_export, default_memory_manager_doctor_report,
    is_default_memory_manager_bootstrapped, open_default_memory_manager_memory,
};
pub use schema::{SchemaMetadata, SchemaMetadataError};
pub use session::{
    AllocationSession, AllocationSessionError, CommittedAllocations, ValidatedAllocations,
};
pub use slot::{
    AllocationSlot, AllocationSlotDescriptor, IC_MEMORY_AUTHORITY_OWNER,
    IC_MEMORY_AUTHORITY_PURPOSE, IC_MEMORY_LEDGER_LABEL, IC_MEMORY_LEDGER_STABLE_KEY,
    IC_MEMORY_STABLE_KEY_PREFIX, MEMORY_MANAGER_GOVERNANCE_MAX_ID, MEMORY_MANAGER_INVALID_ID,
    MEMORY_MANAGER_LEDGER_ID, MEMORY_MANAGER_MAX_ID, MEMORY_MANAGER_MIN_ID,
    MemoryManagerAuthorityRecord, MemoryManagerIdRange, MemoryManagerRangeAuthority,
    MemoryManagerRangeAuthorityError, MemoryManagerRangeError, MemoryManagerRangeMode,
    MemoryManagerSlotError, is_ic_memory_stable_key, memory_manager_governance_range,
    validate_memory_manager_id,
};
pub use stable_cell::{
    STABLE_CELL_HEADER_SIZE, STABLE_CELL_LAYOUT_VERSION, STABLE_CELL_MAGIC,
    STABLE_CELL_VALUE_OFFSET, StableCellLedgerError, StableCellLedgerRecord,
    StableCellPayloadError, decode_stable_cell_ledger_record, decode_stable_cell_payload,
    validate_stable_cell_ledger_memory,
};
pub use substrate::{LedgerAnchor, StorageSubstrate};
pub use validation::{AllocationValidationError, Validate, validate_allocations};

#[doc(hidden)]
pub use runtime::defer_eager_init;

#[doc(hidden)]
pub mod __reexports {
    pub use ctor;
}

/// Register a `MemoryManager` allocation declaration during static initialization.
///
/// The explicit authority is stable policy identity shared with the matching
/// range declaration. Internal `ic-memory` authority is unavailable to callers.
///
/// This macro only registers declaration metadata. It does not open stable
/// memory. The bootstrap owner still has to collect/seal declarations, validate
/// them against the ledger, commit the generation, and then open memory handles.
#[macro_export]
macro_rules! ic_memory_declaration {
    (authority = $authority:literal, key = $stable_key:literal, ty = $label:path, id = $id:expr $(,)?) => {
        const _: () = {
            #[ $crate::__reexports::ctor::ctor(unsafe, anonymous, crate_path = $crate::__reexports::ctor) ]
            fn __ic_memory_register_static_declaration() {
                let _ = core::marker::PhantomData::<$label>;
                $crate::register_static_memory_manager_declaration(
                    $id,
                    $authority,
                    stringify!($label),
                    $stable_key,
                )
                .expect("ic-memory static memory declaration failed");
            }
        };
    };
    (authority = $authority:literal, key = $stable_key:literal, label = $label:literal, id = $id:expr $(,)?) => {
        const _: () = {
            #[ $crate::__reexports::ctor::ctor(unsafe, anonymous, crate_path = $crate::__reexports::ctor) ]
            fn __ic_memory_register_static_declaration() {
                $crate::register_static_memory_manager_declaration(
                    $id,
                    $authority,
                    $label,
                    $stable_key,
                )
                .expect("ic-memory static memory declaration failed");
            }
        };
    };
}

/// Declare a `MemoryManager` allocation range during static initialization.
///
/// The explicit authority must match every declaration that uses this range.
#[macro_export]
macro_rules! ic_memory_range {
    (authority = $authority:literal, start = $start:expr, end = $end:expr $(,)?) => {
        $crate::ic_memory_range!(
            authority = $authority,
            start = $start,
            end = $end,
            mode = Reserved,
        );
    };
    (authority = $authority:literal, start = $start:expr, end = $end:expr, mode = $mode:ident $(,)?) => {
        const _: () = {
            #[ $crate::__reexports::ctor::ctor(unsafe, anonymous, crate_path = $crate::__reexports::ctor) ]
            fn __ic_memory_register_static_range() {
                $crate::register_static_memory_manager_range(
                    $start,
                    $end,
                    $authority,
                    $crate::MemoryManagerRangeMode::$mode,
                    None,
                )
                .expect("ic-memory static memory range declaration failed");
            }
        };
    };
}

/// Declare and open a committed `MemoryManager` slot by stable key.
///
/// The macro registers declaration metadata during static initialization and
/// returns the committed default-runtime memory handle at expression use time.
#[macro_export]
macro_rules! ic_memory_key {
    (authority = $authority:literal, key = $stable_key:literal, ty = $label:path, id = $id:expr $(,)?) => {{
        $crate::ic_memory_declaration!(authority = $authority, key = $stable_key, ty = $label, id = $id,);
        $crate::open_default_memory_manager_memory($stable_key, $id)
            .expect("ic-memory failed to open committed stable memory; bootstrap must run first and the stable key/id must match the committed declaration")
    }};
    (authority = $authority:literal, key = $stable_key:literal, label = $label:literal, id = $id:expr $(,)?) => {{
        $crate::ic_memory_declaration!(authority = $authority, key = $stable_key, label = $label, id = $id,);
        $crate::open_default_memory_manager_memory($stable_key, $id)
            .expect("ic-memory failed to open committed stable memory; bootstrap must run first and the stable key/id must match the committed declaration")
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
