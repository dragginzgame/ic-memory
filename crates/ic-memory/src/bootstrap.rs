use crate::{
    capability::{CommittedAllocations, ValidatedAllocations},
    declaration::DeclarationSnapshot,
    ledger::{
        AllocationStageError, LedgerCommitError, LedgerCommitStore, stage_validated_generation,
    },
    policy::AllocationPolicy,
    validation::{AllocationValidationError, validate_allocations},
};
use std::borrow::Cow;

/// Validate and stage one commit; the runtime persists before confirmation.
pub fn prepare_commit<P>(
    store: &mut LedgerCommitStore,
    prior: crate::RecoveredLedger,
    snapshot: DeclarationSnapshot,
    policy: &P,
) -> Result<PendingBootstrapCommit, BootstrapError<P::Error>>
where
    P: AllocationPolicy,
{
    let validated =
        validate_allocations(&prior, snapshot, policy).map_err(BootstrapError::Validation)?;
    let staged = stage_validated_generation(Cow::Owned(prior.into_ledger()), &validated)
        .map_err(BootstrapError::Staging)?;
    store
        .commit_generation(&staged)
        .map_err(BootstrapError::Ledger)?;

    Ok(PendingBootstrapCommit {
        validated,
        generation: staged.current_generation(),
    })
}

///
/// PendingBootstrapCommit
///
/// Pending result of a successful generic allocation bootstrap commit.
///
/// The embedded [`crate::LedgerCommitStore`] has advanced, but this generic
/// layer does not own stable-memory IO. Persist the owning record first, then
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

///
/// BootstrapError
///
/// Failure to recover, validate, or commit an allocation generation.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum BootstrapError<P> {
    /// Ledger recovery or protected commit failed.
    #[error(transparent)]
    Ledger(LedgerCommitError),
    /// Policy or historical allocation validation failed.
    #[error(transparent)]
    Validation(AllocationValidationError<P>),
    /// Validated declarations could not be staged against the recovered ledger.
    #[error(transparent)]
    Staging(AllocationStageError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AllocationDeclaration, AllocationLedger, GenericAllocationPolicy, MemoryManagerSlot,
        SchemaMetadata,
    };
    #[derive(Debug, Eq, PartialEq)]
    struct RejectActivePolicy;

    impl AllocationPolicy for RejectActivePolicy {
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
    }

    fn ledger() -> AllocationLedger {
        AllocationLedger::empty_genesis()
    }

    fn declaration() -> AllocationDeclaration {
        AllocationDeclaration::new(
            "app.users.v1",
            MemoryManagerSlot::new(100).expect("usable slot"),
            None,
            SchemaMetadata::default(),
        )
        .expect("declaration")
    }

    #[test]
    fn validated_commit_matches_recovered_generation() {
        let mut store = LedgerCommitStore::default();
        let recovered = store.recover_or_initialize(&ledger()).unwrap();
        let snapshot = DeclarationSnapshot::new(vec![declaration()]).unwrap();
        let commit = crate::bootstrap::prepare_commit(
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
        let snapshot = DeclarationSnapshot::new(vec![declaration()]).unwrap();
        let err =
            crate::bootstrap::prepare_commit(&mut store, recovered, snapshot, &RejectActivePolicy)
                .expect_err("active validation must run");
        assert!(matches!(
            err,
            BootstrapError::Validation(AllocationValidationError::Policy("active slot rejected"))
        ));
        assert_eq!(store, before);
        assert_eq!(
            store.recover().unwrap().ledger().records()[0].state(),
            crate::AllocationState::Reserved
        );
    }
}
