use super::*;

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
