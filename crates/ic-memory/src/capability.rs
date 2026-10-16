use crate::{declaration::AllocationDeclaration, key::StableKey, slot::MemoryManagerSlot};
use std::sync::Arc;

// Typed capability callers and the runtime's validated borrowed input share
// one lookup. Comparing text needs no temporary owned StableKey or second index.
pub fn declaration_for_key<'a>(
    declarations: &'a [AllocationDeclaration],
    key: &str,
) -> Option<&'a AllocationDeclaration> {
    declarations
        .iter()
        .find(|declaration| declaration.stable_key.as_str() == key)
}

///
/// CommittedAllocations
///
/// Allocation-open capability confirmed after the validated ledger generation
/// was persisted.
///
/// This type is not serializable, default-constructible, or publicly
/// constructible. [`crate::MemoryRuntime`] publishes it only after its
/// stable-cell write succeeds; callers cannot manually confirm persistence.
///
/// Its immutable declarations have validated keys, slots, with unique keys and slots. Consumers may rely on those invariants
/// without rebuilding uniqueness sets. The capability does not validate live
/// store bytes, application schemas, journals or lifecycle readiness.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommittedAllocations {
    declarations: Arc<Vec<AllocationDeclaration>>,
    generation: u64,
}

impl CommittedAllocations {
    pub(crate) fn from_persisted(
        mut declarations: Vec<AllocationDeclaration>,
        generation: u64,
    ) -> Self {
        // Governance remains durable ownership, never application-open authority.
        declarations
            .retain(|declaration| !crate::is_ic_memory_stable_key(declaration.stable_key.as_str()));
        Self {
            declarations: Arc::new(declarations),
            generation,
        }
    }

    /// Return the persisted ledger generation that grants this capability.
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    /// Borrow the committed allocation declarations.
    #[must_use]
    pub fn declarations(&self) -> &[AllocationDeclaration] {
        &self.declarations
    }

    /// Find a committed slot by stable key.
    #[must_use]
    pub fn slot_for(&self, key: &StableKey) -> Option<&MemoryManagerSlot> {
        declaration_for_key(self.declarations(), key.as_str()).map(AllocationDeclaration::slot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persistence_confirmation_publishes_shared_application_authority() {
        let declarations = vec![
            AllocationDeclaration::memory_manager(crate::IC_MEMORY_LEDGER_STABLE_KEY, 0).unwrap(),
            AllocationDeclaration::memory_manager("app.rows.v1", 100).unwrap(),
        ];
        let committed = CommittedAllocations::from_persisted(declarations, 2);
        let shared = committed.clone();

        assert_eq!(committed.declarations().len(), 1);
        assert_eq!(shared, committed);
        assert!(std::ptr::eq(
            shared.declarations(),
            committed.declarations()
        ));
        assert_eq!(
            committed.declarations()[0].stable_key().as_str(),
            "app.rows.v1"
        );
        assert_eq!(committed.generation(), 2);
        assert_eq!(
            committed
                .slot_for(&StableKey::parse("app.rows.v1").unwrap())
                .unwrap()
                .id(),
            100
        );
        assert!(
            committed
                .slot_for(&StableKey::parse(crate::IC_MEMORY_LEDGER_STABLE_KEY).unwrap())
                .is_none()
        );
    }
}
