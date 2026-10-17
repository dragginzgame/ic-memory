use ic_memory::{RuntimeMemory, Memory, VectorMemory};

fn grow(memory: &RuntimeMemory<VectorMemory>) {
    let _ = Memory::size(memory);
    let _: i64 = memory.grow(1);
}

fn main() {}
