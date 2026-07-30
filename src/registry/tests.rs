use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

static EAGER_INIT_RUNS: AtomicUsize = AtomicUsize::new(0);
static BLOCKING_HOOK_STARTED: AtomicUsize = AtomicUsize::new(0);
static RELEASE_BLOCKING_HOOK: AtomicUsize = AtomicUsize::new(0);

fn register_from_eager_init() {
    EAGER_INIT_RUNS.fetch_add(1, Ordering::SeqCst);
    register_static_memory_manager_declaration(101, "eager", "audit", "eager.audit.v1")
        .expect("eager declaration");
}

fn block_during_sealing() {
    BLOCKING_HOOK_STARTED.store(1, Ordering::SeqCst);
    while RELEASE_BLOCKING_HOOK.load(Ordering::SeqCst) == 0 {
        std::thread::yield_now();
    }
}

fn record_reentrant_seal_error() {
    let error = sealed_declaration_snapshot().expect_err("recursive seal must fail");
    if error == StaticMemoryDeclarationError::ReentrantSealing {
        EAGER_INIT_RUNS.fetch_add(1, Ordering::SeqCst);
    }
}

#[allow(clippy::unnecessary_wraps)]
fn no_registration() -> Result<(), StaticMemoryDeclarationError> {
    Ok(())
}

#[test]
fn registers_and_seals_static_memory_declarations() {
    let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
    reset_static_memory_declarations_for_tests();

    register_static_memory_manager_declaration(100, "icydb", "users", "icydb.users.data.v1")
        .expect("register declaration");

    let snapshot = sealed_declaration_snapshot().expect("snapshot");
    let registrations = snapshot.registered_declarations();
    assert_eq!(registrations.len(), 1);
    assert_eq!(registrations[0].authority(), "icydb");
    assert_eq!(
        registrations[0].declaration().stable_key().as_str(),
        "icydb.users.data.v1"
    );

    assert_eq!(snapshot.allocation_snapshot().len(), 2);

    let err = register_static_memory_manager_declaration(101, "icydb", "orders", "icydb.orders.v1")
        .expect_err("late registration must fail");
    assert_eq!(err, StaticMemoryDeclarationError::RegistrySealed);
}

#[test]
fn registers_static_memory_ranges() {
    let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
    reset_static_memory_declarations_for_tests();

    register_static_memory_manager_range(
        100,
        109,
        "crate_a",
        MemoryManagerRangeMode::Reserved,
        Some("crate A stores".to_string()),
    )
    .expect("register range");

    let snapshot = sealed_declaration_snapshot().expect("snapshot");
    let ranges = snapshot.registered_ranges();
    assert_eq!(ranges.len(), 1);
    assert_eq!(ranges[0].authority(), "crate_a");
    assert_eq!(ranges[0].record().range().start(), 100);
    assert_eq!(ranges[0].record().range().end(), 109);
}

#[test]
fn static_range_declaration_uses_record_authority() {
    let record = MemoryManagerAuthorityRecord::new(
        MemoryManagerIdRange::new(100, 109).expect("range"),
        "record_authority",
        MemoryManagerRangeMode::Reserved,
        None,
    )
    .expect("record");

    let range = StaticMemoryRangeDeclaration::new(record).expect("external range");

    assert_eq!(range.authority(), "record_authority");
}

#[test]
fn static_declaration_rejects_invalid_decoded_declaration() {
    let mut declaration =
        AllocationDeclaration::memory_manager("app.users.v1", 100, "users").expect("declaration");
    declaration.slot =
        crate::AllocationSlotDescriptor::memory_manager_unchecked(crate::MEMORY_MANAGER_INVALID_ID);

    let err = StaticMemoryDeclaration::new("app", declaration)
        .expect_err("decoded invalid declaration must fail at the registry boundary");

    assert!(matches!(
        err,
        StaticMemoryDeclarationError::Declaration(
            crate::DeclarationSnapshotError::MemoryManagerSlot(
                crate::MemoryManagerSlotError::InvalidMemoryManagerId { id }
            )
        ) if id == crate::MEMORY_MANAGER_INVALID_ID
    ));
}

#[test]
fn static_range_declaration_rejects_invalid_decoded_record() {
    let mut record = MemoryManagerAuthorityRecord::new(
        MemoryManagerIdRange::new(100, 109).expect("range"),
        "app",
        MemoryManagerRangeMode::Reserved,
        None,
    )
    .expect("record");
    record.range = MemoryManagerIdRange {
        start: 109,
        end: 100,
    };

    let err = StaticMemoryRangeDeclaration::new(record)
        .expect_err("decoded invalid range record must fail at the registry boundary");

    assert!(matches!(
        err,
        StaticMemoryDeclarationError::Range(MemoryManagerRangeAuthorityError::Range(_))
    ));
}

