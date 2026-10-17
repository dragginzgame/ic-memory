use ic_memory::{RuntimeMemory, VectorMemory};

fn duplicate(memory: RuntimeMemory<VectorMemory>) {
    let _other = memory.clone();
}

fn main() {}
