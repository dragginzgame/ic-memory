use crate::{
    key::{StableKey, StableKeyError},
    schema::SchemaMetadata,
    slot::{MemoryManagerSlot, MemoryManagerSlotError},
};
use serde::{Deserialize, Serialize};

///
/// AllocationDeclaration
///
/// Checked runtime claim that a stable key should own an allocation slot.
///
/// Declarations are supplied by the current binary before opening storage.
/// Construction and decoding check the stable key, slot and schema
/// metadata. A declaration becomes authoritative only after validation against
/// the recovered ledger and commitment in a generation.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AllocationDeclaration {
    /// Durable stable key.
    pub(crate) stable_key: StableKey,
    /// Claimed allocation slot.
    pub(crate) slot: MemoryManagerSlot,
    /// Optional diagnostic schema metadata.
    pub(crate) schema: SchemaMetadata,
}

impl AllocationDeclaration {
    /// Build a declaration from raw parts after validating diagnostic metadata.
    pub fn new(
        stable_key: impl AsRef<str>,
        slot: MemoryManagerSlot,
        schema: SchemaMetadata,
    ) -> Result<Self, DeclarationSnapshotError> {
        let stable_key = StableKey::parse(stable_key).map_err(DeclarationSnapshotError::Key)?;
        Ok(Self {
            stable_key,
            slot,
            schema,
        })
    }

    /// Build a checked `MemoryManager` declaration with default schema metadata.
    pub fn memory_manager(
        stable_key: impl AsRef<str>,
        id: u8,
    ) -> Result<Self, DeclarationSnapshotError> {
        Self::memory_manager_with_schema(stable_key, id, SchemaMetadata::default())
    }

    /// Build a checked `MemoryManager` declaration with schema metadata.
    pub fn memory_manager_with_schema(
        stable_key: impl AsRef<str>,
        id: u8,
        schema: SchemaMetadata,
    ) -> Result<Self, DeclarationSnapshotError> {
        let slot =
            MemoryManagerSlot::new(id).map_err(DeclarationSnapshotError::MemoryManagerSlot)?;
        Self::new(stable_key, slot, schema)
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

    /// Return the optional schema metadata.
    #[must_use]
    pub const fn schema(&self) -> &SchemaMetadata {
        &self.schema
    }
}

///
/// DeclarationSnapshot
///
/// Internal resolved declarations with bounded count and unique keys and slots.
/// Resolution owns canonical order. A snapshot is not permission to open storage.
/// [`crate::MemoryRuntime::bootstrap`] owns validation and persistence before
/// publishing committed allocation authority.
///

#[derive(Debug, Eq, PartialEq)]
pub struct DeclarationSnapshot {
    /// Runtime declarations.
    declarations: Vec<AllocationDeclaration>,
}

impl DeclarationSnapshot {
    pub(crate) fn canonicalize(
        mut declarations: Vec<AllocationDeclaration>,
    ) -> Result<Self, DeclarationSnapshotError> {
        if declarations.len() > crate::constants::MAX_ALLOCATIONS {
            return Err(DeclarationSnapshotError::TooManyDeclarations);
        }
        crate::sort::sort_unstable_by(&mut declarations, |a, b| a.stable_key.cmp(&b.stable_key));
        let mut slots = [false; crate::constants::MAX_ALLOCATIONS];
        for (index, declaration) in declarations.iter().enumerate() {
            let occupied = &mut slots[usize::from(declaration.slot.id())];
            if *occupied {
                return Err(DeclarationSnapshotError::DuplicateSlot(
                    declaration.slot.clone(),
                ));
            }
            *occupied = true;
            if index > 0 && declarations[index - 1].stable_key == declaration.stable_key {
                return Err(DeclarationSnapshotError::DuplicateStableKey(
                    declaration.stable_key.clone(),
                ));
            }
        }
        Ok(Self { declarations })
    }

    /// Borrow the sealed declarations.
    #[must_use]
    pub fn declarations(&self) -> &[AllocationDeclaration] {
        &self.declarations
    }

