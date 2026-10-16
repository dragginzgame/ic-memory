#[path = "support/host.rs"]
mod host;
use host::RUNTIME;

const AUTHORITY: &str = "wasm_core_size_probe";

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
            let Ok(runtime) = host::Runtime::new_with_config(
                ic_memory::ic_stable_structures::DefaultMemoryImpl::default(),
                config,
            ) else {
                return 0;
            };
            *slot = Some(runtime);
        }
        let Some(runtime) = slot.as_mut() else {
            return 0;
        };
        runtime
            .bootstrap(&declarations, &pool(), &ic_memory::GenericAllocationPolicy)
            .map_or(0, ic_memory::CommittedAllocations::generation)
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
                u32::try_from(ic_memory::ic_stable_structures::Memory::size(&memory))
                    .unwrap_or(u32::MAX)
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
