use ic_memory::CommittedAllocations;

fn fabricate(committed: CommittedAllocations, generation: u64) -> CommittedAllocations {
    CommittedAllocations {
        generation,
        ..committed
    }
}

fn main() {}
