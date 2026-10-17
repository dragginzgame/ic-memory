#[path = "support/host.rs"]
mod host;
use host::RUNTIME;

use ic_memory::{
    BootstrapAdmission, MemoryManagerSlot, PolicyIdentity, RuntimeBootstrapPolicy, StableKey,
};

const AUTHORITY: &str = "wasm_core_size_probe";

struct HostPolicy;
impl RuntimeBootstrapPolicy for HostPolicy {
    type Error = &'static str;
    fn validate_allocation(
        &self,
        _key: &StableKey,
        _slot: &MemoryManagerSlot,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn runtime_bootstrap_identity(&self) -> PolicyIdentity {
        PolicyIdentity::new(*b"probe.admission.................")
    }
    fn prepare_bootstrap(&self, admission: &mut BootstrapAdmission<'_>) -> Result<(), Self::Error> {
        let journals: Vec<_> = admission
            .recovered_allocations()
            .filter(|r| {
                r.stable_key().as_str().starts_with("wasm_core_size_probe.")
                    && r.stable_key().as_str().ends_with(".journal.v1")
                    && !admission.is_declared(r.stable_key())
            })
            .map(|r| r.stable_key().clone())
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
    let Ok(declarations) = probe_declarations() else {
        return 0;
    };
    RUNTIME.with(|slot| {
        let Ok(mut slot) = slot.try_borrow_mut() else {
            return 0;
        };
        if slot.is_none() {
            let Ok(config) = ic_memory::MemoryManagerConfig::new(128) else {
                return 0;
            };
            let Ok(runtime) =
                host::Runtime::new_with_config(ic_memory::DefaultMemoryImpl::default(), config)
            else {
                return 0;
            };
            *slot = Some(runtime);
        }
        let Some(runtime) = slot.as_mut() else {
            return 0;
        };
        runtime
            .bootstrap(&declarations, &pool(), &HostPolicy)
            .unwrap_or(0)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn ic_memory_wasm_core_size_probe_open() -> u32 {
    RUNTIME.with(|slot| {
        let Ok(slot) = slot.try_borrow() else {
            return u32::MAX;
        };
        slot.as_ref()
            .and_then(|runtime| runtime.open_memory("wasm_core_size_probe.rows.v1").ok())
            .map_or(u32::MAX, |memory| {
                u32::try_from(ic_memory::Memory::size(&memory)).unwrap_or(u32::MAX)
            })
    })
}

fn pool() -> ic_memory::MemoryAllocationPool {
    ic_memory::MemoryAllocationPool::new(
        vec![
            ic_memory::MemoryAuthority::new("wasm_core_size_probe", "wasm_core_size_probe.")
                .unwrap(),
        ],
        vec![],
    )
    .unwrap()
}

fn probe_declarations()
-> Result<ic_memory::SealedDeclarationSnapshot, ic_memory::MemoryDeclarationError> {
    let request = ic_memory::MemoryRequest::new(AUTHORITY, "wasm_core_size_probe.rows.v1")?;
    ic_memory::SealedDeclarationSnapshot::new(vec![request])
}
