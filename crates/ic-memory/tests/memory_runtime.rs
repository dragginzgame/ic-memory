use ic_memory::{
    Memory, MemoryManagerSlot, MemoryRuntime, PolicyIdentity, RuntimeBootstrapPolicy, StableKey,
    VectorMemory,
};
use ic_stable_structures::Cell;

const AUTHORITY: &str = "explicit_runtime";

fn snapshot() -> ic_memory::SealedDeclarationSnapshot {
    ic_memory::SealedDeclarationSnapshot::new(vec![
        ic_memory::MemoryRequest::new(AUTHORITY, "explicit_runtime.rows.v1").unwrap(),
    ])
    .unwrap()
}

struct AllowAll;

impl RuntimeBootstrapPolicy for AllowAll {
    type Error = core::convert::Infallible;

    fn validate_allocation(
        &self,
        _key: &StableKey,
        _slot: &MemoryManagerSlot,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn runtime_bootstrap_identity(&self) -> PolicyIdentity {
        PolicyIdentity::new(*b"explicit-runtime.allow-all......")
    }
}

#[test]
fn public_explicit_runtime_bootstraps_opens_and_diagnoses_its_memory() {
    let declarations = snapshot();
    let mut runtime = MemoryRuntime::new(VectorMemory::default()).expect("empty backing memory");

    let generation = runtime
        .bootstrap(&declarations, &pool(), &AllowAll)
        .expect("runtime bootstrap");
    let rows = runtime
        .open_memory("explicit_runtime.rows.v1")
        .expect("committed rows memory");
    let mut cell = Cell::init(rows, 7_u64);
    cell.set(9);
    assert_eq!(*cell.get(), 9);

    let export = runtime.diagnostic_export().expect("runtime diagnostics");
    assert_eq!(export.current_generation, generation);
    assert!(
        runtime
            .doctor_report(&declarations, &pool(), &AllowAll)
            .bootstrapped
    );
}

#[test]
fn owned_report_and_detached_lease_support_borrowed_nonclone_backing() {
    struct BorrowedMemory<'a>(&'a VectorMemory);
    impl Memory for BorrowedMemory<'_> {
        fn size(&self) -> u64 {
            self.0.size()
        }
        fn grow(&self, pages: u64) -> i64 {
            self.0.grow(pages)
        }
        fn read(&self, offset: u64, dst: &mut [u8]) {
            self.0.read(offset, dst);
        }
        fn write(&self, offset: u64, src: &[u8]) {
            self.0.write(offset, src);
        }
    }
    let memory = VectorMemory::default();
    let mut runtime = MemoryRuntime::new_with_config(
        BorrowedMemory(&memory),
        ic_memory::MemoryManagerConfig::new(8).unwrap(),
    )
    .unwrap();
    runtime.bootstrap(&snapshot(), &pool(), &AllowAll).unwrap();
    let rows = runtime.open_memory("explicit_runtime.rows.v1").unwrap();
    rows.grow(1).unwrap();
    rows.write(0, &[7]);
    let report = runtime.memory_allocations().unwrap();
    let before = memory.borrow().clone();
    assert_eq!(runtime.diagnostic_export().unwrap().current_generation, 1);
    assert!(
        runtime
            .doctor_report(&snapshot(), &pool(), &AllowAll)
            .bootstrapped
    );
    assert_eq!(*memory.borrow(), before);
    drop(runtime);
    let mut value = [0];
    rows.read(0, &mut value);
    assert_eq!(value, [7]);
    assert_eq!(report.memories[10].virtual_extent.wasm_pages, 1);
    assert_eq!(report.summary.bucket_size_pages, 8);
}

#[test]
fn live_lease_survives_warm_bootstrap_and_collection_transfer() {
    let backing = VectorMemory::default();
    let declarations = snapshot();
    let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
    let committed = runtime
        .bootstrap(&declarations, &pool(), &AllowAll)
        .unwrap();
    let rows = runtime.open_memory("explicit_runtime.rows.v1").unwrap();
    let mut cell = Cell::new(rows, 7_u64);
    let before = backing.borrow().clone();
    let assert_leased = |runtime: &MemoryRuntime<VectorMemory>| {
        assert!(matches!(
            runtime.open_memory("explicit_runtime.rows.v1"),
            Err(ic_memory::RuntimeOpenError::AlreadyOpen { id: 10 })
        ));
    };
    assert_leased(&runtime);
    runtime.verify_authority(&declarations, AUTHORITY).unwrap();
    assert_eq!(
        runtime
            .bootstrap(&declarations, &pool(), &AllowAll)
            .unwrap(),
        committed
    );
    assert_leased(&runtime);
    assert_eq!(*backing.borrow(), before);
    cell.set(9);
    let memory = cell.into_memory();
    assert_leased(&runtime);
    drop(memory);
    let reopened = Cell::init(
        runtime.open_memory("explicit_runtime.rows.v1").unwrap(),
        0_u64,
    );
    assert_eq!(*reopened.get(), 9);
    assert_leased(&runtime);
    drop(reopened);
    drop(runtime.open_memory("explicit_runtime.rows.v1").unwrap());
}

