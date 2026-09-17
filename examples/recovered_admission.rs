//! Allocation-level IcyDB-shaped reconciliation. No control-store read is needed
//! to complete declarations; real journal/commit safety remains consumer-owned.
use ic_memory::ic_stable_structures::{Memory, VectorMemory};
use ic_memory::{
    AllocationPolicy, AllocationSlotDescriptor, AllocationState, BootstrapAdmission,
    MemoryManagerAuthorityRecord, MemoryManagerIdRange, MemoryManagerRangeMode, MemoryRequest,
    MemoryRuntime, PolicyIdentity, PolicyIdentityError, RuntimeBootstrapPolicy, SchemaMetadata,
    SealedDeclarationSnapshot, StableKey, StaticMemoryRangeDeclaration,
};

const CONTROL: &str = "db.main.control.v1";
const OLD_JOURNAL: &str = "db.main.old.journal.v1";

struct HostPolicy;
impl AllocationPolicy for HostPolicy {
    type Error = &'static str;
    fn validate_key(&self, _: &StableKey) -> Result<(), Self::Error> {
        Ok(())
    }
    fn validate_slot(
        &self,
        _: &StableKey,
        _: &AllocationSlotDescriptor,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
    fn validate_reserved_slot(
        &self,
        _: &StableKey,
        _: &AllocationSlotDescriptor,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}
impl RuntimeBootstrapPolicy for HostPolicy {
    fn runtime_bootstrap_identity(&self) -> Result<PolicyIdentity, PolicyIdentityError> {
        PolicyIdentity::new("example.host-with-db-admission", 1)
    }
    fn prepare_bootstrap(&self, admission: &mut BootstrapAdmission<'_>) -> Result<(), Self::Error> {
        prepare_database(admission)
    }
}

// A generated library contributes this function to its host's policy. The role
// grammar here is illustrative, not a claim about IcyDB's integration format.
fn prepare_database(admission: &mut BootstrapAdmission<'_>) -> Result<(), &'static str> {
    let mut omitted = Vec::new();
    for record in admission.recovered_allocations() {
        let key = record.stable_key.as_str();
        if key.starts_with("db.")
            && key.ends_with(".control.v1")
            && (key != CONTROL || !admission.is_declared(record.stable_key))
        {
            return Err(
                "database control identity replacement requires an explicit consumer decision",
            );
        }
        if key.starts_with("db.main.")
            && key.ends_with(".journal.v1")
            && !matches!(record.state, AllocationState::Retired { .. })
            && !admission.is_declared(record.stable_key)
        {
            omitted.push(record.stable_key.clone());
        }
    }
    for key in omitted {
        admission
            .include_historical("db", key.as_str())
            .map_err(|_| "historical journal selection rejected")?;
    }
    Ok(())
}

fn declarations(keys: &[&str]) -> SealedDeclarationSnapshot {
    let grant = StaticMemoryRangeDeclaration::new(
        MemoryManagerAuthorityRecord::new(
            MemoryManagerIdRange::new(100, 110).unwrap(),
            "db",
            MemoryManagerRangeMode::Allowed,
            None,
        )
        .unwrap(),
    )
    .unwrap();
    let requests: Vec<_> = keys
        .iter()
        .map(|key| MemoryRequest::new("db", key, SchemaMetadata::default()).unwrap())
        .collect();
    SealedDeclarationSnapshot::new(&[], &[grant], &requests).unwrap()
}

fn main() {
    let backing = VectorMemory::default();
    let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
    runtime
        .bootstrap(&declarations(&[CONTROL, OLD_JOURNAL]), &HostPolicy)
        .unwrap();
    let journal = runtime.open_memory_by_key(OLD_JOURNAL).unwrap();
    journal.grow(1);
    journal.write(0, b"debt");
    drop(journal);
    drop(runtime);

    let mut runtime = MemoryRuntime::new(backing).unwrap();
    let current = declarations(&[CONTROL, "db.main.new.journal.v1"]);
    let committed = runtime.bootstrap(&current, &HostPolicy).unwrap();
    assert_eq!(committed.generation(), 2); // One commit, including discovered journals.
    let journals: Vec<_> = committed
        .declarations()
        .iter()
        .filter(|d| d.stable_key().as_str().ends_with(".journal.v1"))
        .map(|d| d.stable_key().clone())
        .collect();
    for key in journals {
        let journal = runtime.open_memory_by_key(key.as_str()).unwrap();
        if journal.size() != 0 {
            let mut marker = [0; 4];
            journal.read(0, &mut marker);
            assert_eq!(&marker, b"debt");
            // Consumer must refuse retirement while debt or commit markers remain.
        }
    }
    assert_eq!(
        runtime
            .bootstrap(&current, &HostPolicy)
            .unwrap()
            .generation(),
        2
    );
}
