use super::{
    AllocationLedger, AllocationRecord, AllocationReservationError, AllocationRetirement,
    AllocationRetirementError, AllocationStageError, AllocationState, ClaimConflict, ClaimOutcome,
    GenerationRecord, ReservationClaimConflict, validate_declaration_claim,
    validate_reservation_claim,
};
use crate::{
    capability::ValidatedAllocations,
    declaration::{AllocationDeclaration, DeclarationSnapshotError},
};
use std::borrow::Cow;

impl AllocationLedger {
    /// Return a copy of the ledger with `validated` recorded as the next generation.
    ///
    /// This is a pure logical update. Physical atomicity is the responsibility of
    /// the substrate commit protocol.
    ///
    /// Empty validated generations are valid. They record an explicit generation
    /// boundary, optional runtime fingerprint, and commit timestamp even when no
    /// allocation records changed.
    ///
    /// # Panics
    ///
    /// Panics only if an internal validated-allocation invariant is broken.
    pub fn stage_validated_generation(
        &self,
        validated: &ValidatedAllocations,
        committed_at: Option<u64>,
    ) -> Result<Self, AllocationStageError> {
        stage_validated_generation(Cow::Borrowed(self), validated, committed_at)
    }

    /// Return a copy of the ledger with `reservations` recorded as the next generation.
    ///
    /// This is a pure logical update. The caller is responsible for applying
    /// framework policy before staging reservations.
    ///
    /// Empty reservation generations are valid. They record an explicit
    /// generation boundary and commit timestamp even when no reservation records
    /// changed.
    pub fn stage_reservation_generation(
        &self,
        reservations: &[AllocationDeclaration],
        committed_at: Option<u64>,
    ) -> Result<Self, AllocationReservationError> {
        stage_reservation_generation(Cow::Borrowed(self), reservations, committed_at)
    }

    /// Return a copy of the ledger with one explicit retirement committed.
    ///
    /// Retirement tombstones any known non-retired allocation identity,
    /// including reserved records that never became active.
    pub fn stage_retirement_generation(
        &self,
        retirement: &AllocationRetirement,
        committed_at: Option<u64>,
    ) -> Result<Self, AllocationRetirementError> {
        stage_retirement_generation(Cow::Borrowed(self), retirement, committed_at)
    }
}

// One staging implementation per operation. Public borrowed calls copy only
// after their preflight checks; bootstrap transfers its already-owned ledger
// without copying history. Mutation remains local until protected commit.
pub fn stage_validated_generation(
    ledger: Cow<'_, AllocationLedger>,
    validated: &ValidatedAllocations,
    committed_at: Option<u64>,
) -> Result<AllocationLedger, AllocationStageError> {
    if validated.base_generation() != ledger.current_generation {
        return Err(AllocationStageError::StaleValidatedAllocations {
            validated_generation: validated.base_generation(),
            ledger_generation: ledger.current_generation,
        });
    }
    ledger.validate_staging_bounds()?;
    let parent_generation = ledger.current_generation;
    // The matching proof's base generation passed bounded committed-history
    // validation; adding one cannot overflow even for a different receiver.
    let next_generation = parent_generation + 1;
    let staged_declarations = validated.declarations();
    let declaration_count =
        u32::try_from(staged_declarations.len()).expect("validated declaration count");
    let mut next = ledger.into_owned();
    next.current_generation = next_generation;

    for declaration in staged_declarations {
        record_declaration(&mut next, next_generation, declaration)?;
    }

    next.allocation_history.generations.push(GenerationRecord {
        generation: next_generation,
        parent_generation,
        runtime_fingerprint: validated.runtime_fingerprint().map(str::to_string),
        declaration_count,
        committed_at,
    });

    next.validate_bounds()?;
    Ok(next)
}

pub fn stage_reservation_generation(
    ledger: Cow<'_, AllocationLedger>,
    reservations: &[AllocationDeclaration],
    committed_at: Option<u64>,
) -> Result<AllocationLedger, AllocationReservationError> {
    ledger.validate_staging_bounds()?;
    let parent_generation = ledger.current_generation;
    let next_generation = checked_next_generation(parent_generation)
        .map_err(|generation| AllocationReservationError::GenerationOverflow { generation })?;
    let declaration_count = checked_reservation_count(reservations.len())?;
    let mut next = ledger.into_owned();
    next.current_generation = next_generation;

    for reservation in reservations {
        validate_reservation_declaration(reservation)?;
        record_reservation(&mut next, next_generation, reservation)?;
    }

    next.allocation_history.generations.push(GenerationRecord {
        generation: next_generation,
        parent_generation,
        runtime_fingerprint: None,
        declaration_count,
        committed_at,
    });

    next.validate_bounds()?;
    Ok(next)
}

