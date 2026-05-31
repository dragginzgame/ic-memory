use ic_memory::{AllocationHistory, AllocationLedger, RecoveredLedger};

fn main() {
    let ledger = AllocationLedger::new(0, AllocationHistory::default())
        .expect("structurally valid ledger DTO");

    let _recovered = RecoveredLedger::from_trusted_parts(ledger, 0);
}
