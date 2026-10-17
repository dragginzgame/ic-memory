use ic_memory::{MemoryManagerSlot, PolicyIdentity, RuntimeBootstrapPolicy, StableKey};

const AUTHORITY: &str = "default_custom_policy";
const POLICY_IDENTITY: [u8; 32] = *b"default-custom-policy.v1........";

fn snapshot() -> ic_memory::SealedDeclarationSnapshot {
    ic_memory::SealedDeclarationSnapshot::new(vec![
        ic_memory::MemoryRequest::new(AUTHORITY, "default_custom_policy.rows.v1").unwrap(),
    ])
    .unwrap()
}

struct CustomPolicy;

impl RuntimeBootstrapPolicy for CustomPolicy {
    type Error = core::convert::Infallible;

    fn validate_allocation(
        &self,
        _key: &StableKey,
        _slot: &MemoryManagerSlot,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn runtime_bootstrap_identity(&self) -> PolicyIdentity {
        PolicyIdentity::new(POLICY_IDENTITY)
    }
}

#[test]
fn custom_policy_bootstrap_is_identity_bound_and_idempotent() {
    let mut runtime = ic_memory::MemoryRuntime::new_with_config(
        ic_stable_structures::VectorMemory::default(),
        ic_memory::MemoryManagerConfig::new(16).unwrap(),
    )
    .unwrap();
    assert!(!runtime.is_bootstrapped());
    assert_eq!(
        runtime.committed_allocations(),
        Err(ic_memory::RuntimeOpenError::NotBootstrapped)
    );
    let first = runtime
        .bootstrap(&snapshot(), &pool(), &CustomPolicy)
        .unwrap();
    let before = runtime.memory_allocations().unwrap();
    assert_eq!(before.summary.bucket_size_pages, 16);
    assert!(matches!(
        runtime.bootstrap(&snapshot(), &pool(), &ic_memory::GenericAllocationPolicy),
        Err(ic_memory::RuntimeBootstrapError::PolicyIdentityMismatch { .. })
    ));
    assert_eq!(runtime.memory_allocations().unwrap(), before);
    assert_eq!(
        runtime
            .bootstrap(&snapshot(), &pool(), &CustomPolicy)
            .unwrap(),
        first
    );
    #[cfg(feature = "test-support")]
    {
        let doctor =
            ic_memory::testing::doctor_report(&runtime, &snapshot(), &pool(), &CustomPolicy);
        assert!(matches!(
            doctor.bootstrap_binding,
            ic_memory::testing::DiagnosticCheck::Passed
        ));
    }
    runtime
        .open_memory("default_custom_policy.rows.v1")
        .unwrap();
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
