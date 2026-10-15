use super::RuntimeBootstrapError;
use crate::{
    capability::{CommittedAllocations, ValidatedAllocations},
    declaration::DeclarationSnapshot,
    ledger::{LedgerCommitStore, stage_validated_generation},
    policy::RuntimeBootstrapPolicy,
    validation::validate_allocations,
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
    let validated = validate_allocations(&prior, snapshot, policy)?;
    let staged = stage_validated_generation(prior.into_ledger(), &validated)?;
    store.commit_generation(&staged)?;

    Ok(PendingBootstrapCommit {
        validated,
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
    validated: ValidatedAllocations,
}

impl PendingBootstrapCommit {
    /// Confirm that the owning integration durably persisted this commit.
    ///
    /// Calling this method before the stable-memory write succeeds violates the
    /// allocation protocol. The default runtime performs its stable-cell write
    /// before confirmation.
    #[must_use]
    pub fn confirm_persisted(self) -> CommittedAllocations {
        self.validated.confirm_persisted(self.generation)
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

    fn ledger() -> AllocationLedger {
        AllocationLedger::empty_genesis()
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
        let reserved = ledger()
            .stage_reservation_generation(&[declaration()])
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
}
