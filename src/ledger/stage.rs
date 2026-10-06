use super::{
    AllocationLedger, AllocationRecord, AllocationReservationError, AllocationRetirement,
    AllocationRetirementError, AllocationStageError, AllocationState, ClaimConflict, ClaimOutcome,
    ReservationClaimConflict, validate_declaration_claim, validate_reservation_claim,
};
use crate::{capability::ValidatedAllocations, declaration::AllocationDeclaration};
use std::borrow::Cow;

impl AllocationLedger {
    /// Return a copy of the ledger with `validated` recorded as the next generation.
    ///
    /// This is a pure logical update. Physical atomicity is the responsibility of
    /// the substrate commit protocol.
    ///
    /// Empty validated commits are valid. They advance the counter even when no
    /// allocation records changed, without retaining an event.
    ///
    /// # Panics
    ///
    /// Panics only if an internal validated-allocation invariant is broken.
    pub fn stage_validated_generation(
        &self,
        validated: &ValidatedAllocations,
    ) -> Result<Self, AllocationStageError> {
        stage_validated_generation(Cow::Borrowed(self), validated)
    }

    /// Return a copy of the ledger with `reservations` recorded as the next generation.
    ///
    /// This is a pure logical update. The caller is responsible for applying
    /// framework policy before staging reservations.
    ///
    /// Empty reservation commits are valid. They advance the counter without
    /// retaining an event.
    pub fn stage_reservation_generation(
        &self,
        reservations: &[AllocationDeclaration],
    ) -> Result<Self, AllocationReservationError> {
        stage_reservation_generation(Cow::Borrowed(self), reservations)
    }

    /// Return a copy of the ledger with one explicit retirement committed.
    ///
    /// Retirement tombstones any known non-retired allocation identity,
    /// including reserved records that never became active.
    pub fn stage_retirement_generation(
        &self,
        retirement: &AllocationRetirement,
    ) -> Result<Self, AllocationRetirementError> {
        stage_retirement_generation(Cow::Borrowed(self), retirement)
    }
}

// One staging implementation per operation. Public borrowed calls copy only
// after their preflight checks; bootstrap transfers its already-owned ledger
// without copying its records. Mutation remains local until protected commit.
pub fn stage_validated_generation(
    ledger: Cow<'_, AllocationLedger>,
    validated: &ValidatedAllocations,
) -> Result<AllocationLedger, AllocationStageError> {
    if validated.base_generation() != ledger.current_generation {
        return Err(AllocationStageError::StaleValidatedAllocations {
            validated_generation: validated.base_generation(),
            ledger_generation: ledger.current_generation,
        });
    }
    ledger.validate_bounds()?;
    let parent_generation = ledger.current_generation;
    let next_generation = checked_next_generation(parent_generation)
        .map_err(|generation| AllocationStageError::GenerationOverflow { generation })?;
    let staged_declarations = validated.declarations();
    let mut next = ledger.into_owned();
    next.current_generation = next_generation;

    for declaration in staged_declarations {
        record_declaration(&mut next, declaration)?;
    }

    next.validate_bounds()?;
    Ok(next)
}

pub fn stage_reservation_generation(
    ledger: Cow<'_, AllocationLedger>,
    reservations: &[AllocationDeclaration],
) -> Result<AllocationLedger, AllocationReservationError> {
    ledger.validate_bounds()?;
    let parent_generation = ledger.current_generation;
    let next_generation = checked_next_generation(parent_generation)
        .map_err(|generation| AllocationReservationError::GenerationOverflow { generation })?;
    checked_reservation_count(reservations.len())?;
    let mut next = ledger.into_owned();
    next.current_generation = next_generation;

    for reservation in reservations {
        record_reservation(&mut next, reservation)?;
    }

    next.validate_bounds()?;
    Ok(next)
}

pub fn stage_retirement_generation(
    ledger: Cow<'_, AllocationLedger>,
    retirement: &AllocationRetirement,
) -> Result<AllocationLedger, AllocationRetirementError> {
    ledger.validate_bounds()?;
    let parent_generation = ledger.current_generation;
    let next_generation = checked_next_generation(parent_generation)
        .map_err(|generation| AllocationRetirementError::GenerationOverflow { generation })?;
    let record_index = ledger
        .records()
        .iter()
        .position(|record| record.stable_key == retirement.stable_key)
        .ok_or_else(|| {
            AllocationRetirementError::UnknownStableKey(retirement.stable_key.clone())
        })?;
    let record = &ledger.records()[record_index];

    if record.slot != retirement.slot {
        return Err(AllocationRetirementError::SlotMismatch {
            stable_key: retirement.stable_key.clone(),
            historical_slot: record.slot.clone(),
            retired_slot: retirement.slot.clone(),
        });
    }
    if matches!(record.state, AllocationState::Retired) {
        return Err(AllocationRetirementError::AlreadyRetired {
            stable_key: retirement.stable_key.clone(),
            slot: record.slot.clone(),
        });
    }

    let mut next = ledger.into_owned();
    next.records[record_index].state = AllocationState::Retired;
    next.current_generation = next_generation;

    // Staging checked every bound and counter advancement.
    // Retirement changes neither allocation count nor schema metadata.
    Ok(next)
}

