use ic_memory::{AllocationPolicy, AllocationSlotDescriptor, RuntimeBootstrapPolicy, StableKey};

const AUTHORITY: &str = "default_custom_policy";
const POLICY_IDENTITY: &str = "default-custom-policy.v1";

ic_memory::ic_memory_range!(authority = AUTHORITY, start = 150, end = 150);

ic_memory::ic_memory_declaration!(
    authority = AUTHORITY,
    key = "default_custom_policy.rows.v1",
    label = "rows",
    id = 150,
);

struct CustomPolicy;

impl AllocationPolicy for CustomPolicy {
    type Error = core::convert::Infallible;

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

impl RuntimeBootstrapPolicy for CustomPolicy {
    fn runtime_bootstrap_identity(&self) -> &'static str {
        POLICY_IDENTITY
    }
}

#[test]
fn custom_default_policy_bootstrap_is_identity_bound_and_idempotent() {
    let first = ic_memory::bootstrap_default_memory_manager_with_policy(&CustomPolicy)
        .expect("first custom-policy bootstrap");
    let repeated = ic_memory::bootstrap_default_memory_manager_with_policy(&CustomPolicy)
        .expect("same custom-policy identity");

    assert_eq!(repeated.generation(), first.generation());
    ic_memory::open_default_memory_manager_memory("default_custom_policy.rows.v1", 150)
        .expect("custom-policy committed memory");
}
