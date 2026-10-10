const AUTHORITY: &str = "wasm_runtime_integration_size_probe";
const KEY: &str = "wasm_runtime_integration_size_probe.rows.v1";

ic_memory::ic_memory_declaration!(
    authority = AUTHORITY,
    key = "wasm_runtime_integration_size_probe.rows.v1"
);

#[unsafe(no_mangle)]
pub extern "C" fn ic_memory_wasm_runtime_integration_size_probe_bootstrap() -> u64 {
    let Ok(config) = ic_memory::MemoryManagerConfig::new(16) else {
        return 0;
    };
    ic_memory::bootstrap_default_memory_manager_with_config(
        config,
        &pool(),
        &ic_memory::GenericAllocationPolicy,
    )
    .map_or(0, |allocations| allocations.generation())
}

#[unsafe(no_mangle)]
pub extern "C" fn ic_memory_wasm_runtime_integration_size_probe_grow(pages: u64) -> u32 {
    ic_memory::open_default_memory_manager_memory(KEY)
        .ok()
        .and_then(|memory| memory.grow(pages).ok())
        .map_or(u32::MAX, |previous| {
            u32::try_from(previous).unwrap_or(u32::MAX)
        })
}

#[unsafe(no_mangle)]
pub extern "C" fn ic_memory_wasm_runtime_integration_size_probe_adopt() -> u32 {
    let Ok(requirements) = ic_memory::sealed_declaration_snapshot() else {
        return u32::MAX;
    };
    if ic_memory::verify_default_memory_manager_authority(&requirements, AUTHORITY).is_err() {
        return u32::MAX;
    }
    ic_memory::default_memory_manager_memory_id(KEY).map_or(u32::MAX, u32::from)
}

#[unsafe(no_mangle)]
pub extern "C" fn ic_memory_wasm_runtime_integration_size_probe_summary() -> u32 {
    let Ok(summary) = ic_memory::default_memory_manager_memory_allocation_summary() else {
        return u32::MAX;
    };
    let mut bytes = Vec::new();
    if ciborium::into_writer(&summary, &mut bytes).is_err() {
        return u32::MAX;
    }
    u32::try_from(bytes.len()).unwrap_or(u32::MAX)
}

fn pool() -> ic_memory::MemoryAllocationPool {
    ic_memory::MemoryAllocationPool::new(
        vec![
            ic_memory::MemoryAuthority::new(
                "wasm_runtime_integration_size_probe",
                "wasm_runtime_integration_size_probe.",
            )
            .unwrap(),
        ],
        vec![],
    )
    .unwrap()
}
