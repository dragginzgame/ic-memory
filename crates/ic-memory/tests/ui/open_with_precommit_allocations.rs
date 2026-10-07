use ic_memory::{CommittedAllocations, ValidatedAllocations};

fn requires_committed(_allocations: CommittedAllocations) {}

fn validated() -> ValidatedAllocations {
    loop {}
}

fn main() {
    requires_committed(validated());
}
