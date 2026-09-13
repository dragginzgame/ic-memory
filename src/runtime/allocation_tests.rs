use super::{
    AllocationBinding, MemoryManagerLayoutError, MemoryRuntime, RuntimeConstructionError,
    RuntimeDiagnosticError, RuntimeOpenError, layout, policy::GenericRangePolicy,
};
use crate::{IC_MEMORY_LEDGER_STABLE_KEY, MEMORY_MANAGER_LEDGER_ID, registry::TEST_REGISTRY_LOCK};
use ic_stable_structures::{
    Memory, VectorMemory,
    memory_manager::{MemoryId, MemoryManager},
};
#[path = "../../examples/support/metered.rs"]
mod metered;
use metered::Metered;

fn conservation(report: &super::MemoryAllocations) {
    assert_eq!(report.memories.len(), 255);
    assert_eq!(
        report.physical_extent.bytes,
        report.manager_metadata_bytes + report.allocated_bucket_bytes + report.unmanaged_bytes
    );
    assert_eq!(
        report.allocated_bucket_bytes,
        report
            .memories
            .iter()
            .map(|row| row.allocated_bytes)
            .sum::<u64>()
    );
    assert_eq!(
        report.allocated_bucket_bytes,
        report.known_binding_bytes + report.unknown_binding_bytes
    );
    assert_eq!(
        report.allocated_bucket_bytes,
        report.virtual_extent.bytes + report.bucket_slack_bytes
    );
    assert_eq!(
        report.manager_metadata_bytes,
        report.manager_header_bytes
            + report.manager_bucket_table_bytes
            + report.manager_padding_bytes
    );
    for (id, row) in (0..255).zip(&report.memories) {
        assert_eq!(row.memory_manager_id, id);
        assert_eq!(
            row.allocated_bytes,
            row.virtual_extent.bytes + row.bucket_slack_bytes
        );
        assert_eq!(row.payload_bytes, None);
    }
}

#[test]
fn bounded_conservation_bindings_and_no_effects() {
    let _guard = TEST_REGISTRY_LOCK.lock().unwrap();
    let declarations = super::tests::declarations();
    let memory = Metered::default();
    let mut runtime = MemoryRuntime::new(memory.clone()).unwrap();
    runtime
        .bootstrap(&declarations, &GenericRangePolicy)
        .unwrap();
    let rows = runtime.open_memory("runtime_tests.rows.v1", 120).unwrap();
    let zero = runtime.memory_allocations().unwrap();
    assert_eq!(zero.memories[120].allocated_buckets, 0);
    assert!(matches!(
        zero.memories[120].binding,
        AllocationBinding::Current { .. }
    ));
    assert_eq!(rows.grow(129), 0);
    // Test-only unmanaged backing tail and an ID with no current declaration.
    assert_eq!(runtime.memory(121).grow(1), 0);
    memory.grow(3);
    let before = memory.bytes.borrow().clone();
    let generation = runtime.committed_allocations().unwrap().generation();
    memory.reset();
    let report = runtime.memory_allocations().unwrap();
    conservation(&report);
    assert_eq!(report.memories[120].allocated_buckets, 2);
    assert_eq!(report.memories[121].binding, AllocationBinding::Unknown);
    assert_eq!(report.unknown_binding_bytes, 128 * 65_536);
    assert_eq!(report.unmanaged_bytes, 3 * 65_536);
    assert!(matches!(
        report.memories[usize::from(MEMORY_MANAGER_LEDGER_ID)].binding,
        AllocationBinding::Ledger { .. }
    ));
    assert_eq!(memory.counts().read_bytes, 34_848);
    assert_eq!(memory.counts().reads, 2);
    assert_eq!(memory.counts().writes, 0);
    assert_eq!(memory.counts().grows, 0);
    assert_eq!(memory.bytes.borrow().as_slice(), before);
    assert_eq!(report.current_generation, Some(generation));
    assert!(matches!(
        runtime.open_memory(IC_MEMORY_LEDGER_STABLE_KEY, 0),
        Err(RuntimeOpenError::ReservedStableKey { .. })
    ));
    assert!(matches!(
        runtime.open_memory("unknown.rows.v1", 121),
        Err(RuntimeOpenError::StableKeyNotCommitted(_))
    ));
    assert!(matches!(
        runtime.open_memory("runtime_tests.rows.v1", 121),
        Err(RuntimeOpenError::MemoryIdMismatch { .. })
    ));
    assert_eq!(runtime.memory_allocations().unwrap(), report);
}

