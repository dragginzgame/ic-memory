use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;

///
/// SchemaMetadata
///
/// Optional diagnostic metadata for an in-place store schema.
///
/// This metadata helps humans and frameworks diagnose which schema version was
/// most recently declared. Construction and decoding require a nonzero
/// version when present. It does not perform application schema migrations or
/// validate stable data semantics.
///

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaMetadata {
    /// Optional in-place schema version.
    #[serde(deserialize_with = "crate::cbor::deserialize_present_option")]
    schema_version: Option<NonZeroU32>,
}

impl SchemaMetadata {
    /// Construct schema metadata after validating the persisted encoding bounds.
    pub const fn new(schema_version: Option<u32>) -> Result<Self, SchemaMetadataError> {
        let schema_version = match schema_version {
            Some(version) => match NonZeroU32::new(version) {
                Some(version) => Some(version),
                None => return Err(SchemaMetadataError::InvalidVersion),
            },
            None => None,
        };
        Ok(Self { schema_version })
    }

    /// Return the optional in-place schema version.
    #[must_use]
    pub const fn schema_version(&self) -> Option<u32> {
        match self.schema_version {
            Some(version) => Some(version.get()),
            None => None,
        }
    }
}

///
/// SchemaMetadataError
///
/// Schema metadata validation failure.
///

#[non_exhaustive]
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum SchemaMetadataError {
    /// Schema version zero is reserved for absence.
    #[error("schema_version must be greater than zero when present")]
    InvalidVersion,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_versions_are_checked_and_preserve_current_encoding() {
        assert_eq!(
            SchemaMetadata::new(Some(0)),
            Err(SchemaMetadataError::InvalidVersion)
        );
        for (version, suffix) in [
            (None, vec![0xf6]),
            (Some(1), vec![1]),
            (Some(24), vec![0x18, 24]),
            (Some(u32::MAX), vec![0x1a, 0xff, 0xff, 0xff, 0xff]),
        ] {
            let metadata = SchemaMetadata::new(version).unwrap();
            assert_eq!(metadata.schema_version(), version);
            let json = serde_json::json!({ "schema_version": version });
            assert_eq!(serde_json::to_value(&metadata).unwrap(), json);
            assert_eq!(
                serde_json::from_value::<SchemaMetadata>(json).unwrap(),
                metadata
            );
            let mut expected = b"\xa1\x6eschema_version".to_vec();
            expected.extend(suffix);
            let bytes = crate::test_cbor::to_vec(&metadata).unwrap();
            assert_eq!(bytes, expected);
            assert_eq!(
                crate::test_cbor::from_slice::<SchemaMetadata>(&bytes).unwrap(),
                metadata
            );
        }
        assert_eq!(
            SchemaMetadata::default(),
            SchemaMetadata::new(None).unwrap()
        );
    }

    #[test]
    fn schema_decode_rejects_invalid_versions_and_shapes() {
        for value in [
            serde_json::json!({ "schema_version": 0 }),
            serde_json::json!({ "schema_version": -1 }),
            serde_json::json!({ "schema_version": u64::from(u32::MAX) + 1 }),
            serde_json::json!({ "schema_version": "1" }),
            serde_json::json!({}),
            serde_json::json!({ "schema_version": null, "extra": 1 }),
        ] {
            assert!(serde_json::from_value::<SchemaMetadata>(value.clone()).is_err());
            let bytes = crate::test_cbor::to_vec(&value).unwrap();
            assert!(crate::test_cbor::from_slice::<SchemaMetadata>(&bytes).is_err());
        }

        let mut value = serde_json::to_value(
            crate::AllocationDeclaration::memory_manager("app.rows.v1", 100, "rows").unwrap(),
        )
        .unwrap();
        value["schema"]["schema_version"] = 0.into();
        assert!(serde_json::from_value::<crate::AllocationDeclaration>(value.clone()).is_err());
        let bytes = crate::test_cbor::to_vec(&value).unwrap();
        assert!(crate::test_cbor::from_slice::<crate::AllocationDeclaration>(&bytes).is_err());
    }
}
