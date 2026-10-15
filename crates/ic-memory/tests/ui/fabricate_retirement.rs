use ic_memory::{MemoryManagerSlot, StableKey, testing::AllocationRetirement};

fn main() {
    let stable_key = StableKey::parse("app.orders.v1").expect("valid stable key");
    let slot = MemoryManagerSlot::new(100).expect("valid slot");
    let _retirement = AllocationRetirement { stable_key, slot };
}