#[test]
fn diagnostics_never_read_even_a_corrupt_unbounded_ledger() {
    let _guard = TEST_REGISTRY_LOCK.lock().unwrap();
    let declarations = super::tests::declarations();
    let memory = Metered::default();
    let mut runtime = MemoryRuntime::new(memory.clone()).unwrap();
    runtime
        .bootstrap(&declarations, &GenericRangePolicy)
        .unwrap();
    // Advertise a huge stable-cell value; bounded attribution must not decode it.
    runtime
        .memory(MEMORY_MANAGER_LEDGER_ID)
        .write(4, &u32::MAX.to_le_bytes());
    memory.reset();
    conservation(&runtime.memory_allocations().unwrap());
    assert_eq!(memory.counts().read_bytes, 34_848);
    assert_eq!(memory.counts().writes, 0);
    assert_eq!(memory.counts().grows, 0);
}

#[test]
fn bucket_boundaries_and_same_release_reopen() {
    for bucket in [1_u16, 8, 16, 128] {
        let memory = Metered::default();
        let manager = MemoryManager::init_with_bucket_size(memory.clone(), bucket);
        let rows = manager.get(MemoryId::new(120));
        assert_eq!(rows.grow(1), 0);
        assert_eq!(rows.grow(u64::from(bucket) - 1), 1);
        assert_eq!(rows.grow(1), i64::from(bucket));
        rows.write(u64::from(bucket) * 65_536 - 1, &[1, 2, 3]);
        drop(rows);
        drop(manager);
        memory.reset();
        let runtime = MemoryRuntime::new(memory.clone()).unwrap();
        let report = runtime.memory_allocations().unwrap();
        conservation(&report);
        assert_eq!(report.bucket_size_pages, bucket);
        assert_eq!(report.memories[120].allocated_buckets, 2);
        assert_eq!(report.current_generation, None);
        assert_eq!(report.memories[120].binding, AllocationBinding::Unknown);
        let mut bytes = [0; 3];
        runtime
            .memory(120)
            .read(u64::from(bucket) * 65_536 - 1, &mut bytes);
        assert_eq!(bytes, [1, 2, 3]);
        assert_eq!(memory.counts().writes, 0);
        assert_eq!(memory.counts().grows, 0);
    }
}

#[test]
fn corrupt_layouts_are_typed_and_never_reinitialized() {
    let fixtures: &[(u64, &[u8], MemoryManagerLayoutError)] = &[
        (6, &[0, 0], MemoryManagerLayoutError::ZeroBucketSize),
        (8, &[1], MemoryManagerLayoutError::ReservedHeader),
        (
            4,
            &[1, 128],
            MemoryManagerLayoutError::BucketCount { count: 32769 },
        ),
        (
            layout::HEADER_BYTES as u64,
            &[120],
            MemoryManagerLayoutError::BucketTable { index: 0 },
        ),
        (
            40 + 120 * 8,
            &[1],
            MemoryManagerLayoutError::VirtualExtent { id: 120 },
        ),
        (
            4,
            &[1, 0],
            MemoryManagerLayoutError::TruncatedBacking {
                physical_pages: 1,
                required_pages: 129,
            },
        ),
    ];
    for &(offset, bytes, expected) in fixtures {
        let memory = Metered::default();
        drop(MemoryRuntime::new(memory.clone()).unwrap());
        memory.write(offset, bytes);
        let before = memory.bytes.borrow().clone();
        memory.reset();
        assert!(
            matches!(MemoryRuntime::new(memory.clone()), Err(RuntimeConstructionError::Layout(error)) if error == expected)
        );
        assert_eq!(memory.counts().writes, 0);
        assert_eq!(memory.counts().grows, 0);
        assert_eq!(memory.bytes.borrow().as_slice(), before);
    }
}

