use crate::{
    constants::DIAGNOSTIC_STRING_MAX_BYTES, key::StableKey, slot::AllocationSlotDescriptor,
};
use serde::{Deserialize, Deserializer, Serialize, de::Error as _};

///
/// PolicyIdentity
///
/// Bounded semantic identity for one runtime bootstrap policy configuration.
///
/// The name identifies the policy family, `version` changes when its semantics
/// change, and the optional digest distinguishes runtime configuration. The
/// digest is supplied by the policy implementation; ic-memory does not choose
/// or compute a hashing algorithm.
///
/// This identity is an in-memory repeat-call and diagnostic binding. It is not
/// persisted to the allocation ledger.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyIdentity {
    name: Box<str>,
    version: u32,
    #[serde(deserialize_with = "crate::cbor::deserialize_present_option")]
    configuration_digest: Option<[u8; 32]>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PolicyIdentityRepresentation {
    name: String,
    version: u32,
    #[serde(deserialize_with = "crate::cbor::deserialize_present_option")]
    configuration_digest: Option<[u8; 32]>,
}

impl<'de> Deserialize<'de> for PolicyIdentity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let representation = PolicyIdentityRepresentation::deserialize(deserializer)?;
        let mut identity =
            Self::new(representation.name, representation.version).map_err(D::Error::custom)?;
        identity.configuration_digest = representation.configuration_digest;
        Ok(identity)
    }
}

impl PolicyIdentity {
    /// Construct a validated policy identity without a configuration digest.
    pub fn new(name: impl Into<String>, version: u32) -> Result<Self, PolicyIdentityError> {
        let name = name.into();
        validate_policy_identity_name(&name)?;
        if version == 0 {
            return Err(PolicyIdentityError::ZeroVersion);
        }
        Ok(Self {
            name: name.into_boxed_str(),
            version,
            configuration_digest: None,
        })
    }

    /// Attach a caller-computed configuration digest.
    #[must_use]
    pub const fn with_configuration_digest(mut self, digest: [u8; 32]) -> Self {
        self.configuration_digest = Some(digest);
        self
    }

    /// Borrow the bounded policy-family name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return the nonzero semantic policy version.
    #[must_use]
    pub const fn version(&self) -> u32 {
        self.version
    }

    /// Borrow the optional caller-computed configuration digest.
    #[must_use]
    pub const fn configuration_digest(&self) -> Option<&[u8; 32]> {
        self.configuration_digest.as_ref()
    }
}

///
/// PolicyIdentityError
///
/// Failure to construct a bounded runtime bootstrap policy identity.
///

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, thiserror::Error, PartialEq)]
pub enum PolicyIdentityError {
    /// Policy-family names must not be empty.
    #[error("runtime bootstrap policy identity name must not be empty")]
    EmptyName,
    /// Policy-family names must remain bounded diagnostic metadata.
    #[error("runtime bootstrap policy identity name is {length} bytes; maximum is {maximum} bytes")]
    NameTooLong {
        /// Actual UTF-8 byte length.
        length: usize,
        /// Maximum accepted byte length.
        maximum: usize,
    },
    /// Policy-family names must not require Unicode normalization.
    #[error("runtime bootstrap policy identity name must be ASCII")]
    NonAsciiName,
    /// Policy-family names must be printable diagnostic metadata.
    #[error("runtime bootstrap policy identity name must not contain ASCII control characters")]
    ControlCharacterName,
    /// Semantic policy version zero is reserved as invalid.
    #[error("runtime bootstrap policy identity version must be greater than zero")]
    ZeroVersion,
}

fn validate_policy_identity_name(name: &str) -> Result<(), PolicyIdentityError> {
    if name.is_empty() {
        return Err(PolicyIdentityError::EmptyName);
    }
    if name.len() > DIAGNOSTIC_STRING_MAX_BYTES {
        return Err(PolicyIdentityError::NameTooLong {
            length: name.len(),
            maximum: DIAGNOSTIC_STRING_MAX_BYTES,
        });
    }
    if !name.is_ascii() {
        return Err(PolicyIdentityError::NonAsciiName);
    }
    if name.bytes().any(|byte| byte.is_ascii_control()) {
        return Err(PolicyIdentityError::ControlCharacterName);
    }
    Ok(())
}

