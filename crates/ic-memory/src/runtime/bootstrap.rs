use super::RuntimeBootstrapError;
use crate::{
    capability::CommittedAllocations,
    declaration::{AllocationDeclaration, DeclarationSnapshot},
    ledger::{LedgerCommitStore, stage_generation},
    policy::RuntimeBootstrapPolicy,
};

/// Validate and stage one commit; the runtime persists before confirmation.
pub fn prepare_commit<P>(
    store: &mut LedgerCommitStore,
    prior: crate::RecoveredLedger,
    snapshot: DeclarationSnapshot,
    policy: &P,
) -> Result<PendingBootstrapCommit, RuntimeBootstrapError<P::Error>>
where
    P: RuntimeBootstrapPolicy,
{
    let (staged, declarations) = stage_generation(prior, snapshot, policy)?;
    store.commit_generation(&staged)?;

    Ok(PendingBootstrapCommit {
        declarations,
        generation: staged.current_generation(),
    })
}

///
/// PendingBootstrapCommit
///
/// Pending result of a successful runtime allocation bootstrap commit.
///
/// The owning [`crate::LedgerCommitStore`] has advanced. The runtime must finish
/// stable-memory IO before publishing authority. Persist the owning record first, then
/// call [`PendingBootstrapCommit::confirm_persisted`] to mint the allocation-open
/// capability.
///

#[derive(Debug, Eq, PartialEq)]
#[must_use = "persist the owning ledger record, then confirm_persisted before opening allocations"]
pub struct PendingBootstrapCommit {
    /// Generation accepted by the protected commit, awaiting stable persistence.
    generation: u64,
    /// Validated allocation declarations awaiting persistence confirmation.
    declarations: Vec<AllocationDeclaration>,
}

impl PendingBootstrapCommit {
    /// Confirm that the owning integration durably persisted this commit.
    ///
    /// Calling this method before the stable-memory write succeeds violates the
    /// allocation protocol. The owning runtime performs its stable-cell write
    /// before confirmation.
    #[must_use]
    pub fn confirm_persisted(self) -> CommittedAllocations {
        CommittedAllocations::from_persisted(self.declarations, self.generation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AllocationDeclaration, AllocationLedger, AllocationValidationError,
        GenericAllocationPolicy, MemoryManagerSlot, SchemaMetadata,
    };
    #[derive(Debug, Eq, PartialEq)]
    struct RejectActivePolicy;

    impl RuntimeBootstrapPolicy for RejectActivePolicy {
        type Error = &'static str;

        fn validate_key(&self, _key: &crate::StableKey) -> Result<(), Self::Error> {
            Ok(())
        }

        fn validate_slot(
            &self,
            _key: &crate::StableKey,
            _slot: &MemoryManagerSlot,
        ) -> Result<(), Self::Error> {
            Err("active slot rejected")
        }

        fn runtime_bootstrap_identity(
            &self,
        ) -> Result<crate::PolicyIdentity, crate::PolicyIdentityError> {
            crate::PolicyIdentity::new("validation-test", 1)
        }
    }

    fn declaration() -> AllocationDeclaration {
        AllocationDeclaration::new(
            "app.users.v1",
            MemoryManagerSlot::new(100).expect("usable slot"),
            SchemaMetadata::default(),
        )
        .expect("declaration")
    }

    #[test]
    fn validated_commit_matches_recovered_generation() {
        let mut store = LedgerCommitStore::default();
        let recovered = store.recover_or_genesis().unwrap();
        let snapshot = DeclarationSnapshot::canonicalize(vec![declaration()]).unwrap();
        let commit = crate::runtime::bootstrap::prepare_commit(
            &mut store,
            recovered,
            snapshot,
            &GenericAllocationPolicy,
        )
        .unwrap();
        let recovered = store.recover().unwrap();
        assert_eq!(recovered.current_generation(), 1);
        assert_eq!(recovered.ledger().records().len(), 1);
        assert_eq!(
            recovered.ledger().records()[0],
            crate::AllocationRecord::active(&declaration())
        );
        assert_eq!(commit.confirm_persisted().generation(), 1);
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
        let before = store.clone();
        let recovered = store.recover().unwrap();
        let snapshot = DeclarationSnapshot::canonicalize(vec![declaration()]).unwrap();
        let err = crate::runtime::bootstrap::prepare_commit(
            &mut store,
            recovered,
            snapshot,
            &RejectActivePolicy,
        )
        .expect_err("active validation must run");
        assert!(matches!(
            err,
            RuntimeBootstrapError::Validation(AllocationValidationError::Policy(
                "active slot rejected"
            ))
        ));
        assert_eq!(store, before);
        assert_eq!(
            store.recover().unwrap().ledger().records()[0].state(),
            crate::AllocationState::Reserved
        );
    }

    #[test]
    fn later_policy_failure_discards_local_staging_before_protected_commit() {
        let mut store = LedgerCommitStore::default();
        let prior = store.recover_or_genesis().unwrap();
        // The governance key sorts before this application key, so staging
        // mutates local state before the application policy rejects its slot.
        let snapshot = DeclarationSnapshot::canonicalize(vec![
            AllocationDeclaration::memory_manager(
                crate::IC_MEMORY_LEDGER_STABLE_KEY,
                crate::MEMORY_MANAGER_LEDGER_ID,
            )
            .unwrap(),
            AllocationDeclaration::memory_manager("zoo.rows.v1", 100).unwrap(),
        ])
        .unwrap();
        let before = store.clone();
        assert!(matches!(
            prepare_commit(&mut store, prior, snapshot, &RejectActivePolicy),
            Err(RuntimeBootstrapError::Validation(
                AllocationValidationError::Policy("active slot rejected")
            ))
        ));
        assert_eq!(store, before);
    }

    #[test]
    fn staging_generation_overflow_preserves_protected_store() {
        let mut store = LedgerCommitStore::default();
        let prior = crate::RecoveredLedger::from_trusted_ledger(
            AllocationLedger::new(u64::MAX, Vec::new()).unwrap(),
        );
        let snapshot = DeclarationSnapshot::canonicalize(vec![declaration()]).unwrap();
        // Policy rejection wins even when the successor counter is exhausted.
        assert!(matches!(
            prepare_commit(
                &mut store,
                prior.clone(),
                DeclarationSnapshot::canonicalize(vec![declaration()]).unwrap(),
                &RejectActivePolicy
            ),
            Err(RuntimeBootstrapError::Validation(
                AllocationValidationError::Policy("active slot rejected")
            ))
        ));
        let before = store.clone();
        assert!(matches!(
            prepare_commit(&mut store, prior, snapshot, &GenericAllocationPolicy),
            Err(RuntimeBootstrapError::Staging(
                crate::AllocationStageError::GenerationOverflow {
                    generation: u64::MAX
                }
            ))
        ));
        assert_eq!(store, before);
    }
}
