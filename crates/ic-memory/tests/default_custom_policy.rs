use ic_memory::{
    AllocationPolicy, MemoryManagerSlot, PolicyIdentity, PolicyIdentityError,
    RuntimeBootstrapPolicy, StableKey,
};

const AUTHORITY: &str = "default_custom_policy";
const POLICY_IDENTITY: &str = "default-custom-policy";

ic_memory::ic_memory_declaration!(authority = AUTHORITY, key = "default_custom_policy.rows.v1");

struct CustomPolicy;

impl AllocationPolicy for CustomPolicy {
    type Error = core::convert::Infallible;

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

impl RuntimeBootstrapPolicy for CustomPolicy {
    fn runtime_bootstrap_identity(&self) -> Result<PolicyIdentity, PolicyIdentityError> {
        PolicyIdentity::new(POLICY_IDENTITY, 1)
    }
}

#[test]
fn custom_default_policy_bootstrap_is_identity_bound_and_idempotent() {
    assert!(!ic_memory::is_default_memory_manager_bootstrapped().unwrap());
    assert_eq!(
        ic_memory::committed_allocations(),
        Err(ic_memory::RuntimeOpenError::NotBootstrapped)
    );
    let config = ic_memory::MemoryManagerConfig::new(16).unwrap();
    let first =
        ic_memory::bootstrap_default_memory_manager_with_config(config, &pool(), &CustomPolicy)
            .expect("first custom-policy bootstrap");
    assert!(ic_memory::is_default_memory_manager_bootstrapped().unwrap());
    assert_eq!(ic_memory::committed_allocations().unwrap(), first);
    let before = ic_memory::default_memory_manager_memory_allocations().unwrap();
    assert_eq!(before.bucket_size_pages, 16);
    assert!(matches!(
        ic_memory::bootstrap_default_memory_manager_with_config(
            config,
            &pool(),
            &ic_memory::GenericAllocationPolicy
        ),
        Err(ic_memory::RuntimeBootstrapError::PolicyIdentityMismatch { .. })
    ));
    assert_eq!(
        ic_memory::default_memory_manager_memory_allocations().unwrap(),
        before
    );
    assert_eq!(ic_memory::committed_allocations().unwrap(), first);
    let repeated = ic_memory::bootstrap_default_memory_manager_with_policy(&pool(), &CustomPolicy)
        .expect("same custom-policy identity");

    assert_eq!(repeated.generation(), first.generation());
    let doctor =
        ic_memory::default_memory_manager_doctor_report_with_policy(&pool(), &CustomPolicy)
            .expect("custom-policy doctor report");
    assert!(matches!(
        doctor.bootstrap_binding,
        ic_memory::DiagnosticCheck::Passed
    ));
    assert_eq!(
        doctor
            .tested_policy_identity
            .expect("valid custom-policy identity"),
        PolicyIdentity::new(POLICY_IDENTITY, 1).expect("valid identity")
    );
    ic_memory::open_default_memory_manager_memory("default_custom_policy.rows.v1")
        .expect("custom-policy committed memory");
}

fn pool() -> ic_memory::MemoryAllocationPool {
    ic_memory::MemoryAllocationPool::new(
        vec![
            ic_memory::MemoryAuthority::new("default_custom_policy", "default_custom_policy.")
                .unwrap(),
        ],
        vec![],
    )
    .unwrap()
}
