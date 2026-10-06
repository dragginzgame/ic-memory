use super::{AllocationLedger, LedgerIntegrityError};
use std::collections::BTreeSet;

impl AllocationLedger {
    pub(crate) const fn validate_bounds(&self) -> Result<(), LedgerIntegrityError> {
        if self.records.len() > crate::constants::MAX_ALLOCATIONS {
            return Err(LedgerIntegrityError::LimitExceeded {
                resource: "allocation records",
                limit: crate::constants::MAX_ALLOCATIONS,
            });
        }
        Ok(())
    }

    /// Validate count, unique ownership and empty genesis before recovery or commit.
    /// Checked field types already enforce key, slot and schema invariants.
    pub fn validate_integrity(&self) -> Result<(), LedgerIntegrityError> {
        self.validate_bounds()?;
        let mut stable_keys = BTreeSet::new();
        let mut slots = [false; crate::constants::MAX_ALLOCATIONS];
        for record in self.records() {
            if !stable_keys.insert(record.stable_key()) {
                return Err(LedgerIntegrityError::DuplicateStableKey {
                    stable_key: record.stable_key.clone(),
                });
            }
            if slots[usize::from(record.slot.id())] {
                return Err(LedgerIntegrityError::DuplicateSlot {
                    slot: record.slot.clone(),
                });
            }
            slots[usize::from(record.slot.id())] = true;
        }
        if self.current_generation == 0 && !self.records.is_empty() {
            return Err(LedgerIntegrityError::NonemptyGenesis);
        }
        Ok(())
    }
}
