use crate::{MemoryManagerSlot, PolicyIdentity, RuntimeBootstrapPolicy, StableKey};
use std::convert::Infallible;

///
/// GenericAllocationPolicy
///
/// Built-in bootstrap policy for hosts needing no additional application checks.
/// The runtime enforces host namespace grants, the common pool and governance;
/// this policy adds no application-specific restrictions.
///
/// Use with configured bootstrap when the host does not require a custom
/// policy. It retains the built-in policy identity and does not authorize
/// replacing a different policy already bound to the runtime.
///
pub struct GenericAllocationPolicy;

impl RuntimeBootstrapPolicy for GenericAllocationPolicy {
    type Error = Infallible;

    fn validate_allocation(
        &self,
        _key: &StableKey,
        _slot: &MemoryManagerSlot,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn runtime_bootstrap_identity(&self) -> PolicyIdentity {
        PolicyIdentity::new(*b"ic-memory.noop-policy...........")
    }
}