#[test]
fn snapshot_rejects_duplicate_static_memory_declarations() {
    let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
    reset_static_memory_declarations_for_tests();

    register_static_memory_manager_declaration(100, "icydb", "users", "icydb.users.data.v1")
        .expect("register first declaration");
    register_static_memory_manager_declaration(100, "icydb", "orders", "icydb.orders.v1")
        .expect("register duplicate slot declaration");

    let err = sealed_declaration_snapshot().expect_err("duplicate slot must fail");
    defer_eager_init(|| {});
    let repeated = sealed_declaration_snapshot().expect_err("seal failure is stable");
    assert!(matches!(
        err,
        StaticMemoryDeclarationError::Declaration(crate::DeclarationSnapshotError::DuplicateSlot(
            _
        ))
    ));
    assert_eq!(repeated, err);
}

#[test]
fn external_registration_rejects_internal_stable_key_namespace() {
    let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
    reset_static_memory_declarations_for_tests();

    let err = register_static_memory_manager_declaration(
        1,
        "external",
        "governance",
        "ic_memory.spoof.v1",
    )
    .expect_err("internal stable key must be unavailable externally");

    assert!(matches!(
        err,
        StaticMemoryDeclarationError::ReservedStableKey { stable_key }
            if stable_key == "ic_memory.spoof.v1"
    ));
}

#[test]
fn external_registration_rejects_internal_authority_identity() {
    let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
    reset_static_memory_declarations_for_tests();

    let declaration_err = register_static_memory_manager_declaration(
        100,
        IC_MEMORY_AUTHORITY_OWNER,
        "users",
        "app.users.v1",
    )
    .expect_err("internal declaration authority must be unavailable externally");
    let range_err = register_static_memory_manager_range(
        100,
        109,
        IC_MEMORY_AUTHORITY_OWNER,
        MemoryManagerRangeMode::Reserved,
        None,
    )
    .expect_err("internal range authority must be unavailable externally");

    assert!(matches!(
        declaration_err,
        StaticMemoryDeclarationError::ReservedAuthority { .. }
    ));
    assert!(matches!(
        range_err,
        StaticMemoryDeclarationError::ReservedAuthority { .. }
    ));
}

#[test]
fn eager_hooks_run_once_before_the_canonical_snapshot_is_published() {
    let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
    reset_static_memory_declarations_for_tests();
    EAGER_INIT_RUNS.store(0, Ordering::SeqCst);
    defer_eager_init(register_from_eager_init);

    let first = sealed_declaration_snapshot().expect("first snapshot");
    let second = sealed_declaration_snapshot().expect("second snapshot");

    assert_eq!(EAGER_INIT_RUNS.load(Ordering::SeqCst), 1);
    assert!(first.shares_storage_with(&second));
    assert_eq!(
        first.registered_declarations()[0]
            .declaration()
            .stable_key()
            .as_str(),
        "eager.audit.v1"
    );
}

#[test]
fn deferred_constructor_registration_errors_are_reported_by_snapshot_requests() {
    let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
    reset_static_memory_declarations_for_tests();
    sealed_declaration_snapshot().expect("initial snapshot");

    defer_eager_init(|| {});

    let error = sealed_declaration_snapshot().expect_err("late deferred registration");
    let repeated = sealed_declaration_snapshot().expect_err("preserved registration failure");
    assert_eq!(error, StaticMemoryDeclarationError::RegistrySealed);
    assert_eq!(repeated, error);
}

#[test]
fn snapshot_order_is_independent_of_registration_order() {
    let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
    reset_static_memory_declarations_for_tests();
    register_static_memory_manager_range(
        102,
        102,
        "order",
        MemoryManagerRangeMode::Reserved,
        Some("z".to_string()),
    )
    .expect("z range");
    register_static_memory_manager_range(
        101,
        101,
        "order",
        MemoryManagerRangeMode::Reserved,
        Some("a".to_string()),
    )
    .expect("a range");
    register_static_memory_manager_declaration(102, "order", "z", "order.z.v1")
        .expect("z declaration");
    register_static_memory_manager_declaration(101, "order", "a", "order.a.v1")
        .expect("a declaration");
    let first = sealed_declaration_snapshot().expect("first snapshot");

    reset_static_memory_declarations_for_tests();
    register_static_memory_manager_range(
        101,
        101,
        "order",
        MemoryManagerRangeMode::Reserved,
        Some("a".to_string()),
    )
    .expect("a range");
    register_static_memory_manager_range(
        102,
        102,
        "order",
        MemoryManagerRangeMode::Reserved,
        Some("z".to_string()),
    )
    .expect("z range");
    register_static_memory_manager_declaration(101, "order", "a", "order.a.v1")
        .expect("a declaration");
    register_static_memory_manager_declaration(102, "order", "z", "order.z.v1")
        .expect("z declaration");
    let second = sealed_declaration_snapshot().expect("second snapshot");

    assert_eq!(first, second);
    assert_eq!(
        crate::test_cbor::to_vec(first.allocation_snapshot()).expect("first bytes"),
        crate::test_cbor::to_vec(second.allocation_snapshot()).expect("second bytes")
    );
    assert_eq!(first.fingerprint(), second.fingerprint());
    assert_eq!(first.fingerprint().algorithm_version(), 1);
    assert_eq!(first.fingerprint().value(), 2_010_740_972_202_682_334);
}

