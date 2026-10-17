use crate::{key::StableKey, physical::CommitRecoveryError, slot::MemoryManagerSlot};

///
/// LedgerIntegrityError
///
/// Decoded ledger violates current ownership invariants.
///

#[non_exhaustive]
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum LedgerIntegrityError {
    /// Current structural recovery ceiling exceeded.
    #[error("ledger exceeds {resource} limit {limit}")]
    LimitExceeded {
        resource: &'static str,
        limit: usize,
    },
    /// Stable key appears in more than one retained allocation record.
    #[error("stable key '{stable_key}' appears in more than one allocation record")]
    DuplicateStableKey { stable_key: StableKey },
    /// Memory ID appears in more than one retained allocation record.
    #[error("allocation slot '{slot:?}' appears in more than one allocation record")]
    DuplicateSlot { slot: MemoryManagerSlot },
    /// Commit counter zero is reserved for empty genesis.
    #[error("genesis ledger contains allocation records")]
    NonemptyGenesis,
}

///
/// LedgerCommitError
///
/// Failure to recover or commit a logical allocation ledger.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum LedgerCommitError {
    /// Protected physical commit recovery failed.
    #[error(transparent)]
    Recovery(CommitRecoveryError),
    /// Built-in ledger decoding failed.
    #[error("allocation ledger codec failed: {0}")]
    Codec(String),
    /// Decoded ledger violates current ownership invariants.
    #[error(transparent)]
    Integrity(LedgerIntegrityError),
}

///
/// AllocationStageError
///
/// Failure to stage a validated allocation generation.
///

#[non_exhaustive]
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum AllocationStageError {
    /// The commit counter cannot advance without overflow.
    #[error("ledger generation {generation} cannot be advanced without overflow")]
    GenerationOverflow { generation: u64 },
    #[error(transparent)]
    Integrity(#[from] LedgerIntegrityError),
}