    pub(crate) fn into_declarations(self) -> Vec<AllocationDeclaration> {
        self.declarations
    }
}

///
/// DeclarationSnapshotError
///
/// Declaration snapshot validation failure.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum DeclarationSnapshotError {
    #[error("at most 255 allocation declarations are supported")]
    TooManyDeclarations,
    /// Stable-key grammar failure.
    #[error(transparent)]
    Key(StableKeyError),
    /// `MemoryManager` slot validation failure.
    #[error(transparent)]
    MemoryManagerSlot(MemoryManagerSlotError),
    /// A stable key appeared more than once in one snapshot.
    #[error("stable key '{0}' is declared more than once")]
    DuplicateStableKey(StableKey),
    /// An allocation slot appeared more than once in one snapshot.
    #[error("allocation slot '{0:?}' is declared more than once")]
    DuplicateSlot(MemoryManagerSlot),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::slot::MemoryManagerSlot;

    fn declaration(key: &str, id: u8) -> AllocationDeclaration {
        AllocationDeclaration::new(
            key,
            MemoryManagerSlot::new(id).expect("usable slot"),
            SchemaMetadata::default(),
        )
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
        assert_eq!(declaration.schema, SchemaMetadata::default());
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

    #[test]
    fn rejects_duplicate_keys() {
        let err = DeclarationSnapshot::canonicalize(vec![
            declaration("app.users.v1", 100),
            declaration("app.users.v1", 101),
        ])
        .expect_err("duplicate key");

        assert_eq!(
            err,
            DeclarationSnapshotError::DuplicateStableKey(StableKey::parse("app.users.v1").unwrap())
        );
    }

    #[test]
    fn rejects_duplicate_slots() {
        for second_key in ["app.orders.v1", "app.users.v1"] {
            let err = DeclarationSnapshot::canonicalize(vec![
                declaration("app.users.v1", 100),
                declaration(second_key, 100),
            ])
            .expect_err("duplicate slot precedes duplicate key");

            assert_eq!(
                err,
                DeclarationSnapshotError::DuplicateSlot(MemoryManagerSlot::new(100).unwrap())
            );
        }
    }

    #[test]
    fn canonical_resolution_preserves_identity_failures_and_bounds() {
        // Independently inspect prior rows to check competing slot/key failures
        // in the runtime's canonical traversal order.
        let choices = [
            ("app.a.v1", 10),
            ("app.a.v1", 11),
            ("app.b.v1", 10),
            ("app.b.v1", 11),
        ];
        for code in 0..256 {
            let mut code = code;
            let input: Vec<_> = (0..4)
                .map(|_| {
                    let (key, id) = choices[code % choices.len()];
                    code /= choices.len();
                    declaration(key, id)
                })
                .collect();
            let mut ordered = input.clone();
            crate::sort::sort_unstable_by(&mut ordered, |a, b| a.stable_key.cmp(&b.stable_key));
            let expected = ordered.iter().enumerate().find_map(|(index, declaration)| {
                let prior = &ordered[..index];
                if prior.iter().any(|row| row.slot == declaration.slot) {
                    Some(DeclarationSnapshotError::DuplicateSlot(
                        declaration.slot.clone(),
                    ))
                } else if prior
                    .iter()
                    .any(|row| row.stable_key == declaration.stable_key)
                {
                    Some(DeclarationSnapshotError::DuplicateStableKey(
                        declaration.stable_key.clone(),
                    ))
                } else {
                    None
                }
            });
            assert_eq!(
                DeclarationSnapshot::canonicalize(input).unwrap_err(),
                expected.unwrap()
            );
        }
        for count in [0, 1, 254, 255, 256] {
            let input: Vec<_> = (0..count)
                .rev()
                .map(|index| {
                    declaration(
                        &format!("app.store{index:03}.v1"),
                        u8::try_from(index % 255).unwrap(),
                    )
                })
                .collect();
            let result = DeclarationSnapshot::canonicalize(input);
            if count <= 255 {
                let snapshot = result.unwrap();
                assert_eq!(snapshot.declarations().len(), count);
                assert!(
                    snapshot
                        .declarations()
                        .windows(2)
                        .all(|pair| { pair[0].stable_key < pair[1].stable_key })
                );
            } else {
                assert_eq!(result, Err(DeclarationSnapshotError::TooManyDeclarations));
            }
        }
    }
}
