use ic_memory::{
    AllocationRetirement, MemoryManagerSlot, MemoryManagerAuthorityRecord,
    MemoryManagerIdRange, MemoryManagerRangeMode, SchemaMetadata, StableKey,
};

fn main() {
    let stable_key = StableKey::parse("app.orders.v1").expect("valid stable key");
    let slot = MemoryManagerSlot::new(100).expect("valid slot");

    let _retirement = AllocationRetirement { stable_key, slot };

    let range = MemoryManagerIdRange::new(100, 109).expect("valid range");
    let _authority = MemoryManagerAuthorityRecord {
        range,
        authority: "app".to_string(),
        mode: MemoryManagerRangeMode::Allowed,
        purpose: None,
    };

    let _schema = SchemaMetadata {
        schema_version: None,
    };
}
