use ic_stable_structures::{Memory, memory_manager::VirtualMemory};
use std::rc::Rc;

///
/// RuntimeMemory
///
/// Cloneable virtual memory opened through a runtime's committed authority.
/// Implements [`Memory`] for stable structures without exposing the backing
/// memory or an alternate manager. Cloning does not require `M: Clone`.
/// Reads delegate to the upstream memory implementation, including its
/// optimized support for uninitialized destinations through `Memory::read_unsafe`.
///

pub struct RuntimeMemory<M: Memory>(pub(super) VirtualMemory<Rc<M>>);

impl<M: Memory> Clone for RuntimeMemory<M> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<M: Memory> Memory for RuntimeMemory<M> {
    fn size(&self) -> u64 {
        self.0.size()
    }
    fn grow(&self, pages: u64) -> i64 {
        self.0.grow(pages)
    }
    fn read(&self, offset: u64, dst: &mut [u8]) {
        self.0.read(offset, dst);
    }
    #[allow(
        unsafe_code,
        reason = "delegate the upstream raw-read contract unchanged"
    )]
    unsafe fn read_unsafe(&self, offset: u64, dst: *mut u8, count: usize) {
        // SAFETY: The caller supplies a valid destination disjoint from this
        // memory and its backing. Forwarding preserves the pointer and count;
        // VirtualMemory owns bucket translation and initializes the destination
        // on success. After a panic, initialization must not be assumed.
        unsafe { self.0.read_unsafe(offset, dst, count) }
    }
    fn write(&self, offset: u64, src: &[u8]) {
        self.0.write(offset, src);
    }
}
