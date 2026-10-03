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
        AllocationDeclaration::memory_manager("app.rows.v1", 100, value.as_str()).unwrap();
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
