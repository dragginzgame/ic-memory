use super::*;
use crate::VectorMemory;

#[test]
fn cell_keeps_existing_runtime_bytes_and_bucket_configuration() {
    let backing = VectorMemory::default();
    let config = MemoryManagerConfig::new(16).unwrap();
    // Construct through the ordinary substrate used by independently owned hosts.
    let request = MemoryRequest::new("fixture", "fixture.journal.v1").unwrap();
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
    // Force another bucket after the constructor's runtime has been dropped.
    let bytes = vec![91; 17 * 65_536];
    cell.set(bytes.clone());
    assert_eq!(cell.bytes(), bytes);
    drop(cell);
    let runtime = MemoryRuntime::new_with_config(backing.clone(), config).unwrap();
    assert_eq!(runtime.memory_manager_config(), config);
    drop(runtime);
    let reopened = FixtureCell::new(backing, "fixture", "fixture.", "fixture.journal.v1", config);
    assert_eq!(reopened.bytes(), bytes);
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

#[test]
fn fallible_setup_keeps_payload_and_transfers_the_admitted_memory() {
    let backing = VectorMemory::default();
    let config = MemoryManagerConfig::new(16).unwrap();
    let mut cell = FixtureCell::try_new(
        backing.clone(),
        "fixture",
        "fixture.",
        "fixture.jobs.v1",
        config,
    )
    .unwrap();
    cell.set(b"committed journal".to_vec());
    let memory = cell.into_memory();
    assert_eq!(Cell::init(memory, Vec::new()).get(), b"committed journal");
    let reopened = FixtureCell::try_new(
        backing.clone(),
        "fixture",
        "fixture.",
        "fixture.jobs.v1",
        config,
    )
    .unwrap();
    assert_eq!(reopened.bytes(), b"committed journal");
    drop(reopened);
    let before = backing.borrow().clone();
    let refused = FixtureCell::try_new(
        backing.clone(),
        "fixture",
        "fixture.",
        "fixture.jobs.v1",
        MemoryManagerConfig::new(32).unwrap(),
    );
    assert!(matches!(refused, Err(FixtureCellError::Construction(_))));
    assert_eq!(*backing.borrow(), before);
}

#[test]
fn fallible_setup_preserves_typed_request_grant_and_bootstrap_errors() {
    let config = MemoryManagerConfig::new(16).unwrap();
    let backing = VectorMemory::default();
    let refused = FixtureCell::try_new(backing.clone(), "", "fixture.", "fixture.jobs.v1", config);
    assert!(matches!(refused, Err(FixtureCellError::Declaration(_))));
    assert_eq!(backing.size(), 0);
    let refused = FixtureCell::try_new(
        backing.clone(),
        "fixture",
        "invalid",
        "fixture.jobs.v1",
        config,
    );
    assert!(matches!(refused, Err(FixtureCellError::AllocationPool(_))));
    assert_eq!(backing.size(), 0);
    let refused = FixtureCell::try_new(backing, "fixture", "other.", "fixture.jobs.v1", config);
    assert!(matches!(refused, Err(FixtureCellError::Bootstrap(_))));
}

#[test]
fn fallible_setup_refuses_non_cell_history_without_replacing_its_header() {
    let backing = VectorMemory::default();
    let config = MemoryManagerConfig::new(16).unwrap();
    let cell = FixtureCell::try_new(
        backing.clone(),
        "fixture",
        "fixture.",
        "fixture.jobs.v1",
        config,
    )
    .unwrap();
    let memory = cell.into_memory();
    memory.write(0, b"NOT\x01");
    let mut before = [0; crate::STABLE_CELL_HEADER_SIZE];
    memory.read(0, &mut before);
    let refused = FixtureCell::try_new(backing, "fixture", "fixture.", "fixture.jobs.v1", config);
    assert!(matches!(
        refused,
        Err(FixtureCellError::Payload(
            crate::StableCellPayloadError::NotStableCell
        ))
    ));
    let mut after = [0; crate::STABLE_CELL_HEADER_SIZE];
    memory.read(0, &mut after);
    assert_eq!(after, before);
}

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
    )
    .unwrap();
    cell.try_set(vec![42; 16_384]).unwrap();
    let before = backing.borrow().clone();
    assert!(matches!(
        cell.try_set(vec![0; 16_385]),
        Err(FixtureCellError::ValueTooLarge {
            size: 16_385,
            maximum: 16_384,
        })
    ));
    assert_eq!(cell.bytes(), &[42; 16_384]);
    assert_eq!(*backing.borrow(), before);
    let memory = cell.into_memory();
    let mut payload_before = vec![0; 16_384 + crate::STABLE_CELL_HEADER_SIZE];
    memory.read(0, &mut payload_before);
    assert!(matches!(
        FixtureCell::create_bounded(
            backing.clone(),
            "fixture",
            "fixture.",
            "fixture.consumer.v1",
            config,
            16_384,
        ),
        Err(FixtureCellError::Occupied)
    ));
    assert!(matches!(
        FixtureCell::open_bounded(
            backing.clone(),
            "fixture",
            "fixture.",
            "fixture.consumer.v1",
            config,
            16_383,
        ),
        Err(FixtureCellError::Payload(
            crate::StableCellPayloadError::TooLarge { value_len: 16_384 }
        ))
    ));
    let mut payload_after = vec![0; payload_before.len()];
    memory.read(0, &mut payload_after);
    assert_eq!(payload_after, payload_before);
    let mut reopened = FixtureCell::open_bounded(
        backing.clone(),
        "fixture",
        "fixture.",
        "fixture.consumer.v1",
        config,
        16_384,
    )
    .unwrap();
    assert_eq!(reopened.bytes(), &[42; 16_384]);
    // A short replacement exposes only the new logical value.
    reopened.try_set(vec![7]).unwrap();
    drop(reopened);
    let reopened = FixtureCell::open_bounded(
        backing,
        "fixture",
        "fixture.",
        "fixture.consumer.v1",
        config,
        1,
    )
    .unwrap();
    assert_eq!(reopened.bytes(), &[7]);
}

