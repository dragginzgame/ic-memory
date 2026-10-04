use super::range_authority::MemoryManagerIdRange;
use serde::{Deserialize, Serialize};

///
/// MemoryManagerSlot
///
/// A usable physical `ic-stable-structures::MemoryManager` allocation ID.
///
/// Construction and deserialization reject the unallocated-bucket sentinel
/// (255), so reading the ID is infallible. A slot is an identity, not authority
/// to open memory; that still requires committed allocation validation.
///
/// The private serde representation describes the current durable slot encoding.
/// It preserves that encoding independently of this checked in-memory type.
///

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(try_from = "SlotEncoding", into = "SlotEncoding")]
pub struct MemoryManagerSlot(u8);

impl MemoryManagerSlot {
    /// Construct a slot, rejecting ID 255.
    pub const fn new(id: u8) -> Result<Self, MemoryManagerSlotError> {
        match validate_memory_manager_id(id) {
            Ok(()) => Ok(Self(id)),
            Err(error) => Err(error),
        }
    }

    /// Return the usable virtual memory ID.
    #[must_use]
    pub const fn id(&self) -> u8 {
        self.0
    }
}

// Passive codec fields only: the current format nests the MemoryManagerId tag
// inside a slot field. These values never enter allocation policy or execution.
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SlotEncoding {
    slot: SlotIdEncoding,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
enum SlotIdEncoding {
    MemoryManagerId(u8),
}

impl From<MemoryManagerSlot> for SlotEncoding {
    fn from(slot: MemoryManagerSlot) -> Self {
        Self {
            slot: SlotIdEncoding::MemoryManagerId(slot.id()),
        }
    }
}

impl TryFrom<SlotEncoding> for MemoryManagerSlot {
    type Error = MemoryManagerSlotError;

    fn try_from(encoded: SlotEncoding) -> Result<Self, Self::Error> {
        let SlotIdEncoding::MemoryManagerId(id) = encoded.slot;
        Self::new(id)
    }
}

pub const LEDGER_SLOT: MemoryManagerSlot = match MemoryManagerSlot::new(MEMORY_MANAGER_LEDGER_ID) {
    Ok(slot) => slot,
    Err(_) => panic!("the ledger ID must be usable"),
};

/// First usable `MemoryManager` virtual memory ID.
pub const MEMORY_MANAGER_MIN_ID: u8 = 0;

/// Last usable `MemoryManager` virtual memory ID.
pub const MEMORY_MANAGER_MAX_ID: u8 = 254;

/// `MemoryManager` unallocated-bucket sentinel. This is not a usable slot.
pub const MEMORY_MANAGER_INVALID_ID: u8 = u8::MAX;

/// Stable-key namespace prefix reserved for `ic-memory` allocation-governance infrastructure.
pub const IC_MEMORY_STABLE_KEY_PREFIX: &str = "ic_memory.";

/// Diagnostic owner label for `ic-memory` allocation-governance infrastructure.
pub const IC_MEMORY_AUTHORITY_OWNER: &str = "ic-memory";

/// Diagnostic purpose for the `ic-memory` allocation-governance authority range.
pub const IC_MEMORY_AUTHORITY_PURPOSE: &str = "ic-memory allocation-governance authority";

/// Stable key of the allocation ledger when backed by the current MemoryManager substrate.
pub const IC_MEMORY_LEDGER_STABLE_KEY: &str = "ic_memory.ledger.v1";

/// Diagnostic label of the allocation ledger when backed by the current MemoryManager substrate.
pub const IC_MEMORY_LEDGER_LABEL: &str = "MemoryLayoutLedger";

/// MemoryManager ID used by the allocation ledger in the current MemoryManager substrate.
pub const MEMORY_MANAGER_LEDGER_ID: u8 = MEMORY_MANAGER_MIN_ID;

/// Last MemoryManager ID reserved for `ic-memory` governance in the current substrate.
pub const MEMORY_MANAGER_GOVERNANCE_MAX_ID: u8 = 9;

/// Return true when `stable_key` belongs to the `ic-memory` namespace.
#[must_use]
pub fn is_ic_memory_stable_key(stable_key: &str) -> bool {
    stable_key.starts_with(IC_MEMORY_STABLE_KEY_PREFIX)
}

/// MemoryManager range reserved for `ic-memory` governance in the current substrate.
#[must_use]
pub const fn memory_manager_governance_range() -> MemoryManagerIdRange {
    const RANGE: MemoryManagerIdRange =
        match MemoryManagerIdRange::new(MEMORY_MANAGER_MIN_ID, MEMORY_MANAGER_GOVERNANCE_MAX_ID) {
            Ok(range) => range,
            Err(_) => panic!("the governance range must be usable"),
        };
    RANGE
}

///
/// MemoryManagerSlotError
///
/// Invalid `MemoryManager` allocation ID.
///

#[non_exhaustive]
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum MemoryManagerSlotError {
    /// ID 255 is the unallocated-bucket sentinel.
    #[error("MemoryManager ID {id} is not a usable allocation slot")]
    InvalidMemoryManagerId {
        /// Invalid MemoryManager ID.
        id: u8,
    },
}

/// Validate that a `MemoryManager` ID is usable as an allocation slot.
pub const fn validate_memory_manager_id(id: u8) -> Result<(), MemoryManagerSlotError> {
    if id == MEMORY_MANAGER_INVALID_ID {
        return Err(MemoryManagerSlotError::InvalidMemoryManagerId { id });
    }
    Ok(())
}
