use super::{RuntimeBootstrapError, RuntimePolicyError};
use crate::{
    AllocationPolicy, AllocationSlotDescriptor, PolicyIdentity, PolicyIdentityError,
    RuntimeBootstrapPolicy, StableKey,
    registry::{RuntimeDeclarationAuthority, SealedDeclarationSnapshot},
    slot::{IC_MEMORY_AUTHORITY_OWNER, MemoryManagerRangeAuthorityError},
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
        if matches!(authority, RuntimeDeclarationAuthority::Internal) {
            return Ok(());
        }
        if crate::is_ic_memory_stable_key(key.as_str()) {
            return Err(RuntimePolicyError::ReservedStableKeyAuthority {
                stable_key: key.as_str().to_string(),
                expected_authority: IC_MEMORY_AUTHORITY_OWNER,
            });
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
        self.validate_runtime_range(key, slot)?;
        if matches!(
            self.declaration_authority(key)?,
            RuntimeDeclarationAuthority::Internal
        ) {
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
        self.validate_runtime_range(key, slot)?;
        if matches!(
            self.declaration_authority(key)?,
            RuntimeDeclarationAuthority::Internal
        ) {
            return Ok(());
        }
        self.custom_policy
            .validate_reserved_slot(key, slot)
            .map_err(RuntimePolicyError::Custom)
    }
}

impl<P: AllocationPolicy> RuntimeMemoryManagerPolicy<'_, P> {
    fn declaration_authority(
        &self,
        key: &StableKey,
    ) -> Result<&RuntimeDeclarationAuthority, RuntimePolicyError<P::Error>> {
        self.declarations
            .declaration_authority()
            .get(key.as_str())
            .ok_or_else(|| RuntimePolicyError::MissingDeclarationMetadata(key.as_str().to_string()))
    }

    fn validate_runtime_range(
        &self,
        key: &StableKey,
        slot: &AllocationSlotDescriptor,
    ) -> Result<(), RuntimePolicyError<P::Error>> {
        let authority = self.declaration_authority(key)?;
        if matches!(authority, RuntimeDeclarationAuthority::Internal) {
            self.declarations
                .range_authority()
                .validate_slot_authority(slot, IC_MEMORY_AUTHORITY_OWNER)?;
            return Ok(());
        }

        let RuntimeDeclarationAuthority::External(authority) = authority else {
            return Err(RuntimePolicyError::MissingDeclarationMetadata(
                key.as_str().to_string(),
            ));
        };
        if self.declarations.user_ranges_registered() {
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

pub(super) struct NoopPolicy;

impl AllocationPolicy for NoopPolicy {
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

impl RuntimeBootstrapPolicy for NoopPolicy {
    fn runtime_bootstrap_identity(&self) -> Result<PolicyIdentity, PolicyIdentityError> {
        PolicyIdentity::new("ic-memory.noop-policy", 1)
    }
}