#[test]
fn diagnostic_revalidates_foreign_unsupported_and_stale_metadata() {
    for (offset, bytes) in [(0, &b"BAD"[..]), (3, &[2][..]), (6, &[16, 0][..])] {
        let memory = VectorMemory::default();
        let runtime = MemoryRuntime::new(memory.clone()).unwrap();
        memory.write(offset, bytes);
        assert!(matches!(
            runtime.memory_allocations(),
            Err(RuntimeDiagnosticError::Construction(_))
        ));
    }
}

#[test]
fn default_attribution_does_not_initialize_tls() {
    std::thread::spawn(|| {
        assert!(matches!(
            super::default_memory_manager_memory_allocations(),
            Err(RuntimeDiagnosticError::NotBootstrapped)
        ));
        assert!(matches!(
            super::default_memory_manager_memory_allocations(),
            Err(RuntimeDiagnosticError::NotBootstrapped)
        ));
    })
    .join()
    .unwrap();
}

#[test]
fn explicit_configuration_validates_before_effects_and_replays() {
    let _guard = TEST_REGISTRY_LOCK.lock().unwrap();
    let declarations = super::tests::declarations();
    let memory = Metered::default();
    assert_eq!(
        super::MemoryManagerConfig::new(0),
        Err(RuntimeConstructionError::InvalidBucketSize)
    );
    assert_eq!(memory.size(), 0);
    let config = super::MemoryManagerConfig::new(8).unwrap();
    let mut runtime = MemoryRuntime::new_with_config(memory.clone(), config).unwrap();
    assert_eq!(runtime.memory_manager_config(), config);
    runtime
        .bootstrap(&declarations, &GenericRangePolicy)
        .unwrap();
    let handle = runtime.open_memory("runtime_tests.rows.v1", 120).unwrap();
    handle.grow(9);
    handle.write(8 * 65_536 - 1, &[4, 5, 6]);
    drop(handle);
    let generation = runtime.committed_allocations().unwrap().generation();
    memory.reset();
    runtime
        .bootstrap(&declarations, &GenericRangePolicy)
        .unwrap();
    assert_eq!(
        runtime.committed_allocations().unwrap().generation(),
        generation
    );
    assert_eq!(memory.counts().writes, 0);
    assert_eq!(memory.counts().grows, 0);
    let before = runtime.memory_allocations().unwrap();
    drop(runtime);
    assert!(matches!(
        MemoryRuntime::new_with_config(memory.clone(), super::MemoryManagerConfig::default()),
        Err(RuntimeConstructionError::BucketSizeMismatch {
            persisted: 8,
            requested: 128
        })
    ));
    assert_eq!(memory.counts().writes, 0);
    assert_eq!(memory.counts().grows, 0);
    let mut runtime = MemoryRuntime::new_with_config(memory.clone(), config).unwrap();
    let unbound = runtime.memory_allocations().unwrap();
    assert_eq!(unbound.physical_extent, before.physical_extent);
    assert_eq!(unbound.memories[120].binding, AllocationBinding::Unknown);
    runtime
        .bootstrap(&declarations, &GenericRangePolicy)
        .unwrap();
    let handle = runtime.open_memory("runtime_tests.rows.v1", 120).unwrap();
    let mut bytes = [0; 3];
    handle.read(8 * 65_536 - 1, &mut bytes);
    assert_eq!(bytes, [4, 5, 6]);
    let recovered = runtime.memory_allocations().unwrap();
    assert_eq!(recovered.physical_extent, before.physical_extent);
    assert!(recovered.current_generation > before.current_generation);
    let stable_before = memory.bytes.borrow().clone();
    memory.reset();
    for _ in 0..3 {
        assert_eq!(runtime.memory_allocations().unwrap(), recovered);
    }
    assert_eq!(memory.bytes.borrow().as_slice(), stable_before);
    assert_eq!(memory.counts().writes, 0);
    assert_eq!(memory.counts().grows, 0);
}

