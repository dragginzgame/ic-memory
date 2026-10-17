use crate::{key::StableKey, slot::MemoryManagerSlot};
use serde::Serialize;

///
/// PolicyIdentity
///
/// Host-issued token binding one prepared runtime bootstrap policy configuration.
///
/// All 32-byte values are valid. Hosts choose a stable token and change it when
/// policy semantics or effective configuration change. Names, versions, hashing
/// and fallible configuration preparation stay with the host. The token is an
/// equality binding, not authentication or proof that a policy is correct.
/// It is retained in memory and serialized for diagnostics, never persisted
/// in the allocation ledger. Diagnostic JSON contains a 32-element byte array.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct PolicyIdentity([u8; 32]);

impl PolicyIdentity {
    /// Construct a token from the host's complete policy identity bytes.
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Borrow the exact identity bytes used for equality and diagnostics.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

///
/// RuntimeBootstrapPolicy
///
/// Host policy for recovered admission, allocation checks and runtime identity.
///
/// The runtime admits host grants and pool eligibility before calling the
/// allocation check for each external declaration. Governance stays private;
/// historical ownership checks still apply to every declaration.
///
/// [`crate::MemoryRuntime`] binds its successful bootstrap to this identity.
/// Repeated bootstrap is idempotent only when the caller supplies the same
/// sealed declaration snapshot and the same policy identity. Implementations
/// should change the identity whenever policy configuration or semantics
/// change. Hosts derive the token from all effective configuration and prepare
/// any fallible policy setup before calling the runtime. Labels, versions and
/// hashing algorithms belong to the host; this crate compares the token bytes.
///

pub trait RuntimeBootstrapPolicy {
    /// Policy error type.
    type Error;

    /// Validate one resolved stable-key and allocation-slot claim.
    /// The runtime checks host grants and pool eligibility before this callback;
    /// historical ownership checks follow it. Governance bypasses this callback.
    fn validate_allocation(
        &self,
        key: &StableKey,
        slot: &MemoryManagerSlot,
    ) -> Result<(), Self::Error>;

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

    /// Return the host-issued token for this prepared policy configuration.
    fn runtime_bootstrap_identity(&self) -> PolicyIdentity;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_token_preserves_all_bytes_in_equality_and_diagnostics() {
        let bytes = std::array::from_fn(|i| u8::try_from(i).unwrap());
        let identity = PolicyIdentity::new(bytes);
        assert_eq!(identity.as_bytes(), &bytes);
        assert_eq!(
            serde_json::to_value(identity).unwrap(),
            serde_json::json!(bytes)
        );
        for index in 0..32 {
            let mut changed = bytes;
            changed[index] ^= 1;
            assert_ne!(identity, PolicyIdentity::new(changed));
        }
    }
}
