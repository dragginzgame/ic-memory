use ic_memory::{
    MemoryAllocationPool, MemoryAllocationPoolError, MemoryAuthority, MemoryDeclarationError,
    MemoryRequest,
};

#[test]
fn committed_bindings_serialize_checked_key_and_slot() {
    let mut runtime = ic_memory::MemoryRuntime::new(ic_memory::VectorMemory::default()).unwrap();
    let snapshot = ic_memory::SealedDeclarationSnapshot::new(vec![
        MemoryRequest::new("app", "app.rows.v1").unwrap(),
    ])
    .unwrap();
    let pool =
        MemoryAllocationPool::new(vec![MemoryAuthority::new("app", "app.").unwrap()], vec![])
            .unwrap();
    runtime
        .bootstrap(&snapshot, &pool, &ic_memory::GenericAllocationPolicy)
        .unwrap();
    let declaration = &runtime.committed_allocations().unwrap()[0];
    assert_eq!(
        serde_json::to_value(declaration).unwrap(),
        serde_json::json!({
            "stable_key": "app.rows.v1", "slot": 10,
        })
    );
    let mut bytes = Vec::new();
    ciborium::into_writer(declaration, &mut bytes).unwrap();
    assert_eq!(bytes, b"\xa2\x6astable_key\x6bapp.rows.v1\x64slot\x0a");
}

#[test]
fn diagnostic_metadata_accepts_printable_ascii_through_the_byte_limit() {
    for value in [
        " ".to_string(),
        (b' '..=b'~').map(char::from).collect(),
        "x".repeat(256),
    ] {
        MemoryRequest::new(value.as_str(), "app.rows.v1").unwrap();
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
        (String::new(), "must not be empty"),
        (
            // Length rejection precedes control-character rejection.
            format!("{}\0", "x".repeat(256)),
            "must be at most 256 bytes",
        ),
        (
            // ASCII rejection precedes control-character rejection.
            "é\0".to_string(),
            "must be ASCII",
        ),
        (
            "printable\u{7f}".to_string(),
            "must not contain ASCII control characters",
        ),
    ];
    for (value, reason) in cases {
        assert_eq!(
            MemoryRequest::new(value.as_str(), "app.rows.v1").unwrap_err(),
            MemoryDeclarationError::InvalidAuthority { reason }
        );
        assert_eq!(
            MemoryAuthority::new(value.as_str(), "app.").unwrap_err(),
            MemoryAllocationPoolError::InvalidAuthority { reason }
        );
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
                serde_json::json!({"authorities":[grant],"excluded_ids":[]})
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
