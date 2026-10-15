use ic_memory::CommittedAllocations;

fn fabricate(mut committed: CommittedAllocations, generation: u64) -> CommittedAllocations {
    committed.generation = generation;
    committed
}

fn main() {}
