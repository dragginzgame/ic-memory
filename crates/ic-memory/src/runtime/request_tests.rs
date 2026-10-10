use super::*;
use crate::{
    AllocationBootstrap, AllocationDeclaration, AllocationLedger, AllocationRetirement,
    GenericAllocationPolicy, LedgerCommitStore, MemoryAllocationPool, MemoryAuthority,
    MemoryManagerIdRange, MemoryManagerSlot, MemoryRequest, SchemaMetadata,
    SealedDeclarationSnapshot, StableCellLedgerRecord,
};
use ic_stable_structures::{
    Cell, Memory, VectorMemory,
    memory_manager::{MemoryId, MemoryManager},
};

pub(super) fn snapshot(keys: &[&str], owner: &str) -> SealedDeclarationSnapshot {
    SealedDeclarationSnapshot::new(
        &keys
            .iter()
            .map(|key| MemoryRequest::new(owner, key, SchemaMetadata::default()).unwrap())
            .collect::<Vec<_>>(),
    )
    .unwrap()
}

pub(super) fn pool() -> MemoryAllocationPool {
    MemoryAllocationPool::new(vec![MemoryAuthority::new("app", "app.").unwrap()], vec![]).unwrap()
}

fn composed_pool(exclusions: Vec<MemoryManagerIdRange>) -> MemoryAllocationPool {
    MemoryAllocationPool::new(
        vec![
            MemoryAuthority::new("canic-core", "canic.").unwrap(),
            MemoryAuthority::new("icydb.main", "icydb.main.").unwrap(),
            MemoryAuthority::new("jobs", "jobs.").unwrap(),
        ],
        exclusions,
    )
    .unwrap()
}

fn composed(keys: &[(&str, &str)]) -> SealedDeclarationSnapshot {
    SealedDeclarationSnapshot::new(
        &keys
            .iter()
            .map(|(owner, key)| MemoryRequest::new(*owner, key, SchemaMetadata::default()).unwrap())
            .collect::<Vec<_>>(),
    )
    .unwrap()
}

/// Seed current-format ownership through its existing persistence owner. This
/// models retained fixed claims, without exposing a numeric component request.
fn retained(backing: &VectorMemory, declarations: &[AllocationDeclaration]) {
    let mut store = LedgerCommitStore::default();
    store
        .commit(&AllocationLedger::new(0, vec![]).unwrap())
        .unwrap();
    let pending = AllocationBootstrap::new(&mut store)
        .validate_and_commit(
            crate::DeclarationSnapshot::new(declarations.to_vec()).unwrap(),
            &GenericAllocationPolicy,
        )
        .unwrap();
    let manager = MemoryManager::init(backing.clone());
    drop(Cell::new(
        manager.get(MemoryId::new(0)),
        StableCellLedgerRecord::new(store),
    ));
    drop(pending.confirm_persisted());
}

#[test]
fn deterministic_requests_preserve_history_and_explicit_inspection() {
    let initial = [
        ("jobs", "jobs.rows.v1"),
        ("canic-core", "canic.control.v1"),
        ("icydb.main", "icydb.main.rows.v1"),
    ];
    let pool = composed_pool(vec![]);
    let backing = VectorMemory::default();
    let mut host = MemoryRuntime::new(backing.clone()).unwrap();
    assert_eq!(
        host.memory_id("jobs.rows.v1"),
        Err(RuntimeOpenError::NotBootstrapped)
    );
    host.bootstrap(&composed(&initial), &pool, &GenericAllocationPolicy)
        .unwrap();
    let ids: Vec<_> = initial
        .iter()
        .map(|(_, key)| (*key, host.memory_id(key).unwrap()))
        .collect();
    for (key, _) in &ids {
        let memory = host.open_memory(key).unwrap();
        memory.grow(1).unwrap();
        memory.write(0, key.as_bytes());
    }
    let mut reversed = initial;
    reversed.reverse();
    let generation = host
        .bootstrap(&composed(&reversed), &pool, &GenericAllocationPolicy)
        .unwrap()
        .generation();
    assert_eq!(generation, 1);
    host.verify_authority(&composed(&[("jobs", "jobs.rows.v1")]), "jobs")
        .unwrap();
    drop(host);

    // Omit IcyDB and add a key that sorts before the old keys. Both its retained
    // mapping and payload remain unavailable to the new key, across cold reopen.
    let changed = composed(&[
        ("canic-core", "canic.control.v1"),
        ("canic-core", "canic.added.v1"),
        ("jobs", "jobs.rows.v1"),
    ]);
    let mut host = MemoryRuntime::new(backing.clone()).unwrap();
    host.bootstrap(&changed, &pool, &GenericAllocationPolicy)
        .unwrap();
    let new_id = host.memory_id("canic.added.v1").unwrap();
    assert!(!ids.iter().any(|(_, id)| *id == new_id));
    assert!(matches!(
        host.open_memory("icydb.main.rows.v1"),
        Err(RuntimeOpenError::StableKeyNotCommitted(_))
    ));
    drop(host);
    let mut reopened = MemoryRuntime::new(backing).unwrap();
    reopened
        .bootstrap(&composed(&initial), &pool, &GenericAllocationPolicy)
        .unwrap();
    for (key, id) in ids {
        assert_eq!(reopened.memory_id(key), Ok(id));
        let mut bytes = vec![0; key.len()];
        reopened.open_memory(key).unwrap().read(0, &mut bytes);
        assert_eq!(bytes, key.as_bytes());
    }
}

