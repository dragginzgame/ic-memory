use super::{MemoryManagerConfig, MemoryRuntime, RuntimeMemory};
use ic_stable_structures::{Memory, VectorMemory, memory_manager::MemoryId};
use std::{
    cell::{Cell, RefCell},
    mem::MaybeUninit,
    panic::{AssertUnwindSafe, catch_unwind},
    ptr::NonNull,
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
    RuntimeMemory(runtime.memory_manager.get(MemoryId::new(id)))
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
    assert_eq!(memory.grow(1), 0);
    memory.write(7, &[11, 22, 33, 44]);
    let backing = &runtime.backing;
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
    memory.grow(1);
    runtime.backing.unsafe_reads.borrow_mut().clear();
    runtime.backing.fail_on_read.set(Some(1));
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
    memory.grow(1);
    other.grow(1);
    memory.grow(1);
    other.grow(1);
    memory.grow(1);
    let mut expected = vec![0x6D; COUNT];
    expected[..4].copy_from_slice(&[1, 2, 3, 4]);
    expected[COUNT - 4..].copy_from_slice(&[5, 6, 7, 8]);
    memory.write(PAGE - 4, &expected);
    runtime.backing.unsafe_reads.borrow_mut().clear();
    assert_eq!(
        read_uninitialized::<COUNT>(&memory, PAGE - 4).as_slice(),
        expected.as_slice(),
    );
    assert_eq!(
        *runtime.backing.unsafe_reads.borrow(),
        [(2 * PAGE - 4, 4), (3 * PAGE, 65_536), (5 * PAGE, 4)],
    );

    runtime.backing.unsafe_reads.borrow_mut().clear();
    runtime.backing.fail_on_read.set(Some(2));
    // A panic after the first segment must never reach assume_init or inspect
    // the unread portion. MaybeUninit requires no initialized bytes on drop.
    let result = catch_unwind(AssertUnwindSafe(|| {
        read_uninitialized::<COUNT>(&memory, PAGE - 4)
    }));
    assert!(result.is_err());
    assert_eq!(runtime.backing.unsafe_reads.borrow().len(), 2);
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
    memory.grow(1);
    memory.write(PAGE - 3, &[1, 2, 3]);
    assert_eq!(read_uninitialized::<3>(&memory, PAGE - 3), [1, 2, 3]);
    let warm_clone = memory.clone();
    drop(memory);
    drop(runtime);
    assert_eq!(read_uninitialized::<3>(&cold_clone, PAGE - 3), [1, 2, 3]);
    assert_eq!(read_uninitialized::<3>(&warm_clone, PAGE - 3), [1, 2, 3]);
}

#[test]
fn empty_reads_and_source_bounds_match_upstream_for_cold_and_warm_handles() {
    let runtime = runtime(8);
    let memory = handle(&runtime, 1);
    let empty = NonNull::<u8>::dangling().as_ptr();
    // SAFETY: Non-null aligned pointers are valid for these zero-byte reads.
    unsafe { memory.read_unsafe(0, empty, 0) };
    memory.grow(1);
    unsafe { memory.read_unsafe(PAGE, empty, 0) };
    assert_eq!(read_uninitialized::<1>(&memory, PAGE - 1), [0]);

    for offset in [PAGE, PAGE + 1, 8 * PAGE, u64::MAX] {
        for warm in [false, true] {
            for count in [0, 1] {
                let wrapped = handle(&runtime, 1);
                let direct = runtime.memory_manager.get(MemoryId::new(1));
                if warm {
                    assert_eq!(read_uninitialized::<1>(&wrapped, 0), [0]);
                    assert_eq!(read_uninitialized::<1>(&direct, 0), [0]);
                }
                // Initialized destinations remain readable even after failure.
                // Compare success/failure and successful bytes, not panic text
                // or unspecified destination contents after a failed read.
                let mut wrapped_dst = [0xA5];
                let mut direct_dst = [0xA5];
                let wrapped_result = catch_unwind(AssertUnwindSafe(|| {
                    // SAFETY: count is at most this separate buffer's length.
                    unsafe { wrapped.read_unsafe(offset, wrapped_dst.as_mut_ptr(), count) }
                }));
                let direct_result = catch_unwind(AssertUnwindSafe(|| {
                    // SAFETY: count is at most this separate buffer's length.
                    unsafe { direct.read_unsafe(offset, direct_dst.as_mut_ptr(), count) }
                }));
                assert_eq!(wrapped_result.is_ok(), direct_result.is_ok());
                if wrapped_result.is_ok() {
                    assert_eq!(wrapped_dst, direct_dst);
                }
            }
        }
    }
    // Outside every allocated bucket, both cold and warm paths must reject.
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            read_uninitialized::<1>(&memory, 8 * PAGE)
        }))
        .is_err()
    );
}
