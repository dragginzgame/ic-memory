use super::{
    AllocationLedger, AllocationRecord, AllocationReservationError, AllocationRetirement,
    AllocationRetirementError, AllocationStageError, AllocationState, ClaimConflict, ClaimOutcome,
    GenerationRecord, claim_conflict_record, validate_declaration_claim,
    validate_reservation_claim,
};
use crate::{
    declaration::{AllocationDeclaration, DeclarationSnapshotError},
    session::ValidatedAllocations,
};

impl AllocationLedger {
    /// Return a copy of the ledger with `validated` recorded as the next generation.
    ///
    /// This is a pure logical update. Physical atomicity is the responsibility of
    /// the substrate commit protocol.
    ///
    /// Empty validated generations are valid. They record an explicit generation
    /// boundary, optional runtime fingerprint, and commit timestamp even when no
    /// allocation records changed.
    pub fn stage_validated_generation(
        &self,
        validated: &ValidatedAllocations,
        committed_at: Option<u64>,
    ) -> Result<Self, AllocationStageError> {
        if validated.base_generation() != self.current_generation {
            return Err(AllocationStageError::StaleValidatedAllocations {
                validated_generation: validated.base_generation(),
                ledger_generation: self.current_generation,
            });
        }
        let next_generation = checked_next_generation(self.current_generation)
            .map_err(|generation| AllocationStageError::GenerationOverflow { generation })?;
        let staged_declarations = validated.declarations();
        let Some(declaration_count) = checked_declaration_count(staged_declarations.len()) else {
            return Err(AllocationStageError::TooManyDeclarations {
                count: staged_declarations.len(),
            });
        };
        let mut next = self.clone();
        next.current_generation = next_generation;

        for declaration in staged_declarations {
            declaration.schema.validate().map_err(|error| {
                AllocationStageError::InvalidSchemaMetadata {
                    stable_key: declaration.stable_key.clone(),
                    error,
                }
            })?;
            record_declaration(&mut next, next_generation, declaration)?;
        }

        next.allocation_history.push_generation(GenerationRecord {
            generation: next_generation,
            parent_generation: self.current_generation,
            runtime_fingerprint: validated.runtime_fingerprint().map(str::to_string),
            declaration_count,
            committed_at,
        });

        Ok(next)
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
        let next_generation = checked_next_generation(self.current_generation)
            .map_err(|generation| AllocationReservationError::GenerationOverflow { generation })?;
        let Some(declaration_count) = checked_declaration_count(reservations.len()) else {
            return Err(AllocationReservationError::TooManyReservations {
                count: reservations.len(),
            });
        };
        let mut next = self.clone();
        next.current_generation = next_generation;

        for reservation in reservations {
            validate_reservation_declaration(reservation)?;
            record_reservation(&mut next, next_generation, reservation)?;
        }

        next.allocation_history.push_generation(GenerationRecord {
            generation: next_generation,
            parent_generation: self.current_generation,
            runtime_fingerprint: None,
            declaration_count,
            committed_at,
        });

        Ok(next)
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
        let next_generation = checked_next_generation(self.current_generation)
            .map_err(|generation| AllocationRetirementError::GenerationOverflow { generation })?;
        let mut next = self.clone();
        let record = next
            .allocation_history
            .records_mut()
            .iter_mut()
            .find(|record| record.stable_key == retirement.stable_key)
            .ok_or_else(|| {
                AllocationRetirementError::UnknownStableKey(retirement.stable_key.clone())
            })?;

        if record.slot != retirement.slot {
            return Err(AllocationRetirementError::SlotMismatch {
                stable_key: retirement.stable_key.clone(),
                historical_slot: Box::new(record.slot.clone()),
                retired_slot: Box::new(retirement.slot.clone()),
            });
        }
        if record.state == AllocationState::Retired {
            return Err(AllocationRetirementError::AlreadyRetired {
                stable_key: retirement.stable_key.clone(),
                slot: Box::new(record.slot.clone()),
            });
        }

        record.state = AllocationState::Retired;
        record.retired_generation = Some(next_generation);
        next.current_generation = next_generation;
        next.allocation_history.push_generation(GenerationRecord {
            generation: next_generation,
            parent_generation: self.current_generation,
            runtime_fingerprint: None,
            declaration_count: 0,
            committed_at,
        });

        Ok(next)
    }
}

