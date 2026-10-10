use super::RuntimeBootstrapError;
use crate::{
    AllocationPolicy, MemoryManagerSlot, PolicyIdentity, PolicyIdentityError,
    RuntimeBootstrapPolicy, StableKey, slot::IC_MEMORY_LEDGER_STABLE_KEY,
};
use std::convert::Infallible;

pub(super) fn runtime_bootstrap_error_from_bootstrap<P>(
    err: crate::BootstrapError<P>,
) -> RuntimeBootstrapError<P> {
    match err {
        crate::BootstrapError::Ledger(err) => RuntimeBootstrapError::LedgerCommit(err),
        crate::BootstrapError::Validation(err) => RuntimeBootstrapError::Validation(err),
        crate::BootstrapError::Staging(err) => RuntimeBootstrapError::Staging(err),
    }
}

// Resolution admits every external row under the host pool. This adapter only
// keeps private governance outside application callbacks; it cannot accept raw
// declarations from a public caller or supply a different allocation policy.
pub(super) struct RuntimeMemoryManagerPolicy<'a, P> {
    pub(super) custom_policy: &'a P,
}

impl<P: AllocationPolicy> AllocationPolicy for RuntimeMemoryManagerPolicy<'_, P> {
    type Error = P::Error;

    fn validate_key(&self, key: &StableKey) -> Result<(), Self::Error> {
        if key.as_str() == IC_MEMORY_LEDGER_STABLE_KEY {
            return Ok(());
        }
        self.custom_policy.validate_key(key)
    }

    fn validate_slot(&self, key: &StableKey, slot: &MemoryManagerSlot) -> Result<(), Self::Error> {
        if key.as_str() == IC_MEMORY_LEDGER_STABLE_KEY {
            return Ok(());
        }
        self.custom_policy.validate_slot(key, slot)
    }

    fn validate_reserved_slot(
        &self,
        key: &StableKey,
        slot: &MemoryManagerSlot,
    ) -> Result<(), Self::Error> {
        if key.as_str() == IC_MEMORY_LEDGER_STABLE_KEY {
            return Ok(());
        }
        self.custom_policy.validate_reserved_slot(key, slot)
    }
}

///
/// GenericAllocationPolicy
///
/// Built-in bootstrap policy for hosts needing no additional application checks.
/// The runtime enforces host namespace grants, the common pool and governance;
/// this policy adds no application-specific restrictions. Passing it directly
/// to allocation validation outside the runtime does not enforce host pool admission.
///
/// Use with configured bootstrap when the host does not require a custom
/// policy. It retains the built-in policy identity and does not authorize
/// replacing a different policy already bound to the runtime.
///
pub struct GenericAllocationPolicy;

impl AllocationPolicy for GenericAllocationPolicy {
    type Error = Infallible;

    fn validate_key(&self, _key: &StableKey) -> Result<(), Self::Error> {
        Ok(())
    }

    fn validate_slot(
        &self,
        _key: &StableKey,
        _slot: &MemoryManagerSlot,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn validate_reserved_slot(
        &self,
        _key: &StableKey,
        _slot: &MemoryManagerSlot,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl RuntimeBootstrapPolicy for GenericAllocationPolicy {
    fn runtime_bootstrap_identity(&self) -> Result<PolicyIdentity, PolicyIdentityError> {
        PolicyIdentity::new("ic-memory.noop-policy", 1)
    }
}
