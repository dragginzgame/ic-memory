use ic_memory::{
    AllocationDeclaration, DeclarationSnapshot, DeclarationSnapshotError,
    MemoryManagerAuthorityRecord, MemoryManagerIdRange, MemoryManagerRangeAuthorityError,
    MemoryManagerRangeMode, MemoryRequest, PolicyIdentity, PolicyIdentityError, SchemaMetadata,
    StaticMemoryDeclarationError,
};

#[test]
fn diagnostic_metadata_accepts_printable_ascii_through_the_byte_limit() {
    for value in [
        " ".to_string(),
        (b' '..=b'~').map(char::from).collect(),
        "x".repeat(256),
    ] {
        let declaration =
            AllocationDeclaration::memory_manager("app.rows.v1", 100, value.as_str()).unwrap();
        let json = serde_json::json!({
            "stable_key": "app.rows.v1",
            "slot": { "slot": { "MemoryManagerId": 100 } },
            "label": value,
            "schema": { "schema_version": null },
        });
        assert_eq!(serde_json::to_value(&declaration).unwrap(), json);
        assert_eq!(
            serde_json::from_value::<AllocationDeclaration>(json).unwrap(),
            declaration
        );
        let mut bytes = Vec::new();
        ciborium::into_writer(&declaration, &mut bytes).unwrap();
        assert_eq!(
            ciborium::from_reader::<AllocationDeclaration, _>(bytes.as_slice()).unwrap(),
            declaration
        );
        DeclarationSnapshot::new(Vec::new())
            .unwrap()
            .with_runtime_fingerprint(value.as_str())
            .unwrap();
        PolicyIdentity::new(value.as_str(), 1).unwrap();
        MemoryRequest::new(value.as_str(), "app.rows.v1", SchemaMetadata::default()).unwrap();
        MemoryManagerAuthorityRecord::new(
            MemoryManagerIdRange::new(100, 100).unwrap(),
            value.as_str(),
            MemoryManagerRangeMode::Allowed,
            Some(value.clone()),
        )
        .unwrap();
    }
    let unlabeled = AllocationDeclaration::memory_manager_unlabeled("app.rows.v1", 100).unwrap();
    assert_eq!(unlabeled.label(), None);
    // Pin the existing map and explicit-null encoding independently of decode.
    let expected = b"\xa4\x6astable_key\x6bapp.rows.v1\x64slot\xa1\x64slot\xa1\x6fMemoryManagerId\x18\x64\x65label\xf6\x66schema\xa1\x6eschema_version\xf6";
    let mut bytes = Vec::new();
    ciborium::into_writer(&unlabeled, &mut bytes).unwrap();
    assert_eq!(bytes, expected);
    assert_eq!(
        ciborium::from_reader::<AllocationDeclaration, _>(bytes.as_slice()).unwrap(),
        unlabeled
    );
    let snapshot = DeclarationSnapshot::new(vec![unlabeled]).unwrap();
    assert_eq!(snapshot.runtime_fingerprint(), None);
    assert_eq!(
        MemoryManagerAuthorityRecord::new(
            MemoryManagerIdRange::new(100, 100).unwrap(),
            "owner",
            MemoryManagerRangeMode::Allowed,
            None,
        )
        .unwrap()
        .purpose(),
        None
    );
}

#[test]
fn diagnostic_metadata_preserves_field_errors_and_rejection_order() {
    let cases = [
        (
            String::new(),
            DeclarationSnapshotError::EmptyLabel,
            DeclarationSnapshotError::EmptyRuntimeFingerprint,
            PolicyIdentityError::EmptyName,
            "must not be empty",
        ),
        (
            // Length rejection precedes control-character rejection.
            format!("{}\0", "x".repeat(256)),
            DeclarationSnapshotError::LabelTooLong,
            DeclarationSnapshotError::RuntimeFingerprintTooLong,
            PolicyIdentityError::NameTooLong {
                length: 257,
                maximum: 256,
            },
            "must be at most 256 bytes",
        ),
        (
            // ASCII rejection precedes control-character rejection.
            "é\0".to_string(),
            DeclarationSnapshotError::NonAsciiLabel,
            DeclarationSnapshotError::NonAsciiRuntimeFingerprint,
            PolicyIdentityError::NonAsciiName,
            "must be ASCII",
        ),
        (
            "printable\u{7f}".to_string(),
            DeclarationSnapshotError::ControlCharacterLabel,
            DeclarationSnapshotError::ControlCharacterRuntimeFingerprint,
            PolicyIdentityError::ControlCharacterName,
            "must not contain ASCII control characters",
        ),
    ];
    for (value, label, fingerprint, policy, reason) in cases {
        let mut encoded = serde_json::to_value(
            AllocationDeclaration::memory_manager_unlabeled("app.rows.v1", 100).unwrap(),
        )
        .unwrap();
        encoded["label"] = value.clone().into();
        let error = serde_json::from_value::<AllocationDeclaration>(encoded.clone()).unwrap_err();
        assert!(error.to_string().contains(&label.to_string()));
        let mut bytes = Vec::new();
        ciborium::into_writer(&encoded, &mut bytes).unwrap();
        let error =
            ciborium::from_reader::<AllocationDeclaration, _>(bytes.as_slice()).unwrap_err();
        assert!(error.to_string().contains(&label.to_string()));
        let snapshot = serde_json::json!({
            "declarations": [encoded],
            "runtime_fingerprint": null,
        });
        assert!(serde_json::from_value::<DeclarationSnapshot>(snapshot).is_err());
        assert_eq!(
            AllocationDeclaration::memory_manager("app.rows.v1", 100, value.as_str()).unwrap_err(),
            label
        );
        assert_eq!(
            DeclarationSnapshot::new(Vec::new())
                .unwrap()
                .with_runtime_fingerprint(value.as_str())
                .unwrap_err(),
            fingerprint
        );
        assert_eq!(PolicyIdentity::new(value.as_str(), 1).unwrap_err(), policy);
        assert_eq!(
            MemoryRequest::new(value.as_str(), "app.rows.v1", SchemaMetadata::default())
                .unwrap_err(),
            StaticMemoryDeclarationError::InvalidAuthority { reason }
        );
        let range = MemoryManagerIdRange::new(100, 100).unwrap();
        assert_eq!(
            MemoryManagerAuthorityRecord::new(
                range,
                value.as_str(),
                MemoryManagerRangeMode::Allowed,
                None,
            )
            .unwrap_err(),
            MemoryManagerRangeAuthorityError::InvalidDiagnosticString {
                field: "authority",
                reason,
            }
        );
        assert_eq!(
            MemoryManagerAuthorityRecord::new(
                range,
                "owner",
                MemoryManagerRangeMode::Allowed,
                Some(value),
            )
            .unwrap_err(),
            MemoryManagerRangeAuthorityError::InvalidDiagnosticString {
                field: "purpose",
                reason,
            }
        );
    }
}

#[test]
fn declaration_decode_requires_an_explicit_label_field() {
    let declaration = AllocationDeclaration::memory_manager_unlabeled("app.rows.v1", 100).unwrap();
    let mut value = serde_json::to_value(declaration).unwrap();
    value.as_object_mut().unwrap().remove("label");
    for malformed in [value.clone(), {
        value["label"] = 42.into();
        value
    }] {
        assert!(serde_json::from_value::<AllocationDeclaration>(malformed.clone()).is_err());
        let mut bytes = Vec::new();
        ciborium::into_writer(&malformed, &mut bytes).unwrap();
        assert!(ciborium::from_reader::<AllocationDeclaration, _>(bytes.as_slice()).is_err());
    }
}
