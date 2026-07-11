use crate::{
    capability::ValidatedAllocations,
    declaration::{DeclarationSnapshot, DeclarationSnapshotError},
    key::StableKey,
    ledger::{
        AllocationLedger, ClaimConflict, LedgerIntegrityError, RecoveredLedger,
        claim_conflict_record, validate_declaration_claim,
    },
    policy::AllocationPolicy,
    slot::AllocationSlotDescriptor,
};

///
/// Validate
///
/// Re-check constructor invariants on decoded DTOs before they become
/// authoritative.
pub trait Validate {
    /// Validation error for this DTO.
    type Error;

    /// Validate this value's domain invariants.
    fn validate(&self) -> Result<(), Self::Error>;
}

///
/// AllocationValidationError
///
/// Failure to validate declarations against policy and historical ledger facts.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum AllocationValidationError<P> {
    /// Historical ledger was decoded or assembled with invalid committed state.
    #[error(transparent)]
    LedgerIntegrity(LedgerIntegrityError),
    /// Declaration snapshot was decoded or assembled with invalid DTOs.
    #[error(transparent)]
    Snapshot(DeclarationSnapshotError),
    /// Policy adapter rejected the declaration.
    #[error("allocation policy rejected a declaration")]
    Policy(P),
    /// Stable key was historically bound to a different slot.
    #[error("stable key '{stable_key}' was historically bound to a different allocation slot")]
    StableKeySlotConflict {
        /// Stable key that was redeclared.
        stable_key: StableKey,
        /// Historical slot for the stable key.
        historical_slot: Box<AllocationSlotDescriptor>,
        /// Slot claimed by the current declaration.
        declared_slot: Box<AllocationSlotDescriptor>,
    },
    /// Slot was historically bound to a different stable key.
    #[error("allocation slot '{slot:?}' was historically bound to stable key '{historical_key}'")]
    SlotStableKeyConflict {
        /// Slot claimed by the current declaration.
        slot: Box<AllocationSlotDescriptor>,
        /// Historical stable key for the slot.
        historical_key: StableKey,
        /// Stable key claimed by the current declaration.
        declared_key: StableKey,
    },
    /// Current declaration attempted to revive a retired allocation.
    #[error("stable key '{stable_key}' was explicitly retired and cannot be redeclared")]
    RetiredAllocation {
        /// Retired stable key.
        stable_key: StableKey,
        /// Retired allocation slot.
        slot: Box<AllocationSlotDescriptor>,
    },
    /// Internal claim validation reported an active-allocation conflict where
    /// declaration validation expected only move, reuse, or tombstone conflicts.
    #[error("stable key '{stable_key}' produced an unexpected active-allocation conflict")]
    UnexpectedActiveAllocationConflict {
        /// Active stable key.
        stable_key: StableKey,
        /// Active allocation slot.
        slot: Box<AllocationSlotDescriptor>,
    },
}

/// Validate a committed ledger and current declarations before opening.
///
/// This produces a pre-commit [`ValidatedAllocations`] value: the historical
/// ledger must pass current-format and committed-integrity checks before current
/// declarations are checked against framework policy and ledger history. The
/// result can be staged, but it cannot open storage. Open authority is granted
/// only by [`crate::CommittedAllocations`] after persistence confirmation.
pub fn validate_allocations<P: AllocationPolicy>(
    recovered: &RecoveredLedger,
    snapshot: DeclarationSnapshot,
    policy: &P,
) -> Result<ValidatedAllocations, AllocationValidationError<P::Error>> {
    let ledger = recovered.ledger();

    snapshot
        .validate()
        .map_err(AllocationValidationError::Snapshot)?;

    for declaration in snapshot.declarations() {
        policy
            .validate_key(&declaration.stable_key)
            .map_err(AllocationValidationError::Policy)?;
        policy
            .validate_slot(&declaration.stable_key, &declaration.slot)
            .map_err(AllocationValidationError::Policy)?;

        validate_declaration_history(ledger, declaration)?;
    }

    let (declarations, runtime_fingerprint) = snapshot.into_parts();

    Ok(ValidatedAllocations::new(
        ledger.current_generation,
        declarations,
        runtime_fingerprint,
    ))
}

fn validate_declaration_history<P>(
    ledger: &AllocationLedger,
    declaration: &crate::declaration::AllocationDeclaration,
) -> Result<(), AllocationValidationError<P>> {
    validate_declaration_claim(ledger, declaration)
        .map(|_| ())
        .map_err(|conflict| map_validation_claim_conflict(ledger, declaration, conflict))
}

