use super::{MemoryManagerConfig, MemoryRuntime, RuntimeMemory};
use ic_stable_structures::{Memory, VectorMemory};
use std::{
    cell::{Cell, RefCell},
    mem::MaybeUninit,
    panic::{AssertUnwindSafe, catch_unwind},
};

const PAGE: u64 = 65_536;

#[derive(Default)]
struct ObservedMemory {
    memory: VectorMemory,
    safe_reads: Cell<usize>,
    unsafe_reads: RefCell<Vec<(u64, usize)>>,
    fail_on_read: Cell<Option<usize>>,
    writes: Cell<usize>,
    grows: Cell<usize>,
}

impl Memory for ObservedMemory {
    fn size(&self) -> u64 {
        self.memory.size()
    }

    fn grow(&self, pages: u64) -> i64 {
        self.grows.set(self.grows.get() + 1);
        self.memory.grow(pages)
    }

    fn read(&self, offset: u64, dst: &mut [u8]) {
        self.safe_reads.set(self.safe_reads.get() + 1);
        self.memory.read(offset, dst);
    }

    unsafe fn read_unsafe(&self, offset: u64, dst: *mut u8, count: usize) {
        self.unsafe_reads.borrow_mut().push((offset, count));
        assert_ne!(
            self.fail_on_read.get(),
            Some(self.unsafe_reads.borrow().len()),
            "injected read failure",
        );
        // SAFETY: Forward the caller's valid, non-overlapping destination.
        // Instrumentation never examines destination bytes.
        unsafe { self.memory.read_unsafe(offset, dst, count) }
    }

    fn write(&self, offset: u64, src: &[u8]) {
        self.writes.set(self.writes.get() + 1);
        self.memory.write(offset, src);
    }
}

fn runtime(bucket_pages: u16) -> MemoryRuntime<ObservedMemory> {
    MemoryRuntime::new_with_config(
        ObservedMemory::default(),
        MemoryManagerConfig::new(bucket_pages).unwrap(),
    )
    .unwrap()
}

// Exercise the private transport independently of declaration registration.
// Public open authority and reserved IDs are covered by the runtime tests.
fn handle<M: Memory>(runtime: &MemoryRuntime<M>, id: u8) -> RuntimeMemory<M> {
    runtime.memory(id)
}

fn read_uninitialized<const N: usize>(memory: &impl Memory, offset: u64) -> [u8; N] {
    let mut dst = MaybeUninit::<[u8; N]>::uninit();
    // SAFETY: dst owns N writable bytes, disjoint from the source memory.
    unsafe { memory.read_unsafe(offset, dst.as_mut_ptr().cast(), N) };
    // SAFETY: A successful read initializes all N bytes. A panic skips this.
    unsafe { dst.assume_init() }
}

#[test]
fn safe_and_uninitialized_reads_reach_specialized_backing_without_effects() {
    let runtime = runtime(1);
    let memory = handle(&runtime, 1);
    assert_eq!(memory.grow(1), Ok(0));
    memory.write(7, &[11, 22, 33, 44]);
    let backing = &runtime.growth.backing;
    backing.unsafe_reads.borrow_mut().clear();
    let safe_reads_before = backing.safe_reads.get();
    let effects_before = (backing.writes.get(), backing.grows.get(), backing.size());
    let bytes_before = backing.memory.borrow().clone();

    assert_eq!(read_uninitialized::<4>(&memory, 7), [11, 22, 33, 44]);
    let mut initialized = [0xA5; 4];
    memory.read(7, &mut initialized);
    assert_eq!(initialized, [11, 22, 33, 44]);
    assert_eq!(backing.safe_reads.get(), safe_reads_before);
    assert_eq!(*backing.unsafe_reads.borrow(), [(PAGE + 7, 4); 2]);
    assert_eq!(
        (backing.writes.get(), backing.grows.get(), backing.size()),
        effects_before,
    );
    assert_eq!(*backing.memory.borrow(), bytes_before);
}

#[test]
fn forwarding_does_not_zero_the_destination_before_a_backing_failure() {
    let runtime = runtime(1);
    let memory = handle(&runtime, 1);
    memory.grow(1).unwrap();
    runtime.growth.backing.unsafe_reads.borrow_mut().clear();
    runtime.growth.backing.fail_on_read.set(Some(1));
    let mut dst = [0xA5; 4];
    let result = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: dst is a separate, initialized writable allocation.
        unsafe { memory.read_unsafe(0, dst.as_mut_ptr(), dst.len()) }
    }));
    assert!(result.is_err());
    // This instrumented backing panics before copying. A wrapper fallback
    // would have zeroed these initially initialized bytes before reaching it.
    assert_eq!(dst, [0xA5; 4]);
}

