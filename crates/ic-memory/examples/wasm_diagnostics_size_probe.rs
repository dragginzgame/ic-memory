#[path = "support/host.rs"]
mod host;
use host::RUNTIME;

const AUTHORITY: &str = "wasm_diagnostics_size_probe";

#[unsafe(no_mangle)]
pub extern "C" fn ic_memory_wasm_diagnostics_size_probe_bootstrap() -> u64 {
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
            .bootstrap(&declarations, &pool(), &ic_memory::GenericAllocationPolicy)
            .map_or(0, ic_memory::CommittedAllocations::generation)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn ic_memory_wasm_diagnostics_size_probe_report() -> u32 {
    RUNTIME
        .with(|slot| {
            let slot = slot.try_borrow().ok()?;
            let runtime = slot.as_ref()?;
            let declarations = probe_declarations().ok()?;
            let report =
                runtime.doctor_report(&declarations, &pool(), &ic_memory::GenericAllocationPolicy);
            let mut bytes = Vec::new();
            ciborium::into_writer(&report, &mut bytes).ok()?;
            u32::try_from(bytes.len()).ok()
        })
        .unwrap_or(u32::MAX)
}

#[unsafe(no_mangle)]
pub extern "C" fn ic_memory_wasm_diagnostics_size_probe_export() -> u32 {
    RUNTIME
        .with(|slot| {
            let slot = slot.try_borrow().ok()?;
            let runtime = slot.as_ref()?;
            let report = runtime.diagnostic_export().ok()?;
            let mut bytes = Vec::new();
            ciborium::into_writer(&report, &mut bytes).ok()?;
            u32::try_from(bytes.len()).ok()
        })
        .unwrap_or(u32::MAX)
}

#[unsafe(no_mangle)]
pub extern "C" fn ic_memory_wasm_diagnostics_size_probe_allocations() -> u32 {
    RUNTIME
        .with(|slot| {
            let slot = slot.try_borrow().ok()?;
            let runtime = slot.as_ref()?;
            let report = runtime.memory_allocations().ok()?;
            let mut bytes = Vec::new();
            ciborium::into_writer(&report, &mut bytes).ok()?;
            u32::try_from(bytes.len()).ok()
        })
        .unwrap_or(u32::MAX)
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
    let request = ic_memory::MemoryRequest::new(AUTHORITY, "wasm_diagnostics_size_probe.rows.v1")?;
    ic_memory::SealedDeclarationSnapshot::new(vec![request])
}
