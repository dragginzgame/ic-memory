const AUTHORITY: &str = "wasm_core_size_probe";

ic_memory::ic_memory_range!(authority = AUTHORITY, start = 160, end = 160);

ic_memory::ic_memory_declaration!(
    authority = AUTHORITY,
    key = "wasm_core_size_probe.rows.v1",
    label = "rows",
    id = 160,
);

#[unsafe(no_mangle)]
pub extern "C" fn ic_memory_wasm_core_size_probe_bootstrap() -> u64 {
    ic_memory::bootstrap_default_memory_manager().map_or(0, |allocations| allocations.generation())
}

#[unsafe(no_mangle)]
pub extern "C" fn ic_memory_wasm_core_size_probe_open() -> u32 {
    ic_memory::open_default_memory_manager_memory("wasm_core_size_probe.rows.v1", 160)
        .map_or(u32::MAX, |memory| {
            u32::try_from(ic_stable_structures::Memory::size(&memory)).unwrap_or(u32::MAX)
        })
}
