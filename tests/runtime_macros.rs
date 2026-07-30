use ic_stable_structures::{DefaultMemoryImpl, memory_manager::VirtualMemory};
use std::{
    cell::RefCell,
    sync::atomic::{AtomicBool, Ordering},
};

struct MacroStore;

static EAGER_INIT_RAN: AtomicBool = AtomicBool::new(false);
const MACRO_AUTHORITY: &str = "runtime_macros";

ic_memory::ic_memory_range!(authority = MACRO_AUTHORITY, start = 130, end = 139);

ic_memory::eager_init!({
    EAGER_INIT_RAN.store(true, Ordering::SeqCst);
});

thread_local! {
    static MACRO_MEMORY: RefCell<Option<VirtualMemory<DefaultMemoryImpl>>> = {
        assert!(
            ic_memory::is_default_memory_manager_bootstrapped()
                .expect("default runtime lifecycle")
        );
        RefCell::new(Some(ic_memory::ic_memory_key!(
            authority = MACRO_AUTHORITY,
            key = "macro.integration.users.v1",
            ty = MacroStore,
            id = 130,
        )
        .expect("committed macro memory")))
    };
}

fn bootstrap_and_require_thread_local_ledger() {
    let validated = ic_memory::bootstrap_default_memory_manager().expect("bootstrap");
    ic_memory::default_memory_manager_diagnostic_export()
        .expect("bootstrapped runtime should expose its local ledger");

    assert!(EAGER_INIT_RAN.load(Ordering::SeqCst));
    assert!(
        validated
            .declarations()
            .iter()
            .any(|declaration| declaration.stable_key().as_str() == "macro.integration.users.v1")
    );
    MACRO_MEMORY.with(|memory| assert!(memory.borrow().is_some()));
    ic_memory::open_default_memory_manager_memory("macro.integration.users.v1", 130)
        .expect("open macro memory");
}

#[test]
fn first_libtest_default_runtime_bootstraps_its_own_memory() {
    bootstrap_and_require_thread_local_ledger();
}

#[test]
fn second_libtest_default_runtime_bootstraps_its_own_memory() {
    bootstrap_and_require_thread_local_ledger();
}
