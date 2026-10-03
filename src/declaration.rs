use crate::{
    key::{StableKey, StableKeyError},
    schema::{SchemaMetadata, SchemaMetadataError},
    slot::{AllocationSlotDescriptor, MemoryManagerSlotError},
    text::{DiagnosticTextError, validate_diagnostic_text},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

///
/// AllocationDeclaration
///
/// Checked runtime claim that a stable key should own an allocation slot.
///
/// Declarations are supplied by the current binary before opening storage.
/// Constructors validate the stable key, slot descriptor, label, and schema
/// metadata, but a declaration is not authoritative until it has been validated
/// against the recovered ledger and committed as part of a generation.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AllocationDeclaration {
    /// Durable stable key.
    pub(crate) stable_key: StableKey,
    /// Claimed allocation slot.
    pub(crate) slot: AllocationSlotDescriptor,
    /// Optional diagnostic label.
    #[serde(deserialize_with = "crate::cbor::deserialize_present_option")]
    pub(crate) label: Option<String>,
    /// Optional diagnostic schema metadata.
    pub(crate) schema: SchemaMetadata,
}

impl AllocationDeclaration {
    /// Build a declaration from raw parts after validating diagnostic metadata.
    pub fn new(
        stable_key: impl AsRef<str>,
        slot: AllocationSlotDescriptor,
        label: Option<String>,
        schema: SchemaMetadata,
    ) -> Result<Self, DeclarationSnapshotError> {
        let stable_key = StableKey::parse(stable_key).map_err(DeclarationSnapshotError::Key)?;
        slot.validate()
            .map_err(DeclarationSnapshotError::MemoryManagerSlot)?;
        validate_label(label.as_deref())?;
        schema
            .validate()
            .map_err(DeclarationSnapshotError::SchemaMetadata)?;
        Ok(Self {
            stable_key,
            slot,
            label,
            schema,
        })
    }

    /// Build a `MemoryManager` declaration with a diagnostic label.
    pub fn memory_manager(
        stable_key: impl AsRef<str>,
        id: u8,
        label: impl Into<String>,
    ) -> Result<Self, DeclarationSnapshotError> {
        Self::memory_manager_with_schema(stable_key, id, label, SchemaMetadata::default())
    }

    /// Build an unlabeled `MemoryManager` declaration.
    pub fn memory_manager_unlabeled(
        stable_key: impl AsRef<str>,
        id: u8,
    ) -> Result<Self, DeclarationSnapshotError> {
        Self::memory_manager_unlabeled_with_schema(stable_key, id, SchemaMetadata::default())
    }

    /// Build a `MemoryManager` declaration with a diagnostic label and schema metadata.
    pub fn memory_manager_with_schema(
        stable_key: impl AsRef<str>,
        id: u8,
        label: impl Into<String>,
        schema: SchemaMetadata,
    ) -> Result<Self, DeclarationSnapshotError> {
        let slot = AllocationSlotDescriptor::memory_manager(id)
            .map_err(DeclarationSnapshotError::MemoryManagerSlot)?;
        Self::new(stable_key, slot, Some(label.into()), schema)
    }

    /// Build an unlabeled `MemoryManager` declaration with schema metadata.
    pub fn memory_manager_unlabeled_with_schema(
        stable_key: impl AsRef<str>,
        id: u8,
        schema: SchemaMetadata,
    ) -> Result<Self, DeclarationSnapshotError> {
        let slot = AllocationSlotDescriptor::memory_manager(id)
            .map_err(DeclarationSnapshotError::MemoryManagerSlot)?;
        Self::new(stable_key, slot, None, schema)
    }

    /// Return the durable stable key claimed by this declaration.
    #[must_use]
    pub const fn stable_key(&self) -> &StableKey {
        &self.stable_key
    }

    /// Return the allocation slot claimed by this declaration.
    #[must_use]
    pub const fn slot(&self) -> &AllocationSlotDescriptor {
        &self.slot
    }

    /// Return the optional diagnostic label.
    #[must_use]
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Return the optional schema metadata.
    #[must_use]
    pub const fn schema(&self) -> &SchemaMetadata {
        &self.schema
    }

    /// Validate constructor invariants after decode or manual assembly.
    pub fn validate(&self) -> Result<(), DeclarationSnapshotError> {
        self.stable_key
            .validate()
            .map_err(DeclarationSnapshotError::Key)?;
        self.slot
            .validate()
            .map_err(DeclarationSnapshotError::MemoryManagerSlot)?;
        validate_label(self.label.as_deref())?;
        self.schema
            .validate()
            .map_err(DeclarationSnapshotError::SchemaMetadata)
    }
}

