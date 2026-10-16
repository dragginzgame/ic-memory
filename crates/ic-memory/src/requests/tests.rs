use super::*;

#[test]
fn resolution_fills_free_slots_around_interleaved_retained_requests() {
    let pool = crate::MemoryAllocationPool::new(
        vec![crate::MemoryAuthority::new("app", "app.").unwrap()],
        (12..=13).collect(),
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
                    &AllocationDeclaration::memory_manager(key, id).unwrap(),
                )
            })
            .collect(),
    )
    .unwrap();
    let ledger = crate::RecoveredLedger::from_trusted_ledger(ledger);
    let requests = SealedDeclarationSnapshot::new(
        ["app.e.v1", "app.c.v1", "app.b.v1", "app.a.v1", "app.d.v1"]
            .into_iter()
            .map(|key| MemoryRequest::new("app", key).unwrap())
            .collect(),
    )
    .unwrap();
    let resolved = requests
        .resolve(&ledger, &ledger.ledger().occupied_slots(), vec![], &pool)
        .unwrap();
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
        MemoryRequest::new(IC_MEMORY_AUTHORITY_OWNER, "app.rows.v1"),
        Err(MemoryDeclarationError::ReservedAuthority { .. })
    ));
}

#[test]
fn request_permutations_are_canonical_and_duplicate_keys_reject() {
    let requests: Vec<_> = (0..245)
        .map(|i| MemoryRequest::new("app", &format!("app.rows{i}.v1")).unwrap())
        .collect();
    let first = SealedDeclarationSnapshot::new(requests.clone()).unwrap();
    let reversed: Vec<_> = requests.iter().rev().cloned().collect();
    let second = SealedDeclarationSnapshot::new(reversed).unwrap();
    assert_eq!(first, second);
    let mut conflict = requests;
    conflict.push(MemoryRequest::new("foreign", "app.rows0.v1").unwrap());
    assert!(matches!(
        SealedDeclarationSnapshot::new(conflict),
        Err(MemoryDeclarationError::DuplicateRequest { .. })
    ));
}

#[test]
fn snapshot_equality_covers_linked_declaration_authority() {
    let make = |owner, key| {
        SealedDeclarationSnapshot::new(vec![MemoryRequest::new(owner, key).unwrap()]).unwrap()
    };
    let snapshot = make("app", "app.rows.v1");
    let shared = snapshot.clone();
    assert_eq!(snapshot, shared);
    assert!(snapshot.shares_storage_with(&shared));
    for different in [
        make("foreign", "app.rows.v1"),
        make("app", "app.other.v1"),
        SealedDeclarationSnapshot::new(vec![]).unwrap(),
    ] {
        assert_ne!(snapshot, different);
    }
}

#[test]
fn external_requests_reject_internal_stable_key_namespace() {
    assert!(matches!(
        MemoryRequest::new("external", "ic_memory.spoof.v1"),
        Err(MemoryDeclarationError::ReservedStableKey { .. })
    ));
}

#[test]
fn oversized_composition_rejects_before_snapshot_publication() {
    let requests = (0..255)
        .map(|i| MemoryRequest::new("app", &format!("app.store{i}.v1")).unwrap())
        .collect();
    assert_eq!(
        SealedDeclarationSnapshot::new(requests),
        Err(MemoryDeclarationError::TooManyDeclarations)
    );
}

#[test]
fn decoded_requests_preserve_checked_fields_and_enforce_request_admission() {
    let request = MemoryRequest::new("app", "app.rows.v1").unwrap();
    let json = serde_json::to_value(&request).unwrap();
    assert_eq!(
        json,
        serde_json::json!({
            "authority": "app", "stable_key": "app.rows.v1"
        })
    );
    assert_eq!(
        serde_json::from_value::<MemoryRequest>(json.clone()).unwrap(),
        request
    );
    let bytes = crate::test_cbor::to_vec(&request).unwrap();
    assert_eq!(
        crate::test_cbor::from_slice::<MemoryRequest>(&bytes).unwrap(),
        request
    );
    for (field, value) in [
        ("authority", serde_json::json!("")),
        ("authority", serde_json::json!(IC_MEMORY_AUTHORITY_OWNER)),
        ("authority", serde_json::json!("app\n")),
        ("stable_key", serde_json::json!("ic_memory.rows.v1")),
        ("stable_key", serde_json::json!("App.rows.v1")),
        ("extra", serde_json::json!(true)),
    ] {
        let mut invalid = json.clone();
        invalid[field] = value;
        assert!(serde_json::from_value::<MemoryRequest>(invalid.clone()).is_err());
        let bytes = crate::test_cbor::to_vec(&invalid).unwrap();
        assert!(crate::test_cbor::from_slice::<MemoryRequest>(&bytes).is_err());
    }
    // The raw constructor retains authority-before-key error precedence.
    assert!(matches!(
        MemoryRequest::new("", "BAD"),
        Err(MemoryDeclarationError::InvalidAuthority { .. })
    ));
}
