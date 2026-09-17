//! Run with `cargo run --example key_only`.
use ic_memory::{
    AllocationBootstrap, AllocationDeclaration, AllocationHistory, AllocationLedger,
    GenericRangePolicy, LedgerCommitStore, MemoryManagerAuthorityRecord, MemoryManagerIdRange,
    MemoryManagerRangeMode, MemoryRequest, MemoryRuntime, SchemaMetadata,
    SealedDeclarationSnapshot, StableCellLedgerRecord, StableKey, StaticMemoryDeclaration,
    StaticMemoryRangeDeclaration,
};
use ic_stable_structures::{
    Cell, Memory, VectorMemory,
    memory_manager::{MemoryId, MemoryManager},
};

fn grant(owner: &str, first: u8, last: u8) -> StaticMemoryRangeDeclaration {
    StaticMemoryRangeDeclaration::new(
        MemoryManagerAuthorityRecord::new(
            MemoryManagerIdRange::new(first, last).unwrap(),
            owner,
            MemoryManagerRangeMode::Allowed,
            None,
        )
        .unwrap(),
    )
    .unwrap()
}

fn request(owner: &str, key: &str) -> MemoryRequest {
    MemoryRequest::new(owner, key, SchemaMetadata::default()).unwrap()
}

fn main() {
    // Standalone: the application explicitly owns the entire nongovernance pool.
    let declarations = SealedDeclarationSnapshot::new(
        &[],
        &[grant("app", 10, 254)],
        &[
            request("app", "app.users.v1"),
            request("app", "app.orders.v1"),
        ],
    )
    .unwrap();
    let mut standalone = MemoryRuntime::new(VectorMemory::default()).unwrap();
    standalone
        .bootstrap(&declarations, &GenericRangePolicy)
        .unwrap();
    let users = standalone.open_memory_by_key("app.users.v1").unwrap();
    users.grow(1);
    users.write(0, b"users");

    // A host's existing persistence owner may seed a reservation before runtime
    // bootstrap. This is the privileged ledger-root operation, not a store open.
    let backing = VectorMemory::default();
    let mut store = LedgerCommitStore::default();
    store
        .commit(&AllocationLedger::new(0, AllocationHistory::default()).unwrap())
        .unwrap();
    AllocationBootstrap::new(&mut store)
        .reserve_and_commit(
            &[AllocationDeclaration::memory_manager_unlabeled("db.journal.v1", 101).unwrap()],
            &GenericRangePolicy,
            None,
        )
        .unwrap();
    {
        let manager = MemoryManager::init(backing.clone());
        let _root = Cell::init(
            manager.get(MemoryId::new(0)),
            StableCellLedgerRecord::new(store),
        );
    }
    // Composed: db gets only 100..119. The fixed host claim and reservation are
    // unavailable to new automatic keys, even if declarations change order.
    let declarations = SealedDeclarationSnapshot::new(
        &[StaticMemoryDeclaration::new(
            "host",
            AllocationDeclaration::memory_manager_unlabeled("host.control.v1", 10).unwrap(),
        )
        .unwrap()],
        &[grant("host", 10, 99), grant("db", 100, 119)],
        &[request("db", "db.rows.v1"), request("db", "db.journal.v1")],
    )
    .unwrap();
    let mut host = MemoryRuntime::new(backing).unwrap();
    host.bootstrap(&declarations, &GenericRangePolicy).unwrap();
    let committed = host.committed_allocations().unwrap();
    assert_eq!(
        committed
            .slot_for(&StableKey::parse("db.journal.v1").unwrap())
            .unwrap()
            .memory_manager_id()
            .unwrap(),
        101
    );
    assert_eq!(
        committed
            .slot_for(&StableKey::parse("db.rows.v1").unwrap())
            .unwrap()
            .memory_manager_id()
            .unwrap(),
        100
    );
    // A library adopts current host authority without running bootstrap again.
    let _journal = host.open_memory_by_key("db.journal.v1").unwrap();
    let _rows = host.open_memory_by_key("db.rows.v1").unwrap();
}
