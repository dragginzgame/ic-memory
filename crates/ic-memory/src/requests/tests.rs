use super::*;

#[test]
fn resolution_fills_free_slots_around_interleaved_retained_requests() {
    let pool = crate::MemoryAllocationPool::new(
        vec![crate::MemoryAuthority::new("app", "app.").unwrap()],
        vec![crate::MemoryManagerIdRange::new(12, 13).unwrap()],
    )
    .unwrap();
    // Retained keys occur both before and after new keys in canonical order.
    // An omitted key still occupies its slot.
    let ledger = crate::AllocationLedger::new(
        1,
        [("app.a.v1", 20), ("app.c.v1", 11), ("app.omitted.v1", 15)]
            .into_iter()
            .map(|(key, id)| {
                crate::AllocationRecord::active(
                    &AllocationDeclaration::memory_manager_unlabeled(key, id).unwrap(),
                )
            })
            .collect(),
    )
    .unwrap();
    let requests = SealedDeclarationSnapshot::new(
        ["app.e.v1", "app.c.v1", "app.b.v1", "app.a.v1", "app.d.v1"]
            .into_iter()
            .map(|key| MemoryRequest::new("app", key, SchemaMetadata::default()).unwrap())
            .collect(),
    )
    .unwrap();
    let resolved = requests.resolve(&ledger, vec![], &pool).unwrap();
    let external: Vec<_> = resolved
        .declarations()
        .iter()
        .filter(|declaration| declaration.stable_key().as_str().starts_with("app."))
        .map(|declaration| (declaration.stable_key().as_str(), declaration.slot().id()))
        .collect();
    assert_eq!(
        external,
        [
            ("app.a.v1", 20),
            ("app.b.v1", 10),
            ("app.c.v1", 11),
            ("app.d.v1", 14),
            ("app.e.v1", 16)
        ]
    );
}

#[test]
fn external_requests_reject_internal_authority_identity() {
    assert!(matches!(
        MemoryRequest::new(
            IC_MEMORY_AUTHORITY_OWNER,
            "app.rows.v1",
            SchemaMetadata::default()
        ),
        Err(MemoryDeclarationError::ReservedAuthority { .. })
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
    let first = SealedDeclarationSnapshot::new(requests.clone()).unwrap();
    let reversed: Vec<_> = requests.iter().rev().cloned().collect();
    let second = SealedDeclarationSnapshot::new(reversed).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.fingerprint(), second.fingerprint());
    let mut conflict = requests;
    conflict
        .push(MemoryRequest::new("foreign", "app.rows0.v1", SchemaMetadata::default()).unwrap());
    assert!(matches!(
        SealedDeclarationSnapshot::new(conflict),
        Err(MemoryDeclarationError::DuplicateRequest { .. })
    ));
}

#[test]
fn snapshot_fingerprint_covers_linked_declaration_authority() {
    let make = |owner| {
        SealedDeclarationSnapshot::new(vec![
            MemoryRequest::new(owner, "app.rows.v1", SchemaMetadata::default()).unwrap(),
        ])
        .unwrap()
    };
    assert_ne!(make("app").fingerprint(), make("foreign").fingerprint());
}

#[test]
fn external_requests_reject_internal_stable_key_namespace() {
    assert!(matches!(
        MemoryRequest::new("external", "ic_memory.spoof.v1", SchemaMetadata::default()),
        Err(MemoryDeclarationError::ReservedStableKey { .. })
    ));
}

#[test]
fn oversized_composition_rejects_before_snapshot_publication() {
    let requests = (0..255)
        .map(|i| {
            MemoryRequest::new(
                "app",
                &format!("app.store{i}.v1"),
                SchemaMetadata::default(),
            )
            .unwrap()
        })
        .collect();
    assert_eq!(
        SealedDeclarationSnapshot::new(requests),
        Err(MemoryDeclarationError::TooManyDeclarations)
    );
}
