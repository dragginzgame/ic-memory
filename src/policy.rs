use crate::{key::StableKey, slot::AllocationSlotDescriptor};

///
/// AllocationPolicy
///
/// Framework-supplied rules for whether a key may claim a slot.
///
/// Policy is intentionally separate from the durable ledger invariant. The
/// ledger remembers `stable_key -> allocation_slot`; this trait lets an
/// integration reject declarations that do not belong to its namespace or
/// substrate-specific range before staging a generation.
pub trait AllocationPolicy {
    /// Policy error type.
    type Error;

    /// Validate a stable key against framework naming rules.
    fn validate_key(&self, key: &StableKey) -> Result<(), Self::Error>;

    /// Validate a stable-key to allocation-slot claim.
    fn validate_slot(
        &self,
        key: &StableKey,
        slot: &AllocationSlotDescriptor,
    ) -> Result<(), Self::Error>;

    /// Validate a reserved stable-key to allocation-slot claim.
    fn validate_reserved_slot(
        &self,
        key: &StableKey,
        slot: &AllocationSlotDescriptor,
    ) -> Result<(), Self::Error>;
}
