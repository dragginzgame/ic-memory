//! Run with `cargo run --example composed_host`.
//! Its public-API regressions also run in the ordinary `cargo test` suite.
use ic_memory::ic_stable_structures::{Memory, VectorMemory};
use ic_memory::{
    AllocationPolicy, BootstrapAdmission, MemoryManagerConfig, MemoryManagerSlot, MemoryRequest,
    MemoryRuntime, PolicyIdentity, PolicyIdentityError, RuntimeBootstrapError,
    RuntimeBootstrapPolicy, RuntimeOpenError, SchemaMetadata, SealedDeclarationSnapshot, StableKey,
};
use std::cell::Cell;

const HOST: &str = "host.control.v1";
const CONTROL: &str = "db.main.control.v1";
const ROWS: &str = "db.main.rows.v1";

ic_memory::ic_memory_range!(authority = "host", start = 10, end = 99, mode = Allowed);
ic_memory::ic_memory_range!(authority = "db", start = 100, end = 110, mode = Allowed);
ic_memory::ic_memory_declaration!(
    authority = "host",
    key = "host.control.v1",
    label = "host control",
    id = 10,
);
ic_memory::ic_memory_declaration!(authority = "db", key = "db.main.control.v1");
ic_memory::ic_memory_declaration!(authority = "db", key = "db.main.rows.v1");

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
enum ConsumerRejection {
    #[error("database control replacement requires a consumer decision")]
    ControlReplacement,
}

#[derive(Default)]
struct HostPolicy {
    preparations: Cell<usize>,
    database_preparations: Cell<usize>,
}

impl AllocationPolicy for HostPolicy {
    type Error = ConsumerRejection;

    fn validate_key(&self, _: &StableKey) -> Result<(), Self::Error> {
        Ok(())
    }

    fn validate_slot(&self, _: &StableKey, _: &MemoryManagerSlot) -> Result<(), Self::Error> {
        Ok(())
    }