#[test]
fn uninitialized_reads_translate_discontiguous_buckets_and_partial_failures() {
    const COUNT: usize = 65_536 + 8;

    let runtime = runtime(1);
    let memory = handle(&runtime, 1);
    let other = handle(&runtime, 2);
    memory.grow(1).unwrap();
    other.grow(1).unwrap();
    memory.grow(1).unwrap();
    other.grow(1).unwrap();
    memory.grow(1).unwrap();
    let mut expected = vec![0x6D; COUNT];
    expected[..4].copy_from_slice(&[1, 2, 3, 4]);
    expected[COUNT - 4..].copy_from_slice(&[5, 6, 7, 8]);
    memory.write(PAGE - 4, &expected);
    runtime.growth.backing.unsafe_reads.borrow_mut().clear();
    assert_eq!(
        read_uninitialized::<COUNT>(&memory, PAGE - 4).as_slice(),
        expected.as_slice(),
    );
    assert_eq!(
        *runtime.growth.backing.unsafe_reads.borrow(),
        [(2 * PAGE - 4, 4), (3 * PAGE, 65_536), (5 * PAGE, 4)],
    );

    runtime.growth.backing.unsafe_reads.borrow_mut().clear();
    runtime.growth.backing.fail_on_read.set(Some(2));
    // A panic after the first segment must never reach assume_init or inspect
    // the unread portion. MaybeUninit requires no initialized bytes on drop.
    let result = catch_unwind(AssertUnwindSafe(|| {
        read_uninitialized::<COUNT>(&memory, PAGE - 4)
    }));
    assert!(result.is_err());
    assert_eq!(runtime.growth.backing.unsafe_reads.borrow().len(), 2);
}

#[test]
fn default_read_unsafe_and_clones_support_borrowed_nonclone_backing() {
    struct DefaultOnly<'a>(&'a VectorMemory);
    impl Memory for DefaultOnly<'_> {
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
    let backing = VectorMemory::default();
    let runtime =
        MemoryRuntime::new_with_config(DefaultOnly(&backing), MemoryManagerConfig::new(1).unwrap())
            .unwrap();
    let memory = handle(&runtime, 1);
    let cold_clone = memory.clone();
    memory.grow(1).unwrap();
    memory.write(PAGE - 3, &[1, 2, 3]);
    assert_eq!(read_uninitialized::<3>(&memory, PAGE - 3), [1, 2, 3]);
    let warm_clone = memory.clone();
    drop(memory);
    drop(runtime);
    assert_eq!(read_uninitialized::<3>(&cold_clone, PAGE - 3), [1, 2, 3]);
    assert_eq!(read_uninitialized::<3>(&warm_clone, PAGE - 3), [1, 2, 3]);
}

#[test]
fn io_bounds_are_independent_of_bucket_cache_and_reject_before_backing_access() {
    let runtime = runtime(8);
    let memory = handle(&runtime, 1);
    for pages in [0, 1] {
        memory.grow(pages).unwrap();
        let extent = pages * PAGE;
        let mut spans = vec![(0, 0, true), (extent, 0, true)];
        if extent != 0 {
            spans.extend([(extent - 1, 1, true), (extent - 1, 2, false)]);
        }
        spans.extend([
            (extent, 1, false),
            (extent + 1, 0, false),
            (8 * PAGE, 1, false),
            (u64::MAX, 0, false),
            (u64::MAX, 1, false),
            (u64::MAX - 1, 2, false),
        ]);
        for warm in [false, true] {
            for &(offset, count, valid) in &spans {
                for operation in 0..3 {
                    let memory = handle(&runtime, 1);
                    if warm && extent != 0 {
                        memory.read(0, &mut [0]);
                    }
                    let backing = &runtime.growth.backing;
                    let reads = backing.safe_reads.get();
                    let raw_reads = backing.unsafe_reads.borrow().len();
                    let writes = backing.writes.get();
                    let result = catch_unwind(AssertUnwindSafe(|| {
                        let mut dst = [0xA5; 2];
                        match operation {
                            0 => memory.read(offset, &mut dst[..count]),
                            // SAFETY: count fits in this separate writable buffer.
                            1 => unsafe { memory.read_unsafe(offset, dst.as_mut_ptr(), count) },
                            _ => memory.write(offset, &[0xA5; 2][..count]),
                        }
                    }));
                    assert_eq!(
                        result.is_ok(),
                        valid,
                        "pages={pages}, warm={warm}, operation={operation}, offset={offset}, count={count}",
                    );
                    if !valid {
                        assert_eq!(backing.safe_reads.get(), reads);
                        assert_eq!(backing.unsafe_reads.borrow().len(), raw_reads);
                        assert_eq!(backing.writes.get(), writes);
                    }
                }
            }
        }
    }
}

