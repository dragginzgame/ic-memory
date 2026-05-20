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
//! 5. Only then open stable-memory handles through a validated allocation
//!    session.
//!
//! This crate owns allocation invariants, not framework policy. Namespace
//! rules, range ownership, controller authorization, endpoint lifecycle, schema
//! migrations, and application validation belong to the framework or
//! application.
//!
//! Use these primitives before opening stable-memory handles. Integrations
//! should recover the historical ledger, declare the stores expected by the
//! current binary, validate declarations against history and policy, commit a
//! new generation, and only then publish a validated allocation session that can
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
//! physical slot substrate. The crate still keeps narrow internal abstractions
//! for storage adapters and diagnostics, but the native IC path is
//! `MemoryManager` ID 0 -> `ic-stable-structures::Cell<StableCellLedgerRecord,
//! _>` -> [`LedgerCommitStore`] -> committed [`AllocationLedger`] payloads.
//!
//! `ic-memory` is not a replacement for `ic-stable-structures` collections and
//! does not wrap typed stores such as `StableBTreeMap`.

pub mod bootstrap;
pub mod declaration;
pub mod diagnostics;
pub mod key;
pub mod ledger;
pub mod physical;
pub mod policy;
pub mod registry;
pub mod runtime;
pub mod schema;
pub mod session;
pub mod slot;
pub mod stable_cell;
pub mod substrate;
pub mod validation;

pub use ic_stable_structures as stable_structures;

pub use bootstrap::{
    AllocationBootstrap, BootstrapCommit, BootstrapError, BootstrapReservationError,
    BootstrapRetirementError,
};
pub use declaration::{
    AllocationDeclaration, DeclarationCollector, DeclarationSnapshot, DeclarationSnapshotError,
};
pub use diagnostics::{DiagnosticExport, DiagnosticGeneration, DiagnosticRecord};
pub use key::{StableKey, StableKeyError};
pub use ledger::{
    AllocationHistory, AllocationLedger, AllocationRecord, AllocationReservationError,
    AllocationRetirement, AllocationRetirementError, AllocationStageError, AllocationState,
    CURRENT_LEDGER_SCHEMA_VERSION, CURRENT_PHYSICAL_FORMAT_ID, CborLedgerCodec, GenerationRecord,
    LedgerCodec, LedgerCommitError, LedgerCommitStore, LedgerCompatibility,
    LedgerCompatibilityError, LedgerIntegrityError, SchemaMetadataRecord,
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
    RuntimeBootstrapError, RuntimeOpenError, RuntimePolicyError, bootstrap_default_memory_manager,
    bootstrap_default_memory_manager_with_policy,
};
pub use schema::{SchemaMetadata, SchemaMetadataError};
pub use session::{AllocationSession, AllocationSessionError, ValidatedAllocations};
pub use slot::{
    AllocationSlot, AllocationSlotDescriptor, IC_MEMORY_AUTHORITY_OWNER,
    IC_MEMORY_AUTHORITY_PURPOSE, IC_MEMORY_LEDGER_LABEL, IC_MEMORY_LEDGER_STABLE_KEY,
    IC_MEMORY_STABLE_KEY_PREFIX, MEMORY_MANAGER_DESCRIPTOR_VERSION,
    MEMORY_MANAGER_GOVERNANCE_MAX_ID, MEMORY_MANAGER_INVALID_ID, MEMORY_MANAGER_LEDGER_ID,
    MEMORY_MANAGER_MAX_ID, MEMORY_MANAGER_MIN_ID, MEMORY_MANAGER_SUBSTRATE,
    MemoryManagerAuthorityRecord, MemoryManagerIdRange, MemoryManagerRangeAuthority,
    MemoryManagerRangeAuthorityError, MemoryManagerRangeError, MemoryManagerRangeMode,
    MemoryManagerSlotError, is_ic_memory_stable_key, memory_manager_governance_range,
    validate_memory_manager_id,
};
pub use stable_cell::{
    STABLE_CELL_HEADER_SIZE, STABLE_CELL_LAYOUT_VERSION, STABLE_CELL_MAGIC,
    STABLE_CELL_VALUE_OFFSET, StableCellLedgerRecord, StableCellPayloadError,
    decode_stable_cell_ledger_record, decode_stable_cell_payload,
};
pub use substrate::{LedgerAnchor, StorageSubstrate};
pub use validation::{AllocationValidationError, validate_allocations};

