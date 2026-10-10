use ic_memory::{
    AllocationRetirement, MemoryManagerSlot, MemoryAuthority, SchemaMetadata, StableKey,
};

fn main() {
    let stable_key = StableKey::parse("app.orders.v1").expect("valid stable key");
    let slot = MemoryManagerSlot::new(100).expect("valid slot");

    let _retirement = AllocationRetirement { stable_key, slot };

    let _authority = MemoryAuthority {
        authority: "app".to_string(),
        key_prefix: "app.".to_string(),
    };

    let _schema = SchemaMetadata {
        schema_version: None,
    };
}
