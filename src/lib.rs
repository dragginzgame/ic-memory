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
//! The APIs are generic over storage substrates. `ic-stable-structures`
//! `MemoryManager` IDs are supported as durable slot descriptors, but this crate
//! is not a replacement for `ic-stable-structures` and is not Canic-specific.

pub mod bootstrap;
pub mod declaration;
pub mod diagnostics;
pub mod key;
pub mod ledger;
pub mod physical;
pub mod policy;
pub mod schema;
pub mod session;
pub mod slot;
pub mod substrate;
pub mod validation;

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
    CURRENT_LEDGER_SCHEMA_VERSION, CURRENT_PHYSICAL_FORMAT_ID, GenerationRecord, LedgerCodec,
    LedgerCommitError, LedgerCommitStore, LedgerCompatibility, LedgerCompatibilityError,
    LedgerIntegrityError, SchemaMetadataRecord,
};
pub use physical::{
    AuthoritativeSlot, CommitRecoveryError, CommitSlotDiagnostic, CommitSlotIndex,
    CommitStoreDiagnostic, CommittedGenerationBytes, DualCommitStore, DualProtectedCommitStore,
    ProtectedGenerationSlot, select_authoritative_slot,
};
pub use policy::{AllocationPolicy, NamespaceAuthority, RangeAuthority};
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
pub use substrate::{LedgerAnchor, StorageSubstrate};
pub use validation::{AllocationValidationError, validate_allocations};