///
/// AllocationPolicy
///
/// Framework-supplied rules for whether a key may claim a slot.
///
/// Policy is intentionally separate from the durable ledger invariant. The
/// ledger remembers `stable_key -> allocation_slot`; this trait lets an
/// integration reject declarations that do not belong to its namespace or
/// substrate-specific range before staging a generation.
///
/// In the default `MemoryManager` runtime, registered range claims are checked
/// before this policy, and this policy receives external declarations only.
/// The internal allocation-ledger declaration remains exclusively governed by
/// ic-memory. Framework adapters should decide whether registered range claims
/// or their own policy is authoritative for application ID space, then register
/// ranges accordingly.
///

pub trait AllocationPolicy {
    /// Policy error type.
    type Error;

    /// Validate a stable key against framework naming rules.
    fn validate_key(&self, key: &StableKey) -> Result<(), Self::Error>;

    /// Validate a stable-key to allocation-slot claim.
    fn validate_slot(
        &self,
        key: &StableKey,
        slot: &AllocationSlotDescriptor,
    ) -> Result<(), Self::Error>;

    /// Validate a reserved stable-key to allocation-slot claim.
    fn validate_reserved_slot(
        &self,
        key: &StableKey,
        slot: &AllocationSlotDescriptor,
    ) -> Result<(), Self::Error>;
}

///
/// RuntimeBootstrapPolicy
///
/// Allocation policy with an explicit semantic identity for runtime bootstrap.
///
/// [`crate::MemoryRuntime`] binds its successful bootstrap to this identity.
/// Repeated bootstrap is idempotent only when the caller supplies the same
/// sealed declaration snapshot and the same policy identity. Implementations
/// should change the identity whenever policy configuration or semantics
/// change. Configuration-dependent policies should include a digest derived
/// from their effective configuration.
///

pub trait RuntimeBootstrapPolicy: AllocationPolicy {
    /// Admit recovered identity and complete declarations before resolution.
    ///
    /// Runs once per cold bootstrap attempt after validated recovery. Warm
    /// bootstrap/adoption does not replay it. The default selects no historical
    /// keys. Hosts compose generated consumers here under their existing policy
    /// and bucket profile. Include these semantics in the policy identity.
    fn prepare_bootstrap(
        &self,
        _admission: &mut crate::BootstrapAdmission<'_>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Construct the bounded semantic identity of this policy configuration.
    fn runtime_bootstrap_identity(&self) -> Result<PolicyIdentity, PolicyIdentityError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_identity_validates_name_version_and_digest() {
        let digest = [0xA5; 32];
        let identity = PolicyIdentity::new("canic.memory-bootstrap-policy", 1)
            .expect("valid identity")
            .with_configuration_digest(digest);

        assert_eq!(identity.name(), "canic.memory-bootstrap-policy");
        assert_eq!(identity.version(), 1);
        assert_eq!(identity.configuration_digest(), Some(&digest));
    }

    #[test]
    fn policy_identity_rejects_unbounded_or_noncanonical_metadata() {
        assert_eq!(
            PolicyIdentity::new("", 1).expect_err("empty name"),
            PolicyIdentityError::EmptyName
        );
        assert!(matches!(
            PolicyIdentity::new("x".repeat(DIAGNOSTIC_STRING_MAX_BYTES + 1), 1),
            Err(PolicyIdentityError::NameTooLong { .. })
        ));
        assert_eq!(
            PolicyIdentity::new("policy\nname", 1).expect_err("control character"),
            PolicyIdentityError::ControlCharacterName
        );
        assert_eq!(
            PolicyIdentity::new("policé", 1).expect_err("non-ASCII"),
            PolicyIdentityError::NonAsciiName
        );
        assert_eq!(
            PolicyIdentity::new("policy", 0).expect_err("zero version"),
            PolicyIdentityError::ZeroVersion
        );
    }

    #[test]
    fn policy_identity_deserialization_revalidates_invariants() {
        #[derive(Serialize)]
        struct UncheckedPolicyIdentity<'a> {
            name: &'a str,
            version: u32,
            configuration_digest: Option<[u8; 32]>,
        }

        let bytes = crate::test_cbor::to_vec(&UncheckedPolicyIdentity {
            name: "",
            version: 1,
            configuration_digest: None,
        })
        .expect("invalid diagnostic bytes");
        let error = crate::test_cbor::from_slice::<PolicyIdentity>(&bytes)
            .expect_err("deserialization must revalidate identity");
        assert!(error.to_string().contains("must not be empty"));
    }
}