#[test]
fn bounded_open_refuses_missing_and_malformed_history_without_replacement() {
    let backing = VectorMemory::default();
    let config = MemoryManagerConfig::new(16).unwrap();
    assert!(matches!(
        FixtureCell::open_bounded(
            backing.clone(),
            "fixture",
            "fixture.",
            "fixture.consumer.v1",
            config,
            16_384,
        ),
        Err(FixtureCellError::Payload(
            crate::StableCellPayloadError::NotStableCell
        ))
    ));
    assert_eq!(backing.size(), 0);
    let cell = FixtureCell::create_bounded(
        backing.clone(),
        "fixture",
        "fixture.",
        "fixture.consumer.v1",
        config,
        16_384,
    )
    .unwrap();
    let memory = cell.into_memory();
    // Assert each malformed header survives exactly, including impossible lengths.
    for header in [
        *b"BAD\x01\x00\x00\x00\x00",
        *b"SCL\x02\x00\x00\x00\x00",
        *b"SCL\x01\xff\xff\xff\xff",
    ] {
        memory.write(0, &header);
        assert!(matches!(
            FixtureCell::open_bounded(
                backing.clone(),
                "fixture",
                "fixture.",
                "fixture.consumer.v1",
                config,
                16_384,
            ),
            Err(FixtureCellError::Payload(_))
        ));
        let mut after = [0; crate::STABLE_CELL_HEADER_SIZE];
        memory.read(0, &mut after);
        assert_eq!(after, header);
    }
}

#[test]
fn bounded_open_does_not_initialize_a_missing_cell_in_an_existing_manager() {
    let backing = VectorMemory::default();
    let config = MemoryManagerConfig::new(16).unwrap();
    let mut runtime = MemoryRuntime::new_with_config(backing.clone(), config).unwrap();
    let declarations = SealedDeclarationSnapshot::new(vec![
        MemoryRequest::new("fixture", "fixture.consumer.v1").unwrap(),
    ])
    .unwrap();
    let pool = MemoryAllocationPool::new(
        vec![MemoryAuthority::new("fixture", "fixture.").unwrap()],
        vec![],
    )
    .unwrap();
    runtime
        .bootstrap(&declarations, &pool, &GenericAllocationPolicy)
        .unwrap();
    assert_eq!(
        runtime.open_memory("fixture.consumer.v1").unwrap().size(),
        0
    );
    drop(runtime);
    assert!(matches!(
        FixtureCell::open_bounded(
            backing.clone(),
            "fixture",
            "fixture.",
            "fixture.consumer.v1",
            config,
            0,
        ),
        Err(FixtureCellError::Payload(
            crate::StableCellPayloadError::NotStableCell
        ))
    ));
    // Fresh admission is still available: failed retained admission created no Cell.
    let mut fresh = FixtureCell::create_bounded(
        backing,
        "fixture",
        "fixture.",
        "fixture.consumer.v1",
        config,
        0,
    )
    .unwrap();
    assert_eq!(fresh.bytes(), [0_u8; 0]);
    fresh.try_set(Vec::new()).unwrap();
    assert!(matches!(
        fresh.try_set(vec![1]),
        Err(FixtureCellError::ValueTooLarge {
            size: 1,
            maximum: 0
        })
    ));
}