#[test]
fn common_pool_supplies_available_space_across_all_owners() {
    // Host exclusions deliberately leave three slots. Each owner can consume
    // any free slot; no component has a numeric quota or private partition.
    let pool = composed_pool(vec![MemoryManagerIdRange::new(13, 254).unwrap()]);
    let declarations = composed(&[
        ("canic-core", "canic.a.v1"),
        ("canic-core", "canic.b.v1"),
        ("icydb.main", "icydb.main.a.v1"),
    ]);
    let backing = VectorMemory::default();
    let mut host = MemoryRuntime::new(backing.clone()).unwrap();
    host.bootstrap(&declarations, &pool, &GenericAllocationPolicy)
        .unwrap();
    assert_eq!(host.memory_id("canic.a.v1"), Ok(10));
    assert_eq!(host.memory_id("canic.b.v1"), Ok(11));
    assert_eq!(host.memory_id("icydb.main.a.v1"), Ok(12));
    drop(host);
    let before = backing.borrow().clone();
    let mut host = MemoryRuntime::new(backing.clone()).unwrap();
    assert!(matches!(
        host.bootstrap(
            &composed(&[("jobs", "jobs.extra.v1")]),
            &pool,
            &GenericAllocationPolicy
        ),
        Err(RuntimeBootstrapError::Resolution(
            MemoryResolutionError::Exhausted { .. }
        ))
    ));
    assert_eq!(*backing.borrow(), before);
    assert!(!host.is_bootstrapped());
}

#[test]
fn retained_fixed_claims_and_tombstones_are_never_remapped_or_reused() {
    let backing = VectorMemory::default();
    retained(
        &backing,
        &[
            AllocationDeclaration::memory_manager_unlabeled("app.retained.v1", 100).unwrap(),
            AllocationDeclaration::memory_manager_unlabeled("app.retired.v1", 10).unwrap(),
        ],
    );
    let manager = MemoryManager::init(backing.clone());
    let memory = manager.get(MemoryId::new(100));
    memory.grow(1);
    memory.write(0, b"retained");
    let mut record =
        super::super::decode_stable_cell_ledger_record_from_memory(&manager.get(MemoryId::new(0)))
            .unwrap();
    AllocationBootstrap::new(record.store_mut())
        .retire_and_commit(
            &AllocationRetirement::new("app.retired.v1", MemoryManagerSlot::new(10).unwrap())
                .unwrap(),
        )
        .unwrap();
    drop(Cell::new(manager.get(MemoryId::new(0)), record));
    drop(manager);
    let mut host = MemoryRuntime::new(backing.clone()).unwrap();
    host.bootstrap(
        &snapshot(&["app.retained.v1", "app.new.v1"], "app"),
        &pool(),
        &GenericAllocationPolicy,
    )
    .unwrap();
    assert_eq!(host.memory_id("app.retained.v1"), Ok(100));
    assert_eq!(host.memory_id("app.new.v1"), Ok(11));
    let mut bytes = [0; 8];
    host.open_memory("app.retained.v1")
        .unwrap()
        .read(0, &mut bytes);
    assert_eq!(&bytes, b"retained");
    drop(host);
    let before = backing.borrow().clone();
    let mut host = MemoryRuntime::new(backing.clone()).unwrap();
    assert!(matches!(
        host.bootstrap(
            &snapshot(&["app.retired.v1"], "app"),
            &pool(),
            &GenericAllocationPolicy
        ),
        Err(RuntimeBootstrapError::Validation(
            crate::AllocationValidationError::RetiredAllocation { .. }
        ))
    ));
    assert_eq!(*backing.borrow(), before);
}

