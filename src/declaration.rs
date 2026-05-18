use crate::{
    key::{StableKey, StableKeyError},
    schema::{SchemaMetadata, SchemaMetadataError},
    slot::AllocationSlotDescriptor,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const DIAGNOSTIC_STRING_MAX_BYTES: usize = 256;

///
/// AllocationDeclaration
///
/// Data-only claim that a stable key owns an allocation slot.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AllocationDeclaration {
    /// Durable stable key.
    pub stable_key: StableKey,
    /// Claimed allocation slot.
    pub slot: AllocationSlotDescriptor,
    /// Optional diagnostic label.
    pub label: Option<String>,
    /// Optional diagnostic schema metadata.
    pub schema: SchemaMetadata,
}

impl AllocationDeclaration {
    /// Build a declaration from raw parts.
    pub fn new(
        stable_key: impl AsRef<str>,
        slot: AllocationSlotDescriptor,
        label: Option<String>,
        schema: SchemaMetadata,
    ) -> Result<Self, DeclarationSnapshotError> {
        let stable_key = StableKey::parse(stable_key).map_err(DeclarationSnapshotError::Key)?;
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
}

///
/// DeclarationCollector
///
/// Mutable collection phase before a snapshot is sealed.
#[derive(Clone, Debug, Default)]
pub struct DeclarationCollector {
    declarations: Vec<AllocationDeclaration>,
}

impl DeclarationCollector {
    /// Add one allocation declaration.
    pub fn push(&mut self, declaration: AllocationDeclaration) {
        self.declarations.push(declaration);
    }

    /// Seal collected declarations into a duplicate-free snapshot.
    pub fn seal(self) -> Result<DeclarationSnapshot, DeclarationSnapshotError> {
        DeclarationSnapshot::new(self.declarations)
    }
}

///
/// DeclarationSnapshot
///
/// Immutable runtime declaration snapshot ready for policy and history validation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DeclarationSnapshot {
    /// Runtime declarations.
    declarations: Vec<AllocationDeclaration>,
    /// Optional binary/runtime identity for generation diagnostics.
    runtime_fingerprint: Option<String>,
}

impl DeclarationSnapshot {
    /// Create and validate a declaration snapshot.
    pub fn new(declarations: Vec<AllocationDeclaration>) -> Result<Self, DeclarationSnapshotError> {
        for declaration in &declarations {
            validate_label(declaration.label.as_deref())?;
            declaration
                .schema
                .validate()
                .map_err(DeclarationSnapshotError::SchemaMetadata)?;
        }
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
    pub fn is_empty(&self) -> bool {
        self.declarations.is_empty()
    }

    /// Return the number of declarations in the snapshot.
    #[must_use]
    pub fn len(&self) -> usize {
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

    pub(crate) fn into_parts(self) -> (Vec<AllocationDeclaration>, Option<String>) {
        (self.declarations, self.runtime_fingerprint)
    }
}

///
/// DeclarationSnapshotError
///
/// Declaration snapshot validation failure.
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum DeclarationSnapshotError {
    /// Stable-key grammar failure.
    #[error(transparent)]
    Key(StableKeyError),
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
    if label.is_empty() {
        return Err(DeclarationSnapshotError::EmptyLabel);
    }
    if label.len() > DIAGNOSTIC_STRING_MAX_BYTES {
        return Err(DeclarationSnapshotError::LabelTooLong);
    }
    if !label.is_ascii() {
        return Err(DeclarationSnapshotError::NonAsciiLabel);
    }
    if label.bytes().any(|byte| byte.is_ascii_control()) {
        return Err(DeclarationSnapshotError::ControlCharacterLabel);
    }
    Ok(())
}

pub(crate) fn validate_runtime_fingerprint(
    fingerprint: Option<&str>,
) -> Result<(), DeclarationSnapshotError> {
    let Some(fingerprint) = fingerprint else {
        return Ok(());
    };
    if fingerprint.is_empty() {
        return Err(DeclarationSnapshotError::EmptyRuntimeFingerprint);
    }
    if fingerprint.len() > DIAGNOSTIC_STRING_MAX_BYTES {
        return Err(DeclarationSnapshotError::RuntimeFingerprintTooLong);
    }
    if !fingerprint.is_ascii() {
        return Err(DeclarationSnapshotError::NonAsciiRuntimeFingerprint);
    }
    if fingerprint.bytes().any(|byte| byte.is_ascii_control()) {
        return Err(DeclarationSnapshotError::ControlCharacterRuntimeFingerprint);
    }
    Ok(())
}

fn reject_duplicates(
    declarations: &[AllocationDeclaration],
) -> Result<(), DeclarationSnapshotError> {
    let mut keys = BTreeSet::new();
    let mut slots = BTreeSet::new();

    for declaration in declarations {
        if !slots.insert(declaration.slot.clone()) {
            return Err(DeclarationSnapshotError::DuplicateSlot(
                declaration.slot.clone(),
            ));
        }
        if !keys.insert(declaration.stable_key.clone()) {
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

        assert!(matches!(
            err,
            DeclarationSnapshotError::DuplicateStableKey(_)
        ));
    }

    #[test]
    fn rejects_duplicate_slots() {
        let err = DeclarationSnapshot::new(vec![
            declaration("app.users.v1", 100),
            declaration("app.orders.v1", 100),
        ])
        .expect_err("duplicate slot");

        assert!(matches!(err, DeclarationSnapshotError::DuplicateSlot(_)));
    }
}
