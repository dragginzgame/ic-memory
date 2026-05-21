use ic_memory::ValidatedAllocations;

fn main() {
    let _validated = ValidatedAllocations::new(0, Vec::new(), None);
}