#[doc(hidden)]
pub mod __reexports {
    pub use ctor;
}

/// Register a `MemoryManager` allocation declaration during static initialization.
///
/// This macro only registers declaration metadata. It does not open stable
/// memory. The bootstrap owner still has to collect/seal declarations, validate
/// them against the ledger, commit the generation, and then open memory handles.
#[macro_export]
macro_rules! ic_memory_declaration {
    (key = $stable_key:literal, ty = $label:path, id = $id:expr $(,)?) => {
        const _: () = {
            #[allow(dead_code)]
            type IcMemoryTypeCheck = $label;

            #[ $crate::__reexports::ctor::ctor(unsafe, anonymous, crate_path = $crate::__reexports::ctor) ]
            fn __ic_memory_register_static_declaration() {
                $crate::register_static_memory_manager_declaration(
                    $id,
                    env!("CARGO_PKG_NAME"),
                    stringify!($label),
                    $stable_key,
                )
                .expect("ic-memory static memory declaration failed");
            }
        };
    };
    (key = $stable_key:literal, label = $label:literal, id = $id:expr $(,)?) => {
        const _: () = {
            #[ $crate::__reexports::ctor::ctor(unsafe, anonymous, crate_path = $crate::__reexports::ctor) ]
            fn __ic_memory_register_static_declaration() {
                $crate::register_static_memory_manager_declaration(
                    $id,
                    env!("CARGO_PKG_NAME"),
                    $label,
                    $stable_key,
                )
                .expect("ic-memory static memory declaration failed");
            }
        };
    };
}

/// Declare a `MemoryManager` allocation range during static initialization.
#[macro_export]
macro_rules! ic_memory_range {
    (start = $start:expr, end = $end:expr $(,)?) => {
        $crate::ic_memory_range!(
            start = $start,
            end = $end,
            mode = Reserved,
        );
    };
    (start = $start:expr, end = $end:expr, mode = $mode:ident $(,)?) => {
        const _: () = {
            #[ $crate::__reexports::ctor::ctor(unsafe, anonymous, crate_path = $crate::__reexports::ctor) ]
            fn __ic_memory_register_static_range() {
                $crate::register_static_memory_manager_range(
                    $start,
                    $end,
                    env!("CARGO_PKG_NAME"),
                    $crate::MemoryManagerRangeMode::$mode,
                    None,
                )
                .expect("ic-memory static memory range declaration failed");
            }
        };
    };
}

/// Declare and open a validated `MemoryManager` slot by stable key.
///
/// The macro registers declaration metadata during static initialization and
/// returns the validated default-runtime memory handle at expression use time.
#[macro_export]
macro_rules! ic_memory_key {
    ($stable_key:literal, $label:path, $id:expr $(,)?) => {{
        $crate::ic_memory_declaration!(key = $stable_key, ty = $label, id = $id,);
        $crate::runtime::open_default_memory_manager_memory($stable_key, $id)
            .expect("ic-memory stable memory opened before runtime bootstrap")
    }};
    (key = $stable_key:literal, ty = $label:path, id = $id:expr $(,)?) => {{ $crate::ic_memory_key!($stable_key, $label, $id) }};
    (key = $stable_key:literal, label = $label:literal, id = $id:expr $(,)?) => {{
        $crate::ic_memory_declaration!(key = $stable_key, label = $label, id = $id,);
        $crate::runtime::open_default_memory_manager_memory($stable_key, $id)
            .expect("ic-memory stable memory opened before runtime bootstrap")
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
                $crate::runtime::defer_eager_init(__ic_memory_registered_eager_init_body);
            }
        };
    };
}
