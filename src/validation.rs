use crate::{
    capability::ValidatedAllocations,
    declaration::DeclarationSnapshot,
    key::StableKey,
    ledger::{AllocationLedger, ClaimConflict, RecoveredLedger, validate_declaration_claim},
    policy::AllocationPolicy,
    slot::MemoryManagerSlot,
};

///
/// AllocationValidationError
///
/// Failure to validate declarations against policy and historical ledger facts.
/// Construction and decoding establish snapshot invariants; recovery establishes
/// ledger integrity before this boundary.
///

#[non_exhaustive]
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum AllocationValidationError<P> {
    /// Policy adapter rejected the declaration.
    #[error("allocation policy rejected a declaration")]
    Policy(P),
    /// Stable key was historically bound to a different slot.
    #[error("stable key '{stable_key}' was historically bound to a different allocation slot")]
    StableKeySlotConflict {
        /// Stable key that was redeclared.
        stable_key: StableKey,
        /// Historical slot for the stable key.
        historical_slot: MemoryManagerSlot,
        /// Slot claimed by the current declaration.
        declared_slot: MemoryManagerSlot,
    },
    /// Slot was historically bound to a different stable key.
    #[error("allocation slot '{slot:?}' was historically bound to stable key '{historical_key}'")]
    SlotStableKeyConflict {
        /// Slot claimed by the current declaration.
        slot: MemoryManagerSlot,
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
        slot: MemoryManagerSlot,
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
    check_allocations(recovered, &snapshot, policy)?;
    let (declarations, runtime_fingerprint) = snapshot.into_parts();

    Ok(ValidatedAllocations::new(
        recovered.current_generation(),
        declarations,
        runtime_fingerprint,
    ))
}

// Doctor needs the same checks as bootstrap, but does not consume declarations
// or mint a capability. Keep check ordering and error ownership in one place.
pub fn check_allocations<P: AllocationPolicy>(
    recovered: &RecoveredLedger,
    snapshot: &DeclarationSnapshot,
    policy: &P,
) -> Result<(), AllocationValidationError<P::Error>> {
    let ledger = recovered.ledger();

    for declaration in snapshot.declarations() {
        policy
            .validate_key(&declaration.stable_key)
            .map_err(AllocationValidationError::Policy)?;
        policy
            .validate_slot(&declaration.stable_key, &declaration.slot)
            .map_err(AllocationValidationError::Policy)?;

        validate_declaration_history(ledger, declaration)?;
    }

    Ok(())
}

fn validate_declaration_history<P>(
    ledger: &AllocationLedger,
    declaration: &crate::declaration::AllocationDeclaration,
) -> Result<(), AllocationValidationError<P>> {
    validate_declaration_claim(ledger, declaration)
        .map(|_| ())
        .map_err(|conflict| map_validation_claim_conflict(declaration, conflict))
}

fn map_validation_claim_conflict<P>(
    declaration: &crate::declaration::AllocationDeclaration,
    conflict: ClaimConflict<'_>,
) -> AllocationValidationError<P> {
    match conflict {
        ClaimConflict::StableKeyMoved { record } => {
            AllocationValidationError::StableKeySlotConflict {
                stable_key: declaration.stable_key.clone(),
                historical_slot: record.slot.clone(),
                declared_slot: declaration.slot.clone(),
            }
        }
        ClaimConflict::SlotReused { record } => AllocationValidationError::SlotStableKeyConflict {
            slot: declaration.slot.clone(),
            historical_key: record.stable_key.clone(),
            declared_key: declaration.stable_key.clone(),
        },
        ClaimConflict::Tombstoned { record } => AllocationValidationError::RetiredAllocation {
            stable_key: declaration.stable_key.clone(),
            slot: record.slot.clone(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        declaration::AllocationDeclaration,
        ledger::{AllocationHistory, AllocationRecord, AllocationState, GenerationRecord},
        schema::SchemaMetadata,
        slot::MemoryManagerSlot,
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
            _slot: &MemoryManagerSlot,
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        fn validate_reserved_slot(
            &self,
            _key: &StableKey,
            _slot: &MemoryManagerSlot,
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
            MemoryManagerSlot::new(id).expect("usable slot"),
            None,
            SchemaMetadata::default(),
        )
        .expect("declaration")
    }

    fn active_record(key: &str, id: u8) -> AllocationRecord {
        AllocationRecord::active(1, &declaration(key, id))
    }

    fn recovered(records: Vec<AllocationRecord>) -> RecoveredLedger {
        RecoveredLedger::from_trusted_ledger(ledger(records))
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
        record.state = AllocationState::Retired { generation: 3 };
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

    #[test]
    fn full_slot_domain_validates_stages_and_commits_through_public_boundaries() {
        let mut store = crate::LedgerCommitStore::default();
        let genesis = AllocationLedger::new(0, AllocationHistory::default()).unwrap();
        let recovered = store.recover_or_initialize(&genesis).unwrap();
        let snapshot = DeclarationSnapshot::new(
            (0..=crate::MEMORY_MANAGER_MAX_ID)
                .map(|id| declaration(&format!("app.store{id}.v1"), id))
                .collect(),
        )
        .unwrap();
        let validated = validate_allocations(&recovered, snapshot, &TestPolicy).unwrap();
        let staged = recovered
            .ledger()
            .stage_validated_generation(&validated, None)
            .unwrap();
        assert_eq!(staged.allocation_history().records().len(), 255);
        assert_eq!(
            staged.allocation_history().generations()[0].declaration_count(),
            255
        );
        let committed = store.commit(&staged).unwrap();
        assert_eq!(committed.current_generation(), 1);
        assert_eq!(store.recover().unwrap(), committed);
    }
}