fn map_validation_claim_conflict<P>(
    ledger: &AllocationLedger,
    declaration: &crate::declaration::AllocationDeclaration,
    conflict: ClaimConflict,
) -> AllocationValidationError<P> {
    let record = claim_conflict_record(ledger, conflict);
    match conflict {
        ClaimConflict::StableKeyMoved { .. } => AllocationValidationError::StableKeySlotConflict {
            stable_key: declaration.stable_key.clone(),
            historical_slot: Box::new(record.slot.clone()),
            declared_slot: Box::new(declaration.slot.clone()),
        },
        ClaimConflict::SlotReused { .. } => AllocationValidationError::SlotStableKeyConflict {
            slot: Box::new(declaration.slot.clone()),
            historical_key: record.stable_key.clone(),
            declared_key: declaration.stable_key.clone(),
        },
        ClaimConflict::Tombstoned { .. } => AllocationValidationError::RetiredAllocation {
            stable_key: declaration.stable_key.clone(),
            slot: Box::new(record.slot.clone()),
        },
        ClaimConflict::ActiveAllocation { .. } => {
            AllocationValidationError::UnexpectedActiveAllocationConflict {
                stable_key: record.stable_key.clone(),
                slot: Box::new(record.slot.clone()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        declaration::AllocationDeclaration,
        ledger::{AllocationHistory, AllocationRecord, AllocationState, GenerationRecord},
        schema::SchemaMetadata,
        slot::AllocationSlotDescriptor,
    };

    #[derive(Debug, Eq, PartialEq)]
    struct TestPolicy;

    impl AllocationPolicy for TestPolicy {
        type Error = &'static str;

        fn validate_key(&self, key: &StableKey) -> Result<(), Self::Error> {
            if key.as_str().starts_with("bad.") {
                return Err("bad key");
            }
            Ok(())
        }

        fn validate_slot(
            &self,
            _key: &StableKey,
            slot: &AllocationSlotDescriptor,
        ) -> Result<(), Self::Error> {
            if slot
                == &AllocationSlotDescriptor::memory_manager_unchecked(
                    crate::MEMORY_MANAGER_INVALID_ID,
                )
            {
                return Err("bad slot");
            }
            Ok(())
        }

        fn validate_reserved_slot(
            &self,
            _key: &StableKey,
            _slot: &AllocationSlotDescriptor,
        ) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    fn ledger(records: Vec<AllocationRecord>) -> AllocationLedger {
        let generations = (1..=7)
            .map(|generation| {
                GenerationRecord::new(
                    generation,
                    if generation == 1 { 0 } else { generation - 1 },
                    None,
                    0,
                    None,
                )
                .expect("generation record")
            })
            .collect();

        AllocationLedger {
            current_generation: 7,
            allocation_history: AllocationHistory::from_parts(records, generations),
        }
    }

    fn declaration(key: &str, id: u8) -> AllocationDeclaration {
        AllocationDeclaration::new(
            key,
            AllocationSlotDescriptor::memory_manager(id).expect("usable slot"),
            None,
            SchemaMetadata::default(),
        )
        .expect("declaration")
    }

    fn active_record(key: &str, id: u8) -> AllocationRecord {
        AllocationRecord::from_declaration(1, declaration(key, id), AllocationState::Active)
            .expect("valid schema metadata")
    }

    fn recovered(records: Vec<AllocationRecord>) -> RecoveredLedger {
        RecoveredLedger::from_trusted_parts(ledger(records), 7)
    }

    #[test]
    fn accepts_matching_historical_owner() {
        let snapshot =
            DeclarationSnapshot::new(vec![declaration("app.users.v1", 100)]).expect("snapshot");

        let validated = validate_allocations(
            &recovered(vec![active_record("app.users.v1", 100)]),
            snapshot,
            &TestPolicy,
        )
        .expect("validated");

        assert_eq!(validated.base_generation(), 7);
    }

    #[test]
    fn omitted_historical_records_do_not_fail_validation() {
        let snapshot =
            DeclarationSnapshot::new(vec![declaration("app.users.v1", 100)]).expect("snapshot");

        validate_allocations(
            &recovered(vec![
                active_record("app.users.v1", 100),
                active_record("app.orders.v1", 101),
            ]),
            snapshot,
            &TestPolicy,
        )
        .expect("omitted records are preserved, not retired");
    }

    #[test]
    fn rejects_same_key_different_slot() {
        let snapshot =
            DeclarationSnapshot::new(vec![declaration("app.users.v1", 101)]).expect("snapshot");

        let err = validate_allocations(
            &recovered(vec![active_record("app.users.v1", 100)]),
            snapshot,
            &TestPolicy,
        )
        .expect_err("conflict");

        assert!(matches!(
            err,
            AllocationValidationError::StableKeySlotConflict { .. }
        ));
    }

    #[test]
    fn rejects_same_slot_different_key() {
        let snapshot =
            DeclarationSnapshot::new(vec![declaration("app.orders.v1", 100)]).expect("snapshot");

        let err = validate_allocations(
            &recovered(vec![active_record("app.users.v1", 100)]),
            snapshot,
            &TestPolicy,
        )
        .expect_err("conflict");

        assert!(matches!(
            err,
            AllocationValidationError::SlotStableKeyConflict { .. }
        ));
    }

    #[test]
    fn rejects_retired_redeclaration() {
        let mut record = active_record("app.users.v1", 100);
        record.state = AllocationState::Retired;
        record.retired_generation = Some(3);
        let snapshot =
            DeclarationSnapshot::new(vec![declaration("app.users.v1", 100)]).expect("snapshot");

        let err = validate_allocations(&recovered(vec![record]), snapshot, &TestPolicy)
            .expect_err("retired");

        assert!(matches!(
            err,
            AllocationValidationError::RetiredAllocation { .. }
        ));
    }

    #[test]
    fn policy_rejections_fail_before_validation_succeeds() {
        let snapshot =
            DeclarationSnapshot::new(vec![declaration("bad.users.v1", 100)]).expect("snapshot");

        let err = validate_allocations(&recovered(Vec::new()), snapshot, &TestPolicy)
            .expect_err("policy failure");

        assert_eq!(err, AllocationValidationError::Policy("bad key"));
    }
}
