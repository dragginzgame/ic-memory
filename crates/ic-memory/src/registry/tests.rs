use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

static EAGER_INIT_RUNS: AtomicUsize = AtomicUsize::new(0);
static BLOCKING_HOOK_STARTED: AtomicUsize = AtomicUsize::new(0);
static RELEASE_BLOCKING_HOOK: AtomicUsize = AtomicUsize::new(0);

fn register_from_eager_init() {
    EAGER_INIT_RUNS.fetch_add(1, Ordering::SeqCst);
    register_memory_request(
        MemoryRequest::new("eager", "eager.audit.v1", SchemaMetadata::default()).unwrap(),
    )
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

#[expect(
    clippy::unnecessary_wraps,
    reason = "the registration hook requires a fallible callback even when it does nothing"
)]
fn no_registration() -> Result<(), StaticMemoryDeclarationError> {
    Ok(())
}

#[test]
fn external_registration_rejects_internal_stable_key_namespace() {
    let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
    reset_static_memory_declarations_for_tests();

    let err = MemoryRequest::new("external", "ic_memory.spoof.v1", SchemaMetadata::default())
        .and_then(register_memory_request)
        .expect_err("internal stable key must be unavailable externally");

    assert!(matches!(
        err,
        StaticMemoryDeclarationError::ReservedStableKey { stable_key }
            if stable_key == "ic_memory.spoof.v1"
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
    assert_eq!(first.requests()[0].stable_key().as_str(), "eager.audit.v1");
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
    assert_eq!(first.requests().len(), 1);
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
        let late = MemoryRequest::new("late", "late.rows.v1", SchemaMetadata::default())
            .and_then(register_memory_request)
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

#[test]
fn registers_and_seals_static_memory_declarations() {
    let _guard = TEST_REGISTRY_LOCK.lock().unwrap();
    reset_static_memory_declarations_for_tests();
    register_memory_request(
        MemoryRequest::new("db", "db.rows.v1", SchemaMetadata::default()).unwrap(),
    )
    .unwrap();
    let sealed = sealed_declaration_snapshot().unwrap();
    assert_eq!(sealed.requests().len(), 1);
    assert_eq!(
        register_memory_request(sealed.requests()[0].clone()),
        Err(StaticMemoryDeclarationError::RegistrySealed)
    );
}

#[test]
fn external_registration_rejects_internal_authority_identity() {
    assert!(matches!(
        MemoryRequest::new(
            IC_MEMORY_AUTHORITY_OWNER,
            "app.rows.v1",
            SchemaMetadata::default()
        ),
        Err(StaticMemoryDeclarationError::ReservedAuthority { .. })
    ));
}

#[test]
fn request_permutations_are_canonical_and_duplicate_keys_reject() {
    let requests: Vec<_> = (0..245)
        .map(|i| {
            MemoryRequest::new(
                "app",
                &format!("app.rows{i}.v1"),
                SchemaMetadata::new(Some(i + 1)).unwrap(),
            )
            .unwrap()
        })
        .collect();
    let first = SealedDeclarationSnapshot::new(&requests).unwrap();
    let reversed: Vec<_> = requests.iter().rev().cloned().collect();
    let second = SealedDeclarationSnapshot::new(&reversed).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.fingerprint(), second.fingerprint());
    let mut conflict = requests;
    conflict
        .push(MemoryRequest::new("foreign", "app.rows0.v1", SchemaMetadata::default()).unwrap());
    assert!(matches!(
        SealedDeclarationSnapshot::new(&conflict),
        Err(StaticMemoryDeclarationError::DuplicateRequest { .. })
    ));
}

#[test]
fn snapshot_fingerprint_covers_linked_declaration_authority() {
    let make = |owner| {
        SealedDeclarationSnapshot::new(&[MemoryRequest::new(
            owner,
            "app.rows.v1",
            SchemaMetadata::default(),
        )
        .unwrap()])
        .unwrap()
    };
    assert_ne!(make("app").fingerprint(), make("foreign").fingerprint());
}