fn record_declaration(
    ledger: &mut AllocationLedger,
    generation: u64,
    declaration: &AllocationDeclaration,
) -> Result<(), AllocationStageError> {
    match validate_declaration_claim(ledger, declaration) {
        Ok(ClaimOutcome::Existing { record_index }) => {
            ledger.allocation_history.records_mut()[record_index]
                .observe_declaration(generation, declaration)
                .map_err(|error| AllocationStageError::InvalidSchemaMetadata {
                    stable_key: declaration.stable_key.clone(),
                    error,
                })?;
            Ok(())
        }
        Ok(ClaimOutcome::New) => {
            let record = AllocationRecord::from_declaration(
                generation,
                declaration.clone(),
                AllocationState::Active,
            )
            .map_err(|error| AllocationStageError::InvalidSchemaMetadata {
                stable_key: declaration.stable_key.clone(),
                error,
            })?;
            ledger.allocation_history.push_record(record);
            Ok(())
        }
        Err(conflict) => Err(map_declaration_stage_conflict(
            ledger,
            declaration,
            conflict,
        )),
    }
}

fn record_reservation(
    ledger: &mut AllocationLedger,
    generation: u64,
    reservation: &AllocationDeclaration,
) -> Result<(), AllocationReservationError> {
    match validate_reservation_claim(ledger, reservation) {
        Ok(ClaimOutcome::Existing { record_index }) => {
            ledger.allocation_history.records_mut()[record_index]
                .observe_reservation(generation, reservation)
                .map_err(|error| AllocationReservationError::InvalidSchemaMetadata {
                    stable_key: reservation.stable_key.clone(),
                    error,
                })?;
            Ok(())
        }
        Ok(ClaimOutcome::New) => {
            let record =
                AllocationRecord::reserved(generation, reservation.clone()).map_err(|error| {
                    AllocationReservationError::InvalidSchemaMetadata {
                        stable_key: reservation.stable_key.clone(),
                        error,
                    }
                })?;
            ledger.allocation_history.push_record(record);
            Ok(())
        }
        Err(conflict) => Err(map_reservation_stage_conflict(
            ledger,
            reservation,
            conflict,
        )),
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

fn checked_declaration_count(count: usize) -> Option<u32> {
    u32::try_from(count).ok()
}

fn map_declaration_stage_conflict(
    ledger: &AllocationLedger,
    declaration: &AllocationDeclaration,
    conflict: ClaimConflict,
) -> AllocationStageError {
    let record = claim_conflict_record(ledger, conflict);
    match conflict {
        ClaimConflict::StableKeyMoved { .. } => AllocationStageError::StableKeySlotConflict {
            stable_key: declaration.stable_key.clone(),
            historical_slot: Box::new(record.slot.clone()),
            declared_slot: Box::new(declaration.slot.clone()),
        },
        ClaimConflict::SlotReused { .. } => AllocationStageError::SlotStableKeyConflict {
            slot: Box::new(declaration.slot.clone()),
            historical_key: record.stable_key.clone(),
            declared_key: declaration.stable_key.clone(),
        },
        ClaimConflict::Tombstoned { .. } => AllocationStageError::RetiredAllocation {
            stable_key: declaration.stable_key.clone(),
            slot: Box::new(record.slot.clone()),
        },
        ClaimConflict::ActiveAllocation { .. } => {
            AllocationStageError::UnexpectedActiveAllocationConflict {
                stable_key: record.stable_key.clone(),
                slot: Box::new(record.slot.clone()),
            }
        }
    }
}

fn map_reservation_stage_conflict(
    ledger: &AllocationLedger,
    reservation: &AllocationDeclaration,
    conflict: ClaimConflict,
) -> AllocationReservationError {
    let record = claim_conflict_record(ledger, conflict);
    match conflict {
        ClaimConflict::StableKeyMoved { .. } => AllocationReservationError::StableKeySlotConflict {
            stable_key: reservation.stable_key.clone(),
            historical_slot: Box::new(record.slot.clone()),
            reserved_slot: Box::new(reservation.slot.clone()),
        },
        ClaimConflict::SlotReused { .. } => AllocationReservationError::SlotStableKeyConflict {
            slot: Box::new(reservation.slot.clone()),
            historical_key: record.stable_key.clone(),
            reserved_key: reservation.stable_key.clone(),
        },
        ClaimConflict::Tombstoned { .. } => AllocationReservationError::RetiredAllocation {
            stable_key: reservation.stable_key.clone(),
            slot: Box::new(record.slot.clone()),
        },
        ClaimConflict::ActiveAllocation { .. } => AllocationReservationError::ActiveAllocation {
            stable_key: reservation.stable_key.clone(),
            slot: Box::new(record.slot.clone()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declaration_count_fails_closed_on_overflow() {
        assert_eq!(checked_declaration_count(u32::MAX as usize), Some(u32::MAX));
        assert_eq!(checked_declaration_count(u32::MAX as usize + 1), None);
    }
}
