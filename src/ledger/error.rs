use crate::{
    declaration::DeclarationSnapshotError,
    key::{StableKey, StableKeyError},
    ledger::LedgerPayloadEnvelopeError,
    physical::CommitRecoveryError,
    schema::SchemaMetadataError,
    slot::{AllocationSlotDescriptor, AllocationSlotDescriptorError},
};

///
/// LedgerCompatibilityError
///
/// Decoded ledger format is unsupported by this reader.
#[derive(Clone, Copy, Debug, Eq, thiserror::Error, PartialEq)]
pub enum LedgerCompatibilityError {
    /// Ledger schema version is outside the supported range.
    #[error(
        "ledger_schema_version {found} is unsupported; supported range is {min_supported}-{max_supported}"
    )]
    UnsupportedLedgerSchemaVersion {
        /// Version found in the ledger.
        found: u32,
        /// Minimum supported version.
        min_supported: u32,
        /// Maximum supported version.
        max_supported: u32,
    },
    /// Physical format ID is not supported by this reader.
    #[error("physical_format_id {found} is unsupported; supported format is {supported}")]
    UnsupportedPhysicalFormat {
        /// Format found in the ledger.
        found: u32,
        /// Supported format ID.
        supported: u32,
    },
}

///
/// LedgerIntegrityError
///
/// Decoded ledger violates structural allocation-history invariants.
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum LedgerIntegrityError {
    /// Stable-key grammar was invalid after durable decode.
    #[error(transparent)]
    InvalidStableKey(StableKeyError),
    /// Allocation slot descriptor was invalid after durable decode.
    #[error(transparent)]
    InvalidSlotDescriptor(AllocationSlotDescriptorError),
    /// Stable key appears in more than one allocation record.
    #[error("stable key '{stable_key}' appears in more than one allocation record")]
    DuplicateStableKey {
        /// Duplicate stable key.
        stable_key: StableKey,
    },
    /// Allocation slot appears in more than one allocation record.
    #[error("allocation slot '{slot:?}' appears in more than one allocation record")]
    DuplicateSlot {
        /// Duplicate allocation slot.
        slot: Box<AllocationSlotDescriptor>,
    },
    /// Allocation record generation ordering is invalid.
    #[error("stable key '{stable_key}' has first_generation after last_seen_generation")]
    InvalidRecordGenerationOrder {
        /// Stable key whose record is invalid.
        stable_key: StableKey,
        /// First generation in the record.
        first_generation: u64,
        /// Last seen generation in the record.
        last_seen_generation: u64,
    },
    /// Allocation record points past the current generation.
    #[error(
        "stable key '{stable_key}' references generation {generation} after current generation {current_generation}"
    )]
    FutureRecordGeneration {
        /// Stable key whose record is invalid.
        stable_key: StableKey,
        /// Generation referenced by the record.
        generation: u64,
        /// Current ledger generation.
        current_generation: u64,
    },
    /// Non-retired allocation carries retired metadata.
    #[error("stable key '{stable_key}' is not retired but has retired_generation metadata")]
    UnexpectedRetiredGeneration {
        /// Stable key whose record is invalid.
        stable_key: StableKey,
    },
    /// Retired allocation is missing retired metadata.
    #[error("stable key '{stable_key}' is retired but retired_generation is missing")]
    MissingRetiredGeneration {
        /// Stable key whose record is invalid.
        stable_key: StableKey,
    },
    /// Retired generation predates the allocation record.
    #[error("stable key '{stable_key}' has retired_generation before first_generation")]
    RetiredBeforeFirstGeneration {
        /// Stable key whose record is invalid.
        stable_key: StableKey,
        /// First generation in the record.
        first_generation: u64,
        /// Retired generation in the record.
        retired_generation: u64,
    },
    /// Allocation record has no schema metadata history.
    #[error("stable key '{stable_key}' has empty schema metadata history")]
    EmptySchemaHistory {
        /// Stable key whose record is invalid.
        stable_key: StableKey,
    },
    /// Schema metadata generation history is not strictly increasing.
    #[error("stable key '{stable_key}' has non-increasing schema metadata generation history")]
    NonIncreasingSchemaHistory {
        /// Stable key whose record is invalid.
        stable_key: StableKey,
    },
    /// Schema metadata generation is outside the allocation record lifetime.
    #[error("stable key '{stable_key}' has schema metadata generation outside the ledger bounds")]
    SchemaHistoryOutOfBounds {
        /// Stable key whose record is invalid.
        stable_key: StableKey,
        /// Schema metadata generation.
        generation: u64,
    },
    /// Schema metadata in committed allocation history is invalid.
    #[error("stable key '{stable_key}' has invalid schema metadata at generation {generation}")]
    InvalidSchemaMetadata {
        /// Stable key whose schema metadata is invalid.
        stable_key: StableKey,
        /// Generation that recorded the invalid schema metadata.
        generation: u64,
        /// Schema metadata validation error.
        error: SchemaMetadataError,
    },
    /// Generation record appears more than once.
    #[error("generation {generation} appears more than once")]
    DuplicateGeneration {
        /// Duplicate generation.
        generation: u64,
    },
    /// Generation record points past the current generation.
    #[error("generation {generation} is after current generation {current_generation}")]
    FutureGeneration {
        /// Generation record value.
        generation: u64,
        /// Current ledger generation.
        current_generation: u64,
    },
    /// Generation parent does not precede the child generation.
    #[error("generation {generation} has invalid parent generation {parent_generation:?}")]
    InvalidParentGeneration {
        /// Generation record value.
        generation: u64,
        /// Invalid parent generation.
        parent_generation: Option<u64>,
    },
    /// Current ledger generation has no committed generation record.
    #[error("current generation {current_generation} has no committed generation record")]
    MissingCurrentGenerationRecord {
        /// Current ledger generation.
        current_generation: u64,
    },
    /// Generation records are not strictly increasing in durable order.
    #[error("generation records are not strictly increasing at generation {generation}")]
    NonIncreasingGenerationRecords {
        /// Non-increasing generation.
        generation: u64,
    },
    /// Generation record parent does not match the previous committed generation.
    #[error(
        "generation {generation} does not link to previous committed generation {expected_parent:?}"
    )]
    BrokenGenerationChain {
        /// Generation whose parent link is invalid.
        generation: u64,
        /// Expected parent generation.
        expected_parent: Option<u64>,
        /// Actual parent generation.
        actual_parent: Option<u64>,
    },
    /// Allocation record refers to a generation absent from committed history.
    #[error("stable key '{stable_key}' references unknown generation {generation}")]
    UnknownRecordGeneration {
        /// Stable key whose record is invalid.
        stable_key: StableKey,
        /// Unknown generation.
        generation: u64,
    },
    /// Generation diagnostic metadata is invalid.
    #[error(transparent)]
    DiagnosticMetadata(DeclarationSnapshotError),
}

