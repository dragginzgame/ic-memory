use crate::{
    declaration::AllocationDeclaration, key::StableKey, slot::AllocationSlotDescriptor,
    substrate::StorageSubstrate,
};
use std::sync::Arc;

///
/// ValidatedAllocations
///
/// Pre-commit allocation declarations accepted by policy and historical ledger
/// validation.
///
/// This value is produced by [`crate::validate_allocations`] and may be staged
/// into the next ledger generation. It cannot construct an
/// [`AllocationSession`] or open storage. Only a [`CommittedAllocations`]
/// capability confirmed after persistence can do that.
///
/// This is an in-memory capability, not a serde DTO. It has no public
/// constructor and should only be produced by validation or bootstrap paths.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedAllocations {
    inner: Arc<ValidatedState>,
    _private: (),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ValidatedState {
    /// Recovered generation against which these declarations were validated.
    base_generation: u64,
    /// Validated declarations.
    declarations: Vec<AllocationDeclaration>,
    /// Optional binary/runtime identity for generation diagnostics.
    runtime_fingerprint: Option<String>,
}

impl ValidatedAllocations {
    pub(crate) fn new(
        base_generation: u64,
        declarations: Vec<AllocationDeclaration>,
        runtime_fingerprint: Option<String>,
    ) -> Self {
        Self {
            inner: Arc::new(ValidatedState {
                base_generation,
                declarations,
                runtime_fingerprint,
            }),
            _private: (),
        }
    }

    /// Return the recovered generation used as the validation base.
    #[must_use]
    pub fn base_generation(&self) -> u64 {
        self.inner.base_generation
    }

    /// Borrow the validated declarations.
    #[must_use]
    pub fn declarations(&self) -> &[AllocationDeclaration] {
        &self.inner.declarations
    }

    /// Borrow the optional runtime fingerprint.
    #[must_use]
    pub fn runtime_fingerprint(&self) -> Option<&str> {
        self.inner.runtime_fingerprint.as_deref()
    }

    /// Find a validated slot by stable key.
    #[must_use]
    pub fn slot_for(&self, key: &StableKey) -> Option<&AllocationSlotDescriptor> {
        self.declarations()
            .iter()
            .find(|declaration| &declaration.stable_key == key)
            .map(|declaration| &declaration.slot)
    }

    pub(crate) const fn confirm_persisted(self, generation: u64) -> CommittedAllocations {
        CommittedAllocations {
            validated: self,
            generation,
            _private: (),
        }
    }
}

///
/// CommittedAllocations
///
/// Allocation-open capability confirmed after the validated ledger generation
/// was persisted.
///
/// This type is not serializable, default-constructible, or publicly
/// constructible. Generic persistence owners obtain it only by explicitly
/// confirming a successful [`crate::PendingBootstrapCommit`]. The default runtime
/// publishes it only after its stable cell write succeeds.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommittedAllocations {
    validated: ValidatedAllocations,
    generation: u64,
    _private: (),
}

impl CommittedAllocations {
    /// Return the persisted ledger generation that grants this capability.
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    /// Borrow the committed allocation declarations.
    #[must_use]
    pub fn declarations(&self) -> &[AllocationDeclaration] {
        self.validated.declarations()
    }

    /// Borrow the optional runtime fingerprint.
    #[must_use]
    pub fn runtime_fingerprint(&self) -> Option<&str> {
        self.validated.runtime_fingerprint()
    }

    /// Find a committed slot by stable key.
    #[must_use]
    pub fn slot_for(&self, key: &StableKey) -> Option<&AllocationSlotDescriptor> {
        self.validated.slot_for(key)
    }

    pub(crate) fn without_stable_key_prefix(mut self, prefix: &str) -> Self {
        let mut state = (*self.validated.inner).clone();
        state
            .declarations
            .retain(|declaration| !declaration.stable_key.as_str().starts_with(prefix));
        self.validated.inner = Arc::new(state);
        self
    }
}

///
/// AllocationSession
///
/// Persisted allocation capability required before opening allocation slots.
///
/// Integrations should construct sessions only after recovering the ledger,
/// validating declarations, and committing the next generation. Opening storage
/// through this type keeps handle creation tied to the validated stable-key
/// snapshot.
pub struct AllocationSession<S: StorageSubstrate> {
    substrate: S,
    committed: CommittedAllocations,
}

impl<S: StorageSubstrate> AllocationSession<S> {
    /// Construct a session from a substrate and committed allocation set.
    #[must_use]
    pub const fn new(substrate: S, committed: CommittedAllocations) -> Self {
        Self {
            substrate,
            committed,
        }
    }

    /// Borrow the committed allocation set.
    #[must_use]
    pub const fn committed(&self) -> &CommittedAllocations {
        &self.committed
    }

    /// Open an allocation by stable key.
    pub fn open(
        &self,
        key: &StableKey,
    ) -> Result<S::MemoryHandle, AllocationSessionError<S::Error>> {
        let slot = self
            .committed
            .slot_for(key)
            .ok_or_else(|| AllocationSessionError::UnknownStableKey(key.clone()))?;
        self.substrate
            .open_slot(slot)
            .map_err(AllocationSessionError::Substrate)
    }
}

///
/// AllocationSessionError
///
/// Failure to open through a committed allocation session.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum AllocationSessionError<E> {
    /// Stable key was not part of the committed allocation snapshot.
    #[error("stable key '{0}' was not committed for this allocation session")]
    UnknownStableKey(StableKey),
    /// Storage substrate failed to open the committed slot.
    #[error("storage substrate failed to open allocation slot")]
    Substrate(E),
}
