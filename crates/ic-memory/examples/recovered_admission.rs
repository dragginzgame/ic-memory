//! Retain an explicitly selected known allocation across a cold bootstrap.
//! Applications own key meaning and any checks on the retained contents.
use ic_memory::{
    BootstrapAdmission, MemoryManagerSlot, MemoryRequest, MemoryRuntime, PolicyIdentity,
    RuntimeBootstrapPolicy, SealedDeclarationSnapshot, StableKey,
};
use ic_memory::{Memory, VectorMemory};

const CURRENT: &str = "component.current.v1";
const RETAINED: &str = "component.retained.v1";

struct HostPolicy;
impl RuntimeBootstrapPolicy for HostPolicy {
    type Error = &'static str;
    fn validate_allocation(
        &self,
        _key: &StableKey,
        _slot: &MemoryManagerSlot,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
    fn runtime_bootstrap_identity(&self) -> PolicyIdentity {
        PolicyIdentity::new(*b"example.known-retention.v1......")
    }
    fn prepare_bootstrap(&self, admission: &mut BootstrapAdmission<'_>) -> Result<(), Self::Error> {
        let key = StableKey::parse(RETAINED).unwrap();
        if !admission.is_declared(&key) {
            admission
                .include_historical("component", RETAINED)
                .map_err(|_| "known allocation retention rejected")?;
        }
        Ok(())
    }
}

fn declarations(keys: &[&str]) -> SealedDeclarationSnapshot {
    SealedDeclarationSnapshot::new(
        keys.iter()
            .map(|key| MemoryRequest::new("component", key).unwrap())
            .collect(),
    )
    .unwrap()
}

fn main() {
    let backing = VectorMemory::default();
    let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
    runtime
        .bootstrap(&declarations(&[CURRENT, RETAINED]), &pool(), &HostPolicy)
        .unwrap();
    let retained = runtime.open_memory(RETAINED).unwrap();
    retained.grow(1).unwrap();
    retained.write(0, b"kept");
    let id = runtime.memory_id(RETAINED).unwrap();
    drop(retained);
    drop(runtime);

    let mut runtime = MemoryRuntime::new(backing).unwrap();
    let current = declarations(&[CURRENT, "component.added.v1"]);
    assert_eq!(
        runtime.bootstrap(&current, &pool(), &HostPolicy).unwrap(),
        2
    );
    assert_eq!(runtime.memory_id(RETAINED).unwrap(), id);
    let mut marker = [0; 4];
    runtime.open_memory(RETAINED).unwrap().read(0, &mut marker);
    assert_eq!(&marker, b"kept");
    assert_eq!(
        runtime.bootstrap(&current, &pool(), &HostPolicy).unwrap(),
        2
    );
}

fn pool() -> ic_memory::MemoryAllocationPool {
    ic_memory::MemoryAllocationPool::new(
        vec![ic_memory::MemoryAuthority::new("component", "component.").unwrap()],
        vec![],
    )
    .unwrap()
}
