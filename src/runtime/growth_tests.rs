use super::{MemoryManagerConfig, MemoryRuntime, RuntimeGrowError, RuntimeMemory};
use ic_stable_structures::{Memory, VectorMemory};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[derive(Clone, Default)]
struct RefusableMemory {
    bytes: VectorMemory,
    refuse: Rc<Cell<bool>>,
    reads: Rc<Cell<usize>>,
    grows: Rc<Cell<usize>>,
}

impl Memory for RefusableMemory {
    fn size(&self) -> u64 {
        self.bytes.size()
    }
    fn grow(&self, pages: u64) -> i64 {
        self.grows.set(self.grows.get() + 1);
        if self.refuse.get() {
            -1
        } else {
            self.bytes.grow(pages)
        }
    }
    fn read(&self, offset: u64, dst: &mut [u8]) {
        self.reads.set(self.reads.get() + 1);
        self.bytes.read(offset, dst);
    }
    fn write(&self, offset: u64, src: &[u8]) {
        self.bytes.write(offset, src);
    }
}

#[test]
fn backing_reentry_cannot_assign_buckets_during_an_outer_reservation() {
    struct ReentrantMemory {
        bytes: VectorMemory,
        nested: RefCell<Option<RuntimeMemory<Rc<Self>>>>,
        result: Cell<Option<Result<u64, RuntimeGrowError>>>,
    }
    impl Memory for ReentrantMemory {
        fn size(&self) -> u64 {
            self.bytes.size()
        }
        fn grow(&self, pages: u64) -> i64 {
            if let Some(memory) = self.nested.borrow().as_ref() {
                self.result.set(Some(memory.grow(1)));
            }
            self.bytes.grow(pages)
        }
        fn read(&self, offset: u64, dst: &mut [u8]) {
            self.bytes.read(offset, dst);
        }
        fn write(&self, offset: u64, src: &[u8]) {
            self.bytes.write(offset, src);
        }
    }
    let backing = Rc::new(ReentrantMemory {
        bytes: VectorMemory::default(),
        nested: RefCell::new(None),
        result: Cell::new(None),
    });
    let runtime =
        MemoryRuntime::new_with_config(Rc::clone(&backing), MemoryManagerConfig::new(1).unwrap())
            .unwrap();
    *backing.nested.borrow_mut() = Some(runtime.memory(121));
    assert_eq!(runtime.memory(120).grow(1), Ok(0));
    assert_eq!(
        backing.result.get(),
        Some(Err(RuntimeGrowError::ReentrantAccess))
    );
    assert_eq!(runtime.memory(121).size(), 0);
    assert_eq!(runtime.memory_allocations().unwrap().allocated_buckets, 1);
    // Release the callback's retained handle before dropping its backing owner.
    backing.nested.borrow_mut().take();
}

#[test]
fn refused_application_growth_preserves_bytes_extents_and_retry() {
    for bucket in [1, 8, 16, 128, u16::MAX] {
        // The maximum bucket is checked with refusal only, without allocating GiBs.
        let backing = RefusableMemory::default();
        let runtime = MemoryRuntime::new_with_config(
            backing.clone(),
            MemoryManagerConfig::new(bucket).unwrap(),
        )
        .unwrap();
        let rows = runtime.memory(120);
        let before = backing.bytes.borrow().clone();
        backing.refuse.set(true);
        assert_eq!(
            rows.grow(1),
            Err(RuntimeGrowError::BackingRefused {
                additional_pages: u64::from(bucket),
            })
        );
        // Generic stable structures must still receive the upstream sentinel.
        assert_eq!(Memory::grow(&rows, 1), -1);
        assert_eq!(rows.size(), 0);
        assert_eq!(*backing.bytes.borrow(), before);
        assert_eq!(runtime.memory_allocations().unwrap().allocated_buckets, 0);
        assert_eq!(backing.reads.get(), 2);
        if bucket == u16::MAX {
            continue;
        }
        backing.refuse.set(false);
        assert_eq!(rows.grow(1), Ok(0));
        rows.write(0, b"retained");
        let before = backing.bytes.borrow().clone();
        let report = runtime.memory_allocations().unwrap();
        backing.refuse.set(true);
        let clone = rows.clone();
        assert_eq!(
            clone.grow(u64::from(bucket)),
            Err(RuntimeGrowError::BackingRefused {
                additional_pages: u64::from(bucket),
            })
        );
        assert_eq!(*backing.bytes.borrow(), before);
        assert_eq!(runtime.memory_allocations().unwrap(), report);
        // Growth within an already assigned bucket requires no physical growth.
        let grows = backing.grows.get();
        assert_eq!(rows.grow(u64::from(bucket) - 1), Ok(1));
        assert_eq!(backing.grows.get(), grows);
        backing.refuse.set(false);
        assert_eq!(rows.grow(1), Ok(u64::from(bucket)));
        let mut bytes = [0; 8];
        rows.read(0, &mut bytes);
        assert_eq!(&bytes, b"retained");
        drop(rows);
        assert_eq!(clone.grow(0), Ok(u64::from(bucket) + 1));
    }
}

#[test]
fn interleaved_clones_reopen_and_detached_handles_share_growth_accounting() {
    let backing = RefusableMemory::default();
    let runtime =
        MemoryRuntime::new_with_config(backing.clone(), MemoryManagerConfig::new(8).unwrap())
            .unwrap();
    let a = runtime.memory(120);
    let b = runtime.memory(121);
    let reads = backing.reads.get();
    assert_eq!(a.grow(9), Ok(0));
    assert_eq!(b.grow(1), Ok(0));
    let clone = a.clone();
    drop(a);
    assert_eq!(clone.grow(8), Ok(9));
    // Capacity admission reads no persisted metadata, even at bucket boundaries.
    assert_eq!(backing.reads.get(), reads);
    assert_eq!(runtime.memory_allocations().unwrap().allocated_buckets, 4);
    assert_eq!(
        clone.grow(u64::MAX),
        Err(RuntimeGrowError::ArithmeticOverflow)
    );
    assert_eq!(clone.size(), 17);
    drop(clone);
    drop(b);
    drop(runtime);
    let runtime = MemoryRuntime::new(backing.clone()).unwrap();
    assert_eq!(runtime.memory(122).grow(1), Ok(0));
    assert_eq!(runtime.memory_allocations().unwrap().allocated_buckets, 5);
    let detached = runtime.memory(120);
    drop(runtime);
    assert_eq!(detached.grow(8), Ok(17));
    drop(detached);
    assert_eq!(
        MemoryRuntime::new(backing)
            .unwrap()
            .memory_allocations()
            .unwrap()
            .allocated_buckets,
        6
    );
}
