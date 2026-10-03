use crate::{
    LedgerCommitError, PolicyIdentity, PolicyIdentityError, StableCellLedgerError,
    registry::StaticMemoryDeclarationError, slot::MemoryManagerRangeAuthorityError,
};

///
/// RuntimeGrowError
///
/// Failure to grow a runtime memory before assigning new manager buckets.
/// Ordinary capacity failures preserve virtual extents and manager metadata.
/// Backing traps and partial writes remain outside this guarantee.
///

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, thiserror::Error, PartialEq)]
pub enum RuntimeGrowError {
    /// The requested virtual extent overflows the page count.
    #[error("virtual memory page count overflows")]
    ArithmeticOverflow,
    /// The sole manager has insufficient bucket slots.
    #[error("growth requires {required_buckets} buckets, exceeding capacity {capacity}")]
    BucketExhausted {
        required_buckets: u64,
        capacity: u16,
    },
    /// The backing memory refused the physical capacity reservation.
    #[error("backing memory refused growth by {additional_pages} pages")]
    BackingRefused { additional_pages: u64 },
    /// Growth re-entered while another handle held a capacity reservation.
    #[error("runtime memory growth is already in progress")]
    ReentrantAccess,
    /// The manager refused growth despite the runtime's successful preflight.
    #[error("memory manager refused preflighted growth")]
    ManagerRefused,
}

///
/// RuntimeConstructionError
///
/// Failure to construct a memory runtime without overwriting unrecognized
/// backing memory.
///

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, thiserror::Error, PartialEq)]
pub enum RuntimeConstructionError {
    /// Fresh manager metadata could not reserve physical capacity.
    #[error(transparent)]
    Growth(#[from] RuntimeGrowError),
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
}

///
/// RuntimeBootstrapError
///
/// Failure to bootstrap one `MemoryRuntime`.
///

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum RuntimeBootstrapError<P> {
    /// A known-only historical selection failed before commitment.
    #[error(transparent)]
    Admission(#[from] super::BootstrapAdmissionError),
    /// Consumer identity or key-set admission rejected this attempt.
    #[error("bootstrap admission policy rejected recovered allocation metadata")]
    AdmissionPolicy(P),
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
    /// Protected ledger recovery or commit failed.
    #[error(transparent)]
    LedgerCommit(#[from] crate::LedgerCommitError),
    /// Stable-cell ledger storage is corrupt before protected recovery can run.
    #[error(transparent)]
    StableCellLedger(#[from] StableCellLedgerError),
    /// The encoded stable-cell ledger record exceeds its bounded size ceiling.
    #[error("stable-cell ledger record size {value_size} cannot be written to stable memory")]
    StableCellLedgerWriteTooLarge {
        /// Encoded stable-cell ledger record size in bytes.
        value_size: usize,
    },
    /// Stable-cell ledger capacity reservation failed before commitment.
    #[error(transparent)]
    LedgerGrowth(#[from] RuntimeGrowError),
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
    /// No default runtime exists, or this operation requires completed bootstrap.
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
