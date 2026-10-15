use super::*;
use crate::ic_stable_structures::VectorMemory;

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
