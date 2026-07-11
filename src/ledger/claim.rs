use super::{AllocationLedger, AllocationRecord, AllocationState};
use crate::declaration::AllocationDeclaration;
use crate::key::StableKey;
use crate::slot::AllocationSlotDescriptor;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClaimOutcome {
    Existing { record_index: usize },
    New,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClaimConflict {
    StableKeyMoved { record_index: usize },
    SlotReused { record_index: usize },
    Tombstoned { record_index: usize },
    ActiveAllocation { record_index: usize },
}

impl ClaimConflict {
    pub const fn record_index(self) -> usize {
        match self {
            Self::StableKeyMoved { record_index }
            | Self::SlotReused { record_index }
            | Self::Tombstoned { record_index }
            | Self::ActiveAllocation { record_index } => record_index,
        }
    }
}

pub fn claim_conflict_record(
    ledger: &AllocationLedger,
    conflict: ClaimConflict,
) -> &AllocationRecord {
    &ledger.allocation_history.records()[conflict.record_index()]
}

pub fn validate_declaration_claim(
    ledger: &AllocationLedger,
    declaration: &AllocationDeclaration,
) -> Result<ClaimOutcome, ClaimConflict> {
    if let Some(record_index) = find_by_key_index(ledger, &declaration.stable_key) {
        let record = &ledger.allocation_history.records()[record_index];
        if matches!(record.state, AllocationState::Retired { .. }) {
            return Err(ClaimConflict::Tombstoned { record_index });
        }
        if record.slot != declaration.slot {
            return Err(ClaimConflict::StableKeyMoved { record_index });
        }
        return Ok(ClaimOutcome::Existing { record_index });
    }

    if let Some(record_index) = find_by_slot_index(ledger, &declaration.slot) {
        return Err(ClaimConflict::SlotReused { record_index });
    }

    Ok(ClaimOutcome::New)
}

pub fn validate_reservation_claim(
    ledger: &AllocationLedger,
    reservation: &AllocationDeclaration,
) -> Result<ClaimOutcome, ClaimConflict> {
    if let Some(record_index) = find_by_key_index(ledger, &reservation.stable_key) {
        let record = &ledger.allocation_history.records()[record_index];
        if record.slot != reservation.slot {
            return Err(ClaimConflict::StableKeyMoved { record_index });
        }

        return match record.state {
            AllocationState::Reserved => Ok(ClaimOutcome::Existing { record_index }),
            AllocationState::Active => Err(ClaimConflict::ActiveAllocation { record_index }),
            AllocationState::Retired { .. } => Err(ClaimConflict::Tombstoned { record_index }),
        };
    }

    if let Some(record_index) = find_by_slot_index(ledger, &reservation.slot) {
        return Err(ClaimConflict::SlotReused { record_index });
    }

    Ok(ClaimOutcome::New)
}

fn find_by_key_index(ledger: &AllocationLedger, stable_key: &StableKey) -> Option<usize> {
    ledger
        .allocation_history
        .records()
        .iter()
        .position(|record| &record.stable_key == stable_key)
}

fn find_by_slot_index(ledger: &AllocationLedger, slot: &AllocationSlotDescriptor) -> Option<usize> {
    ledger
        .allocation_history
        .records()
        .iter()
        .position(|record| &record.slot == slot)
}
