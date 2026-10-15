use super::*;
use crate::ic_stable_structures::VectorMemory;

#[test]
fn bounded_cell_preserves_history_and_refuses_overwrites_or_oversized_writes() {
    let backing = VectorMemory::default();
    let config = MemoryManagerConfig::new(16).unwrap();
    let mut cell = FixtureCell::create_bounded(
        backing.clone(),
        "fixture",
        "fixture.",
        "fixture.consumer.v1",
        config,
        16_384,
    );
    cell.set(vec![42; 16_384]);
    let memory = cell._runtime.open_memory("fixture.consumer.v1").unwrap();
    let mut payload_before = vec![0; 16_384 + crate::STABLE_CELL_HEADER_SIZE];
    memory.read(0, &mut payload_before);
    let before = backing.borrow().clone();
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            cell.set(vec![0; 16_385]);
        }))
        .is_err()
    );
    assert_eq!(cell.bytes(), &[42; 16_384]);
    assert_eq!(*backing.borrow(), before);
    drop(cell);
    for create in [true, false] {
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if create {
                    FixtureCell::create_bounded(
                        backing.clone(),
                        "fixture",
                        "fixture.",
                        "fixture.consumer.v1",
                        config,
                        16_384,
                    )
                } else {
                    FixtureCell::open_bounded(
                        backing.clone(),
                        "fixture",
                        "fixture.",
                        "fixture.consumer.v1",
                        config,
                        16_383,
                    )
                }
            }))
            .is_err()
        );
        // Bootstrap may advance the ledger; refusal must not replace the journal.
        let mut payload_after = vec![0; payload_before.len()];
        memory.read(0, &mut payload_after);
        assert_eq!(payload_after, payload_before);
    }
    let reopened = FixtureCell::open_bounded(
        backing,
        "fixture",
        "fixture.",
        "fixture.consumer.v1",
        config,
        16_384,
    );
    assert_eq!(reopened.bytes(), &[42; 16_384]);
}

#[test]
fn bounded_open_refuses_missing_and_non_cell_history_without_replacement() {
    let backing = VectorMemory::default();
    let config = MemoryManagerConfig::new(16).unwrap();
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            FixtureCell::open_bounded(
                backing.clone(),
                "fixture",
                "fixture.",
                "fixture.consumer.v1",
                config,
                16_384,
            )
        }))
        .is_err()
    );
    assert_eq!(backing.size(), 0);
    let cell = FixtureCell::create_bounded(
        backing.clone(),
        "fixture",
        "fixture.",
        "fixture.consumer.v1",
        config,
        16_384,
    );
    let memory = cell._runtime.open_memory("fixture.consumer.v1").unwrap();
    memory.write(0, b"CONS\x01");
    drop(cell);
    let mut before = [0; crate::STABLE_CELL_HEADER_SIZE];
    memory.read(0, &mut before);
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            FixtureCell::open_bounded(
                backing.clone(),
                "fixture",
                "fixture.",
                "fixture.consumer.v1",
                config,
                16_384,
            )
        }))
        .is_err()
    );
    let mut after = [0; crate::STABLE_CELL_HEADER_SIZE];
    memory.read(0, &mut after);
    assert_eq!(after, before);
}

#[test]
fn cell_keeps_existing_runtime_bytes_and_bucket_configuration() {
    let backing = VectorMemory::default();
    let config = MemoryManagerConfig::new(16).unwrap();
    // Construct through the ordinary substrate used by independently owned hosts.
    let request =
        MemoryRequest::new("fixture", "fixture.journal.v1", SchemaMetadata::default()).unwrap();
    let pool = MemoryAllocationPool::new(
        vec![MemoryAuthority::new("fixture", "fixture.").unwrap()],
        vec![],
    )
    .unwrap();
    let declarations = SealedDeclarationSnapshot::new(vec![request]).unwrap();
    let mut runtime = MemoryRuntime::new_with_config(backing.clone(), config).unwrap();
    runtime
        .bootstrap(&declarations, &pool, &GenericAllocationPolicy)
        .unwrap();
    let mut original = Cell::init(
        runtime.open_memory("fixture.journal.v1").unwrap(),
        Vec::new(),
    );
    original.set(vec![0, 255, 42]);
    drop(original);
    drop(runtime);

    let mut cell = FixtureCell::new(
        backing.clone(),
        "fixture",
        "fixture.",
        "fixture.journal.v1",
        config,
    );
    assert_eq!(cell.bytes(), &[0, 255, 42]);
    assert_eq!(
        cell._runtime
            .memory_allocations()
            .unwrap()
            .summary
            .bucket_size_pages,
        16
    );
    cell.set(vec![91]);
    drop(cell);
    let reopened = FixtureCell::new(backing, "fixture", "fixture.", "fixture.journal.v1", config);
    assert_eq!(reopened.bytes(), &[91]);
}

#[test]
fn fresh_empty_cell_is_distinct_from_incompatible_retained_configuration() {
    let backing = VectorMemory::default();
    let mut cell = FixtureCell::new(
        backing.clone(),
        "fixture",
        "fixture.",
        "fixture.journal.v1",
        MemoryManagerConfig::new(16).unwrap(),
    );
    assert_eq!(cell.bytes(), &[0_u8; 0]);
    cell.set(vec![37]);
    drop(cell);
    let before = backing.borrow().clone();
    let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        FixtureCell::new(
            backing.clone(),
            "fixture",
            "fixture.",
            "fixture.journal.v1",
            MemoryManagerConfig::new(128).unwrap(),
        )
    }));
    assert!(failure.is_err());
    assert_eq!(*backing.borrow(), before);
}
