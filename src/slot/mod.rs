mod descriptor;
mod memory_manager;
mod range_authority;

pub use descriptor::{AllocationSlot, AllocationSlotDescriptor};
pub use memory_manager::{
    IC_MEMORY_AUTHORITY_OWNER, IC_MEMORY_AUTHORITY_PURPOSE, IC_MEMORY_LEDGER_LABEL,
    IC_MEMORY_LEDGER_STABLE_KEY, IC_MEMORY_STABLE_KEY_PREFIX, MEMORY_MANAGER_GOVERNANCE_MAX_ID,
    MEMORY_MANAGER_INVALID_ID, MEMORY_MANAGER_LEDGER_ID, MEMORY_MANAGER_MAX_ID,
    MEMORY_MANAGER_MIN_ID, MemoryManagerSlotError, is_ic_memory_stable_key,
    memory_manager_governance_range, validate_memory_manager_id,
};
pub use range_authority::{
    MemoryManagerAuthorityRecord, MemoryManagerIdRange, MemoryManagerRangeAuthority,
    MemoryManagerRangeAuthorityError, MemoryManagerRangeError, MemoryManagerRangeMode,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_manager_default_constructor_rejects_sentinel() {
        let err = AllocationSlotDescriptor::memory_manager(MEMORY_MANAGER_INVALID_ID)
            .expect_err("sentinel must fail");

        assert_eq!(
            err,
            MemoryManagerSlotError::InvalidMemoryManagerId {
                id: MEMORY_MANAGER_INVALID_ID
            }
        );
    }

    #[test]
    fn memory_manager_usable_domain_is_u8_with_255_sentinel() {
        assert_eq!(MEMORY_MANAGER_MIN_ID, 0);
        assert_eq!(MEMORY_MANAGER_MAX_ID, 254);
        assert_eq!(MEMORY_MANAGER_INVALID_ID, 255);
        assert_eq!(MEMORY_MANAGER_INVALID_ID, u8::MAX);

        AllocationSlotDescriptor::memory_manager(MEMORY_MANAGER_MAX_ID)
            .expect("254 is the last usable MemoryManager ID");
        AllocationSlotDescriptor::memory_manager(MEMORY_MANAGER_INVALID_ID)
            .expect_err("255 is always the unallocated sentinel");
    }

    #[test]
    fn memory_manager_id_validates_sentinel() {
        let slot = AllocationSlotDescriptor::memory_manager(42).expect("usable slot");
        assert_eq!(slot.memory_manager_id().expect("usable ID"), 42);

        let err = AllocationSlotDescriptor {
            slot: AllocationSlot::MemoryManagerId(MEMORY_MANAGER_INVALID_ID),
        }
        .memory_manager_id()
        .expect_err("sentinel should fail");
        assert_eq!(
            err,
            MemoryManagerSlotError::InvalidMemoryManagerId {
                id: MEMORY_MANAGER_INVALID_ID
            }
        );
    }

    #[test]
    fn memory_manager_range_accepts_usable_ranges() {
        let range = MemoryManagerIdRange::new(MEMORY_MANAGER_MIN_ID, MEMORY_MANAGER_MAX_ID)
            .expect("usable full range");

        assert!(range.contains(MEMORY_MANAGER_MIN_ID));
        assert!(range.contains(MEMORY_MANAGER_MAX_ID));
        assert!(!range.contains(MEMORY_MANAGER_INVALID_ID));
    }

    #[test]
    fn memory_manager_range_all_usable_matches_usable_bounds() {
        assert_eq!(
            MemoryManagerIdRange::all_usable(),
            MemoryManagerIdRange::new(MEMORY_MANAGER_MIN_ID, MEMORY_MANAGER_MAX_ID)
                .expect("usable full range")
        );
    }

    #[test]
    fn memory_manager_governance_range_is_owned_by_ic_memory() {
        let range = memory_manager_governance_range();

        assert_eq!(range.start(), MEMORY_MANAGER_MIN_ID);
        assert_eq!(MEMORY_MANAGER_LEDGER_ID, range.start());
        assert!(range.contains(MEMORY_MANAGER_LEDGER_ID));
        assert!(is_ic_memory_stable_key(IC_MEMORY_LEDGER_STABLE_KEY));
        assert_eq!(IC_MEMORY_AUTHORITY_OWNER, "ic-memory");
    }

    #[test]
    fn memory_manager_range_rejects_reversed_bounds() {
        let err = MemoryManagerIdRange::new(10, 9).expect_err("reversed range");

        assert_eq!(
            err,
            MemoryManagerRangeError::InvalidRange { start: 10, end: 9 }
        );
    }

    #[test]
    fn memory_manager_range_rejects_sentinel_bounds() {
        let err =
            MemoryManagerIdRange::new(240, MEMORY_MANAGER_INVALID_ID).expect_err("sentinel range");

        assert_eq!(
            err,
            MemoryManagerRangeError::InvalidMemoryManagerId {
                id: MEMORY_MANAGER_INVALID_ID
            }
        );
    }

    #[test]
    fn memory_manager_range_authority_accepts_non_overlapping_construction() {
        let authority = MemoryManagerRangeAuthority::new()
            .reserve(memory_manager_governance_range(), IC_MEMORY_AUTHORITY_OWNER)
            .expect("ic-memory range")
            .reserve_ids(10, 99, "framework")
            .expect("framework range")
            .allow_ids(100, MEMORY_MANAGER_MAX_ID, "applications")
            .expect("app range");

        assert_eq!(authority.authorities().len(), 3);
        assert_eq!(
            authority.authorities()[0].range,
            memory_manager_governance_range()
        );
        assert_eq!(
            authority.authorities()[0].mode,
            MemoryManagerRangeMode::Reserved
        );
        assert_eq!(authority.authorities()[1].range.start(), 10);
        assert_eq!(authority.authorities()[2].range.start(), 100);
    }

    #[test]
    fn memory_manager_range_authority_id_bound_builders_reject_invalid_ranges() {
        let err = MemoryManagerRangeAuthority::new()
            .allow_ids(100, MEMORY_MANAGER_INVALID_ID, "applications")
            .expect_err("sentinel must fail");

        assert_eq!(
            err,
            MemoryManagerRangeAuthorityError::Range(
                MemoryManagerRangeError::InvalidMemoryManagerId {
                    id: MEMORY_MANAGER_INVALID_ID
                }
            )
        );
    }

    #[test]
    fn memory_manager_range_authority_from_records_rejects_decoded_reversed_range() {
        let err = MemoryManagerRangeAuthority::from_records(vec![MemoryManagerAuthorityRecord {
            range: MemoryManagerIdRange {
                start: 100,
                end: 99,
            },
            authority: "applications".to_string(),
            mode: MemoryManagerRangeMode::Allowed,
            purpose: None,
        }])
        .expect_err("decoded reversed range must fail");

        assert_eq!(
            err,
            MemoryManagerRangeAuthorityError::Range(MemoryManagerRangeError::InvalidRange {
                start: 100,
                end: 99,
            })
        );
    }

    #[test]
    fn memory_manager_range_authority_from_records_rejects_decoded_sentinel_range() {
        let err = MemoryManagerRangeAuthority::from_records(vec![MemoryManagerAuthorityRecord {
            range: MemoryManagerIdRange {
                start: 100,
                end: MEMORY_MANAGER_INVALID_ID,
            },
            authority: "applications".to_string(),
            mode: MemoryManagerRangeMode::Allowed,
            purpose: None,
        }])
        .expect_err("decoded sentinel range must fail");

        assert_eq!(
            err,
            MemoryManagerRangeAuthorityError::Range(
                MemoryManagerRangeError::InvalidMemoryManagerId {
                    id: MEMORY_MANAGER_INVALID_ID,
                }
            )
        );
    }

    #[test]
    fn memory_manager_range_authority_rejects_overlap() {
        let err = MemoryManagerRangeAuthority::new()
            .reserve(
                MemoryManagerIdRange::new(10, 99).expect("framework range"),
                "framework",
            )
            .expect("framework range")
            .allow(
                MemoryManagerIdRange::new(99, 120).expect("overlapping app range"),
                "applications",
            )
            .expect_err("overlap must fail");

        assert_eq!(
            err,
            MemoryManagerRangeAuthorityError::OverlappingRanges {
                existing_start: 10,
                existing_end: 99,
                candidate_start: 99,
                candidate_end: 120,
            }
        );
    }

    #[test]
    fn memory_manager_range_authority_rejects_invalid_diagnostic_strings() {
        let err = MemoryManagerRangeAuthority::new()
            .reserve(
                MemoryManagerIdRange::new(10, 99).expect("framework range"),
                "",
            )
            .expect_err("empty authority must fail");
        assert_eq!(
            err,
            MemoryManagerRangeAuthorityError::InvalidDiagnosticString {
                field: "authority",
                reason: "must not be empty",
            }
        );

        let err = MemoryManagerRangeAuthority::new()
            .allow_with_purpose(
                MemoryManagerIdRange::new(100, MEMORY_MANAGER_MAX_ID).expect("app range"),
                "applications",
                Some("bad\npurpose".to_string()),
            )
            .expect_err("control character purpose must fail");
        assert_eq!(
            err,
            MemoryManagerRangeAuthorityError::InvalidDiagnosticString {
                field: "purpose",
                reason: "must not contain ASCII control characters",
            }
        );
    }

    #[test]
    fn memory_manager_range_authority_rejects_sentinel_lookup() {
        let err = MemoryManagerRangeAuthority::new()
            .authority_for_id(MEMORY_MANAGER_INVALID_ID)
            .expect_err("sentinel lookup must fail");

        assert_eq!(
            err,
            MemoryManagerRangeAuthorityError::Slot(
                MemoryManagerSlotError::InvalidMemoryManagerId {
                    id: MEMORY_MANAGER_INVALID_ID
                }
            )
        );
    }

    #[test]
    fn memory_manager_range_authority_finds_authority_for_id() {
        let authority = MemoryManagerRangeAuthority::new()
            .allow(
                MemoryManagerIdRange::new(100, MEMORY_MANAGER_MAX_ID).expect("app range"),
                "applications",
            )
            .expect("app range")
            .reserve(memory_manager_governance_range(), IC_MEMORY_AUTHORITY_OWNER)
            .expect("ic-memory range");

        let record = authority
            .authority_for_id(100)
            .expect("valid ID")
            .expect("authority record");
        assert_eq!(record.authority, "applications");
        assert_eq!(record.mode, MemoryManagerRangeMode::Allowed);

        assert!(
            authority
                .authority_for_id(99)
                .expect("valid unclaimed ID")
                .is_none()
        );
    }

    #[test]
    fn memory_manager_range_authority_validates_slot_authority() {
        let authority = MemoryManagerRangeAuthority::new()
            .reserve(
                MemoryManagerIdRange::new(10, 99).expect("framework range"),
                "framework",
            )
            .expect("framework range")
            .allow(
                MemoryManagerIdRange::new(100, MEMORY_MANAGER_MAX_ID).expect("app range"),
                "applications",
            )
            .expect("app range");

        let record = authority
            .validate_slot_authority(
                &AllocationSlotDescriptor::memory_manager(42).expect("framework slot"),
                "framework",
            )
            .expect("framework authority");
        assert_eq!(record.mode, MemoryManagerRangeMode::Reserved);

        let err = authority
            .validate_slot_authority(
                &AllocationSlotDescriptor::memory_manager(42).expect("framework slot"),
                "applications",
            )
            .expect_err("wrong authority must fail");
        assert_eq!(
            err,
            MemoryManagerRangeAuthorityError::AuthorityMismatch {
                id: 42,
                expected_authority: "applications".to_string(),
                actual_authority: "framework".to_string(),
            }
        );
    }

    #[test]
    fn memory_manager_range_authority_validates_slot_authority_mode() {
        let authority = MemoryManagerRangeAuthority::new()
            .reserve(
                MemoryManagerIdRange::new(10, 99).expect("framework range"),
                "framework",
            )
            .expect("framework range")
            .allow(
                MemoryManagerIdRange::new(100, MEMORY_MANAGER_MAX_ID).expect("app range"),
                "applications",
            )
            .expect("app range");

        let record = authority
            .validate_slot_authority_mode(
                &AllocationSlotDescriptor::memory_manager(42).expect("framework slot"),
                "framework",
                MemoryManagerRangeMode::Reserved,
            )
            .expect("framework reserved authority");
        assert_eq!(record.authority, "framework");

        let err = authority
            .validate_slot_authority_mode(
                &AllocationSlotDescriptor::memory_manager(42).expect("framework slot"),
                "framework",
                MemoryManagerRangeMode::Allowed,
            )
            .expect_err("wrong mode must fail");
        assert_eq!(
            err,
            MemoryManagerRangeAuthorityError::ModeMismatch {
                id: 42,
                authority: "framework".to_string(),
                expected_mode: MemoryManagerRangeMode::Allowed,
                actual_mode: MemoryManagerRangeMode::Reserved,
            }
        );
    }

    #[test]
    fn memory_manager_range_authority_validates_id_authority() {
        let authority = MemoryManagerRangeAuthority::new()
            .reserve(
                MemoryManagerIdRange::new(10, 99).expect("framework range"),
                "framework",
            )
            .expect("framework range")
            .allow(
                MemoryManagerIdRange::new(100, MEMORY_MANAGER_MAX_ID).expect("app range"),
                "applications",
            )
            .expect("app range");

        assert_eq!(
            authority
                .validate_id_authority(100, "applications")
                .expect("application authority")
                .mode,
            MemoryManagerRangeMode::Allowed
        );
        assert_eq!(
            authority
                .validate_id_authority_mode(42, "framework", MemoryManagerRangeMode::Reserved)
                .expect("framework reserved authority")
                .range,
            MemoryManagerIdRange::new(10, 99).expect("framework range")
        );

        let err = authority
            .validate_id_authority_mode(100, "applications", MemoryManagerRangeMode::Reserved)
            .expect_err("wrong mode must fail");
        assert_eq!(
            err,
            MemoryManagerRangeAuthorityError::ModeMismatch {
                id: 100,
                authority: "applications".to_string(),
                expected_mode: MemoryManagerRangeMode::Reserved,
                actual_mode: MemoryManagerRangeMode::Allowed,
            }
        );
    }

    #[test]
    fn memory_manager_range_authority_reports_authority_mismatch_before_mode_mismatch() {
        let authority = MemoryManagerRangeAuthority::new()
            .allow(
                MemoryManagerIdRange::new(100, MEMORY_MANAGER_MAX_ID).expect("app range"),
                "applications",
            )
            .expect("app range");

        let err = authority
            .validate_id_authority_mode(100, "framework", MemoryManagerRangeMode::Reserved)
            .expect_err("authority mismatch must be distinct");
        assert_eq!(
            err,
            MemoryManagerRangeAuthorityError::AuthorityMismatch {
                id: 100,
                expected_authority: "framework".to_string(),
                actual_authority: "applications".to_string(),
            }
        );
    }

    #[test]
    fn memory_manager_range_authority_preserves_reserve_and_allow_modes() {
        let authority = MemoryManagerRangeAuthority::new()
            .reserve(
                MemoryManagerIdRange::new(10, 99).expect("framework range"),
                "framework",
            )
            .expect("framework range")
            .allow(
                MemoryManagerIdRange::new(100, MEMORY_MANAGER_MAX_ID).expect("app range"),
                "applications",
            )
            .expect("app range");

        assert_eq!(
            authority.authorities()[0].mode,
            MemoryManagerRangeMode::Reserved
        );
        assert_eq!(
            authority.authorities()[1].mode,
            MemoryManagerRangeMode::Allowed
        );
    }

    #[test]
    fn memory_manager_range_authority_validates_complete_coverage() {
        let authority = MemoryManagerRangeAuthority::new()
            .reserve(memory_manager_governance_range(), IC_MEMORY_AUTHORITY_OWNER)
            .expect("ic-memory range")
            .reserve(
                MemoryManagerIdRange::new(10, 99).expect("framework range"),
                "framework",
            )
            .expect("framework range")
            .allow(
                MemoryManagerIdRange::new(100, MEMORY_MANAGER_MAX_ID).expect("app range"),
                "applications",
            )
            .expect("app range");

        authority
            .validate_complete_coverage(
                MemoryManagerIdRange::new(MEMORY_MANAGER_MIN_ID, MEMORY_MANAGER_MAX_ID)
                    .expect("full range"),
            )
            .expect("complete coverage");
    }

    #[test]
    fn memory_manager_range_authority_rejects_complete_coverage_gaps() {
        let authority = MemoryManagerRangeAuthority::new()
            .reserve(memory_manager_governance_range(), IC_MEMORY_AUTHORITY_OWNER)
            .expect("ic-memory range")
            .allow(
                MemoryManagerIdRange::new(100, MEMORY_MANAGER_MAX_ID).expect("app range"),
                "applications",
            )
            .expect("app range");

        let err = authority
            .validate_complete_coverage(
                MemoryManagerIdRange::new(MEMORY_MANAGER_MIN_ID, MEMORY_MANAGER_MAX_ID)
                    .expect("full range"),
            )
            .expect_err("coverage gap must fail");
        assert_eq!(
            err,
            MemoryManagerRangeAuthorityError::MissingCoverage { start: 10, end: 99 }
        );

        let err = MemoryManagerRangeAuthority::new()
            .validate_complete_coverage(
                MemoryManagerIdRange::new(MEMORY_MANAGER_MIN_ID, MEMORY_MANAGER_MAX_ID)
                    .expect("full range"),
            )
            .expect_err("empty coverage must fail");
        assert_eq!(
            err,
            MemoryManagerRangeAuthorityError::MissingCoverage {
                start: MEMORY_MANAGER_MIN_ID,
                end: MEMORY_MANAGER_MAX_ID,
            }
        );
    }

    #[test]
    fn memory_manager_range_authority_rejects_complete_coverage_outside_target() {
        let authority = MemoryManagerRangeAuthority::new()
            .reserve(memory_manager_governance_range(), IC_MEMORY_AUTHORITY_OWNER)
            .expect("ic-memory range")
            .reserve(
                MemoryManagerIdRange::new(10, 99).expect("framework range"),
                "framework",
            )
            .expect("framework range");

        let err = authority
            .validate_complete_coverage(MemoryManagerIdRange::new(10, 99).expect("target range"))
            .expect_err("outside range must fail");
        assert_eq!(
            err,
            MemoryManagerRangeAuthorityError::RangeOutsideCoverageTarget {
                start: MEMORY_MANAGER_MIN_ID,
                end: MEMORY_MANAGER_GOVERNANCE_MAX_ID,
                target_start: 10,
                target_end: 99,
            }
        );
    }

    #[test]
    fn memory_manager_range_authority_from_records_sorts_and_validates() {
        let err = MemoryManagerRangeAuthority::from_records(vec![MemoryManagerAuthorityRecord {
            range: MemoryManagerIdRange::new(10, 99).expect("framework range"),
            authority: String::new(),
            mode: MemoryManagerRangeMode::Reserved,
            purpose: None,
        }])
        .expect_err("empty authority must fail");
        assert_eq!(
            err,
            MemoryManagerRangeAuthorityError::InvalidDiagnosticString {
                field: "authority",
                reason: "must not be empty",
            }
        );

        let authority = MemoryManagerRangeAuthority::from_records(vec![
            MemoryManagerAuthorityRecord {
                range: MemoryManagerIdRange::new(100, MEMORY_MANAGER_MAX_ID).expect("app range"),
                authority: "applications".to_string(),
                mode: MemoryManagerRangeMode::Allowed,
                purpose: Some("application stable stores".to_string()),
            },
            MemoryManagerAuthorityRecord {
                range: memory_manager_governance_range(),
                authority: IC_MEMORY_AUTHORITY_OWNER.to_string(),
                mode: MemoryManagerRangeMode::Reserved,
                purpose: Some(IC_MEMORY_AUTHORITY_PURPOSE.to_string()),
            },
        ])
        .expect("records");

        assert_eq!(
            authority.authorities()[0].authority,
            IC_MEMORY_AUTHORITY_OWNER
        );
        assert_eq!(authority.authorities()[1].authority, "applications");
    }

    #[test]
    fn memory_manager_authority_record_constructor_validates_metadata() {
        let record = MemoryManagerAuthorityRecord::new(
            MemoryManagerIdRange::new(10, 99).expect("framework range"),
            "framework",
            MemoryManagerRangeMode::Reserved,
            Some("framework-owned stores".to_string()),
        )
        .expect("authority record");

        assert_eq!(record.authority, "framework");
        assert_eq!(record.purpose.as_deref(), Some("framework-owned stores"));

        let err = MemoryManagerAuthorityRecord::new(
            MemoryManagerIdRange::new(10, 99).expect("framework range"),
            "",
            MemoryManagerRangeMode::Reserved,
            None,
        )
        .expect_err("empty authority must fail");

        assert_eq!(
            err,
            MemoryManagerRangeAuthorityError::InvalidDiagnosticString {
                field: "authority",
                reason: "must not be empty",
            }
        );
    }

    #[test]
    fn memory_manager_range_authority_from_records_rejects_overlap() {
        let err = MemoryManagerRangeAuthority::from_records(vec![
            MemoryManagerAuthorityRecord {
                range: MemoryManagerIdRange::new(10, 99).expect("framework range"),
                authority: "framework".to_string(),
                mode: MemoryManagerRangeMode::Reserved,
                purpose: None,
            },
            MemoryManagerAuthorityRecord {
                range: MemoryManagerIdRange::new(90, 120).expect("overlap range"),
                authority: "applications".to_string(),
                mode: MemoryManagerRangeMode::Allowed,
                purpose: None,
            },
        ])
        .expect_err("overlap must fail");

        assert_eq!(
            err,
            MemoryManagerRangeAuthorityError::OverlappingRanges {
                existing_start: 10,
                existing_end: 99,
                candidate_start: 90,
                candidate_end: 120,
            }
        );
    }

    #[test]
    fn memory_manager_range_authority_diagnostic_export_is_stable() {
        let authority = MemoryManagerRangeAuthority::new()
            .allow_with_purpose(
                MemoryManagerIdRange::new(100, MEMORY_MANAGER_MAX_ID).expect("app range"),
                "applications",
                Some("application stable stores".to_string()),
            )
            .expect("app range")
            .reserve_with_purpose(
                memory_manager_governance_range(),
                IC_MEMORY_AUTHORITY_OWNER,
                Some(IC_MEMORY_AUTHORITY_PURPOSE.to_string()),
            )
            .expect("ic-memory range");

        assert_eq!(
            authority.authorities(),
            vec![
                MemoryManagerAuthorityRecord {
                    range: memory_manager_governance_range(),
                    authority: IC_MEMORY_AUTHORITY_OWNER.to_string(),
                    mode: MemoryManagerRangeMode::Reserved,
                    purpose: Some(IC_MEMORY_AUTHORITY_PURPOSE.to_string()),
                },
                MemoryManagerAuthorityRecord {
                    range: MemoryManagerIdRange::new(100, MEMORY_MANAGER_MAX_ID)
                        .expect("app range"),
                    authority: "applications".to_string(),
                    mode: MemoryManagerRangeMode::Allowed,
                    purpose: Some("application stable stores".to_string()),
                },
            ]
            .as_slice()
        );
    }
}