///
/// LedgerCommitError
///
/// Failure to recover or commit a logical allocation ledger.
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum LedgerCommitError {
    /// Protected physical commit recovery failed.
    #[error(transparent)]
    Recovery(CommitRecoveryError),
    /// Logical ledger payload envelope could not be decoded.
    #[error(transparent)]
    PayloadEnvelope(LedgerPayloadEnvelopeError),
    /// Logical ledger payload envelope version is unsupported.
    #[error(
        "ledger payload envelope version {found} is unsupported; supported version is {supported}"
    )]
    UnsupportedEnvelopeVersion {
        /// Version found in the envelope.
        found: u16,
        /// Supported version.
        supported: u16,
    },
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
    /// Logical payload envelope and decoded ledger headers disagree.
    #[error("ledger payload envelope metadata does not match decoded ledger metadata")]
    PayloadEnvelopeLedgerMismatch {
        /// Schema version declared by the envelope.
        envelope_ledger_schema_version: u32,
        /// Schema version decoded from the ledger.
        ledger_schema_version: u32,
        /// Physical format ID declared by the envelope.
        envelope_physical_format_id: u32,
        /// Physical format ID decoded from the ledger.
        ledger_physical_format_id: u32,
    },
    /// Built-in ledger codec failed.
    #[error("allocation ledger codec failed")]
    Codec(String),
    /// Decoded ledger format is not compatible with this reader.
    #[error(transparent)]
    Compatibility(LedgerCompatibilityError),
    /// Decoded ledger violates structural allocation-history invariants.
    #[error(transparent)]
    Integrity(LedgerIntegrityError),
}