#[test]
fn snapshot_fingerprint_covers_linked_declaration_authority() {
    let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
    reset_static_memory_declarations_for_tests();
    register_static_memory_manager_declaration(101, "authority_a", "rows", "fingerprint.rows.v1")
        .expect("first declaration");
    let first = sealed_declaration_snapshot()
        .expect("first snapshot")
        .fingerprint();

    reset_static_memory_declarations_for_tests();
    register_static_memory_manager_declaration(101, "authority_b", "rows", "fingerprint.rows.v1")
        .expect("second declaration");
    let second = sealed_declaration_snapshot()
        .expect("second snapshot")
        .fingerprint();

    assert_ne!(first, second);
}

#[test]
fn concurrent_snapshot_requests_share_one_complete_seal() {
    let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
    reset_static_memory_declarations_for_tests();
    defer_eager_init(register_from_eager_init);

    let (first, second) = std::thread::scope(|scope| {
        let first = scope.spawn(sealed_declaration_snapshot);
        let second = scope.spawn(sealed_declaration_snapshot);
        (
            first.join().expect("first thread").expect("first seal"),
            second.join().expect("second thread").expect("second seal"),
        )
    });

    assert!(first.shares_storage_with(&second));
    assert_eq!(first.registered_declarations().len(), 1);
}

#[test]
fn registration_from_another_thread_fails_after_sealing_begins() {
    let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
    reset_static_memory_declarations_for_tests();
    BLOCKING_HOOK_STARTED.store(0, Ordering::SeqCst);
    RELEASE_BLOCKING_HOOK.store(0, Ordering::SeqCst);
    defer_eager_init(block_during_sealing);

    std::thread::scope(|scope| {
        let sealing = scope.spawn(sealed_declaration_snapshot);
        while BLOCKING_HOOK_STARTED.load(Ordering::SeqCst) == 0 {
            std::thread::yield_now();
        }
        let late = register_static_memory_manager_declaration(101, "late", "late", "late.rows.v1")
            .expect_err("concurrent late registration");
        assert_eq!(late, StaticMemoryDeclarationError::RegistrySealed);
        RELEASE_BLOCKING_HOOK.store(1, Ordering::SeqCst);
        sealing.join().expect("sealing thread").expect("snapshot");
    });
}

#[test]
fn deferred_registration_during_sealing_fails_the_active_snapshot() {
    let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
    reset_static_memory_declarations_for_tests();
    BLOCKING_HOOK_STARTED.store(0, Ordering::SeqCst);
    RELEASE_BLOCKING_HOOK.store(0, Ordering::SeqCst);
    defer_eager_init(block_during_sealing);

    std::thread::scope(|scope| {
        let sealing = scope.spawn(sealed_declaration_snapshot);
        while BLOCKING_HOOK_STARTED.load(Ordering::SeqCst) == 0 {
            std::thread::yield_now();
        }
        defer_static_memory_registration(no_registration);
        RELEASE_BLOCKING_HOOK.store(1, Ordering::SeqCst);
        let error = sealing
            .join()
            .expect("sealing thread")
            .expect_err("deferred registration must fail active sealing");
        assert_eq!(error, StaticMemoryDeclarationError::RegistrySealed);
    });

    let repeated = sealed_declaration_snapshot().expect_err("preserved failure");
    assert_eq!(repeated, StaticMemoryDeclarationError::RegistrySealed);
}

#[test]
fn recursive_snapshot_request_from_eager_hook_is_typed() {
    let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
    reset_static_memory_declarations_for_tests();
    EAGER_INIT_RUNS.store(0, Ordering::SeqCst);
    defer_eager_init(record_reentrant_seal_error);

    sealed_declaration_snapshot().expect("outer snapshot");

    assert_eq!(EAGER_INIT_RUNS.load(Ordering::SeqCst), 1);
}