#[test]
fn overflowing_writes_preserve_neighbor_and_growth_zeroes() {
    let runtime = runtime(8);
    let neighbor = handle(&runtime, 1);
    neighbor.grow(8).unwrap();
    neighbor.write(8 * PAGE - 1, &[0x6D]);
    let memory = handle(&runtime, 2);
    memory.grow(1).unwrap();
    memory.write(0, &[0x42]); // Warm virtual bucket zero, after the neighbor.
    let before = runtime.growth.backing.memory.borrow().clone();
    for (offset, src) in [(u64::MAX, &[0xFF][..]), (PAGE, &[0xEE][..])] {
        let result = catch_unwind(AssertUnwindSafe(|| memory.write(offset, src)));
        assert_eq!(read_uninitialized::<1>(&neighbor, 8 * PAGE - 1), [0x6D]);
        assert!(result.is_err());
        assert_eq!(*runtime.growth.backing.memory.borrow(), before);
    }
    assert_eq!(read_uninitialized::<1>(&neighbor, 8 * PAGE - 1), [0x6D]);
    memory.grow(1).unwrap();
    assert_eq!(read_uninitialized::<1>(&memory, PAGE), [0]);
}

#[test]
fn byte_io_crosses_four_gib_with_discontiguous_buckets() {
    // Keep real bytes for manager metadata and touched payload only. This
    // exercises u64 byte addresses, unlike metadata-only capacity fixtures.
    struct Sparse {
        pages: Cell<u64>,
        header: VectorMemory,
        bytes: RefCell<std::collections::BTreeMap<u64, u8>>,
    }
    impl Memory for Sparse {
        fn size(&self) -> u64 {
            self.pages.get()
        }
        fn grow(&self, pages: u64) -> i64 {
            let old = self.pages.get();
            self.pages.set(old.checked_add(pages).unwrap());
            i64::try_from(old).unwrap()
        }
        fn read(&self, offset: u64, dst: &mut [u8]) {
            assert!(offset.checked_add(dst.len() as u64).unwrap() <= self.size() * PAGE);
            if offset < PAGE {
                self.header.read(offset, dst);
            } else {
                let bytes = self.bytes.borrow();
                for (index, byte) in dst.iter_mut().enumerate() {
                    *byte = bytes.get(&(offset + index as u64)).copied().unwrap_or(0);
                }
            }
        }
        fn write(&self, offset: u64, src: &[u8]) {
            assert!(offset.checked_add(src.len() as u64).unwrap() <= self.size() * PAGE);
            if offset < PAGE {
                self.header.write(offset, src);
            } else {
                let mut bytes = self.bytes.borrow_mut();
                for (index, byte) in src.iter().enumerate() {
                    bytes.insert(offset + index as u64, *byte);
                }
            }
        }
    }
    let header = VectorMemory::default();
    header.grow(1);
    let runtime = MemoryRuntime::new_with_config(
        Sparse {
            pages: Cell::new(0),
            header,
            bytes: RefCell::default(),
        },
        MemoryManagerConfig::new(128).unwrap(),
    )
    .unwrap();
    let memory = handle(&runtime, 1);
    memory.grow(65_536).unwrap();
    let neighbor = handle(&runtime, 2);
    neighbor.grow(1).unwrap();
    neighbor.write(0, &[0x6D]);
    memory.grow(1).unwrap();
    let offset = (1_u64 << 32) - 1;
    memory.write(offset, &[1, 2, 3]);
    let cold = handle(&runtime, 1);
    assert_eq!(read_uninitialized::<3>(&cold, offset), [1, 2, 3]);
    assert_eq!(read_uninitialized::<3>(&cold, offset), [1, 2, 3]);
    let mut dst = [0; 3];
    memory.read(offset, &mut dst);
    assert_eq!(dst, [1, 2, 3]);
    assert_eq!(read_uninitialized::<1>(&neighbor, 0), [0x6D]);
    assert_eq!(read_uninitialized::<1>(&memory, offset + 3), [0]);
}
