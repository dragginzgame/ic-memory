use crate::{declaration::AllocationDeclaration, key::StableKey, slot::AllocationSlotDescriptor};
use std::sync::Arc;

///
/// ValidatedAllocations
///
/// Pre-commit allocation declarations accepted by policy and historical ledger
/// validation.
///
/// This value is produced by [`crate::validate_allocations`] and may be staged
/// into the next ledger generation. It cannot open storage. Only a
/// [`CommittedAllocations`] capability confirmed after persistence can do that.
///
/// This is an in-memory capability, not a serde DTO. It has no public
/// constructor and should only be produced by validation or bootstrap paths.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedAllocations {
    inner: Arc<ValidatedState>,
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
/// confirming a successful [`crate::PendingBootstrapCommit`]. A
/// [`crate::MemoryRuntime`] stores it only after that runtime's stable-cell write
/// succeeds.
///
/// Its immutable declarations have validated keys, slots and diagnostic
/// metadata, with unique keys and slots. Consumers may rely on those invariants
/// without rebuilding uniqueness sets. The capability does not validate live
/// store bytes, application schemas, journals or lifecycle readiness.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommittedAllocations {
    validated: ValidatedAllocations,
    generation: u64,
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
        let mut state = Arc::unwrap_or_clone(self.validated.inner);
        state
            .declarations
            .retain(|declaration| !declaration.stable_key.as_str().starts_with(prefix));
        self.validated.inner = Arc::new(state);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filtering_governance_does_not_change_shared_capabilities() {
        let validated = ValidatedAllocations::new(
            1,
            vec![
                AllocationDeclaration::memory_manager(
                    crate::IC_MEMORY_LEDGER_STABLE_KEY,
                    0,
                    "ledger",
                )
                .unwrap(),
                AllocationDeclaration::memory_manager("app.rows.v1", 100, "rows").unwrap(),
            ],
            Some("host".to_string()),
        );
        let committed = validated.clone().confirm_persisted(2);
        let filtered = committed
            .clone()
            .without_stable_key_prefix(crate::IC_MEMORY_STABLE_KEY_PREFIX);

        assert_eq!(validated.declarations().len(), 2);
        assert_eq!(committed.declarations().len(), 2);
        assert_eq!(filtered.declarations().len(), 1);
        assert_eq!(
            filtered.declarations()[0].stable_key().as_str(),
            "app.rows.v1"
        );
        assert_eq!(filtered.generation(), committed.generation());
        assert_eq!(
            filtered.runtime_fingerprint(),
            committed.runtime_fingerprint()
        );
    }
}
