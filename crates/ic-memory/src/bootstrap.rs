use crate::{
    capability::{CommittedAllocations, ValidatedAllocations},
    declaration::DeclarationSnapshot,
    ledger::{
        AllocationLedger, AllocationStageError, LedgerCommitError, LedgerCommitStore,
        stage_validated_generation,
    },
    policy::AllocationPolicy,
    validation::{AllocationValidationError, validate_allocations},
};
use std::borrow::Cow;

///
/// AllocationBootstrap
///
/// Internal validation and protected commit step used by `MemoryRuntime`.
/// The runtime persists the owning stable-cell record before confirming the
/// pending commit.
///

#[derive(Debug)]
pub struct AllocationBootstrap<'store> {
    store: &'store mut LedgerCommitStore,
}

impl<'store> AllocationBootstrap<'store> {
    /// Build a bootstrap pipeline over a protected ledger commit store.
    pub const fn new(store: &'store mut LedgerCommitStore) -> Self {
        Self { store }
    }

    pub(crate) fn validate_against<P>(
        &mut self,
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
        self.store
            .commit_generation(&staged)
            .map_err(BootstrapError::Ledger)?;

        Ok(PendingBootstrapCommit {
            validated,
            ledger: staged,
        })
    }
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
    /// Staged ledger accepted by the protected generation commit.
    ledger: AllocationLedger,
    /// Validated allocation declarations awaiting persistence confirmation.
    validated: ValidatedAllocations,
}

impl PendingBootstrapCommit {
    /// Borrow the committed logical ledger for diagnostics.
    ///
    /// The persistence owner must write the owning record that contains the
    /// mutated [`crate::LedgerCommitStore`], not serialize this ledger DTO as a
    /// replacement protocol.
    #[must_use]
    #[cfg(test)]
    pub const fn ledger(&self) -> &AllocationLedger {
        &self.ledger
    }

    /// Confirm that the owning integration durably persisted this commit.
    ///
    /// Calling this method before the stable-memory write succeeds violates the
    /// allocation protocol. The default runtime performs its stable-cell write
    /// before confirmation.
    #[must_use]
    pub fn confirm_persisted(self) -> CommittedAllocations {
        self.validated
            .confirm_persisted(self.ledger.current_generation())
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
        AllocationDeclaration, GenericAllocationPolicy, MemoryManagerSlot, SchemaMetadata,
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

        fn validate_reserved_slot(
            &self,
            _key: &crate::StableKey,
            _slot: &MemoryManagerSlot,
        ) -> Result<(), Self::Error> {
            Ok(())
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
        let commit = AllocationBootstrap::new(&mut store)
            .validate_against(recovered, snapshot, &GenericAllocationPolicy)
            .unwrap();
        assert_eq!(commit.ledger().current_generation(), 1);
        assert_eq!(commit.ledger().records().len(), 1);
        assert_eq!(store.recover().unwrap().ledger(), commit.ledger());
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
        let err = AllocationBootstrap::new(&mut store)
            .validate_against(recovered, snapshot, &RejectActivePolicy)
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