#[test]
fn foreign_namespace_and_excluded_historical_claims_fail_before_commit() {
    let backing = VectorMemory::default();
    retained(
        &backing,
        &[AllocationDeclaration::memory_manager_unlabeled("app.rows.v1", 100).unwrap()],
    );
    let before = backing.borrow().clone();
    let mut host = MemoryRuntime::new(backing.clone()).unwrap();
    assert!(matches!(
        host.bootstrap(
            &snapshot(&["app.rows.v1"], "foreign"),
            &pool(),
            &GenericAllocationPolicy
        ),
        Err(RuntimeBootstrapError::Resolution(
            MemoryResolutionError::Pool(crate::MemoryAllocationPoolError::AuthorityMismatch { .. })
        ))
    ));
    assert_eq!(*backing.borrow(), before);
    let excluded = MemoryAllocationPool::new(
        pool().authorities().to_vec(),
        vec![MemoryManagerIdRange::new(100, 100).unwrap()],
    )
    .unwrap();
    assert!(matches!(
        host.bootstrap(
            &snapshot(&["app.rows.v1"], "app"),
            &excluded,
            &GenericAllocationPolicy
        ),
        Err(RuntimeBootstrapError::Resolution(
            MemoryResolutionError::Pool(crate::MemoryAllocationPoolError::ExcludedSlot { id: 100 })
        ))
    ));
    assert_eq!(*backing.borrow(), before);
}

#[test]
fn unmanaged_physical_custody_requires_explicit_host_exclusion() {
    let backing = VectorMemory::default();
    let manager = MemoryManager::init(backing.clone());
    let raw = manager.get(MemoryId::new(30));
    raw.grow(1);
    raw.write(0, b"unmanaged");
    drop(raw);
    drop(manager);
    let before = backing.borrow().clone();
    let mut host = MemoryRuntime::new(backing.clone()).unwrap();
    let requests = snapshot(&["app.rows.v1"], "app");
    let report = host.doctor_report(&requests, &pool(), &GenericAllocationPolicy);
    assert!(matches!(
        report.validation,
        crate::DiagnosticCheck::Failed { .. }
    ));
    assert_eq!(*backing.borrow(), before);
    assert!(matches!(
        host.bootstrap(&requests, &pool(), &GenericAllocationPolicy),
        Err(RuntimeBootstrapError::Resolution(
            MemoryResolutionError::UnmanagedAllocation { id: 30 }
        ))
    ));
    assert_eq!(*backing.borrow(), before);
    let excluded = MemoryAllocationPool::new(
        pool().authorities().to_vec(),
        vec![MemoryManagerIdRange::new(30, 30).unwrap()],
    )
    .unwrap();
    host.bootstrap(&requests, &excluded, &GenericAllocationPolicy)
        .unwrap();
    assert_eq!(host.memory_id("app.rows.v1"), Ok(10));
    let raw = MemoryManager::init(backing).get(MemoryId::new(30));
    let mut bytes = [0; 9];
    raw.read(0, &mut bytes);
    assert_eq!(&bytes, b"unmanaged");
}

#[test]
fn doctor_resolves_logical_requests_without_writes() {
    let source = snapshot(&["app.rows.v1"], "app");
    let backing = VectorMemory::default();
    let mut host = MemoryRuntime::new(backing.clone()).unwrap();
    for bootstrapped in [false, true] {
        if bootstrapped {
            host.bootstrap(&source, &pool(), &GenericAllocationPolicy)
                .unwrap();
        }
        let before = backing.borrow().clone();
        let report = host.doctor_report(&source, &pool(), &GenericAllocationPolicy);
        assert_eq!(report.validation, crate::DiagnosticCheck::Passed);
        assert_eq!(report.bootstrapped, bootstrapped);
        assert_eq!(*backing.borrow(), before);
    }
}

#[test]
fn current_host_grants_replace_owner_labels_without_moving_keys() {
    let backing = VectorMemory::default();
    let source = snapshot(&["app.rows.v1"], "app");
    let mut original = MemoryRuntime::new(backing.clone()).unwrap();
    original
        .bootstrap(&source, &pool(), &GenericAllocationPolicy)
        .unwrap();
    let id = original.memory_id("app.rows.v1").unwrap();
    let rows = original.open_memory("app.rows.v1").unwrap();
    rows.grow(1).unwrap();
    rows.write(0, b"retained");
    drop(rows);
    drop(original);
    let current = MemoryAllocationPool::new(
        vec![MemoryAuthority::new("replacement", "app.").unwrap()],
        vec![],
    )
    .unwrap();
    let mut reopened = MemoryRuntime::new(backing).unwrap();
    reopened
        .bootstrap(
            &snapshot(&["app.rows.v1"], "replacement"),
            &current,
            &GenericAllocationPolicy,
        )
        .unwrap();
    assert_eq!(reopened.memory_id("app.rows.v1"), Ok(id));
    let mut bytes = [0; 8];
    reopened
        .open_memory("app.rows.v1")
        .unwrap()
        .read(0, &mut bytes);
    assert_eq!(&bytes, b"retained");
}
