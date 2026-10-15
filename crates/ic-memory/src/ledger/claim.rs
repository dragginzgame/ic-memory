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

fn find_by_key_index(ledger: &AllocationLedger, stable_key: &StableKey) -> Option<usize> {
    ledger
        .records()
        .iter()
        .position(|record| &record.stable_key == stable_key)
}

pub fn validate_new_claim<'ledger>(
    ledger: &'ledger AllocationLedger,
    slot: &MemoryManagerSlot,
) -> Result<ClaimOutcome, ClaimConflict<'ledger>> {
    if let Some(record) = ledger.records().iter().find(|record| &record.slot == slot) {
        return Err(ClaimConflict::SlotReused { record });
    }
    Ok(ClaimOutcome::New)
}
