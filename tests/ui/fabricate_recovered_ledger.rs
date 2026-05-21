use ic_memory::{
    AllocationHistory, AllocationLedger, CURRENT_LEDGER_SCHEMA_VERSION, CURRENT_PHYSICAL_FORMAT_ID,
    RecoveredLedger,
};

fn main() {
    let ledger = AllocationLedger::new(
        CURRENT_LEDGER_SCHEMA_VERSION,
        CURRENT_PHYSICAL_FORMAT_ID,
        0,
        AllocationHistory::default(),
    )
    .expect("structurally valid ledger DTO");

    let _recovered = RecoveredLedger::from_trusted_parts(ledger, 0, 1);
}
