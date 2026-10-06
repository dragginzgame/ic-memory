use crate::{
    key::{StableKey, StableKeyError},
    ledger::LedgerPayloadEnvelopeError,
    physical::CommitRecoveryError,
    slot::MemoryManagerSlot,
};

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
    /// Logical ledger payload envelope could not be decoded.
    #[error(transparent)]
    PayloadEnvelope(LedgerPayloadEnvelopeError),
    /// Physical slot generation and decoded logical ledger generation disagree.
    #[error(
        "physical generation {physical_generation} does not match logical ledger generation {logical_generation}"
    )]
    PhysicalLogicalGenerationMismatch {
        /// Generation encoded in the physical commit slot.
        physical_generation: u64,
        /// Generation decoded from the logical allocation ledger.
        logical_generation: u64,
    },
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
    /// Validated declarations were produced against a different ledger generation.
    #[error(
        "validated allocations were produced at generation {validated_generation}, but ledger is at generation {ledger_generation}"
    )]
    StaleValidatedAllocations {
        /// Generation carried by the validated allocation capability.
        validated_generation: u64,
        /// Current ledger generation.
        ledger_generation: u64,
    },
    /// Stable key was historically bound to a different slot.
    #[error("stable key '{stable_key}' was historically bound to a different allocation slot")]
    StableKeySlotConflict {
        /// Stable key being declared.
        stable_key: StableKey,
        /// Historical slot for the stable key.
        historical_slot: MemoryManagerSlot,
        /// Slot claimed by the declaration.
        declared_slot: MemoryManagerSlot,
    },
    /// Slot was historically bound to a different stable key.
    #[error("allocation slot '{slot:?}' was historically bound to stable key '{historical_key}'")]
    SlotStableKeyConflict {
        /// Slot being declared.
        slot: MemoryManagerSlot,
        /// Historical stable key for the slot.
        historical_key: StableKey,
        /// Stable key claimed by the declaration.
        declared_key: StableKey,
    },
    /// Current declaration attempted to revive a retired allocation.
    #[error("stable key '{stable_key}' was explicitly retired and cannot be redeclared")]
    RetiredAllocation {
        /// Retired stable key.
        stable_key: StableKey,
        /// Retired allocation slot.
        slot: MemoryManagerSlot,
    },
}

///
/// AllocationReservationError
///
/// Failure to stage a reservation generation.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum AllocationReservationError {
    #[error(transparent)]
    Integrity(#[from] LedgerIntegrityError),
    /// Ledger generation cannot be advanced without overflow.
    #[error("ledger generation {generation} cannot be advanced without overflow")]
    GenerationOverflow {
        /// Current ledger generation.
        generation: u64,
    },
    /// Reservation count exceeds the usable allocation-slot domain.
    #[error("generation contains {count} reservations, exceeding the 255-allocation limit")]
    TooManyReservations {
        /// Number of reservations in the staged generation.
        count: usize,
    },
    /// Stable key was historically bound to a different slot.
    #[error("stable key '{stable_key}' was historically bound to a different allocation slot")]
    StableKeySlotConflict {
        /// Stable key being reserved.
        stable_key: StableKey,
        /// Historical slot for the stable key.
        historical_slot: MemoryManagerSlot,
        /// Slot claimed by the reservation.
        reserved_slot: MemoryManagerSlot,
    },
    /// Slot was historically bound to a different stable key.
    #[error("allocation slot '{slot:?}' was historically bound to stable key '{historical_key}'")]
    SlotStableKeyConflict {
        /// Slot being reserved.
        slot: MemoryManagerSlot,
        /// Historical stable key for the slot.
        historical_key: StableKey,
        /// Stable key claimed by the reservation.
        reserved_key: StableKey,
    },
    /// Allocation already exists as an active record.
    #[error("stable key '{stable_key}' is already active and cannot be reserved")]
    ActiveAllocation {
        /// Active stable key.
        stable_key: StableKey,
        /// Active allocation slot.
        slot: MemoryManagerSlot,
    },
    /// Allocation was already retired and cannot be reserved.
    #[error("stable key '{stable_key}' was explicitly retired and cannot be reserved")]
    RetiredAllocation {
        /// Retired stable key.
        stable_key: StableKey,
        /// Retired allocation slot.
        slot: MemoryManagerSlot,
    },
}

///
/// AllocationRetirementError
///
/// Failure to stage an explicit retirement generation.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum AllocationRetirementError {
    #[error(transparent)]
    Integrity(#[from] LedgerIntegrityError),
    /// Stable-key grammar failure.
    #[error(transparent)]
    Key(StableKeyError),
    /// Ledger generation cannot be advanced without overflow.
    #[error("ledger generation {generation} cannot be advanced without overflow")]
    GenerationOverflow {
        /// Current ledger generation.
        generation: u64,
    },
    /// Stable key has no historical allocation record.
    #[error("stable key '{0}' has no allocation record to retire")]
    UnknownStableKey(StableKey),
    /// Stable key was historically bound to a different slot.
    #[error("stable key '{stable_key}' cannot be retired for a different allocation slot")]
    SlotMismatch {
        /// Stable key being retired.
        stable_key: StableKey,
        /// Historical slot for the stable key.
        historical_slot: MemoryManagerSlot,
        /// Slot named by the retirement request.
        retired_slot: MemoryManagerSlot,
    },
    /// Allocation was already retired.
    #[error("stable key '{stable_key}' was already retired")]
    AlreadyRetired {
        /// Retired stable key.
        stable_key: StableKey,
        /// Retired allocation slot.
        slot: MemoryManagerSlot,
    },
}
