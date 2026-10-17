use super::{
    AllocationLedger, AllocationRecord, AllocationStageError, AllocationState, ClaimOutcome,
    RecoveredLedger,
};
use crate::{
    RuntimeBootstrapError, RuntimeBootstrapPolicy, declaration::AllocationDeclaration,
    requests::DeclarationSnapshot,
};

// Own the exact recovered ledger throughout validation and staging. A claim's
// checked index is consumed immediately; no proof can be rebound to another
// same-generation history and no second ownership scan is needed.
pub fn stage_generation<P: RuntimeBootstrapPolicy>(
    recovered: RecoveredLedger,
    snapshot: DeclarationSnapshot,
    policy: &P,
) -> Result<(AllocationLedger, Vec<AllocationDeclaration>), RuntimeBootstrapError<P::Error>> {
    let mut next = recovered.into_ledger();
    for declaration in snapshot.declarations() {
        match crate::validation::check_declaration(&next, declaration, policy)? {
            ClaimOutcome::Existing { record_index } => {
                let record = &mut next.records[record_index];
                record.state = AllocationState::Active;
            }
            ClaimOutcome::New => next.records.push(AllocationRecord::active(declaration)),
        }
    }
    // Keep successor failures after all policy/history callbacks, as before.
    // Staged mutation remains local and is discarded on every failure.
    next.validate_bounds().map_err(AllocationStageError::from)?;
    next.current_generation = checked_next_generation(next.current_generation)
        .map_err(|generation| AllocationStageError::GenerationOverflow { generation })?;
    Ok((next, snapshot.into_declarations()))
}

pub const fn checked_next_generation(current_generation: u64) -> Result<u64, u64> {
    match current_generation.checked_add(1) {
        Some(next_generation) => Ok(next_generation),
        None => Err(current_generation),
    }
}

#[cfg(test)]
mod tests {
    use super::stage_generation;
    use crate::requests::DeclarationSnapshot;
    use crate::{
        AllocationDeclaration, AllocationLedger, AllocationValidationError,
        GenericAllocationPolicy, MemoryManagerSlot, RuntimeBootstrapPolicy,
    };
    use crate::{LedgerCommitStore, RuntimeBootstrapError};
    #[derive(Debug, Eq, PartialEq)]
    struct RejectActivePolicy;

    impl RuntimeBootstrapPolicy for RejectActivePolicy {
        type Error = &'static str;

        fn validate_allocation(
            &self,
            _key: &crate::StableKey,
            _slot: &MemoryManagerSlot,
        ) -> Result<(), Self::Error> {
            Err("active slot rejected")
        }

        fn runtime_bootstrap_identity(&self) -> crate::PolicyIdentity {
            crate::PolicyIdentity::new(*b"validation-test.................")
        }
    }

    fn declaration() -> AllocationDeclaration {
        AllocationDeclaration::new(
            "app.users.v1",
            MemoryManagerSlot::new(100).expect("usable slot"),
        )
        .expect("declaration")
    }

    #[test]
    fn validated_commit_matches_recovered_generation() {
        let mut store = LedgerCommitStore::default();
        let recovered = store.recover_or_genesis().unwrap();
        let snapshot = DeclarationSnapshot::for_test(vec![declaration()]);
        let commit = stage_generation(recovered, snapshot, &GenericAllocationPolicy).unwrap();
        let (next, declarations) = commit;
        store.commit_generation(&next).unwrap();
        let recovered = store.recover().unwrap();
        assert_eq!(recovered.current_generation(), 1);
        assert_eq!(recovered.ledger().records().len(), 1);
        assert_eq!(
            recovered.ledger().records()[0],
            crate::AllocationRecord::active(&declaration())
        );
        assert_eq!(declarations, vec![declaration()]);
    }

    #[test]
    fn reservation_policy_alone_does_not_activate_reserved_allocation() {
        let mut store = LedgerCommitStore::default();
        let reserved = crate::AllocationLedger::new(
            1,
            vec![crate::testing::allocation_record(
                &declaration(),
                crate::AllocationState::Reserved,
            )],
        )
        .unwrap();
        store.commit(&reserved).unwrap();
        let recovered = store.recover().unwrap();
        let snapshot = DeclarationSnapshot::for_test(vec![declaration()]);
        let err = stage_generation(recovered, snapshot, &RejectActivePolicy)
            .expect_err("active validation must run");
        assert!(matches!(
            err,
            RuntimeBootstrapError::Validation(AllocationValidationError::Policy(
                "active slot rejected"
            ))
        ));
        assert_eq!(
            store.recover().unwrap().ledger().records()[0].state(),
            crate::AllocationState::Reserved
        );
    }

    #[test]
    fn later_policy_failure_discards_local_staging() {
        let store = LedgerCommitStore::default();
        let prior = store.recover_or_genesis().unwrap();
        // The governance key sorts before this application key, so staging
        // mutates local state before the application policy rejects its slot.
        let snapshot = DeclarationSnapshot::for_test(vec![
            AllocationDeclaration::memory_manager(
                crate::IC_MEMORY_LEDGER_STABLE_KEY,
                crate::MEMORY_MANAGER_LEDGER_ID,
            )
            .unwrap(),
            AllocationDeclaration::memory_manager("zoo.rows.v1", 100).unwrap(),
        ]);
        assert!(matches!(
            stage_generation(prior, snapshot, &RejectActivePolicy),
            Err(RuntimeBootstrapError::Validation(
                AllocationValidationError::Policy("active slot rejected")
            ))
        ));
    }

    #[test]
    fn staging_generation_overflow_follows_validation() {
        let prior = crate::RecoveredLedger::from_trusted_ledger(
            AllocationLedger::new(u64::MAX, Vec::new()).unwrap(),
        );
        let snapshot = DeclarationSnapshot::for_test(vec![declaration()]);
        // Policy rejection wins even when the successor counter is exhausted.
        assert!(matches!(
            stage_generation(
                prior.clone(),
                DeclarationSnapshot::for_test(vec![declaration()]),
                &RejectActivePolicy
            ),
            Err(RuntimeBootstrapError::Validation(
                AllocationValidationError::Policy("active slot rejected")
            ))
        ));
        assert!(matches!(
            stage_generation(prior, snapshot, &GenericAllocationPolicy),
            Err(RuntimeBootstrapError::Staging(
                crate::AllocationStageError::GenerationOverflow {
                    generation: u64::MAX
                }
            ))
        ));
    }
}