    fn validate_reserved_slot(
        &self,
        _: &StableKey,
        _: &MemoryManagerSlot,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl RuntimeBootstrapPolicy for HostPolicy {
    fn runtime_bootstrap_identity(&self) -> Result<PolicyIdentity, PolicyIdentityError> {
        PolicyIdentity::new("example.composed-host", 1)
    }

    fn prepare_bootstrap(&self, admission: &mut BootstrapAdmission<'_>) -> Result<(), Self::Error> {
        self.preparations.set(self.preparations.get() + 1);
        prepare_database(admission, &self.database_preparations)
    }
}

// A library contributes admission to the host's single policy. This illustrative
// identity rule uses allocation metadata only; real database readiness, journal
// debt and pending-commit checks remain consumer responsibilities after opening.
fn prepare_database(
    admission: &BootstrapAdmission<'_>,
    preparations: &Cell<usize>,
) -> Result<(), ConsumerRejection> {
    preparations.set(preparations.get() + 1);
    for record in admission.recovered_allocations() {
        if record.stable_key.as_str().starts_with("db.")
            && record.stable_key.as_str().ends_with(".control.v1")
            && !admission.is_declared(record.stable_key)
        {
            return Err(ConsumerRejection::ControlReplacement);
        }
    }
    Ok(())
}

fn database_requirements() -> SealedDeclarationSnapshot {
    let requests = [CONTROL, ROWS]
        .map(|key| MemoryRequest::new("db", key, SchemaMetadata::default()).unwrap());
    // Consumer requirements do not replace the host's grants or policy.
    SealedDeclarationSnapshot::new(&[], &[], &requests).unwrap()
}

fn cold_reopens() {
    let backing = VectorMemory::default();
    let declarations = ic_memory::sealed_declaration_snapshot().unwrap();
    let requirements = database_requirements();
    let config = MemoryManagerConfig::new(16).unwrap();
    let policy = HostPolicy::default();
    {
        let mut host = MemoryRuntime::new_with_config(backing.clone(), config).unwrap();
        host.bootstrap(&declarations, &policy).unwrap();
        host.verify_authority(&requirements, "db").unwrap();
        for (key, marker) in [(HOST, b"host"), (CONTROL, b"ctrl"), (ROWS, b"rows")] {
            let memory = host.open_memory_by_key(key).unwrap();
            memory.grow(1).unwrap();
            memory.write(0, marker);
        }
    }

    for reopen in 1..=2 {
        let before = backing.borrow().clone();
        // A disallowed control replacement fails before commitment. A subsequent
        // cold attempt still sees the prior IDs and data under the same geometry.
        let changed = SealedDeclarationSnapshot::new(
            declarations.registered_declarations(),
            declarations.registered_ranges(),
            &[
                MemoryRequest::new("db", "db.replacement.control.v1", SchemaMetadata::default())
                    .unwrap(),
                MemoryRequest::new("db", ROWS, SchemaMetadata::default()).unwrap(),
            ],
        )
        .unwrap();
        {
            let mut rejected = MemoryRuntime::new_with_config(backing.clone(), config).unwrap();
            assert!(matches!(
                rejected.bootstrap(&changed, &policy),
                Err(RuntimeBootstrapError::AdmissionPolicy(
                    ConsumerRejection::ControlReplacement
                ))
            ));
            assert!(!rejected.is_bootstrapped());
            assert_eq!(
                rejected.committed_allocations(),
                Err(RuntimeOpenError::NotBootstrapped)
            );
            assert!(matches!(
                rejected.open_memory_by_key(ROWS),
                Err(RuntimeOpenError::NotBootstrapped)
            ));
            assert_eq!(*backing.borrow(), before);
        }

        let mut host = MemoryRuntime::new_with_config(backing.clone(), config).unwrap();
        let committed = host.bootstrap(&declarations, &policy).unwrap().clone();
        assert_eq!(committed.generation(), reopen + 1);
        let before_adoption = backing.borrow().clone();
        for _ in 0..2 {
            host.verify_authority(&declarations, "host").unwrap();
            host.verify_authority(&requirements, "db").unwrap();
            for (key, id, marker) in [
                (HOST, 10, b"host"),
                (CONTROL, 100, b"ctrl"),
                (ROWS, 101, b"rows"),
            ] {
                assert_eq!(host.memory_id(key), Ok(id));
                let memory = host.open_memory_by_key(key).unwrap();
                assert_eq!(memory.size(), 1);
                let mut retained = [0; 4];
                memory.read(0, &mut retained);
                assert_eq!(&retained, marker);
            }
        }
        assert_eq!(host.bootstrap(&declarations, &policy).unwrap(), &committed);
        assert_eq!(host.memory_manager_config(), config);
        assert_eq!(*backing.borrow(), before_adoption);
        assert_eq!(
            policy.preparations.get(),
            1 + 2 * usize::try_from(reopen).unwrap()
        );
        assert_eq!(
            policy.database_preparations.get(),
            policy.preparations.get()
        );
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn native_threads() {
    // The declaration registry is program-wide; the native default runtime and
    // its backing are thread-local. Every worker bootstraps the host before
    // library adoption or any thread-local stable-store initializer is touched.
    let workers: Vec<_> = (0..2)
        .map(|_| {
            std::thread::spawn(|| {
                let policy = HostPolicy::default();
                let requirements = database_requirements();
                assert_eq!(
                    ic_memory::verify_default_memory_manager_authority(&requirements, "db"),
                    Err(ic_memory::RuntimeAdoptionError::Open(
                        RuntimeOpenError::NotBootstrapped
                    ))
                );
                ic_memory::bootstrap_default_memory_manager_with_config(
                    MemoryManagerConfig::new(16).unwrap(),
                    &policy,
                )
                .unwrap();
                ic_memory::verify_default_memory_manager_authority(&requirements, "db").unwrap();
                let rows = ic_memory::open_default_memory_manager_memory_by_key(ROWS).unwrap();
                rows.grow(1).unwrap();
                rows.write(0, b"rows");
                let mut retained = [0; 4];
                rows.read(0, &mut retained);
                assert_eq!(&retained, b"rows");
                assert_eq!(ic_memory::default_memory_manager_memory_id(ROWS), Ok(101));
                assert_eq!(
                    ic_memory::default_memory_manager_memory_allocation_summary()
                        .unwrap()
                        .bucket_size_pages,
                    16
                );
                assert_eq!(policy.preparations.get(), 1);
                assert_eq!(policy.database_preparations.get(), 1);
            })
        })
        .collect();
    for worker in workers {
        worker.join().unwrap();
    }
}

fn main() {
    cold_reopens();
    #[cfg(not(target_arch = "wasm32"))]
    native_threads();
}

#[test]
fn composed_host_retains_authority_and_data_through_two_cold_reopens() {
    cold_reopens();
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn every_native_worker_bootstraps_host_before_consumer_adoption() {
    native_threads();
}