fn record_declaration(
    ledger: &mut AllocationLedger,
    declaration: &AllocationDeclaration,
) -> Result<(), AllocationStageError> {
    match validate_declaration_claim(ledger, declaration) {
        Ok(ClaimOutcome::Existing { record_index }) => {
            // Claim validation rejected retired identities. Both an existing
            // active claim and a matching reservation become active here.
            let record = &mut ledger.records[record_index];
            record.state = AllocationState::Active;
            record.schema.clone_from(&declaration.schema);
            Ok(())
        }
        Ok(ClaimOutcome::New) => {
            let record = AllocationRecord::active(declaration);
            ledger.records.push(record);
            Ok(())
        }
        Err(conflict) => Err(map_declaration_stage_conflict(declaration, conflict)),
    }
}

fn record_reservation(
    ledger: &mut AllocationLedger,
    reservation: &AllocationDeclaration,
) -> Result<(), AllocationReservationError> {
    match validate_reservation_claim(ledger, reservation) {
        Ok(ClaimOutcome::Existing { record_index }) => {
            ledger.records[record_index]
                .schema
                .clone_from(&reservation.schema);
            Ok(())
        }
        Ok(ClaimOutcome::New) => {
            let record = AllocationRecord::reserved(reservation);
            ledger.records.push(record);
            Ok(())
        }
        Err(conflict) => Err(map_reservation_stage_conflict(reservation, conflict)),
    }
}

const fn checked_next_generation(current_generation: u64) -> Result<u64, u64> {
    match current_generation.checked_add(1) {
        Some(next_generation) => Ok(next_generation),
        None => Err(current_generation),
    }
}

pub const fn checked_reservation_count(count: usize) -> Result<(), AllocationReservationError> {
    if count > crate::constants::MAX_ALLOCATIONS {
        return Err(AllocationReservationError::TooManyReservations { count });
    }
    Ok(())
}

fn map_declaration_stage_conflict(
    declaration: &AllocationDeclaration,
    conflict: ClaimConflict<'_>,
) -> AllocationStageError {
    match conflict {
        ClaimConflict::StableKeyMoved { record } => AllocationStageError::StableKeySlotConflict {
            stable_key: declaration.stable_key.clone(),
            historical_slot: record.slot.clone(),
            declared_slot: declaration.slot.clone(),
        },
        ClaimConflict::SlotReused { record } => AllocationStageError::SlotStableKeyConflict {
            slot: declaration.slot.clone(),
            historical_key: record.stable_key.clone(),
            declared_key: declaration.stable_key.clone(),
        },
        ClaimConflict::Tombstoned { record } => AllocationStageError::RetiredAllocation {
            stable_key: declaration.stable_key.clone(),
            slot: record.slot.clone(),
        },
    }
}

fn map_reservation_stage_conflict(
    reservation: &AllocationDeclaration,
    conflict: ReservationClaimConflict<'_>,
) -> AllocationReservationError {
    let conflict = match conflict {
        ReservationClaimConflict::ActiveAllocation { record } => {
            return AllocationReservationError::ActiveAllocation {
                stable_key: reservation.stable_key.clone(),
                slot: record.slot.clone(),
            };
        }
        ReservationClaimConflict::Claim(conflict) => conflict,
    };
    match conflict {
        ClaimConflict::StableKeyMoved { record } => {
            AllocationReservationError::StableKeySlotConflict {
                stable_key: reservation.stable_key.clone(),
                historical_slot: record.slot.clone(),
                reserved_slot: reservation.slot.clone(),
            }
        }
        ClaimConflict::SlotReused { record } => AllocationReservationError::SlotStableKeyConflict {
            slot: reservation.slot.clone(),
            historical_key: record.stable_key.clone(),
            reserved_key: reservation.stable_key.clone(),
        },
        ClaimConflict::Tombstoned { record } => AllocationReservationError::RetiredAllocation {
            stable_key: reservation.stable_key.clone(),
            slot: record.slot.clone(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reservation_count_fails_closed_on_overflow() {
        assert_eq!(checked_reservation_count(0), Ok(()));
        assert_eq!(checked_reservation_count(255), Ok(()));
        for count in [256, usize::MAX] {
            assert_eq!(
                checked_reservation_count(count),
                Err(AllocationReservationError::TooManyReservations { count })
            );
        }
    }
}
