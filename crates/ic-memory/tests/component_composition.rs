use ic_memory::RuntimeMemory;
use ic_stable_structures::DefaultMemoryImpl;
use std::cell::RefCell;

const MACRO_AUTHORITY: &str = "runtime_macros";

fn snapshot() -> ic_memory::SealedDeclarationSnapshot {
    ic_memory::SealedDeclarationSnapshot::new(vec![
        ic_memory::MemoryRequest::new(
            MACRO_AUTHORITY,
            "macro.logical.rows.v1",
            ic_memory::SchemaMetadata::default(),
        )
        .unwrap(),
        ic_memory::MemoryRequest::new(
            MACRO_AUTHORITY,
            "macro.integration.users.v1",
            ic_memory::SchemaMetadata::default(),
        )
        .unwrap(),
    ])
    .unwrap()
}

thread_local! {
    static MACRO_MEMORY: RefCell<Option<RuntimeMemory<DefaultMemoryImpl>>> = const { RefCell::new(None) };
}

#[test]
fn host_injects_component_memory_that_outlives_its_runtime() {
    let mut host = ic_memory::MemoryRuntime::new(DefaultMemoryImpl::default()).unwrap();
    let validated = host
        .bootstrap(&snapshot(), &pool(), &ic_memory::GenericAllocationPolicy)
        .expect("bootstrap")
        .clone();
    host.diagnostic_export()
        .expect("bootstrapped runtime should expose its local ledger");

    assert!(
        validated
            .declarations()
            .iter()
            .any(|declaration| declaration.stable_key().as_str() == "macro.integration.users.v1")
    );
    let logical = host
        .open_memory("macro.logical.rows.v1")
        .expect("key-only host adoption");
    assert_eq!(ic_stable_structures::Memory::size(&logical), 0);
    assert_eq!(
        validated
            .slot_for(&ic_memory::StableKey::parse("macro.logical.rows.v1").unwrap())
            .unwrap()
            .id(),
        11
    );
    let component_memory = host.open_memory("macro.integration.users.v1").unwrap();
    MACRO_MEMORY.with(|memory| *memory.borrow_mut() = Some(component_memory));
    drop(host);
    MACRO_MEMORY.with(|memory| {
        assert_eq!(
            ic_stable_structures::Memory::size(memory.borrow().as_ref().unwrap()),
            0,
        );
    });
}

fn pool() -> ic_memory::MemoryAllocationPool {
    ic_memory::MemoryAllocationPool::new(
        vec![ic_memory::MemoryAuthority::new("runtime_macros", "macro.").unwrap()],
        vec![],
    )
    .unwrap()
}
