use ic_memory::{
    AllocationDeclaration, DeclarationSnapshot, DeclarationSnapshotError, MemoryAllocationPool,
    MemoryAllocationPoolError, MemoryAuthority, MemoryDeclarationError, MemoryRequest,
    PolicyIdentity, PolicyIdentityError, SchemaMetadata,
};

#[test]
fn declarations_round_trip_checked_key_slot_and_schema() {
    let declaration = AllocationDeclaration::memory_manager("app.rows.v1", 100).unwrap();
    let json = serde_json::json!({
        "stable_key": "app.rows.v1",
        "slot": 100,
        "schema": { "schema_version": null },
    });
    assert_eq!(serde_json::to_value(&declaration).unwrap(), json);
    assert_eq!(
        serde_json::from_value::<AllocationDeclaration>(json).unwrap(),
        declaration
    );
    // Pin the current DTO bytes independently of decode.
    let expected =
        b"\xa3\x6astable_key\x6bapp.rows.v1\x64slot\x18\x64\x66schema\xa1\x6eschema_version\xf6";
    let mut bytes = Vec::new();
    ciborium::into_writer(&declaration, &mut bytes).unwrap();
    assert_eq!(bytes, expected);
    assert_eq!(
        ciborium::from_reader::<AllocationDeclaration, _>(bytes.as_slice()).unwrap(),
        declaration
    );
}

#[test]
fn diagnostic_metadata_accepts_printable_ascii_through_the_byte_limit() {
    for value in [
        " ".to_string(),
        (b' '..=b'~').map(char::from).collect(),
        "x".repeat(256),
    ] {
        PolicyIdentity::new(value.as_str(), 1).unwrap();
        MemoryRequest::new(value.as_str(), "app.rows.v1", SchemaMetadata::default()).unwrap();
        let grant = MemoryAuthority::new(value.as_str(), "app.").unwrap();
        let json = serde_json::json!({"authority":value,"key_prefix":"app."});
        assert_eq!(serde_json::to_value(&grant).unwrap(), json);
        assert_eq!(
            serde_json::from_value::<MemoryAuthority>(json).unwrap(),
            grant
        );
        let mut bytes = Vec::new();
        ciborium::into_writer(&grant, &mut bytes).unwrap();
        assert_eq!(
            ciborium::from_reader::<MemoryAuthority, _>(bytes.as_slice()).unwrap(),
            grant
        );
    }
}

#[test]
fn diagnostic_metadata_preserves_field_errors_and_rejection_order() {
    let cases = [
        (
            String::new(),
            PolicyIdentityError::EmptyName,
            "must not be empty",
        ),
        (
            // Length rejection precedes control-character rejection.
            format!("{}\0", "x".repeat(256)),
            PolicyIdentityError::NameTooLong {
                length: 257,
                maximum: 256,
            },
            "must be at most 256 bytes",
        ),
        (
            // ASCII rejection precedes control-character rejection.
            "é\0".to_string(),
            PolicyIdentityError::NonAsciiName,
            "must be ASCII",
        ),
        (
            "printable\u{7f}".to_string(),
            PolicyIdentityError::ControlCharacterName,
            "must not contain ASCII control characters",
        ),
    ];
    for (value, policy, reason) in cases {
        assert_eq!(PolicyIdentity::new(value.as_str(), 1).unwrap_err(), policy);
        assert_eq!(
            MemoryRequest::new(value.as_str(), "app.rows.v1", SchemaMetadata::default())
                .unwrap_err(),
            MemoryDeclarationError::InvalidAuthority { reason }
        );
        assert_eq!(
            MemoryAuthority::new(value.as_str(), "app.").unwrap_err(),
            MemoryAllocationPoolError::InvalidAuthority { reason }
        );
    }
}

fn snapshot_decode_error(value: &serde_json::Value, expected: &DeclarationSnapshotError) {
    let json = serde_json::from_value::<DeclarationSnapshot>(value.clone());
    let mut bytes = Vec::new();
    ciborium::into_writer(value, &mut bytes).unwrap();
    let cbor = ciborium::from_reader::<DeclarationSnapshot, _>(bytes.as_slice());
    assert_eq!([json.is_err(), cbor.is_err()], [true, true], "{expected}");
    assert!(
        json.unwrap_err()
            .to_string()
            .contains(&expected.to_string())
    );
    assert!(
        cbor.unwrap_err()
            .to_string()
            .contains(&expected.to_string())
    );
}