///
/// DeclarationSnapshot
///
/// Immutable runtime declaration snapshot ready for policy and history validation.
///
/// A snapshot is duplicate-free, but it is still not permission to open storage.
/// Integrations should call [`crate::validate_allocations`], commit the staged
/// generation, and only then expose committed allocation authority.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DeclarationSnapshot {
    /// Runtime declarations.
    declarations: Vec<AllocationDeclaration>,
    /// Optional binary/runtime identity for generation diagnostics.
    #[serde(deserialize_with = "crate::cbor::deserialize_present_option")]
    runtime_fingerprint: Option<String>,
}

impl DeclarationSnapshot {
    /// Create and validate a declaration snapshot.
    pub fn new(declarations: Vec<AllocationDeclaration>) -> Result<Self, DeclarationSnapshotError> {
        validate_declarations(&declarations)?;
        reject_duplicates(&declarations)?;
        Ok(Self {
            declarations,
            runtime_fingerprint: None,
        })
    }

    /// Attach an optional runtime fingerprint.
    pub fn with_runtime_fingerprint(
        mut self,
        fingerprint: impl Into<String>,
    ) -> Result<Self, DeclarationSnapshotError> {
        let fingerprint = fingerprint.into();
        validate_runtime_fingerprint(Some(&fingerprint))?;
        self.runtime_fingerprint = Some(fingerprint);
        Ok(self)
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

    /// Borrow the optional runtime fingerprint.
    #[must_use]
    pub fn runtime_fingerprint(&self) -> Option<&str> {
        self.runtime_fingerprint.as_deref()
    }

    /// Validate decoded snapshot invariants before allocation validation.
    pub fn validate(&self) -> Result<(), DeclarationSnapshotError> {
        validate_declarations(&self.declarations)?;
        reject_duplicates(&self.declarations)?;
        validate_runtime_fingerprint(self.runtime_fingerprint.as_deref())
    }

    pub(crate) fn into_parts(self) -> (Vec<AllocationDeclaration>, Option<String>) {
        (self.declarations, self.runtime_fingerprint)
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
    /// Schema metadata encoding failure.
    #[error(transparent)]
    SchemaMetadata(SchemaMetadataError),
    /// A stable key appeared more than once in one snapshot.
    #[error("stable key '{0}' is declared more than once")]
    DuplicateStableKey(StableKey),
    /// An allocation slot appeared more than once in one snapshot.
    #[error("allocation slot '{0:?}' is declared more than once")]
    DuplicateSlot(AllocationSlotDescriptor),
    /// Present declaration labels must be non-empty.
    #[error("allocation declaration label must not be empty when present")]
    EmptyLabel,
    /// Declaration labels must stay bounded for durable ledger storage.
    #[error("allocation declaration label must be at most 256 bytes")]
    LabelTooLong,
    /// Declaration labels must not require Unicode normalization.
    #[error("allocation declaration label must be ASCII")]
    NonAsciiLabel,
    /// Declaration labels must be printable metadata.
    #[error("allocation declaration label must not contain ASCII control characters")]
    ControlCharacterLabel,
    /// Present runtime fingerprints must be non-empty.
    #[error("runtime_fingerprint must not be empty when present")]
    EmptyRuntimeFingerprint,
    /// Runtime fingerprints must stay bounded for durable ledger storage.
    #[error("runtime_fingerprint must be at most 256 bytes")]
    RuntimeFingerprintTooLong,
    /// Runtime fingerprints must not require Unicode normalization.
    #[error("runtime_fingerprint must be ASCII")]
    NonAsciiRuntimeFingerprint,
    /// Runtime fingerprints must be printable metadata.
    #[error("runtime_fingerprint must not contain ASCII control characters")]
    ControlCharacterRuntimeFingerprint,
}

fn validate_label(label: Option<&str>) -> Result<(), DeclarationSnapshotError> {
    let Some(label) = label else {
        return Ok(());
    };
    validate_diagnostic_text(label).map_err(|error| match error {
        DiagnosticTextError::Empty => DeclarationSnapshotError::EmptyLabel,
        DiagnosticTextError::TooLong => DeclarationSnapshotError::LabelTooLong,
        DiagnosticTextError::NonAscii => DeclarationSnapshotError::NonAsciiLabel,
        DiagnosticTextError::ControlCharacter => DeclarationSnapshotError::ControlCharacterLabel,
    })
}

fn validate_declarations(
    declarations: &[AllocationDeclaration],
) -> Result<(), DeclarationSnapshotError> {
    if declarations.len() > 255 {
        return Err(DeclarationSnapshotError::TooManyDeclarations);
    }
    for declaration in declarations {
        declaration.validate()?;
    }
    Ok(())
}

pub fn validate_runtime_fingerprint(
    fingerprint: Option<&str>,
) -> Result<(), DeclarationSnapshotError> {
    let Some(fingerprint) = fingerprint else {
        return Ok(());
    };
    validate_diagnostic_text(fingerprint).map_err(|error| match error {
        DiagnosticTextError::Empty => DeclarationSnapshotError::EmptyRuntimeFingerprint,
        DiagnosticTextError::TooLong => DeclarationSnapshotError::RuntimeFingerprintTooLong,
        DiagnosticTextError::NonAscii => DeclarationSnapshotError::NonAsciiRuntimeFingerprint,
        DiagnosticTextError::ControlCharacter => {
            DeclarationSnapshotError::ControlCharacterRuntimeFingerprint
        }
    })
}

fn reject_duplicates(
    declarations: &[AllocationDeclaration],
) -> Result<(), DeclarationSnapshotError> {
    let mut keys = BTreeSet::new();
    let mut slots = BTreeSet::new();

    for declaration in declarations {
        if !slots.insert(&declaration.slot) {
            return Err(DeclarationSnapshotError::DuplicateSlot(
                declaration.slot.clone(),
            ));
        }
        if !keys.insert(&declaration.stable_key) {
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
    use crate::slot::AllocationSlotDescriptor;

    fn declaration(key: &str, id: u8) -> AllocationDeclaration {
        AllocationDeclaration::new(
            key,
            AllocationSlotDescriptor::memory_manager(id).expect("usable slot"),
            None,
            SchemaMetadata::default(),
        )
        .expect("declaration")
    }

    #[test]
    fn declaration_rejects_unbounded_label_metadata() {
        let err = AllocationDeclaration::new(
            "app.users.v1",
            AllocationSlotDescriptor::memory_manager(100).expect("usable slot"),
            Some("x".repeat(257)),
            SchemaMetadata::default(),
        )
        .expect_err("label too long");

        assert_eq!(err, DeclarationSnapshotError::LabelTooLong);
    }

    #[test]
    fn memory_manager_declaration_constructor_builds_common_declaration() {
        let declaration = AllocationDeclaration::memory_manager("app.orders.v1", 100, "orders")
            .expect("declaration");

        assert_eq!(declaration.stable_key.as_str(), "app.orders.v1");
        assert_eq!(
            declaration.slot,
            AllocationSlotDescriptor::memory_manager(100).expect("usable slot")
        );
        assert_eq!(declaration.label.as_deref(), Some("orders"));
        assert_eq!(declaration.schema, SchemaMetadata::default());
    }

    #[test]
    fn memory_manager_declaration_constructor_rejects_invalid_slot() {
        let err = AllocationDeclaration::memory_manager("app.orders.v1", u8::MAX, "orders")
            .expect_err("sentinel must fail");

        assert!(matches!(
            err,
            DeclarationSnapshotError::MemoryManagerSlot(_)
        ));
    }

    #[test]
    fn snapshot_rejects_decoded_invalid_memory_manager_slot() {
        let mut declaration = declaration("app.orders.v1", 100);
        declaration.slot =
            AllocationSlotDescriptor::memory_manager_unchecked(crate::MEMORY_MANAGER_INVALID_ID);

        let err = DeclarationSnapshot::new(vec![declaration]).expect_err("snapshot must fail");

        assert!(matches!(
            err,
            DeclarationSnapshotError::MemoryManagerSlot(
                MemoryManagerSlotError::InvalidMemoryManagerId { id }
            ) if id == crate::MEMORY_MANAGER_INVALID_ID
        ));
    }

    #[test]
    fn snapshot_rejects_unbounded_runtime_fingerprint() {
        let snapshot =
            DeclarationSnapshot::new(vec![declaration("app.users.v1", 100)]).expect("snapshot");

        let err = snapshot
            .with_runtime_fingerprint("x".repeat(257))
            .expect_err("fingerprint too long");

        assert_eq!(err, DeclarationSnapshotError::RuntimeFingerprintTooLong);
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
        let err = DeclarationSnapshot::new(vec![
            declaration("app.users.v1", 100),
            declaration("app.orders.v1", 100),
        ])
        .expect_err("duplicate slot");

        assert_eq!(
            err,
            DeclarationSnapshotError::DuplicateSlot(
                AllocationSlotDescriptor::memory_manager(100).unwrap()
            )
        );
    }
}
