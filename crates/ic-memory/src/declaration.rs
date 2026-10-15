use crate::{
    key::{StableKey, StableKeyError},
    schema::SchemaMetadata,
    slot::{MemoryManagerSlot, MemoryManagerSlotError},
};
use serde::{Deserialize, Deserializer, Serialize, de::Error as _};

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
/// Immutable runtime declaration snapshot ready for policy and ownership validation.
///
/// Construction and decoding enforce the declaration count and unique keys and
/// slots. A snapshot is not permission to open storage.
/// [`crate::MemoryRuntime::bootstrap`] owns validation and persistence before
/// publishing committed allocation authority.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DeclarationSnapshot {
    /// Runtime declarations.
    declarations: Vec<AllocationDeclaration>,
}

impl<'de> Deserialize<'de> for DeclarationSnapshot {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename = "DeclarationSnapshot", deny_unknown_fields)]
        struct Snapshot {
            #[serde(deserialize_with = "crate::cbor::deserialize_records")]
            declarations: Vec<AllocationDeclaration>,
        }

        let snapshot = Snapshot::deserialize(deserializer)?;
        Self::new(snapshot.declarations).map_err(D::Error::custom)
    }
}

impl DeclarationSnapshot {
    /// Create and validate a declaration snapshot.
    pub fn new(declarations: Vec<AllocationDeclaration>) -> Result<Self, DeclarationSnapshotError> {
        validate_declaration_set(&declarations)?;
        Ok(Self { declarations })
    }

    // Runtime resolution owns this order; public construction and decoding keep
    // their original traversal and error precedence. Share identity checks below.
    pub(crate) fn canonicalize(
        mut declarations: Vec<AllocationDeclaration>,
    ) -> Result<Self, DeclarationSnapshotError> {
        if declarations.len() > crate::constants::MAX_ALLOCATIONS {
            return Err(DeclarationSnapshotError::TooManyDeclarations);
        }
        crate::sort::sort_unstable_by(&mut declarations, |a, b| a.stable_key.cmp(&b.stable_key));
        let mut duplicate_keys = [false; crate::constants::MAX_ALLOCATIONS];
        for (index, pair) in declarations.windows(2).enumerate() {
            duplicate_keys[index + 1] = pair[0].stable_key == pair[1].stable_key;
        }
        validate_declaration_identities(&declarations, &duplicate_keys)?;
        Ok(Self { declarations })
    }

    /// Return true when the snapshot has no declarations.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.declarations.is_empty()
    }

    /// Return the number of declarations in the snapshot.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.declarations.len()
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

fn validate_declaration_set(
    declarations: &[AllocationDeclaration],
) -> Result<(), DeclarationSnapshotError> {
    if declarations.len() > crate::constants::MAX_ALLOCATIONS {
        return Err(DeclarationSnapshotError::TooManyDeclarations);
    }
    let duplicate_keys = crate::key::duplicate_key_positions(
        declarations
            .iter()
            .enumerate()
            .map(|(index, declaration)| (&declaration.stable_key, index))
            .collect(),
    );
    validate_declaration_identities(declarations, &duplicate_keys)
}

fn validate_declaration_identities(
    declarations: &[AllocationDeclaration],
    duplicate_keys: &[bool; crate::constants::MAX_ALLOCATIONS],
) -> Result<(), DeclarationSnapshotError> {
    let mut slots = [false; crate::constants::MAX_ALLOCATIONS];

    for (index, declaration) in declarations.iter().enumerate() {
        let occupied = &mut slots[usize::from(declaration.slot.id())];
        if *occupied {
            return Err(DeclarationSnapshotError::DuplicateSlot(
                declaration.slot.clone(),
            ));
        }
        *occupied = true;
        if duplicate_keys[index] {
            return Err(DeclarationSnapshotError::DuplicateStableKey(
                declaration.stable_key.clone(),
            ));
        }
    }

    Ok(())
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
    fn snapshot_decode_rejects_unusable_memory_manager_slot() {
        let snapshot = DeclarationSnapshot::new(vec![declaration("app.orders.v1", 100)]).unwrap();
        let mut value = serde_json::to_value(snapshot).unwrap();
        value["declarations"][0]["slot"] = serde_json::json!(255);
        assert!(serde_json::from_value::<DeclarationSnapshot>(value).is_err());
    }

    #[test]
    fn rejects_duplicate_keys() {
        let err = DeclarationSnapshot::new(vec![
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
            let err = DeclarationSnapshot::new(vec![
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
        // Compare the runtime boundary with ordinary validation in canonical
        // traversal order, including competing duplicate-key/slot failures.
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
            assert_eq!(
                DeclarationSnapshot::canonicalize(input),
                DeclarationSnapshot::new(ordered)
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
                assert_eq!(snapshot.len(), count);
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
