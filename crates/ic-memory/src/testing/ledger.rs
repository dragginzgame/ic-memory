//! Pure ledger edits for constructing fixture histories, never runtime persistence authority.

use crate::{
    AllocationDeclaration, AllocationLedger, AllocationRecord, AllocationState,
    LedgerIntegrityError, MemoryManagerSlot, StableKey,
    key::StableKeyError,
    ledger::{ClaimConflict, ClaimOutcome, checked_next_generation, validate_new_claim},
};
use serde::{Deserialize, Serialize};

///
/// AllocationReservationError
///
/// Failure to stage a fixture reservation generation.
///

#[non_exhaustive]
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum AllocationReservationError {
    #[error(transparent)]
    Integrity(#[from] LedgerIntegrityError),
    /// Ledger generation cannot be advanced without overflow.
    #[error("ledger generation {generation} cannot be advanced without overflow")]
    GenerationOverflow {
        /// Current ledger generation.
        generation: u64,
    },
    /// Reservation count exceeds the usable allocation-slot domain.
    #[error("generation contains {count} reservations, exceeding the 255-allocation limit")]
    TooManyReservations {
        /// Number of reservations in the staged generation.
        count: usize,
    },
    /// Stable key was historically bound to a different slot.
    #[error("stable key '{stable_key}' was historically bound to a different allocation slot")]
    StableKeySlotConflict {
        /// Stable key being reserved.
        stable_key: StableKey,
        /// Historical slot for the stable key.
        historical_slot: MemoryManagerSlot,
        /// Slot claimed by the reservation.
        reserved_slot: MemoryManagerSlot,
    },
    /// Slot was historically bound to a different stable key.
    #[error("allocation slot '{slot:?}' was historically bound to stable key '{historical_key}'")]
    SlotStableKeyConflict {
        /// Slot being reserved.
        slot: MemoryManagerSlot,
        /// Historical stable key for the slot.
        historical_key: StableKey,
        /// Stable key claimed by the reservation.
        reserved_key: StableKey,
    },
    /// Allocation already exists as an active record.
    #[error("stable key '{stable_key}' is already active and cannot be reserved")]
    ActiveAllocation {
        /// Active stable key.
        stable_key: StableKey,
        /// Active allocation slot.
        slot: MemoryManagerSlot,
    },
    /// Allocation was already retired and cannot be reserved.
    #[error("stable key '{stable_key}' was explicitly retired and cannot be reserved")]
    RetiredAllocation {
        /// Retired stable key.
        stable_key: StableKey,
        /// Retired allocation slot.
        slot: MemoryManagerSlot,
    },
}

///
/// AllocationRetirementError
///
/// Failure to stage an explicit fixture retirement generation.
///

#[non_exhaustive]
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum AllocationRetirementError {
    #[error(transparent)]
    Integrity(#[from] LedgerIntegrityError),
    /// Stable-key grammar failure.
    #[error(transparent)]
    Key(StableKeyError),
    /// Ledger generation cannot be advanced without overflow.
    #[error("ledger generation {generation} cannot be advanced without overflow")]
    GenerationOverflow {
        /// Current ledger generation.
        generation: u64,
    },
    /// Stable key has no historical allocation record.
    #[error("stable key '{0}' has no allocation record to retire")]
    UnknownStableKey(StableKey),
    /// Stable key was historically bound to a different slot.
    #[error("stable key '{stable_key}' cannot be retired for a different allocation slot")]
    SlotMismatch {
        /// Stable key being retired.
        stable_key: StableKey,
        /// Historical slot for the stable key.
        historical_slot: MemoryManagerSlot,
        /// Slot named by the retirement request.
        retired_slot: MemoryManagerSlot,
    },
    /// Allocation was already retired.
    #[error("stable key '{stable_key}' was already retired")]
    AlreadyRetired {
        /// Retired stable key.
        stable_key: StableKey,
        /// Retired allocation slot.
        slot: MemoryManagerSlot,
    },
}

///
/// AllocationRetirement
///
/// Explicit fixture request to tombstone one retained allocation identity. Retirement
/// prevents redeclaration and never frees the physical slot for another key.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AllocationRetirement {
    pub(crate) stable_key: StableKey,
    pub(crate) slot: MemoryManagerSlot,
}

impl AllocationRetirement {
    /// Build an explicit retirement request from checked raw parts.
    pub fn new(
        stable_key: impl AsRef<str>,
        slot: MemoryManagerSlot,
    ) -> Result<Self, AllocationRetirementError> {
        let stable_key = StableKey::parse(stable_key).map_err(AllocationRetirementError::Key)?;
        Ok(Self { stable_key, slot })
    }

    /// Return the stable key being retired.
    #[must_use]
    pub const fn stable_key(&self) -> &StableKey {
        &self.stable_key
    }

    /// Return the allocation slot named by the request.
    #[must_use]
    pub const fn slot(&self) -> &MemoryManagerSlot {
        &self.slot
    }
}

impl AllocationLedger {
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
        self.validate_bounds()?;
        let parent_generation = self.current_generation;
        let next_generation = checked_next_generation(parent_generation)
            .map_err(|generation| AllocationReservationError::GenerationOverflow { generation })?;
        checked_reservation_count(reservations.len())?;
        let mut next = self.clone();
        next.current_generation = next_generation;

        for reservation in reservations {
            record_reservation(&mut next, reservation)?;
        }

        next.validate_bounds()?;
        Ok(next)
    }

    /// Return a copy of the ledger with one explicit retirement committed.
    ///
    /// Retirement tombstones any known non-retired allocation identity,
    /// including reserved records that never became active.
    pub fn stage_retirement_generation(
        &self,
        retirement: &AllocationRetirement,
    ) -> Result<Self, AllocationRetirementError> {
        self.validate_bounds()?;
        let parent_generation = self.current_generation;
        let next_generation = checked_next_generation(parent_generation)
            .map_err(|generation| AllocationRetirementError::GenerationOverflow { generation })?;
        let record_index = self
            .records()
            .iter()
            .position(|record| record.stable_key == retirement.stable_key)
            .ok_or_else(|| {
                AllocationRetirementError::UnknownStableKey(retirement.stable_key.clone())
            })?;
        let record = &self.records()[record_index];

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

        let mut next = self.clone();
        next.records[record_index].state = AllocationState::Retired;
        next.current_generation = next_generation;

        // Staging checked every bound and counter advancement.
        // Retirement changes neither allocation count nor schema metadata.
        Ok(next)
    }
}

///
/// ReservationClaimConflict
///
/// Internal reservation failure, including refusal to reserve an active record.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReservationClaimConflict<'ledger> {
    Claim(ClaimConflict<'ledger>),
    ActiveAllocation { record: &'ledger AllocationRecord },
}

fn validate_reservation_claim<'ledger>(
    ledger: &'ledger AllocationLedger,
    reservation: &AllocationDeclaration,
) -> Result<ClaimOutcome, ReservationClaimConflict<'ledger>> {
    if let Some(record_index) = ledger
        .records()
        .iter()
        .position(|record| record.stable_key() == reservation.stable_key())
    {
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

const fn checked_reservation_count(count: usize) -> Result<(), AllocationReservationError> {
    if count > crate::constants::MAX_ALLOCATIONS {
        return Err(AllocationReservationError::TooManyReservations { count });
    }
    Ok(())
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
    use crate::ledger::validate_declaration_claim;

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
