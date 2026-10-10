use ic_memory::{
    AllocationPolicy, MemoryManagerSlot, MemoryRuntime, PolicyIdentity, PolicyIdentityError,
    RuntimeBootstrapPolicy, StableKey,
    ic_stable_structures::{Cell, Memory, VectorMemory},
    sealed_declaration_snapshot,
};

const AUTHORITY: &str = "explicit_runtime";

ic_memory::ic_memory_declaration!(authority = AUTHORITY, key = "explicit_runtime.rows.v1");

struct AllowAll;

impl AllocationPolicy for AllowAll {
    type Error = core::convert::Infallible;

    fn validate_key(&self, _key: &StableKey) -> Result<(), Self::Error> {
        Ok(())
    }

    fn validate_slot(
        &self,
        _key: &StableKey,
        _slot: &MemoryManagerSlot,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn validate_reserved_slot(
        &self,
        _key: &StableKey,
        _slot: &MemoryManagerSlot,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl RuntimeBootstrapPolicy for AllowAll {
    fn runtime_bootstrap_identity(&self) -> Result<PolicyIdentity, PolicyIdentityError> {
        PolicyIdentity::new("explicit-runtime.allow-all", 1)
    }
}

#[test]
fn public_explicit_runtime_bootstraps_opens_and_diagnoses_its_memory() {
    let declarations = sealed_declaration_snapshot().expect("linked declarations");
    let mut runtime = MemoryRuntime::new(VectorMemory::default()).expect("empty backing memory");

    let generation = runtime
        .bootstrap(&declarations, &pool(), &AllowAll)
        .expect("runtime bootstrap")
        .generation();
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
fn owned_report_and_cloned_handles_support_borrowed_nonclone_backing() {
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
    runtime
        .bootstrap(&sealed_declaration_snapshot().unwrap(), &pool(), &AllowAll)
        .unwrap();
    let rows = runtime.open_memory("explicit_runtime.rows.v1").unwrap();
    rows.grow(1).unwrap();
    rows.write(0, &[7]);
    let clone = rows.clone();
    let report = runtime.memory_allocations().unwrap();
    let before = memory.borrow().clone();
    assert_eq!(runtime.diagnostic_export().unwrap().current_generation, 1);
    assert!(
        runtime
            .doctor_report(&sealed_declaration_snapshot().unwrap(), &pool(), &AllowAll)
            .bootstrapped
    );
    assert_eq!(*memory.borrow(), before);
    drop(runtime);
    drop(rows);
    let mut value = [0];
    clone.read(0, &mut value);
    assert_eq!(value, [7]);
    assert_eq!(report.memories[10].virtual_extent.wasm_pages, 1);
    assert_eq!(report.bucket_size_pages, 8);
}

fn pool() -> ic_memory::MemoryAllocationPool {
    ic_memory::MemoryAllocationPool::new(
        vec![ic_memory::MemoryAuthority::new("explicit_runtime", "explicit_runtime.").unwrap()],
        vec![],
    )
    .unwrap()
}
