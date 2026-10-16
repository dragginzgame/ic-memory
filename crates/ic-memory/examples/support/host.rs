//! This fixture binary owns its instance; the library owns no global runtime.
use ic_memory::{MemoryRuntime, ic_stable_structures::DefaultMemoryImpl};
use std::cell::RefCell;

/// Fixture-owned runtime with the platform default backing.
pub type Runtime = MemoryRuntime<DefaultMemoryImpl>;

thread_local! {
    pub static RUNTIME: RefCell<Option<Runtime>> = const { RefCell::new(None) };
}
