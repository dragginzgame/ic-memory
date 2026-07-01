use serde::{Deserialize, Serialize};

///
/// SchemaMetadata
///
/// Optional diagnostic metadata for an in-place store schema.
///
/// This metadata helps humans and frameworks diagnose which schema version was
/// declared in each generation. It is bounded and validated for durable ledger
/// encoding, but it does not perform application schema migrations or validate
/// stable data semantics.
///

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaMetadata {
    /// Optional in-place schema version.
    pub(crate) schema_version: Option<u32>,
}

impl SchemaMetadata {
    /// Construct schema metadata after validating the persisted encoding bounds.
    pub fn new(schema_version: Option<u32>) -> Result<Self, SchemaMetadataError> {
        let metadata = Self { schema_version };
        metadata.validate()?;
        Ok(metadata)
    }

    /// Validate schema metadata encoding rules.
    pub fn validate(&self) -> Result<(), SchemaMetadataError> {
        if self.schema_version == Some(0) {
            return Err(SchemaMetadataError::InvalidVersion);
        }
        Ok(())
    }

    /// Return the optional in-place schema version.
    #[must_use]
    pub const fn schema_version(&self) -> Option<u32> {
        self.schema_version
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
