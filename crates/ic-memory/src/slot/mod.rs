mod id_range;
mod memory_manager;

pub use id_range::{MemoryManagerIdRange, MemoryManagerRangeError};
pub use memory_manager::LEDGER_SLOT;
pub use memory_manager::{
    IC_MEMORY_AUTHORITY_OWNER, IC_MEMORY_LEDGER_LABEL, IC_MEMORY_LEDGER_STABLE_KEY,
    IC_MEMORY_STABLE_KEY_PREFIX, MEMORY_MANAGER_GOVERNANCE_MAX_ID, MEMORY_MANAGER_INVALID_ID,
    MEMORY_MANAGER_LEDGER_ID, MEMORY_MANAGER_MAX_ID, MEMORY_MANAGER_MIN_ID, MemoryManagerSlot,
    MemoryManagerSlotError, is_ic_memory_stable_key, memory_manager_governance_range,
    validate_memory_manager_id,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_manager_default_constructor_rejects_sentinel() {
        let err =
            MemoryManagerSlot::new(MEMORY_MANAGER_INVALID_ID).expect_err("sentinel must fail");

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

        MemoryManagerSlot::new(MEMORY_MANAGER_MAX_ID)
            .expect("254 is the last usable MemoryManager ID");
        MemoryManagerSlot::new(MEMORY_MANAGER_INVALID_ID)
            .expect_err("255 is always the unallocated sentinel");
    }

    #[test]
    fn slots_validate_ids_at_construction_and_decode() {
        for id in 0..MEMORY_MANAGER_INVALID_ID {
            let slot = MemoryManagerSlot::new(id).unwrap();
            assert_eq!(slot.id(), id);
            let json = serde_json::to_value(&slot).unwrap();
            assert_eq!(
                json,
                serde_json::json!({ "slot": { "MemoryManagerId": id } })
            );
            assert_eq!(
                serde_json::from_value::<MemoryManagerSlot>(json).unwrap(),
                slot
            );
            let bytes = crate::test_cbor::to_vec(&slot).unwrap();
            assert_eq!(
                crate::cbor::from_slice_exact::<MemoryManagerSlot>(&bytes).unwrap(),
                slot
            );
        }
        for json in [
            serde_json::json!({ "slot": { "MemoryManagerId": 255 } }),
            serde_json::json!({ "slot": { "MemoryManagerId": 256 } }),
            serde_json::json!({ "slot": { "MemoryManagerId": -1 } }),
            serde_json::json!({ "slot": { "MemoryManagerId": 1 }, "extra": true }),
            serde_json::json!({ "slot": { "Other": 1 } }),
            serde_json::json!({ "slot": null }),
            serde_json::json!({}),
        ] {
            assert!(serde_json::from_value::<MemoryManagerSlot>(json.clone()).is_err());
            let bytes = crate::test_cbor::to_vec(&json).unwrap();
            assert!(crate::cbor::from_slice_exact::<MemoryManagerSlot>(&bytes).is_err());
        }
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
        for (start, end) in [(10, 9), (MEMORY_MANAGER_INVALID_ID, MEMORY_MANAGER_MAX_ID)] {
            assert_eq!(
                MemoryManagerIdRange::new(start, end).unwrap_err(),
                MemoryManagerRangeError::InvalidRange { start, end }
            );
        }
    }

    #[test]
    fn memory_manager_range_rejects_sentinel_bounds() {
        for start in [240, MEMORY_MANAGER_INVALID_ID] {
            assert_eq!(
                MemoryManagerIdRange::new(start, MEMORY_MANAGER_INVALID_ID).unwrap_err(),
                MemoryManagerRangeError::InvalidMemoryManagerId {
                    id: MEMORY_MANAGER_INVALID_ID
                }
            );
        }
    }

    #[test]
    fn memory_manager_range_decode_enforces_usable_bounds() {
        for (start, end) in [(0, 0), (0, 254), (100, 109), (254, 254)] {
            let range = MemoryManagerIdRange::new(start, end).unwrap();
            let json = serde_json::to_value(range).unwrap();
            assert_eq!(json, serde_json::json!({ "start": start, "end": end }));
            assert_eq!(
                serde_json::from_value::<MemoryManagerIdRange>(json).unwrap(),
                range
            );
            let bytes = crate::test_cbor::to_vec(&range).unwrap();
            assert_eq!(
                crate::test_cbor::from_slice::<MemoryManagerIdRange>(&bytes).unwrap(),
                range
            );
        }
        let range = MemoryManagerIdRange::new(100, 109).unwrap();
        assert_eq!(
            crate::test_cbor::to_vec(&range).unwrap(),
            b"\xa2\x65start\x18\x64\x63end\x18\x6d"
        );

        for json in [
            serde_json::json!({ "start": 100, "end": 99 }),
            serde_json::json!({ "start": 100, "end": 255 }),
            serde_json::json!({ "start": 255, "end": 255 }),
            serde_json::json!({ "start": 0, "end": 256 }),
            serde_json::json!({ "start": 0 }),
            serde_json::json!({ "start": 0, "end": 1, "extra": true }),
        ] {
            assert!(serde_json::from_value::<MemoryManagerIdRange>(json.clone()).is_err());
            let bytes = crate::test_cbor::to_vec(&json).unwrap();
            assert!(crate::test_cbor::from_slice::<MemoryManagerIdRange>(&bytes).is_err());
        }
    }
}