pub fn stage_retirement_generation(
    ledger: Cow<'_, AllocationLedger>,
    retirement: &AllocationRetirement,
    committed_at: Option<u64>,
) -> Result<AllocationLedger, AllocationRetirementError> {
    ledger.validate_staging_bounds()?;
    let parent_generation = ledger.current_generation;
    let next_generation = checked_next_generation(parent_generation)
        .map_err(|generation| AllocationRetirementError::GenerationOverflow { generation })?;
    let record_index = ledger
        .allocation_history
        .records()
        .iter()
        .position(|record| record.stable_key == retirement.stable_key)
        .ok_or_else(|| {
            AllocationRetirementError::UnknownStableKey(retirement.stable_key.clone())
        })?;
    let record = &ledger.allocation_history.records()[record_index];

    if record.slot != retirement.slot {
        return Err(AllocationRetirementError::SlotMismatch {
            stable_key: retirement.stable_key.clone(),
            historical_slot: record.slot.clone(),
            retired_slot: retirement.slot.clone(),
        });
    }
    if matches!(record.state, AllocationState::Retired { .. }) {
        return Err(AllocationRetirementError::AlreadyRetired {
            stable_key: retirement.stable_key.clone(),
            slot: record.slot.clone(),
        });
    }

    let mut next = ledger.into_owned();
    next.allocation_history.records[record_index].state = AllocationState::Retired {
        generation: next_generation,
    };
    next.current_generation = next_generation;
    next.allocation_history.generations.push(GenerationRecord {
        generation: next_generation,
        parent_generation,
        runtime_fingerprint: None,
        declaration_count: 0,
        committed_at,
    });

    // Staging checked every bound and reserved one generation entry.
    // Retirement changes neither allocation count nor schema history.
    Ok(next)
}

fn record_declaration(
    ledger: &mut AllocationLedger,
    generation: u64,
    declaration: &AllocationDeclaration,
) -> Result<(), AllocationStageError> {
    match validate_declaration_claim(ledger, declaration) {
        Ok(ClaimOutcome::Existing { record_index }) => {
            // Claim validation rejected retired identities. Both an existing
            // active claim and a matching reservation become active here.
            let record = &mut ledger.allocation_history.records[record_index];
            record.state = AllocationState::Active;
            record.observe_schema(generation, &declaration.schema);
            Ok(())
        }
        Ok(ClaimOutcome::New) => {
            let record = AllocationRecord::active(generation, declaration);
            ledger.allocation_history.records.push(record);
            Ok(())
        }
        Err(conflict) => Err(map_declaration_stage_conflict(declaration, conflict)),
    }
}

fn record_reservation(
    ledger: &mut AllocationLedger,
    generation: u64,
    reservation: &AllocationDeclaration,
) -> Result<(), AllocationReservationError> {
    match validate_reservation_claim(ledger, reservation) {
        Ok(ClaimOutcome::Existing { record_index }) => {
            ledger.allocation_history.records[record_index]
                .observe_schema(generation, &reservation.schema);
            Ok(())
        }
        Ok(ClaimOutcome::New) => {
            let record = AllocationRecord::reserved(generation, reservation);
            ledger.allocation_history.records.push(record);
            Ok(())
        }
        Err(conflict) => Err(map_reservation_stage_conflict(reservation, conflict)),
    }
}

pub fn validate_reservation_declaration(
    reservation: &AllocationDeclaration,
) -> Result<(), AllocationReservationError> {
    reservation.validate().map_err(|err| match err {
        DeclarationSnapshotError::SchemaMetadata(error) => {
            AllocationReservationError::InvalidSchemaMetadata {
                stable_key: reservation.stable_key.clone(),
                error,
            }
        }
        err => AllocationReservationError::InvalidDeclaration(err),
    })
}

const fn checked_next_generation(current_generation: u64) -> Result<u64, u64> {
    match current_generation.checked_add(1) {
        Some(next_generation) => Ok(next_generation),
        None => Err(current_generation),
    }
}

pub fn checked_reservation_count(count: usize) -> Result<u32, AllocationReservationError> {
    if count > crate::constants::MAX_ALLOCATIONS {
        return Err(AllocationReservationError::TooManyReservations { count });
    }
    Ok(u32::try_from(count).expect("bounded reservation count"))
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
        assert_eq!(checked_reservation_count(0), Ok(0));
        assert_eq!(checked_reservation_count(255), Ok(255));
        for count in [256, usize::MAX] {
            assert_eq!(
                checked_reservation_count(count),
                Err(AllocationReservationError::TooManyReservations { count })
            );
        }
    }
}