///
/// AllocationStageError
///
/// Failure to stage a validated allocation generation.
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum AllocationStageError {
    /// Validated declarations were produced against a different ledger generation.
    #[error(
        "validated allocations were produced at generation {validated_generation}, but ledger is at generation {ledger_generation}"
    )]
    StaleValidatedAllocations {
        /// Generation carried by the validated allocation session.
        validated_generation: u64,
        /// Current ledger generation.
        ledger_generation: u64,
    },
    /// Ledger generation cannot be advanced without overflow.
    #[error("ledger generation {generation} cannot be advanced without overflow")]
    GenerationOverflow {
        /// Current ledger generation.
        generation: u64,
    },
    /// Declaration count does not fit in durable generation diagnostics.
    #[error("generation contains {count} declarations, exceeding the durable u32 diagnostic limit")]
    TooManyDeclarations {
        /// Number of declarations in the staged generation.
        count: usize,
    },
    /// A staged declaration carries invalid schema metadata.
    #[error("stable key '{stable_key}' has invalid schema metadata")]
    InvalidSchemaMetadata {
        /// Stable key whose schema metadata is invalid.
        stable_key: StableKey,
        /// Schema metadata validation error.
        error: SchemaMetadataError,
    },
    /// Stable key was historically bound to a different slot.
    #[error("stable key '{stable_key}' was historically bound to a different allocation slot")]
    StableKeySlotConflict {
        /// Stable key being declared.
        stable_key: StableKey,
        /// Historical slot for the stable key.
        historical_slot: Box<AllocationSlotDescriptor>,
        /// Slot claimed by the declaration.
        declared_slot: Box<AllocationSlotDescriptor>,
    },
    /// Slot was historically bound to a different stable key.
    #[error("allocation slot '{slot:?}' was historically bound to stable key '{historical_key}'")]
    SlotStableKeyConflict {
        /// Slot being declared.
        slot: Box<AllocationSlotDescriptor>,
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
        slot: Box<AllocationSlotDescriptor>,
    },
}

///
/// AllocationReservationError
///
/// Failure to stage a reservation generation.
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum AllocationReservationError {
    /// Ledger generation cannot be advanced without overflow.
    #[error("ledger generation {generation} cannot be advanced without overflow")]
    GenerationOverflow {
        /// Current ledger generation.
        generation: u64,
    },
    /// Declaration count does not fit in durable generation diagnostics.
    #[error("generation contains {count} reservations, exceeding the durable u32 diagnostic limit")]
    TooManyReservations {
        /// Number of reservations in the staged generation.
        count: usize,
    },
    /// A staged reservation carries invalid schema metadata.
    #[error("stable key '{stable_key}' has invalid schema metadata")]
    InvalidSchemaMetadata {
        /// Stable key whose schema metadata is invalid.
        stable_key: StableKey,
        /// Schema metadata validation error.
        error: SchemaMetadataError,
    },
    /// Stable key was historically bound to a different slot.
    #[error("stable key '{stable_key}' was historically bound to a different allocation slot")]
    StableKeySlotConflict {
        /// Stable key being reserved.
        stable_key: StableKey,
        /// Historical slot for the stable key.
        historical_slot: Box<AllocationSlotDescriptor>,
        /// Slot claimed by the reservation.
        reserved_slot: Box<AllocationSlotDescriptor>,
    },
    /// Slot was historically bound to a different stable key.
    #[error("allocation slot '{slot:?}' was historically bound to stable key '{historical_key}'")]
    SlotStableKeyConflict {
        /// Slot being reserved.
        slot: Box<AllocationSlotDescriptor>,
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
        slot: Box<AllocationSlotDescriptor>,
    },
    /// Allocation was already retired and cannot be reserved.
    #[error("stable key '{stable_key}' was explicitly retired and cannot be reserved")]
    RetiredAllocation {
        /// Retired stable key.
        stable_key: StableKey,
        /// Retired allocation slot.
        slot: Box<AllocationSlotDescriptor>,
    },
}

///
/// AllocationRetirementError
///
/// Failure to stage an explicit retirement generation.
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum AllocationRetirementError {
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
        historical_slot: Box<AllocationSlotDescriptor>,
        /// Slot named by the retirement request.
        retired_slot: Box<AllocationSlotDescriptor>,
    },
    /// Allocation was already retired.
    #[error("stable key '{stable_key}' was already retired")]
    AlreadyRetired {
        /// Retired stable key.
        stable_key: StableKey,
        /// Retired allocation slot.
        slot: Box<AllocationSlotDescriptor>,
    },
}
