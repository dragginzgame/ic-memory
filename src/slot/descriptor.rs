use crate::validation::Validate;
use serde::{Deserialize, Serialize};

///
/// AllocationSlot
///
/// Durable physical `ic-stable-structures::MemoryManager` allocation identity.
///
/// Stable keys are logical identities; allocation slots are the physical
/// locations those keys are bound to in the ledger.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub enum AllocationSlot {
    /// `ic-stable-structures::MemoryManager` virtual memory ID.
    MemoryManagerId(u8),
}

///
/// AllocationSlotDescriptor
///
/// Encoded allocation slot persisted in the ledger.
///
/// Use [`AllocationSlotDescriptor::memory_manager`] so ID 255 is rejected.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AllocationSlotDescriptor {
    /// Durable allocation slot.
    pub(crate) slot: AllocationSlot,
    /// Fixed substrate marker for the current `MemoryManager` slot protocol.
    pub(crate) substrate: String,
    /// Descriptor encoding version.
    pub(crate) descriptor_version: u32,
}

impl AllocationSlotDescriptor {
    /// Return the durable allocation slot value.
    #[must_use]
    pub const fn slot(&self) -> &AllocationSlot {
        &self.slot
    }

    /// Return the fixed substrate marker for this slot protocol.
    #[must_use]
    pub fn substrate(&self) -> &str {
        &self.substrate
    }

    /// Return the descriptor encoding version.
    #[must_use]
    pub const fn descriptor_version(&self) -> u32 {
        self.descriptor_version
    }
}

impl Validate for AllocationSlotDescriptor {
    type Error = AllocationSlotDescriptorError;

    fn validate(&self) -> Result<(), Self::Error> {
        self.memory_manager_id()
            .map(|_| ())
            .map_err(AllocationSlotDescriptorError::MemoryManager)
    }
}

///
/// AllocationSlotDescriptorError
///
/// Allocation slot descriptor validation failure.
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum AllocationSlotDescriptorError {
    /// `MemoryManager` descriptor invariants failed.
    #[error(transparent)]
    MemoryManager(super::memory_manager::MemoryManagerSlotError),
}
