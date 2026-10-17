use crate::{
    key::StableKey,
    ledger::{AllocationLedger, ClaimConflict, RecoveredLedger, validate_declaration_claim},
    policy::RuntimeBootstrapPolicy,
    requests::DeclarationSnapshot,
    slot::MemoryManagerSlot,
};

///
/// AllocationValidationError
///
/// Failure to validate declarations against policy and historical ledger facts.
/// Resolution establishes snapshot invariants; recovery establishes
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

// Doctor needs the same checks as bootstrap, but does not consume declarations
// or mint a capability. Keep check ordering and error ownership in one place.
pub fn check_allocations<P: RuntimeBootstrapPolicy>(
    recovered: &RecoveredLedger,
    snapshot: &DeclarationSnapshot,
    policy: &P,
) -> Result<(), AllocationValidationError<P::Error>> {
    for declaration in snapshot.declarations() {
        check_declaration(recovered.ledger(), declaration, policy)?;
    }

    Ok(())
}

pub fn check_declaration<P: RuntimeBootstrapPolicy>(
    ledger: &AllocationLedger,
    declaration: &crate::declaration::AllocationDeclaration,
    policy: &P,
) -> Result<crate::ledger::ClaimOutcome, AllocationValidationError<P::Error>> {
    // Keep callback and history error precedence identical in doctor and staging.
    // Governance is checked against history without invoking application policy.
    if declaration.stable_key.as_str() != crate::IC_MEMORY_LEDGER_STABLE_KEY {
        policy
            .validate_allocation(&declaration.stable_key, &declaration.slot)
            .map_err(AllocationValidationError::Policy)?;
    }
    validate_declaration_claim(ledger, declaration)
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
        ledger::{AllocationRecord, AllocationState},
        slot::MemoryManagerSlot,
    };

    #[derive(Debug, Eq, PartialEq)]
    struct TestPolicy;

    impl RuntimeBootstrapPolicy for TestPolicy {
        type Error = &'static str;

        fn validate_allocation(
            &self,
            key: &StableKey,
            _slot: &MemoryManagerSlot,
        ) -> Result<(), Self::Error> {
            if key.as_str().starts_with("bad.") {
                return Err("bad key");
            }
            assert!(!key.as_str().starts_with("bad."));
            Ok(())
        }

        fn runtime_bootstrap_identity(&self) -> crate::PolicyIdentity {
            crate::PolicyIdentity::new(*b"validation-test.................")
        }
    }

    fn ledger(records: Vec<AllocationRecord>) -> AllocationLedger {
        AllocationLedger {
            current_generation: 7,
            records,
        }
    }

    fn declaration(key: &str, id: u8) -> AllocationDeclaration {
        AllocationDeclaration::new(key, MemoryManagerSlot::new(id).expect("usable slot"))
            .expect("declaration")
    }

    fn active_record(key: &str, id: u8) -> AllocationRecord {
        AllocationRecord::active(&declaration(key, id))
    }

    fn recovered(records: Vec<AllocationRecord>) -> RecoveredLedger {
        RecoveredLedger::from_trusted_ledger(ledger(records))
    }

    #[test]
    fn accepts_matching_historical_owner() {
        let snapshot = DeclarationSnapshot::for_test(vec![declaration("app.users.v1", 100)]);

        check_allocations(
            &recovered(vec![active_record("app.users.v1", 100)]),
            &snapshot,
            &TestPolicy,
        )
        .expect("validated");
    }

    #[test]
    fn omitted_historical_records_do_not_fail_validation() {
        let snapshot = DeclarationSnapshot::for_test(vec![declaration("app.users.v1", 100)]);

        check_allocations(
            &recovered(vec![
                active_record("app.users.v1", 100),
                active_record("app.orders.v1", 101),
            ]),
            &snapshot,
            &TestPolicy,
        )
        .expect("omitted records are preserved, not retired");
    }

    #[test]
    fn rejects_same_key_different_slot() {
        let snapshot = DeclarationSnapshot::for_test(vec![declaration("app.users.v1", 101)]);

        let err = check_allocations(
            &recovered(vec![active_record("app.users.v1", 100)]),
            &snapshot,
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
        let snapshot = DeclarationSnapshot::for_test(vec![declaration("app.orders.v1", 100)]);

        let err = check_allocations(
            &recovered(vec![active_record("app.users.v1", 100)]),
            &snapshot,
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
        let snapshot = DeclarationSnapshot::for_test(vec![declaration("app.users.v1", 100)]);

        let err = check_allocations(&recovered(vec![record]), &snapshot, &TestPolicy)
            .expect_err("retired");

        assert!(matches!(
            err,
            AllocationValidationError::RetiredAllocation { .. }
        ));
    }

    #[test]
    fn policy_rejections_fail_before_validation_succeeds() {
        let snapshot = DeclarationSnapshot::for_test(vec![declaration("bad.users.v1", 100)]);

        let err = check_allocations(&recovered(Vec::new()), &snapshot, &TestPolicy)
            .expect_err("policy failure");

        assert_eq!(err, AllocationValidationError::Policy("bad key"));
    }

    #[test]
    fn full_slot_domain_validates_stages_and_commits() {
        let mut store = crate::LedgerCommitStore::default();
        let recovered = store.recover_or_genesis().unwrap();
        let snapshot = DeclarationSnapshot::for_test(
            (0..=crate::MEMORY_MANAGER_MAX_ID)
                .map(|id| declaration(&format!("app.store{id}.v1"), id))
                .collect(),
        );
        let (staged, _) =
            crate::ledger::stage_generation(recovered, snapshot, &TestPolicy).unwrap();
        assert_eq!(staged.records().len(), 255);

        let committed = store.commit(&staged).unwrap();
        assert_eq!(committed.current_generation(), 1);
        assert_eq!(store.recover().unwrap(), committed);
    }
}
