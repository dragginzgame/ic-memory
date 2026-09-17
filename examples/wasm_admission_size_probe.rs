use ic_memory::{
    AllocationPolicy, AllocationSlotDescriptor, BootstrapAdmission, PolicyIdentity,
    PolicyIdentityError, RuntimeBootstrapPolicy, StableKey,
};

const AUTHORITY: &str = "wasm_core_size_probe";
ic_memory::ic_memory_range!(
    authority = AUTHORITY,
    start = 160,
    end = 160,
    mode = Allowed
);
ic_memory::ic_memory_declaration!(authority = AUTHORITY, key = "wasm_core_size_probe.rows.v1");

struct HostPolicy;
impl AllocationPolicy for HostPolicy {
    type Error = &'static str;
    fn validate_key(&self, _: &StableKey) -> Result<(), Self::Error> {
        Ok(())
    }
    fn validate_slot(
        &self,
        _: &StableKey,
        _: &AllocationSlotDescriptor,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
    fn validate_reserved_slot(
        &self,
        _: &StableKey,
        _: &AllocationSlotDescriptor,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}
impl RuntimeBootstrapPolicy for HostPolicy {
    fn runtime_bootstrap_identity(&self) -> Result<PolicyIdentity, PolicyIdentityError> {
        PolicyIdentity::new("probe.admission", 1)
    }
    fn prepare_bootstrap(&self, admission: &mut BootstrapAdmission<'_>) -> Result<(), Self::Error> {
        let journals: Vec<_> = admission
            .recovered_allocations()
            .filter(|r| {
                r.stable_key.as_str().starts_with("wasm_core_size_probe.")
                    && r.stable_key.as_str().ends_with(".journal.v1")
                    && !admission.is_declared(r.stable_key)
            })
            .map(|r| r.stable_key.clone())
            .collect();
        for key in journals {
            admission
                .include_historical(AUTHORITY, key.as_str())
                .map_err(|_| "selection rejected")?;
        }
        Ok(())
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ic_memory_wasm_core_size_probe_bootstrap() -> u64 {
    ic_memory::bootstrap_default_memory_manager_with_policy(&HostPolicy)
        .map_or(0, |allocations| allocations.generation())
}

#[unsafe(no_mangle)]
pub extern "C" fn ic_memory_wasm_core_size_probe_open() -> u32 {
    ic_memory::open_default_memory_manager_memory_by_key("wasm_core_size_probe.rows.v1").map_or(
        u32::MAX,
        |memory| {
            u32::try_from(ic_memory::ic_stable_structures::Memory::size(&memory))
                .unwrap_or(u32::MAX)
        },
    )
}