#[test]
fn snapshot_decode_checks_collection_invariants() {
    let declaration = AllocationDeclaration::memory_manager("app.rows.v1", 100).unwrap();
    for (second, expected) in [
        (
            declaration.clone(),
            DeclarationSnapshotError::DuplicateSlot(declaration.slot().clone()),
        ),
        (
            AllocationDeclaration::memory_manager("app.other.v1", 100).unwrap(),
            DeclarationSnapshotError::DuplicateSlot(declaration.slot().clone()),
        ),
        (
            AllocationDeclaration::memory_manager("app.rows.v1", 101).unwrap(),
            DeclarationSnapshotError::DuplicateStableKey(declaration.stable_key().clone()),
        ),
    ] {
        let value = serde_json::json!({
            "declarations": [declaration, second],
        });
        snapshot_decode_error(&value, &expected);
    }
    let mut value =
        serde_json::to_value(DeclarationSnapshot::new(vec![declaration]).unwrap()).unwrap();
    value["declarations"] = serde_json::Value::Array(vec![value["declarations"][0].clone(); 256]);
    assert!(serde_json::from_value::<DeclarationSnapshot>(value.clone()).is_err());
    let mut bytes = Vec::new();
    ciborium::into_writer(&value, &mut bytes).unwrap();
    assert!(ciborium::from_reader::<DeclarationSnapshot, _>(bytes.as_slice()).is_err());
}

#[test]
fn snapshot_decode_rejects_excess_before_decoding_the_extra_declaration() {
    let declarations = (0..=254)
        .map(|id| AllocationDeclaration::memory_manager(format!("app.store{id}.v1"), id).unwrap())
        .collect();
    let value = serde_json::to_value(DeclarationSnapshot::new(declarations).unwrap()).unwrap();
    let mut sequence = serde_json::to_string(&value["declarations"]).unwrap();
    sequence.pop(); // Replace the end with an extra element that cannot be parsed.
    let input = format!("{{\"declarations\":{sequence},!");
    let error = serde_json::from_str::<DeclarationSnapshot>(&input).unwrap_err();
    // A count refusal is a data error; decoding the malformed extra element
    // would instead produce a syntax error. No error prose is a contract.
    assert_eq!(error.classify(), serde_json::error::Category::Data);

    // Advertise 256 entries, without providing even the first declaration.
    // A semantic refusal proves the size hint was checked before an EOF read.
    let bytes = b"\xa1\x6cdeclarations\x99\x01\x00";
    let error = ciborium::from_reader::<DeclarationSnapshot, _>(bytes.as_slice()).unwrap_err();
    assert!(matches!(error, ciborium::de::Error::Semantic(..)));
}

#[test]
fn snapshot_decode_preserves_current_shape_order_and_full_slot_domain() {
    let empty = DeclarationSnapshot::new(Vec::new()).unwrap();
    let mut bytes = Vec::new();
    ciborium::into_writer(&empty, &mut bytes).unwrap();
    assert_eq!(bytes, b"\xa1\x6cdeclarations\x80");
    let full = DeclarationSnapshot::new(
        (0..=254)
            .rev()
            .map(|id| {
                AllocationDeclaration::memory_manager(format!("app.store{id}.v1"), id).unwrap()
            })
            .collect(),
    )
    .unwrap();
    for snapshot in [empty, full] {
        let value = serde_json::to_value(&snapshot).unwrap();
        assert_eq!(
            serde_json::from_value::<DeclarationSnapshot>(value).unwrap(),
            snapshot
        );
        let mut bytes = Vec::new();
        ciborium::into_writer(&snapshot, &mut bytes).unwrap();
        assert_eq!(
            ciborium::from_reader::<DeclarationSnapshot, _>(bytes.as_slice()).unwrap(),
            snapshot
        );
    }
    for value in [
        serde_json::json!({}),
        serde_json::json!({"declarations": 42}),
        serde_json::json!({"declarations": [], "extra": true}),
    ] {
        assert!(serde_json::from_value::<DeclarationSnapshot>(value.clone()).is_err());
        let mut bytes = Vec::new();
        ciborium::into_writer(&value, &mut bytes).unwrap();
        assert!(ciborium::from_reader::<DeclarationSnapshot, _>(bytes.as_slice()).is_err());
    }
}

#[test]
fn authority_decode_rejects_invalid_metadata_before_pool_assembly() {
    for grant in [
        serde_json::json!({"authority":"","key_prefix":"app."}),
        serde_json::json!({"authority":"app","key_prefix":"ic_memory."}),
        serde_json::json!({"authority":"app","key_prefix":"App."}),
        serde_json::json!({"authority":"app","key_prefix":"app"}),
        serde_json::json!({"authority":"app","key_prefix":"app.","extra":true}),
    ] {
        assert!(serde_json::from_value::<MemoryAuthority>(grant.clone()).is_err());
        let mut bytes = Vec::new();
        ciborium::into_writer(&grant, &mut bytes).unwrap();
        assert!(ciborium::from_reader::<MemoryAuthority, _>(bytes.as_slice()).is_err());
        assert!(
            serde_json::from_value::<MemoryAllocationPool>(
                serde_json::json!({"authorities":[grant],"excluded_ranges":[]})
            )
            .is_err()
        );
    }
}
#[test]
fn authority_decode_preserves_current_shape_and_required_fields() {
    let grant = MemoryAuthority::new("app", "app.").unwrap();
    let json = serde_json::json!({"authority":"app","key_prefix":"app."});
    assert_eq!(serde_json::to_value(&grant).unwrap(), json);
    for field in ["authority", "key_prefix"] {
        let mut incomplete = json.clone();
        incomplete.as_object_mut().unwrap().remove(field);
        assert!(serde_json::from_value::<MemoryAuthority>(incomplete).is_err());
    }
}
