use ic_stable_structures::{Memory, memory_manager::VirtualMemory};
use std::rc::Rc;

// Sharing the owned backing preserves support for non-Clone memories. Only the
// manager writes it; attribution borrows it read-only and never creates a manager.
pub(super) struct SharedBacking<M>(pub(super) Rc<M>);

impl<M> Clone for SharedBacking<M> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<M: Memory> Memory for SharedBacking<M> {
    fn size(&self) -> u64 {
        self.0.size()
    }
    fn grow(&self, pages: u64) -> i64 {
        self.0.grow(pages)
    }
    fn read(&self, offset: u64, dst: &mut [u8]) {
        self.0.read(offset, dst);
    }
    fn write(&self, offset: u64, src: &[u8]) {
        self.0.write(offset, src);
    }
}

///
/// RuntimeMemory
///
/// Cloneable virtual memory opened through a runtime's committed authority.
/// Implements [`Memory`] for stable structures without exposing the backing
/// memory or an alternate manager. Cloning does not require `M: Clone`.
///

pub struct RuntimeMemory<M: Memory>(pub(super) VirtualMemory<SharedBacking<M>>);

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
    fn write(&self, offset: u64, src: &[u8]) {
        self.0.write(offset, src);
    }
}
