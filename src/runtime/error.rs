use crate::{
    LedgerCommitError, PolicyIdentity, PolicyIdentityError, StableCellLedgerError,
    registry::StaticMemoryDeclarationError,
    slot::{MemoryManagerRangeAuthorityError, MemoryManagerSlotError},
};

///
/// RuntimeConstructionError
///
/// Failure to construct a memory runtime without overwriting unrecognized
/// backing memory.
///

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, thiserror::Error, PartialEq)]
pub enum RuntimeConstructionError {
    /// Zero pages cannot form a bucket.
    #[error("bucket size must be nonzero")]
    InvalidBucketSize,
    /// Explicit policy differs from the actual durable setting.
    #[error("persisted bucket size {persisted} pages differs from requested {requested}")]
    BucketSizeMismatch { persisted: u16, requested: u16 },
    /// Persisted manager metadata failed bounded validation.
    #[error(transparent)]
    Layout(#[from] super::MemoryManagerLayoutError),
    /// Nonempty backing memory does not contain a `MemoryManager` header.
    #[error(
        "nonempty backing memory is not an ic-stable-structures MemoryManager \
         (expected magic 'MGR', found bytes {observed_magic:?})"
    )]
    ForeignMemory {
        /// First three bytes found in the nonempty backing memory.
        observed_magic: [u8; 3],
    },
    /// Backing memory contains an unsupported `MemoryManager` layout version.
    #[error(
        "unsupported ic-stable-structures MemoryManager layout version {observed}; \
         expected {supported}"
    )]
    UnsupportedMemoryManagerVersion {
        /// Version byte found after the `MemoryManager` magic.
        observed: u8,
        /// Version supported by the pinned `ic-stable-structures` dependency.
        supported: u8,
    },
}

