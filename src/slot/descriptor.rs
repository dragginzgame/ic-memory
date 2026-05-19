use serde::{Deserialize, Serialize};

///
/// AllocationSlot
///
/// Durable physical allocation identity interpreted by a storage substrate.
///
/// Stable keys are logical identities; allocation slots are the physical
/// locations those keys are bound to in the ledger.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum AllocationSlot {
    /// `ic-stable-structures::MemoryManager` virtual memory ID.
    MemoryManagerId(u8),
    /// Named substrate partition.
    NamedPartition(String),
    /// Forward-compatible custom slot descriptor.
    Custom {
        /// Substrate-defined slot kind.
        kind: String,
        /// Slot descriptor version.
        version: u32,
        /// Canonical substrate-defined value.
        value: Vec<u8>,
    },
}

///
/// AllocationSlotDescriptor
///
/// Encoded allocation slot persisted in the ledger.
///
/// Constructors for built-in substrates validate their invariants before a
/// descriptor can be used publicly. For `MemoryManager` slots, use
/// [`AllocationSlotDescriptor::memory_manager`] so ID 255 is rejected.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct AllocationSlotDescriptor {
    /// Durable allocation slot.
    pub(crate) slot: AllocationSlot,
    /// Substrate identifier that interprets the slot.
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

    /// Return the substrate identifier that interprets this slot.
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
