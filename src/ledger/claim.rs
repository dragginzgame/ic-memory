use super::{AllocationLedger, AllocationRecord, AllocationState};
use crate::declaration::AllocationDeclaration;
use crate::key::StableKey;
use crate::slot::AllocationSlotDescriptor;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClaimOutcome {
    Existing { record_index: usize },
    New,
}

///
/// ClaimConflict
///
/// Internal claim failure carrying the historical record needed for error
/// projection. Successful claims retain indexes for subsequent ledger mutation.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClaimConflict<'ledger> {
    StableKeyMoved { record: &'ledger AllocationRecord },
    SlotReused { record: &'ledger AllocationRecord },
    Tombstoned { record: &'ledger AllocationRecord },
}

///
/// ReservationClaimConflict
///
/// Internal reservation failure, including refusal to reserve an active record.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReservationClaimConflict<'ledger> {
    Claim(ClaimConflict<'ledger>),
    ActiveAllocation { record: &'ledger AllocationRecord },
}

pub fn validate_declaration_claim<'ledger>(
    ledger: &'ledger AllocationLedger,
    declaration: &AllocationDeclaration,
) -> Result<ClaimOutcome, ClaimConflict<'ledger>> {
    if let Some(record_index) = find_by_key_index(ledger, &declaration.stable_key) {
        let record = &ledger.allocation_history.records()[record_index];
        if matches!(record.state, AllocationState::Retired { .. }) {
            return Err(ClaimConflict::Tombstoned { record });
        }
        if record.slot != declaration.slot {
            return Err(ClaimConflict::StableKeyMoved { record });
        }
        return Ok(ClaimOutcome::Existing { record_index });
    }

    if let Some(record) = find_by_slot(ledger, &declaration.slot) {
        return Err(ClaimConflict::SlotReused { record });
    }

    Ok(ClaimOutcome::New)
}

pub fn validate_reservation_claim<'ledger>(
    ledger: &'ledger AllocationLedger,
    reservation: &AllocationDeclaration,
) -> Result<ClaimOutcome, ReservationClaimConflict<'ledger>> {
    if let Some(record_index) = find_by_key_index(ledger, &reservation.stable_key) {
        let record = &ledger.allocation_history.records()[record_index];
        if record.slot != reservation.slot {
            return Err(ReservationClaimConflict::Claim(
                ClaimConflict::StableKeyMoved { record },
            ));
        }

        return match record.state {
            AllocationState::Reserved => Ok(ClaimOutcome::Existing { record_index }),
            AllocationState::Active => Err(ReservationClaimConflict::ActiveAllocation { record }),
            AllocationState::Retired { .. } => {
                Err(ReservationClaimConflict::Claim(ClaimConflict::Tombstoned {
                    record,
                }))
            }
        };
    }

    if let Some(record) = find_by_slot(ledger, &reservation.slot) {
        return Err(ReservationClaimConflict::Claim(ClaimConflict::SlotReused {
            record,
        }));
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

fn find_by_slot<'ledger>(
    ledger: &'ledger AllocationLedger,
    slot: &AllocationSlotDescriptor,
) -> Option<&'ledger AllocationRecord> {
    ledger
        .allocation_history
        .records()
        .iter()
        .find(|record| &record.slot == slot)
}
