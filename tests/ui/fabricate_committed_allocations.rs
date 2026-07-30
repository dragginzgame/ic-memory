use ic_memory::ValidatedAllocations;

fn fabricate(validated: ValidatedAllocations) {
    let _committed = validated.confirm_persisted(0);
}

fn main() {}
