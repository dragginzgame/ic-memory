use crate::{
    declaration::AllocationDeclaration, key::StableKey, slot::AllocationSlotDescriptor,
    substrate::StorageSubstrate,
};
use std::sync::Arc;

///
/// ValidatedAllocations
///
/// Allocation declarations accepted by policy and historical ledger validation.
///
/// This value is produced by [`crate::validate_allocations`] and is the bridge
/// between declaration validation and opening storage. It is not a durable
/// ledger record; staging commits it into the next generation before an
/// integration should expose memory handles.
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
    /// Committed generation that validated these allocations.
    generation: u64,
    /// Validated declarations.
    declarations: Vec<AllocationDeclaration>,
    /// Optional binary/runtime identity for generation diagnostics.
    runtime_fingerprint: Option<String>,
}

impl ValidatedAllocations {
    pub(crate) fn new(
        generation: u64,
        declarations: Vec<AllocationDeclaration>,
        runtime_fingerprint: Option<String>,
    ) -> Self {
        Self {
            inner: Arc::new(ValidatedState {
                generation,
                declarations,
                runtime_fingerprint,
            }),
            _private: (),
        }
    }

    pub(crate) fn with_generation(self, generation: u64) -> Self {
        let mut state = (*self.inner).clone();
        state.generation = generation;
        Self {
            inner: Arc::new(state),
            _private: (),
        }
    }

    pub(crate) fn without_stable_key(self, stable_key: &str) -> Self {
        let mut state = (*self.inner).clone();
        state
            .declarations
            .retain(|declaration| declaration.stable_key.as_str() != stable_key);
        Self {
            inner: Arc::new(state),
            _private: (),
        }
    }

    /// Return the committed generation that validated these allocations.
    #[must_use]
    pub fn generation(&self) -> u64 {
        self.inner.generation
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
}

///
/// AllocationSession
///
/// Validated capability required before opening allocation slots.
///
/// Integrations should construct sessions only after recovering the ledger,
/// validating declarations, and committing the next generation. Opening storage
/// through this type keeps handle creation tied to the validated stable-key
/// snapshot.
pub struct AllocationSession<S: StorageSubstrate> {
    substrate: S,
    validated: ValidatedAllocations,
}

impl<S: StorageSubstrate> AllocationSession<S> {
    /// Construct a session from a substrate and validated allocation set.
    #[must_use]
    pub const fn new(substrate: S, validated: ValidatedAllocations) -> Self {
        Self {
            substrate,
            validated,
        }
    }

    /// Borrow the validated allocation set.
    #[must_use]
    pub const fn validated(&self) -> &ValidatedAllocations {
        &self.validated
    }

    /// Open an allocation by stable key.
    pub fn open(
        &self,
        key: &StableKey,
    ) -> Result<S::MemoryHandle, AllocationSessionError<S::Error>> {
        let slot = self
            .validated
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
/// Failure to open through a validated allocation session.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum AllocationSessionError<E> {
    /// Stable key was not part of the validated allocation snapshot.
    #[error("stable key '{0}' was not validated for this allocation session")]
    UnknownStableKey(StableKey),
    /// Storage substrate failed to open the validated slot.
    #[error("storage substrate failed to open allocation slot")]
    Substrate(E),
}