#[test]
fn failed_collection_construction_and_unwinding_release_the_lease() {
    let mut runtime = MemoryRuntime::new(VectorMemory::default()).unwrap();
    runtime.bootstrap(&snapshot(), &pool(), &AllowAll).unwrap();
    let memory = runtime.open_memory("explicit_runtime.rows.v1").unwrap();
    memory.grow(1).unwrap();
    memory.write(0, b"bad");
    let refused = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
        ic_stable_structures::StableVec::<u64, _>::init(memory)
    }));
    assert!(refused.is_err());
    let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _cell = Cell::new(
            runtime.open_memory("explicit_runtime.rows.v1").unwrap(),
            13_u64,
        );
        panic!("injected collection owner failure");
    }));
    assert!(failure.is_err());
    let reopened = Cell::init(
        runtime.open_memory("explicit_runtime.rows.v1").unwrap(),
        0_u64,
    );
    assert_eq!(*reopened.get(), 13);
}

#[test]
fn all_application_slots_have_independent_leases() {
    let keys: Vec<_> = (0..245)
        .map(|id| format!("explicit_runtime.store{id:03}.v1"))
        .collect();
    let declarations = ic_memory::SealedDeclarationSnapshot::new(
        keys.iter()
            .map(|key| ic_memory::MemoryRequest::new(AUTHORITY, key).unwrap())
            .collect(),
    )
    .unwrap();
    let mut runtime = MemoryRuntime::new(VectorMemory::default()).unwrap();
    runtime
        .bootstrap(&declarations, &pool(), &AllowAll)
        .unwrap();
    let mut leases: Vec<_> = keys
        .iter()
        .map(|key| Some(runtime.open_memory(key).unwrap()))
        .collect();
    for (key, lease) in keys.iter().zip(&mut leases) {
        assert!(matches!(
            runtime.open_memory(key),
            Err(ic_memory::RuntimeOpenError::AlreadyOpen { .. })
        ));
        drop(lease.take());
        let replacement = runtime.open_memory(key).unwrap();
        assert!(matches!(
            runtime.open_memory(key),
            Err(ic_memory::RuntimeOpenError::AlreadyOpen { .. })
        ));
        *lease = Some(replacement);
    }
    drop(leases);
    for key in &keys {
        drop(runtime.open_memory(key).unwrap());
    }
    // A two-memory collection must release its first lease if the second open
    // refuses, and must retain both distinct leases after successful creation.
    let held_data = runtime.open_memory(&keys[1]).unwrap();
    let refused = runtime.open_memory(&keys[0]).and_then(|index| {
        runtime
            .open_memory(&keys[1])
            .map(|data| ic_stable_structures::StableLog::<u64, _, _>::new(index, data))
    });
    assert!(matches!(
        refused,
        Err(ic_memory::RuntimeOpenError::AlreadyOpen { id: 11 })
    ));
    drop(held_data);
    let log = ic_stable_structures::StableLog::<u64, _, _>::new(
        runtime.open_memory(&keys[0]).unwrap(),
        runtime.open_memory(&keys[1]).unwrap(),
    );
    log.append(&23).unwrap();
    assert_eq!(log.get(0), Some(23));
    for key in &keys[..2] {
        assert!(matches!(
            runtime.open_memory(key),
            Err(ic_memory::RuntimeOpenError::AlreadyOpen { .. })
        ));
    }
    drop(log);
    for key in &keys[..2] {
        drop(runtime.open_memory(key).unwrap());
    }
}

fn pool() -> ic_memory::MemoryAllocationPool {
    ic_memory::MemoryAllocationPool::new(
        vec![ic_memory::MemoryAuthority::new("explicit_runtime", "explicit_runtime.").unwrap()],
        vec![],
    )
    .unwrap()
}
