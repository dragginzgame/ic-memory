use ic_memory::{AllocationLedger, RecoveredLedger};

fn main() {
    let ledger = AllocationLedger::new(0, Vec::new())
        .expect("structurally valid ledger DTO");

    let _recovered = RecoveredLedger::from_trusted_ledger(ledger);
}
