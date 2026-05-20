use ic_stable_structures::{DefaultMemoryImpl, memory_manager::VirtualMemory};
use std::{
    cell::RefCell,
    sync::atomic::{AtomicBool, Ordering},
};

struct MacroStore;

static EAGER_INIT_RAN: AtomicBool = AtomicBool::new(false);

ic_memory::ic_memory_range!(start = 130, end = 139);

ic_memory::eager_init!({
    EAGER_INIT_RAN.store(true, Ordering::SeqCst);
});

thread_local! {
    static MACRO_MEMORY: RefCell<Option<VirtualMemory<DefaultMemoryImpl>>> = {
        assert!(ic_memory::runtime::is_default_memory_manager_bootstrapped());
        RefCell::new(Some(ic_memory::ic_memory_key!(
            "macro.integration.users.v1",
            MacroStore,
            130,
        )))
    };
}

#[test]
fn downstream_style_range_and_key_macros_register_then_open_memory() {
    let validated = ic_memory::bootstrap_default_memory_manager().expect("bootstrap");

    assert!(EAGER_INIT_RAN.load(Ordering::SeqCst));
    assert!(
        validated
            .declarations()
            .iter()
            .any(|declaration| declaration.stable_key().as_str() == "macro.integration.users.v1")
    );
    MACRO_MEMORY.with(|memory| assert!(memory.borrow().is_some()));
    ic_memory::runtime::open_default_memory_manager_memory("macro.integration.users.v1", 130)
        .expect("open macro memory");
}
