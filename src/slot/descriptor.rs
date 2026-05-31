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
}

impl AllocationSlotDescriptor {
    /// Return the durable allocation slot value.
    #[must_use]
    pub const fn slot(&self) -> &AllocationSlot {
        &self.slot
    }
}

impl Validate for AllocationSlotDescriptor {
    type Error = super::memory_manager::MemoryManagerSlotError;

    fn validate(&self) -> Result<(), Self::Error> {
        self.memory_manager_id().map(|_| ())
    }
}
