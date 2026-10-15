use super::{AllocationLedger, AllocationRecord, AllocationState};
use crate::declaration::AllocationDeclaration;
use crate::key::StableKey;
use crate::slot::MemoryManagerSlot;

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
        let record = &ledger.records()[record_index];
        if matches!(record.state, AllocationState::Retired) {
            return Err(ClaimConflict::Tombstoned { record });
        }
        if record.slot != declaration.slot {
            return Err(ClaimConflict::StableKeyMoved { record });
        }
        return Ok(ClaimOutcome::Existing { record_index });
    }

    validate_new_claim(ledger, &declaration.slot)
}

pub fn validate_reservation_claim<'ledger>(
    ledger: &'ledger AllocationLedger,
    reservation: &AllocationDeclaration,
) -> Result<ClaimOutcome, ReservationClaimConflict<'ledger>> {
    if let Some(record_index) = find_by_key_index(ledger, &reservation.stable_key) {
        let record = &ledger.records()[record_index];
        if record.slot != reservation.slot {
            return Err(ReservationClaimConflict::Claim(
                ClaimConflict::StableKeyMoved { record },
            ));
        }

        return match record.state {
            AllocationState::Reserved => Ok(ClaimOutcome::Existing { record_index }),
            AllocationState::Active => Err(ReservationClaimConflict::ActiveAllocation { record }),
            AllocationState::Retired => {
                Err(ReservationClaimConflict::Claim(ClaimConflict::Tombstoned {
                    record,
                }))
            }
        };
    }

    validate_new_claim(ledger, &reservation.slot).map_err(ReservationClaimConflict::Claim)
}

fn find_by_key_index(ledger: &AllocationLedger, stable_key: &StableKey) -> Option<usize> {
    ledger
        .records()
        .iter()
        .position(|record| &record.stable_key == stable_key)
}

fn validate_new_claim<'ledger>(
    ledger: &'ledger AllocationLedger,
    slot: &MemoryManagerSlot,
) -> Result<ClaimOutcome, ClaimConflict<'ledger>> {
    if let Some(record) = ledger.records().iter().find(|record| &record.slot == slot) {
        return Err(ClaimConflict::SlotReused { record });
    }
    Ok(ClaimOutcome::New)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retired_moved_key_keeps_each_operations_error_precedence() {
        let original = AllocationDeclaration::memory_manager("app.rows.v1", 100).unwrap();
        let mut record = AllocationRecord::active(&original);
        record.state = AllocationState::Retired;
        let ledger = AllocationLedger::new(1, vec![record]).unwrap();
        let moved = AllocationDeclaration::memory_manager("app.rows.v1", 101).unwrap();
        assert!(matches!(
            validate_declaration_claim(&ledger, &moved),
            Err(ClaimConflict::Tombstoned { .. })
        ));
        assert!(matches!(
            validate_reservation_claim(&ledger, &moved),
            Err(ReservationClaimConflict::Claim(
                ClaimConflict::StableKeyMoved { .. }
            ))
        ));
        assert!(matches!(
            validate_reservation_claim(&ledger, &original),
            Err(ReservationClaimConflict::Claim(
                ClaimConflict::Tombstoned { .. }
            ))
        ));
    }
}
