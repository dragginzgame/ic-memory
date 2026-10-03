use super::{RuntimeBootstrapError, RuntimePolicyError};
use crate::{
    AllocationPolicy, AllocationSlotDescriptor, PolicyIdentity, PolicyIdentityError,
    RuntimeBootstrapPolicy, StableKey,
    registry::SealedDeclarationSnapshot,
    slot::{
        IC_MEMORY_AUTHORITY_OWNER, IC_MEMORY_LEDGER_STABLE_KEY, MemoryManagerRangeAuthorityError,
    },
};
use std::convert::Infallible;

pub(super) fn runtime_bootstrap_error_from_bootstrap<P>(
    err: crate::BootstrapError<RuntimePolicyError<P>>,
) -> RuntimeBootstrapError<P> {
    match err {
        crate::BootstrapError::Ledger(err) => RuntimeBootstrapError::LedgerCommit(err),
        crate::BootstrapError::Validation(err) => RuntimeBootstrapError::Validation(err),
        crate::BootstrapError::Staging(err) => RuntimeBootstrapError::Staging(err),
    }
}

pub(super) struct RuntimeMemoryManagerPolicy<'a, P> {
    pub(super) declarations: &'a SealedDeclarationSnapshot,
    pub(super) custom_policy: &'a P,
}

impl<P: AllocationPolicy> AllocationPolicy for RuntimeMemoryManagerPolicy<'_, P> {
    type Error = RuntimePolicyError<P::Error>;

    fn validate_key(&self, key: &StableKey) -> Result<(), Self::Error> {
        let authority = self.declaration_authority(key)?;
        if authority == IC_MEMORY_AUTHORITY_OWNER {
            return Ok(());
        }
        self.custom_policy
            .validate_key(key)
            .map_err(RuntimePolicyError::Custom)
    }

    fn validate_slot(
        &self,
        key: &StableKey,
        slot: &AllocationSlotDescriptor,
    ) -> Result<(), Self::Error> {
        let authority = self.declaration_authority(key)?;
        self.validate_runtime_range(authority, slot)?;
        if authority == IC_MEMORY_AUTHORITY_OWNER {
            return Ok(());
        }
        self.custom_policy
            .validate_slot(key, slot)
            .map_err(RuntimePolicyError::Custom)
    }

    fn validate_reserved_slot(
        &self,
        key: &StableKey,
        slot: &AllocationSlotDescriptor,
    ) -> Result<(), Self::Error> {
        let authority = self.declaration_authority(key)?;
        self.validate_runtime_range(authority, slot)?;
        if authority == IC_MEMORY_AUTHORITY_OWNER {
            return Ok(());
        }
        self.custom_policy
            .validate_reserved_slot(key, slot)
            .map_err(RuntimePolicyError::Custom)
    }
}

impl<P: AllocationPolicy> RuntimeMemoryManagerPolicy<'_, P> {
    fn declaration_authority(&self, key: &StableKey) -> Result<&str, RuntimePolicyError<P::Error>> {
        if key.as_str() == IC_MEMORY_LEDGER_STABLE_KEY {
            return Ok(IC_MEMORY_AUTHORITY_OWNER);
        }
        self.declarations
            .registered_declaration(key)
            .map(crate::StaticMemoryDeclaration::authority)
            .ok_or_else(|| RuntimePolicyError::MissingDeclarationMetadata(key.as_str().to_string()))
    }

    fn validate_runtime_range(
        &self,
        authority: &str,
        slot: &AllocationSlotDescriptor,
    ) -> Result<(), RuntimePolicyError<P::Error>> {
        if authority == IC_MEMORY_AUTHORITY_OWNER || self.declarations.user_ranges_registered() {
            self.declarations
                .range_authority()
                .validate_slot_authority(slot, authority)?;
            return Ok(());
        }

        let id = slot
            .memory_manager_id()
            .map_err(MemoryManagerRangeAuthorityError::Slot)?;
        if self
            .declarations
            .range_authority()
            .authority_for_id(id)?
            .is_some()
        {
            self.declarations
                .range_authority()
                .validate_slot_authority(slot, authority)?;
        }
        Ok(())
    }
}

///
/// GenericRangePolicy
///
/// Built-in bootstrap policy used by the no-argument default-runtime helpers.
/// The runtime enforces registered range ownership and internal reservations;
/// this policy adds no application-specific restrictions. Passing it directly
/// to allocation validation outside the runtime does not enforce those ranges.
///
/// Use with configured bootstrap when the host does not require a custom
/// policy. It retains the built-in policy identity and does not authorize
/// replacing a different policy already bound to the runtime.
///
pub struct GenericRangePolicy;

impl AllocationPolicy for GenericRangePolicy {
    type Error = Infallible;

    fn validate_key(&self, _key: &StableKey) -> Result<(), Self::Error> {
        Ok(())
    }

    fn validate_slot(
        &self,
        _key: &StableKey,
        _slot: &AllocationSlotDescriptor,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn validate_reserved_slot(
        &self,
        _key: &StableKey,
        _slot: &AllocationSlotDescriptor,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl RuntimeBootstrapPolicy for GenericRangePolicy {
    fn runtime_bootstrap_identity(&self) -> Result<PolicyIdentity, PolicyIdentityError> {
        PolicyIdentity::new("ic-memory.noop-policy", 1)
    }
}