#[test]
fn default_configuration_is_bound_before_repeated_bootstrap() {
    let _guard = TEST_REGISTRY_LOCK.lock().unwrap();
    super::tests::declarations();
    std::thread::spawn(|| {
        let config = super::MemoryManagerConfig::new(16).unwrap();
        let committed =
            super::bootstrap_default_memory_manager_with_config(config, &GenericRangePolicy)
                .unwrap();
        let before = super::default_memory_manager_memory_allocations().unwrap();
        assert_eq!(before.bucket_size_pages, 16);
        assert!(
            super::bootstrap_default_memory_manager_with_config(
                super::MemoryManagerConfig::default(),
                &GenericRangePolicy
            )
            .is_err()
        );
        let replay =
            super::bootstrap_default_memory_manager_with_config(config, &GenericRangePolicy)
                .unwrap();
        assert_eq!(replay.generation(), committed.generation());
        assert_eq!(
            super::default_memory_manager_memory_allocations().unwrap(),
            before
        );
        assert_eq!(
            super::bootstrap_default_memory_manager()
                .unwrap()
                .generation(),
            committed.generation()
        );
    })
    .join()
    .unwrap();
}

#[test]
fn finite_bucket_table_capacity_and_overflow_are_checked() {
    // Model address capacity without allocating GiBs of host RAM. No payload
    // access is performed in this metadata-only exhaustion fixture.
    struct Sparse {
        pages: std::cell::Cell<u64>,
        header: std::cell::RefCell<Vec<u8>>,
    }
    impl Memory for Sparse {
        fn size(&self) -> u64 {
            self.pages.get()
        }
        fn grow(&self, pages: u64) -> i64 {
            let old = self.pages.get();
            self.pages.set(old + pages);
            i64::try_from(old).unwrap()
        }
        fn read(&self, offset: u64, dst: &mut [u8]) {
            let offset = usize::try_from(offset).unwrap();
            dst.copy_from_slice(&self.header.borrow()[offset..offset + dst.len()]);
        }
        fn write(&self, offset: u64, src: &[u8]) {
            let offset = usize::try_from(offset).unwrap();
            self.header.borrow_mut()[offset..offset + src.len()].copy_from_slice(src);
        }
    }
    struct Overflow;
    impl Memory for Overflow {
        fn size(&self) -> u64 {
            u64::MAX
        }
        fn grow(&self, _: u64) -> i64 {
            panic!("no grow")
        }
        fn read(&self, _: u64, _: &mut [u8]) {
            panic!("bound before read")
        }
        fn write(&self, _: u64, _: &[u8]) {
            panic!("no write")
        }
    }
    for bucket in [1_u16, 8, 16, 128, u16::MAX] {
        let runtime = MemoryRuntime::new_with_config(
            Sparse {
                pages: std::cell::Cell::new(0),
                header: std::cell::RefCell::new(vec![0; 65_536]),
            },
            super::MemoryManagerConfig::new(bucket).unwrap(),
        )
        .unwrap();
        let pages = 32_768 * u64::from(bucket);
        assert_eq!(runtime.memory(120).grow(pages), 0);
        let full = runtime.memory_allocations().unwrap();
        conservation(&full);
        assert_eq!(full.remaining_buckets, 0);
        assert_eq!(full.maximum_bucket_bytes, pages * 65_536);
        assert_eq!(runtime.memory(121).grow(1), -1);
        assert_eq!(runtime.memory_allocations().unwrap(), full);
    }
    assert!(matches!(
        MemoryRuntime::new(Overflow),
        Err(RuntimeConstructionError::Layout(
            MemoryManagerLayoutError::ExtentOverflow
        ))
    ));
}
