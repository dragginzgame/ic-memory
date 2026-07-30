const AUTHORITY: &str = "wasm_diagnostics_size_probe";

ic_memory::ic_memory_range!(authority = AUTHORITY, start = 161, end = 161);

ic_memory::ic_memory_declaration!(
    authority = AUTHORITY,
    key = "wasm_diagnostics_size_probe.rows.v1",
    label = "rows",
    id = 161,
);

#[unsafe(no_mangle)]
pub extern "C" fn ic_memory_wasm_diagnostics_size_probe_bootstrap() -> u64 {
    ic_memory::bootstrap_default_memory_manager().map_or(0, |allocations| allocations.generation())
}

#[unsafe(no_mangle)]
pub extern "C" fn ic_memory_wasm_diagnostics_size_probe_report() -> u32 {
    let Ok(report) = ic_memory::default_memory_manager_doctor_report() else {
        return u32::MAX;
    };
    let mut bytes = Vec::new();
    if ciborium::into_writer(&report, &mut bytes).is_err() {
        return u32::MAX;
    }
    u32::try_from(bytes.len()).unwrap_or(u32::MAX)
}

#[unsafe(no_mangle)]
pub extern "C" fn ic_memory_wasm_diagnostics_size_probe_export() -> u32 {
    let Ok(export) = ic_memory::default_memory_manager_diagnostic_export() else {
        return u32::MAX;
    };
    let mut bytes = Vec::new();
    if ciborium::into_writer(&export, &mut bytes).is_err() {
        return u32::MAX;
    }
    u32::try_from(bytes.len()).unwrap_or(u32::MAX)
}
