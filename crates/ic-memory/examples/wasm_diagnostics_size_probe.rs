const AUTHORITY: &str = "wasm_diagnostics_size_probe";

#[unsafe(no_mangle)]
pub extern "C" fn ic_memory_wasm_diagnostics_size_probe_bootstrap() -> u64 {
    let Ok(declarations) = probe_declarations() else {
        return 0;
    };
    ic_memory::bootstrap_default_memory_manager(&declarations, &pool())
        .map_or(0, |allocations| allocations.generation())
}

#[unsafe(no_mangle)]
pub extern "C" fn ic_memory_wasm_diagnostics_size_probe_report() -> u32 {
    let Ok(declarations) = probe_declarations() else {
        return u32::MAX;
    };
    let Ok(report) = ic_memory::default_memory_manager_doctor_report(&declarations, &pool()) else {
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

#[unsafe(no_mangle)]
pub extern "C" fn ic_memory_wasm_diagnostics_size_probe_allocations() -> u32 {
    let Ok(report) = ic_memory::default_memory_manager_memory_allocations() else {
        return u32::MAX;
    };
    let mut bytes = Vec::new();
    if ciborium::into_writer(&report, &mut bytes).is_err() {
        return u32::MAX;
    }
    u32::try_from(bytes.len()).unwrap_or(u32::MAX)
}

fn pool() -> ic_memory::MemoryAllocationPool {
    ic_memory::MemoryAllocationPool::new(
        vec![
            ic_memory::MemoryAuthority::new(
                "wasm_diagnostics_size_probe",
                "wasm_diagnostics_size_probe.",
            )
            .unwrap(),
        ],
        vec![],
    )
    .unwrap()
}

fn probe_declarations()
-> Result<ic_memory::SealedDeclarationSnapshot, ic_memory::MemoryDeclarationError> {
    let request = ic_memory::MemoryRequest::new(
        AUTHORITY,
        "wasm_diagnostics_size_probe.rows.v1",
        ic_memory::SchemaMetadata::default(),
    )?;
    ic_memory::SealedDeclarationSnapshot::new(vec![request])
}
