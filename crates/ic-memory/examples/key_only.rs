//! Run with `cargo run --example key_only`.
use ic_memory::ic_stable_structures::{Memory, VectorMemory};
use ic_memory::{
    GenericAllocationPolicy, MemoryAllocationPool, MemoryAuthority, MemoryRequest, MemoryRuntime,
    SchemaMetadata, SealedDeclarationSnapshot,
};
fn request(owner: &str, key: &str) -> MemoryRequest {
    MemoryRequest::new(owner, key, SchemaMetadata::default()).unwrap()
}
fn main() {
    // The final host owns one pool. Owners receive namespaces, not numeric ranges.
    let pool = MemoryAllocationPool::new(
        vec![
            MemoryAuthority::new("app", "app.").unwrap(),
            MemoryAuthority::new("db", "db.").unwrap(),
        ],
        vec![],
    )
    .unwrap();
    let declarations = SealedDeclarationSnapshot::new(&[
        request("app", "app.control.v1"),
        request("db", "db.journal.v1"),
        request("db", "db.rows.v1"),
    ])
    .unwrap();
    let backing = VectorMemory::default();
    let mut host = MemoryRuntime::new(backing.clone()).unwrap();
    host.bootstrap(&declarations, &pool, &GenericAllocationPolicy)
        .unwrap();
    let id = host.memory_id("db.rows.v1").unwrap();
    let rows = host.open_memory("db.rows.v1").unwrap();
    rows.grow(1).unwrap();
    rows.write(0, b"rows");
    drop(rows);
    drop(host);
    let mut host = MemoryRuntime::new(backing).unwrap();
    host.bootstrap(&declarations, &pool, &GenericAllocationPolicy)
        .unwrap();
    // Library adoption verifies only its requirements and does not bootstrap again.
    let requirements = SealedDeclarationSnapshot::new(&[
        request("db", "db.journal.v1"),
        request("db", "db.rows.v1"),
    ])
    .unwrap();
    host.verify_authority(&requirements, "db").unwrap();
    assert_eq!(host.memory_id("db.rows.v1"), Ok(id));
    let mut bytes = [0; 4];
    host.open_memory("db.rows.v1").unwrap().read(0, &mut bytes);
    assert_eq!(&bytes, b"rows");
}
