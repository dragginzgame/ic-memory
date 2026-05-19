use super::descriptor::{AllocationSlot, AllocationSlotDescriptor};
use super::range_authority::MemoryManagerIdRange;

/// Substrate identifier for `ic-stable-structures::MemoryManager` slots.
pub const MEMORY_MANAGER_SUBSTRATE: &str = "ic-stable-structures.memory_manager";

/// Descriptor version for current `MemoryManagerId` slots.
pub const MEMORY_MANAGER_DESCRIPTOR_VERSION: u32 = 1;

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
    MemoryManagerIdRange {
        start: MEMORY_MANAGER_MIN_ID,
        end: MEMORY_MANAGER_GOVERNANCE_MAX_ID,
    }
}

impl AllocationSlotDescriptor {
    /// Construct a descriptor for a usable `MemoryManager` virtual memory ID.
    ///
    /// ID 255 is the `ic-stable-structures` unallocated-bucket sentinel and is
    /// rejected.
    pub fn memory_manager(id: u8) -> Result<Self, MemoryManagerSlotError> {
        validate_memory_manager_id(id)?;
        Ok(Self::memory_manager_unchecked(id))
    }

    /// Construct a descriptor for a `MemoryManager` virtual memory ID without validating it.
    #[must_use]
    pub(crate) fn memory_manager_unchecked(id: u8) -> Self {
        Self {
            slot: AllocationSlot::MemoryManagerId(id),
            substrate: MEMORY_MANAGER_SUBSTRATE.to_string(),
            descriptor_version: MEMORY_MANAGER_DESCRIPTOR_VERSION,
        }
    }

    /// Construct a descriptor for a usable `MemoryManager` virtual memory ID.
    ///
    /// This is an explicit alias for [`AllocationSlotDescriptor::memory_manager`].
    pub fn memory_manager_checked(id: u8) -> Result<Self, MemoryManagerSlotError> {
        Self::memory_manager(id)
    }

    /// Return the usable `MemoryManager` virtual memory ID represented by this descriptor.
    ///
    /// This validates substrate, descriptor version, slot kind, and sentinel ID
    /// rules before returning the numeric ID.
    pub fn memory_manager_id(&self) -> Result<u8, MemoryManagerSlotError> {
        if self.substrate != MEMORY_MANAGER_SUBSTRATE {
            return Err(MemoryManagerSlotError::UnsupportedSubstrate {
                substrate: self.substrate.clone(),
            });
        }
        if self.descriptor_version != MEMORY_MANAGER_DESCRIPTOR_VERSION {
            return Err(MemoryManagerSlotError::UnsupportedDescriptorVersion {
                version: self.descriptor_version,
            });
        }

        let AllocationSlot::MemoryManagerId(id) = self.slot else {
            return Err(MemoryManagerSlotError::UnsupportedSlot);
        };

        validate_memory_manager_id(id)?;
        Ok(id)
    }
}

///
/// MemoryManagerSlotError
///
/// Invalid or unsupported `MemoryManager` allocation slot descriptor.
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum MemoryManagerSlotError {
    /// Descriptor is not a `MemoryManagerId` slot.
    #[error("allocation slot is not a MemoryManager virtual memory ID")]
    UnsupportedSlot,
    /// Descriptor is attached to another substrate.
    #[error("allocation slot substrate '{substrate}' is not supported as a MemoryManager slot")]
    UnsupportedSubstrate {
        /// Unsupported substrate identifier.
        substrate: String,
    },
    /// Descriptor uses an unsupported encoding version.
    #[error("MemoryManager slot descriptor version {version} is unsupported")]
    UnsupportedDescriptorVersion {
        /// Unsupported descriptor version.
        version: u32,
    },
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
