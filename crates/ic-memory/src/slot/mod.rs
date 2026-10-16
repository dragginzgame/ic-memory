mod memory_manager;

pub use memory_manager::{
    IC_MEMORY_AUTHORITY_OWNER, IC_MEMORY_LEDGER_STABLE_KEY, IC_MEMORY_STABLE_KEY_PREFIX,
    MEMORY_MANAGER_GOVERNANCE_MAX_ID, MEMORY_MANAGER_INVALID_ID, MEMORY_MANAGER_LEDGER_ID,
    MEMORY_MANAGER_MAX_ID, MEMORY_MANAGER_MIN_ID, MemoryManagerSlot, MemoryManagerSlotError,
    is_ic_memory_stable_key, validate_memory_manager_id,
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
            assert_eq!(json, serde_json::json!(id));
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
            serde_json::json!(255),
            serde_json::json!(256),
            serde_json::json!(-1),
            serde_json::json!(1.5),
            serde_json::json!("1"),
            serde_json::json!(null),
            serde_json::json!({}),
        ] {
            assert!(serde_json::from_value::<MemoryManagerSlot>(json.clone()).is_err());
            let bytes = crate::test_cbor::to_vec(&json).unwrap();
            assert!(crate::cbor::from_slice_exact::<MemoryManagerSlot>(&bytes).is_err());
        }
    }
}