///
/// RuntimeStateError
///
/// Failure to enter or maintain one memory runtime's in-memory lifecycle.
///

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, thiserror::Error, PartialEq)]
pub enum RuntimeStateError {
    /// This thread's default runtime could not safely claim its backing memory.
    #[error(transparent)]
    Construction(#[from] RuntimeConstructionError),
    /// A default-runtime operation re-entered while that TLS runtime was borrowed.
    #[error("ic-memory default runtime is already borrowed by an active operation")]
    ReentrantAccess,
    /// The thread-local default runtime is being destroyed and cannot be entered.
    #[error("ic-memory default runtime is unavailable during thread-local destruction")]
    Unavailable,
    /// Internal runtime lifecycle state was inconsistent.
    #[error("ic-memory runtime lifecycle is internally inconsistent")]
    InconsistentLifecycle,
}

///
/// RuntimeBootstrapError
///
/// Failure to bootstrap one `MemoryRuntime`.
///

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum RuntimeBootstrapError<P> {
    #[error(transparent)]
    Resolution(#[from] MemoryResolutionError),
    /// The policy did not provide a valid bounded semantic identity.
    #[error(transparent)]
    PolicyIdentity(#[from] PolicyIdentityError),
    /// A bootstrapped runtime was called with a different declaration snapshot.
    #[error("runtime bootstrap declaration snapshot differs from the established binding")]
    DeclarationSnapshotMismatch,
    /// A bootstrapped runtime was called with a different policy identity.
    #[error("runtime bootstrap policy identity changed from {established:?} to {requested:?}")]
    PolicyIdentityMismatch {
        /// Policy identity established by successful bootstrap.
        established: PolicyIdentity,
        /// Policy identity supplied by the repeated call.
        requested: PolicyIdentity,
    },
    /// Linked-program declaration snapshot sealing failed.
    #[error(transparent)]
    Registry(#[from] StaticMemoryDeclarationError),
    /// Runtime ledger genesis construction failed.
    #[error(transparent)]
    LedgerIntegrity(#[from] crate::LedgerIntegrityError),
    /// Protected ledger recovery or commit failed.
    #[error(transparent)]
    LedgerCommit(#[from] crate::LedgerCommitError),
    /// Stable-cell ledger storage is corrupt before protected recovery can run.
    #[error(transparent)]
    StableCellLedger(#[from] StableCellLedgerError),
    /// Stable-cell ledger storage cannot fit the next protected ledger record.
    #[error("stable-cell ledger record size {value_size} cannot be written to stable memory")]
    StableCellLedgerWriteTooLarge {
        /// Encoded stable-cell ledger record size in bytes.
        value_size: usize,
    },
    /// Declaration validation failed.
    #[error(transparent)]
    Validation(#[from] crate::AllocationValidationError<RuntimePolicyError<P>>),
    /// Validated declarations could not be staged.
    #[error(transparent)]
    Staging(#[from] crate::AllocationStageError),
    /// Runtime lifecycle or default TLS access failed.
    #[error(transparent)]
    State(#[from] RuntimeStateError),
}

///
/// RuntimeOpenError
///
/// Failure to open an allocation through one memory runtime.
///

#[non_exhaustive]
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum RuntimeOpenError {
    /// This runtime has not published committed allocations.
    #[error("ic-memory runtime has not completed bootstrap validation")]
    NotBootstrapped,
    /// Runtime lifecycle or default TLS access failed.
    #[error(transparent)]
    State(#[from] RuntimeStateError),
    /// Stable-key grammar failure.
    #[error(transparent)]
    StableKey(#[from] crate::StableKeyError),
    /// The stable key was not present in this runtime's committed declaration set.
    #[error("stable key '{0}' was not committed by ic-memory runtime bootstrap")]
    StableKeyNotCommitted(String),
    /// Runtime governance stable keys are internal and cannot be opened publicly.
    #[error("stable key '{stable_key}' is reserved for ic-memory runtime governance")]
    ReservedStableKey {
        /// Reserved stable key.
        stable_key: String,
    },
    /// The committed slot is not a usable `MemoryManager` ID.
    #[error(transparent)]
    MemoryManagerSlot(#[from] MemoryManagerSlotError),
    /// The requested memory ID does not match the committed stable-key binding.
    #[error(
        "stable key '{stable_key}' is committed for MemoryManager ID {committed_id}, not requested ID {requested_id}"
    )]
    MemoryIdMismatch {
        /// Stable key being opened.
        stable_key: String,
        /// Committed MemoryManager ID.
        committed_id: u8,
        /// Requested MemoryManager ID.
        requested_id: u8,
    },
}

///
/// RuntimeDiagnosticError
///
/// Failure to build diagnostics for one memory runtime.
///

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum RuntimeDiagnosticError {
    /// Persisted manager metadata is invalid or unsupported.
    #[error(transparent)]
    Construction(#[from] RuntimeConstructionError),
    /// Current binding metadata exceeds the fixed usable ID domain.
    #[error("allocation bindings exceed the bounded manager domain")]
    AllocationBound,
    /// This runtime has not opened and validated its ledger cell.
    #[error("ic-memory runtime has not completed bootstrap validation")]
    NotBootstrapped,
    /// Linked-program declaration snapshot sealing failed.
    #[error(transparent)]
    Registry(#[from] StaticMemoryDeclarationError),
    /// Runtime lifecycle or default TLS access failed.
    #[error(transparent)]
    State(#[from] RuntimeStateError),
    /// The recovered allocation ledger failed protected commit validation.
    #[error(transparent)]
    LedgerCommit(#[from] LedgerCommitError),
    /// Stable-cell ledger storage is corrupt before protected recovery can run.
    #[error(transparent)]
    StableCellLedger(#[from] StableCellLedgerError),
    /// A committed allocation slot was not a usable `MemoryManager` ID.
    #[error(transparent)]
    MemoryManagerSlot(#[from] MemoryManagerSlotError),
}

///
/// RuntimePolicyError
///
/// Failure in generic runtime range policy or caller-supplied policy.
///

#[non_exhaustive]
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum RuntimePolicyError<P> {
    /// Runtime range authority rejected the declaration.
    #[error(transparent)]
    Range(#[from] MemoryManagerRangeAuthorityError),
    /// Runtime metadata is internally inconsistent.
    #[error("runtime declaration metadata is missing for stable key '{0}'")]
    MissingDeclarationMetadata(String),
    /// `ic_memory.*` stable keys are reserved to the `ic-memory` authority.
    #[error("stable key '{stable_key}' is reserved to authority '{expected_authority}'")]
    ReservedStableKeyAuthority {
        /// Stable key being declared.
        stable_key: String,
        /// Required declaring authority.
        expected_authority: &'static str,
    },
    /// Caller-supplied policy rejected the declaration.
    #[error(transparent)]
    Custom(P),
}

///
/// MemoryResolutionError
///
/// Logical placement failed before publishing allocation authority.
///

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum MemoryResolutionError {
    #[error("no eligible free slot for {stable_key} under authority {authority}")]
    Exhausted {
        stable_key: crate::StableKey,
        authority: String,
    },
    #[error(transparent)]
    Range(#[from] crate::MemoryManagerRangeAuthorityError),
    #[error(transparent)]
    Registry(#[from] StaticMemoryDeclarationError),
    #[error(transparent)]
    Declaration(#[from] crate::DeclarationSnapshotError),
}
