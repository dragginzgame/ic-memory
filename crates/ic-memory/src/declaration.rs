use crate::{key::StableKey, slot::MemoryManagerSlot};
use serde::Serialize;

///
/// AllocationDeclaration
///
/// Resolved key-to-slot binding borrowed from a committed runtime.
///
/// Production inputs are key-only [`crate::MemoryRequest`] values. The runtime
/// resolves slots, validates recovered ownership and persists its generation
/// before exposing these bindings. Numeric construction and decoding are
/// available only with `test-support` for explicit fixtures.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(any(test, feature = "test-support"), derive(serde::Deserialize))]
#[serde(deny_unknown_fields)]
pub struct AllocationDeclaration {
    /// Durable stable key.
    pub(crate) stable_key: StableKey,
    /// Claimed allocation slot.
    pub(crate) slot: MemoryManagerSlot,
}

impl AllocationDeclaration {
    /// Return the durable stable key claimed by this declaration.
    #[must_use]
    pub const fn stable_key(&self) -> &StableKey {
        &self.stable_key
    }

    /// Return the allocation slot claimed by this declaration.
    #[must_use]
    pub const fn slot(&self) -> &MemoryManagerSlot {
        &self.slot
    }
}

// Borrowed runtime queries share
// one lookup. Comparing text needs no temporary owned StableKey or second index.
pub fn declaration_for_key<'a>(
    declarations: &'a [AllocationDeclaration],
    key: &str,
) -> Option<&'a AllocationDeclaration> {
    declarations
        .iter()
        .find(|declaration| declaration.stable_key.as_str() == key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{slot::MemoryManagerSlot, testing::DeclarationSnapshotError};

    fn declaration(key: &str, id: u8) -> AllocationDeclaration {
        AllocationDeclaration::new(key, MemoryManagerSlot::new(id).expect("usable slot"))
            .expect("declaration")
    }

    #[test]
    fn memory_manager_declaration_constructor_builds_common_declaration() {
        let declaration =
            AllocationDeclaration::memory_manager("app.orders.v1", 100).expect("declaration");

        assert_eq!(declaration.stable_key.as_str(), "app.orders.v1");
        assert_eq!(
            declaration.slot,
            MemoryManagerSlot::new(100).expect("usable slot")
        );
    }

    #[test]
    fn memory_manager_declaration_constructor_rejects_invalid_slot() {
        let err = AllocationDeclaration::memory_manager("app.orders.v1", u8::MAX)
            .expect_err("sentinel must fail");

        assert!(matches!(
            err,
            DeclarationSnapshotError::MemoryManagerSlot(_)
        ));
    }

    #[test]
    fn declaration_decode_rejects_unusable_memory_manager_slot() {
        let mut value = serde_json::to_value(declaration("app.orders.v1", 100)).unwrap();
        value["slot"] = serde_json::json!(255);
        assert!(serde_json::from_value::<AllocationDeclaration>(value.clone()).is_err());
        let bytes = crate::test_cbor::to_vec(&value).unwrap();
        assert!(crate::test_cbor::from_slice::<AllocationDeclaration>(&bytes).is_err());
    }
}
