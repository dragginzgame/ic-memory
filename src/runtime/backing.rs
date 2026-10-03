use ic_stable_structures::{Memory, memory_manager::VirtualMemory};
use std::{cell::RefCell, rc::Rc};

// Backing, immutable geometry and one live bucket count for the sole manager,
// shared by the runtime and all handles, including the ledger. The count is
// seeded from validated metadata on reopen and updated only after grow.
pub(super) struct GrowthState<M: Memory> {
    pub backing: Rc<M>,
    pub bucket_size_pages: u16,
    pub allocated_buckets: RefCell<u16>,
}

///
/// RuntimeMemory
///
/// Cloneable virtual memory opened through a runtime's committed authority.
/// Implements [`Memory`] for stable structures without exposing the backing
/// memory or an alternate manager. Cloning does not require `M: Clone`.
/// Reads delegate to the upstream memory implementation, including its
/// optimized support for uninitialized destinations through `Memory::read_unsafe`.
/// Growth reserves physical capacity before assigning manager buckets. Ordinary
/// backing refusal, arithmetic overflow and bucket exhaustion return typed errors
/// without changing virtual extents or manager metadata. Backing implementations
/// must obey the `Memory` contract; traps and partial writes are not transactions
/// on native memory. Typed collections retain their own failure behavior.
///

pub struct RuntimeMemory<M: Memory> {
    pub(super) memory: VirtualMemory<Rc<M>>,
    pub(super) growth: Rc<GrowthState<M>>,
}

impl<M: Memory> Clone for RuntimeMemory<M> {
    fn clone(&self) -> Self {
        Self {
            memory: self.memory.clone(),
            growth: Rc::clone(&self.growth),
        }
    }
}

impl<M: Memory> RuntimeMemory<M> {
    /// Grow by the requested pages, returning the previous virtual page count.
    ///
    /// Capacity is reserved before assigning manager buckets. Ordinary refusal
    /// preserves virtual extents and manager metadata and permits retry. The
    /// upstream [`Memory::grow`] adapter translates errors into its required
    /// `-1` sentinel; direct runtime callers receive [`super::RuntimeGrowError`].
    pub fn grow(&self, pages: u64) -> Result<u64, super::RuntimeGrowError> {
        use super::RuntimeGrowError;
        let mut allocated = self
            .growth
            .allocated_buckets
            .try_borrow_mut()
            .map_err(|_| RuntimeGrowError::ReentrantAccess)?;
        let old_pages = self.memory.size();
        let new_pages = old_pages
            .checked_add(pages)
            .ok_or(RuntimeGrowError::ArithmeticOverflow)?;
        let bucket_pages = u64::from(self.growth.bucket_size_pages);
        let extra = new_pages.div_ceil(bucket_pages) - old_pages.div_ceil(bucket_pages);
        let total = u64::from(*allocated)
            .checked_add(extra)
            .ok_or(RuntimeGrowError::ArithmeticOverflow)?;
        if total > u64::from(super::layout::BUCKET_CAPACITY) {
            return Err(RuntimeGrowError::BucketExhausted {
                required_buckets: total,
                capacity: super::layout::BUCKET_CAPACITY,
            });
        }
        let total_buckets =
            u16::try_from(total).map_err(|_| RuntimeGrowError::ArithmeticOverflow)?;
        let required_pages = 1 + total * bucket_pages;
        let physical_pages = self.growth.backing.size();
        if required_pages > physical_pages
            && self.growth.backing.grow(required_pages - physical_pages) < 0
        {
            return Err(RuntimeGrowError::BackingRefused {
                additional_pages: required_pages - physical_pages,
            });
        }
        let previous =
            u64::try_from(self.memory.grow(pages)).map_err(|_| RuntimeGrowError::ManagerRefused)?;
        *allocated = total_buckets;
        Ok(previous)
    }
}

impl<M: Memory> Memory for RuntimeMemory<M> {
    fn size(&self) -> u64 {
        self.memory.size()
    }
    fn grow(&self, pages: u64) -> i64 {
        // The upstream trait fixes the sentinel contract. Direct calls use the
        // inherent typed method; virtual extents fit in i64 by bucket capacity.
        Self::grow(self, pages).map_or(-1, |previous| i64::try_from(previous).unwrap_or(-1))
    }
    fn read(&self, offset: u64, dst: &mut [u8]) {
        self.memory.read(offset, dst);
    }
    #[expect(
        unsafe_code,
        reason = "delegate the upstream raw-read contract unchanged"
    )]
    unsafe fn read_unsafe(&self, offset: u64, dst: *mut u8, count: usize) {
        // SAFETY: The caller supplies a valid destination disjoint from this
        // memory and its backing. Forwarding preserves the pointer and count;
        // VirtualMemory owns bucket translation and initializes the destination
        // on success. After a panic, initialization must not be assumed.
        unsafe { self.memory.read_unsafe(offset, dst, count) }
    }
    fn write(&self, offset: u64, src: &[u8]) {
        self.memory.write(offset, src);
    }
}
