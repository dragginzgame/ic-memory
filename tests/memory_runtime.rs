use ic_memory::{
    AllocationPolicy, AllocationSlotDescriptor, MemoryRuntime, PolicyIdentity, PolicyIdentityError,
    RuntimeBootstrapPolicy, StableKey, sealed_declaration_snapshot,
};
use ic_stable_structures::{Memory, VectorMemory};

const AUTHORITY: &str = "explicit_runtime";

ic_memory::ic_memory_range!(authority = AUTHORITY, start = 140, end = 140,);

ic_memory::ic_memory_declaration!(
    authority = AUTHORITY,
    key = "explicit_runtime.rows.v1",
    label = "rows",
    id = 140,
);

struct AllowAll;

impl AllocationPolicy for AllowAll {
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

impl RuntimeBootstrapPolicy for AllowAll {
    fn runtime_bootstrap_identity(&self) -> Result<PolicyIdentity, PolicyIdentityError> {
        PolicyIdentity::new("explicit-runtime.allow-all", 1)
    }
}

#[test]
fn public_explicit_runtime_bootstraps_opens_and_diagnoses_its_memory() {
    let declarations = sealed_declaration_snapshot().expect("linked declarations");
    let mut runtime = MemoryRuntime::new(VectorMemory::default()).expect("empty backing memory");

    let generation = runtime
        .bootstrap(&declarations, &AllowAll)
        .expect("runtime bootstrap")
        .generation();
    let rows = runtime
        .open_memory("explicit_runtime.rows.v1", 140)
        .expect("committed rows memory");
    rows.grow(1);

    let export = runtime.diagnostic_export().expect("runtime diagnostics");
    assert_eq!(export.current_generation, generation);
    assert!(runtime.doctor_report(&declarations, &AllowAll).bootstrapped);
}
