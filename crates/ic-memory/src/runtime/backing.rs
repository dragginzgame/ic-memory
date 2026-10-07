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
/// IO checks the virtual extent before upstream bucket translation. Empty IO
/// is valid at the end of memory, but not beyond it. Reads preserve the upstream
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
    // The manager bounds page counts by 32768 buckets of at most u16::MAX
    // pages, so converting its virtual extent to bytes cannot overflow u64.
    // Check before the upstream cache: its buckets can include virtual slack,
    // and its span/translation arithmetic is unchecked in release builds.
    #[expect(
        clippy::inline_always,
        reason = "matched PocketIC measurements reduce IO instructions with bounded Wasm growth"
    )]
    #[inline(always)]
    fn check_io_bounds(&self, offset: u64, count: usize) {
        let extent = self.memory.size() * crate::constants::WASM_PAGE_SIZE_BYTES;
        assert!(
            offset <= extent && count as u64 <= extent - offset,
            "virtual memory access out of bounds",
        );
    }

    /// Grow by the requested pages, returning the previous virtual page count.
    ///
    /// Capacity is reserved before assigning manager buckets. Ordinary refusal
    /// preserves virtual extents and manager metadata and permits retry. The
    /// upstream [`Memory::grow`] adapter translates errors into its required
    /// `-1` sentinel; direct runtime callers receive [`super::RuntimeGrowError`].
    /// A zero-page request returns the current extent without backing IO or
    /// manager mutation, after checking for reentrant growth.
    ///
    /// # Panics
    ///
    /// Panics if a private growth-accounting invariant is broken or backing
    /// memory panics.
    pub fn grow(&self, pages: u64) -> Result<u64, super::RuntimeGrowError> {
        use super::RuntimeGrowError;
        let mut allocated = self
            .growth
            .allocated_buckets
            .try_borrow_mut()
            .map_err(|_| RuntimeGrowError::ReentrantAccess)?;
        let old_pages = self.memory.size();
        if pages == 0 {
            return Ok(old_pages);
        }
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
        #[expect(
            clippy::cast_possible_truncation,
            reason = "admission bounds total by the u16 bucket capacity"
        )]
        let total_buckets = total as u16;
        let required_pages = 1 + total * bucket_pages;
        let physical_pages = self.growth.backing.size();
        if required_pages > physical_pages
            && self.growth.backing.grow(required_pages - physical_pages) < 0
        {
            return Err(RuntimeGrowError::BackingRefused {
                additional_pages: required_pages - physical_pages,
            });
        }
        // The pinned manager's only refusal is bucket exhaustion, already
        // checked above while all handles share this exclusive reservation.
        // Physical capacity is reserved before it assigns any buckets.
        assert_eq!(
            self.memory.grow(pages),
            old_pages.cast_signed(),
            "preflighted manager growth returns the previous virtual extent"
        );
        *allocated = total_buckets;
        Ok(old_pages)
    }
}

impl<M: Memory> Memory for RuntimeMemory<M> {
    fn size(&self) -> u64 {
        self.memory.size()
    }
    fn grow(&self, pages: u64) -> i64 {
        // The upstream trait fixes the sentinel contract. Direct calls use the
        // inherent typed method; virtual extents fit in i64 by bucket capacity.
        Self::grow(self, pages).map_or(-1, u64::cast_signed)
    }
    fn read(&self, offset: u64, dst: &mut [u8]) {
        self.check_io_bounds(offset, dst.len());
        self.memory.read(offset, dst);
    }
    #[expect(
        unsafe_code,
        reason = "delegate the upstream raw-read contract unchanged"
    )]
    unsafe fn read_unsafe(&self, offset: u64, dst: *mut u8, count: usize) {
        self.check_io_bounds(offset, count);
        // SAFETY: The caller supplies a valid destination disjoint from this
        // memory and its backing. Forwarding preserves the pointer and count;
        // VirtualMemory owns bucket translation and initializes the destination
        // on success. After a panic, initialization must not be assumed.
        unsafe { self.memory.read_unsafe(offset, dst, count) }
    }
    fn write(&self, offset: u64, src: &[u8]) {
        self.check_io_bounds(offset, src.len());
        self.memory.write(offset, src);
    }
}
