use crate::{declaration::AllocationDeclaration, key::StableKey, slot::MemoryManagerSlot};
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
/// Its declarations have valid schema metadata and at most 255 unique keys and
/// slots. These facts are established before the proof is constructed.
/// The base generation has passed bounded ownership validation.
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
}

impl ValidatedAllocations {
    pub(crate) fn new(base_generation: u64, declarations: Vec<AllocationDeclaration>) -> Self {
        Self {
            inner: Arc::new(ValidatedState {
                base_generation,
                declarations,
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

    /// Find a validated slot by stable key.
    #[must_use]
    pub fn slot_for(&self, key: &StableKey) -> Option<&MemoryManagerSlot> {
        slot_for_key(self.declarations(), key.as_str())
    }

    pub(crate) const fn confirm_persisted(self, generation: u64) -> CommittedAllocations {
        CommittedAllocations {
            validated: self,
            generation,
        }
    }
}

// Typed capability callers and the runtime's validated borrowed input share
// one lookup. Comparing text needs no temporary owned StableKey or second index.
pub fn slot_for_key<'a>(
    declarations: &'a [AllocationDeclaration],
    key: &str,
) -> Option<&'a MemoryManagerSlot> {
    declarations
        .iter()
        .find(|declaration| declaration.stable_key.as_str() == key)
        .map(|declaration| &declaration.slot)
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

    /// Find a committed slot by stable key.
    #[must_use]
    pub fn slot_for(&self, key: &StableKey) -> Option<&MemoryManagerSlot> {
        self.validated.slot_for(key)
    }

    // Runtime publication exposes application allocations only. Manual commit
    // owners retain the complete capability returned by persistence confirmation.
    pub(crate) fn into_application_allocations(mut self) -> Self {
        Arc::make_mut(&mut self.validated.inner)
            .declarations
            .retain(|declaration| !crate::is_ic_memory_stable_key(declaration.stable_key.as_str()));
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
        );
        let committed = validated.clone().confirm_persisted(2);
        let filtered = committed.clone().into_application_allocations();

        assert_eq!(validated.declarations().len(), 2);
        assert_eq!(committed.declarations().len(), 2);
        assert_eq!(filtered.declarations().len(), 1);
        assert_eq!(
            filtered.declarations()[0].stable_key().as_str(),
            "app.rows.v1"
        );
        assert_eq!(filtered.generation(), committed.generation());
    }
}
