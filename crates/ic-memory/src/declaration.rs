use crate::{
    key::{StableKey, StableKeyError},
    slot::{MemoryManagerSlot, MemoryManagerSlotError},
};
use serde::{Deserialize, Serialize};

///
/// AllocationDeclaration
///
/// Checked runtime claim that a stable key should own an allocation slot.
///
/// Declarations are supplied by the current binary before opening storage.
/// Construction and decoding check the stable key and slot. A declaration
/// becomes authoritative only after validation against the recovered ledger
/// and commitment in a generation.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AllocationDeclaration {
    /// Durable stable key.
    pub(crate) stable_key: StableKey,
    /// Claimed allocation slot.
    pub(crate) slot: MemoryManagerSlot,
}

impl AllocationDeclaration {
    /// Build a declaration from raw parts after validating allocation identity.
    pub fn new(
        stable_key: impl AsRef<str>,
        slot: MemoryManagerSlot,
    ) -> Result<Self, DeclarationSnapshotError> {
        let stable_key = StableKey::parse(stable_key).map_err(DeclarationSnapshotError::Key)?;
        Ok(Self { stable_key, slot })
    }

    /// Build a checked `MemoryManager` declaration with a usable slot.
    pub fn memory_manager(
        stable_key: impl AsRef<str>,
        id: u8,
    ) -> Result<Self, DeclarationSnapshotError> {
        let slot =
            MemoryManagerSlot::new(id).map_err(DeclarationSnapshotError::MemoryManagerSlot)?;
        Self::new(stable_key, slot)
    }

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

///
/// DeclarationSnapshotError
///
/// Checked allocation identity construction failure.
///

#[non_exhaustive]
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum DeclarationSnapshotError {
    /// Stable-key grammar failure.
    #[error(transparent)]
    Key(StableKeyError),
    /// `MemoryManager` slot validation failure.
    #[error(transparent)]
    MemoryManagerSlot(MemoryManagerSlotError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::slot::MemoryManagerSlot;

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
