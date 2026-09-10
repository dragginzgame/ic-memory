use super::{
    MemoryRuntime, RuntimeConstructionError, RuntimeDiagnosticError, RuntimeOpenError,
    RuntimeStateError,
    default::{is_default_memory_manager_bootstrapped, with_default_runtime_borrowed},
    diagnostics::diagnostic_validation_ledger,
    policy::NoopPolicy,
};
use crate::{
    AllocationHistory, AllocationLedger, AllocationPolicy, AllocationRecord,
    AllocationSlotDescriptor, DiagnosticCheck, DiagnosticCode, DiagnosticMemorySize,
    DiagnosticMemorySizeOutcome, LedgerCommitError, LedgerPayloadEnvelopeError, PolicyIdentity,
    PolicyIdentityError, RuntimeBootstrapPolicy, StableKey,
    registry::{
        SealedDeclarationSnapshot, TEST_REGISTRY_LOCK, register_static_memory_manager_declaration,
        register_static_memory_manager_range, reset_static_memory_declarations_for_tests,
        sealed_declaration_snapshot,
    },
};
use ic_stable_structures::{Memory, VectorMemory};
use std::convert::Infallible;

pub(super) fn declarations() -> SealedDeclarationSnapshot {
    reset_static_memory_declarations_for_tests();
    register_static_memory_manager_range(
        120,
        120,
        "runtime_tests",
        crate::MemoryManagerRangeMode::Reserved,
        None,
    )
    .expect("test range");
    register_static_memory_manager_declaration(
        120,
        "runtime_tests",
        "rows",
        "runtime_tests.rows.v1",
    )
    .expect("test declaration");
    sealed_declaration_snapshot().expect("sealed declarations")
}

fn empty_runtime() -> MemoryRuntime<VectorMemory> {
    MemoryRuntime::new(VectorMemory::default()).expect("empty backing memory")
}

const WASM_PAGE_SIZE_BYTES: usize = 65_536;

fn one_page_backing(prefix: [u8; 4]) -> (VectorMemory, Vec<u8>) {
    let memory = VectorMemory::default();
    assert_eq!(memory.grow(1), 0);
    let mut bytes = vec![0xA5; WASM_PAGE_SIZE_BYTES];
    bytes[..prefix.len()].copy_from_slice(&prefix);
    memory.write(0, &bytes);
    (memory, bytes)
}

fn read_one_page(memory: &VectorMemory) -> Vec<u8> {
    assert_eq!(memory.size(), 1);
    let mut bytes = vec![0; WASM_PAGE_SIZE_BYTES];
    memory.read(0, &mut bytes);
    bytes
}

struct CountingPolicy(std::cell::Cell<usize>);

impl AllocationPolicy for CountingPolicy {
    type Error = Infallible;

    fn validate_key(&self, _key: &StableKey) -> Result<(), Self::Error> {
        self.0.set(self.0.get() + 1);
        Ok(())
    }

    fn validate_slot(
        &self,
        _key: &StableKey,
        _slot: &AllocationSlotDescriptor,
    ) -> Result<(), Self::Error> {
        self.0.set(self.0.get() + 1);
        Ok(())
    }

    fn validate_reserved_slot(
        &self,
        _key: &StableKey,
        _slot: &AllocationSlotDescriptor,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl RuntimeBootstrapPolicy for CountingPolicy {
    fn runtime_bootstrap_identity(&self) -> Result<PolicyIdentity, PolicyIdentityError> {
        PolicyIdentity::new("runtime-tests.counting-policy", 1)
    }
}

struct IdentityPolicy {
    name: &'static str,
    version: u32,
    configuration_digest: Option<[u8; 32]>,
}

impl IdentityPolicy {
    const fn new(name: &'static str, version: u32) -> Self {
        Self {
            name,
            version,
            configuration_digest: None,
        }
    }

    const fn configured(name: &'static str, version: u32, configuration_digest: [u8; 32]) -> Self {
        Self {
            name,
            version,
            configuration_digest: Some(configuration_digest),
        }
    }
}

impl AllocationPolicy for IdentityPolicy {
    type Error = Infallible;

    fn validate_key(&self, _key: &StableKey) -> Result<(), Self::Error> {
        Ok(())
    }

    fn validate_slot(
        &self,
        _key: &StableKey,
        _slot: &AllocationSlotDescriptor,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn validate_reserved_slot(
        &self,
        _key: &StableKey,
        _slot: &AllocationSlotDescriptor,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl RuntimeBootstrapPolicy for IdentityPolicy {
    fn runtime_bootstrap_identity(&self) -> Result<PolicyIdentity, PolicyIdentityError> {
        let identity = PolicyIdentity::new(self.name, self.version)?;
        Ok(match self.configuration_digest {
            Some(digest) => identity.with_configuration_digest(digest),
            None => identity,
        })
    }
}

#[test]
fn construction_initializes_empty_memory_and_accepts_current_manager_memory() {
    let backing = VectorMemory::default();
    let runtime =
        MemoryRuntime::new(backing.clone()).expect("empty backing memory should initialize");
    assert_eq!(&read_one_page(&backing)[..4], b"MGR\x01");

    drop(runtime);
    let recovered =
        MemoryRuntime::new(backing).expect("current MemoryManager backing should be accepted");
    assert!(!recovered.is_bootstrapped());
}

#[test]
fn construction_rejects_foreign_nonempty_memory_without_writing() {
    let (backing, original) = one_page_backing(*b"DATA");

    let Err(error) = MemoryRuntime::new(backing.clone()) else {
        panic!("foreign backing memory must be rejected");
    };

    assert_eq!(
        error,
        RuntimeConstructionError::ForeignMemory {
            observed_magic: *b"DAT",
        }
    );
    assert_eq!(read_one_page(&backing), original);
}

#[test]
fn construction_rejects_unsupported_manager_version_without_writing() {
    let (backing, original) = one_page_backing(*b"MGR\x02");

    let Err(error) = MemoryRuntime::new(backing.clone()) else {
        panic!("unsupported MemoryManager backing must be rejected");
    };

    assert_eq!(
        error,
        RuntimeConstructionError::UnsupportedMemoryManagerVersion {
            observed: 2,
            supported: 1,
        }
    );
    assert_eq!(read_one_page(&backing), original);
}

#[test]
fn separate_runtimes_have_independent_bootstrap_authority_and_memory() {
    let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock");
    let declarations = declarations();
    let mut runtime_a = empty_runtime();
    let mut runtime_b = empty_runtime();
    let policy = CountingPolicy(std::cell::Cell::new(0));

    runtime_a
        .bootstrap(&declarations, &policy)
        .expect("runtime A bootstrap");
    assert_eq!(policy.0.get(), 2);
    assert!(runtime_a.is_bootstrapped());
    assert!(!runtime_b.is_bootstrapped());
    assert_eq!(
        runtime_b.committed_allocations().expect_err("runtime B"),
        RuntimeOpenError::NotBootstrapped
    );

    let memory_a = runtime_a
        .open_memory("runtime_tests.rows.v1", 120)
        .expect("runtime A memory");
    memory_a.grow(1);
    memory_a.write(0, b"runtime-a");

    runtime_b
        .bootstrap(&declarations, &policy)
        .expect("runtime B bootstrap");
    assert_eq!(policy.0.get(), 4);
    let memory_b = runtime_b
        .open_memory("runtime_tests.rows.v1", 120)
        .expect("runtime B memory");
    assert_eq!(memory_b.size(), 0);
    memory_b.grow(1);
    let mut bytes = [0; 9];
    memory_b.read(0, &mut bytes);
    assert_eq!(&bytes, &[0; 9]);
    assert_eq!(
        runtime_a
            .diagnostic_export()
            .expect("runtime A diagnostics")
            .current_generation,
        1
    );
    assert_eq!(
        runtime_b
            .diagnostic_export()
            .expect("runtime B diagnostics")
            .current_generation,
        1
    );
}

#[test]
fn concurrent_independent_runtimes_do_not_share_bootstrap_state() {
    let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock");
    let declarations = declarations();
    let first_declarations = declarations.clone();
    let second_declarations = declarations;

    let (first, second) = std::thread::scope(|scope| {
        let first = scope.spawn(move || {
            let mut runtime = empty_runtime();
            let generation = runtime
                .bootstrap(&first_declarations, &NoopPolicy)
                .expect("first bootstrap")
                .generation();
            let diagnostic_generation = runtime
                .diagnostic_export()
                .expect("first diagnostics")
                .current_generation;
            (generation, diagnostic_generation)
        });
        let second = scope.spawn(move || {
            let mut runtime = empty_runtime();
            let generation = runtime
                .bootstrap(&second_declarations, &NoopPolicy)
                .expect("second bootstrap")
                .generation();
            let diagnostic_generation = runtime
                .diagnostic_export()
                .expect("second diagnostics")
                .current_generation;
            (generation, diagnostic_generation)
        });
        (
            first.join().expect("first runtime thread"),
            second.join().expect("second runtime thread"),
        )
    });

    assert_eq!(first, (1, 1));
    assert_eq!(second, (1, 1));
}

#[test]
fn repeated_bootstrap_is_idempotent_and_existing_memory_recovers() {
    let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock");
    let declarations = declarations();
    let backing = VectorMemory::default();
    let generation = {
        let mut runtime = MemoryRuntime::new(backing.clone()).expect("empty backing memory");
        let first = runtime
            .bootstrap(&declarations, &NoopPolicy)
            .expect("first bootstrap")
            .generation();
        let second = runtime
            .bootstrap(&declarations, &NoopPolicy)
            .expect("idempotent bootstrap")
            .generation();
        assert_eq!(first, second);
        let memory = runtime
            .open_memory("runtime_tests.rows.v1", 120)
            .expect("first runtime memory");
        memory.grow(1);
        memory.write(0, b"persisted");
        first
    };

    let mut recovered_runtime =
        MemoryRuntime::new(backing).expect("existing MemoryManager backing memory");
    let recovered_generation = recovered_runtime
        .bootstrap(&declarations, &NoopPolicy)
        .expect("recover existing backing memory")
        .generation();
    assert!(recovered_generation >= generation);
    assert_eq!(
        recovered_runtime
            .diagnostic_export()
            .expect("runtime diagnostic")
            .current_generation,
        recovered_generation
    );
    let memory = recovered_runtime
        .open_memory("runtime_tests.rows.v1", 120)
        .expect("recovered runtime memory");
    let mut bytes = [0; 9];
    memory.read(0, &mut bytes);
    assert_eq!(&bytes, b"persisted");
}

#[test]
fn repeated_bootstrap_is_bound_to_declarations_and_policy_identity() {
    let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock");
    let declarations = declarations();
    let mut runtime = empty_runtime();

    let generation = runtime
        .bootstrap(
            &declarations,
            &IdentityPolicy::new("runtime-tests.identity-policy", 1),
        )
        .expect("first bootstrap")
        .generation();
    let repeated_generation = runtime
        .bootstrap(
            &declarations,
            &IdentityPolicy::new("runtime-tests.identity-policy", 1),
        )
        .expect("same semantic policy identity")
        .generation();
    assert_eq!(repeated_generation, generation);

    let policy_error = runtime
        .bootstrap(
            &declarations,
            &IdentityPolicy::new("runtime-tests.identity-policy", 2),
        )
        .expect_err("changed policy identity");
    assert!(matches!(
        policy_error,
        super::RuntimeBootstrapError::PolicyIdentityMismatch { .. }
    ));

    reset_static_memory_declarations_for_tests();
    register_static_memory_manager_range(
        121,
        121,
        "alternate_runtime_tests",
        crate::MemoryManagerRangeMode::Reserved,
        None,
    )
    .expect("alternate range");
    register_static_memory_manager_declaration(
        121,
        "alternate_runtime_tests",
        "rows",
        "alternate_runtime_tests.rows.v1",
    )
    .expect("alternate declaration");
    let alternate_declarations =
        sealed_declaration_snapshot().expect("alternate sealed declarations");
    let declaration_error = runtime
        .bootstrap(
            &alternate_declarations,
            &IdentityPolicy::new("runtime-tests.identity-policy", 1),
        )
        .expect_err("changed declaration snapshot");
    assert!(matches!(
        declaration_error,
        super::RuntimeBootstrapError::DeclarationSnapshotMismatch
    ));
    assert_eq!(
        runtime
            .diagnostic_export()
            .expect("unchanged ledger")
            .current_generation,
        generation
    );
}

#[test]
fn policy_identity_validation_and_configuration_digest_are_runtime_bound() {
    let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock");
    let declarations = declarations();
    let mut empty_identity_runtime = empty_runtime();
    let empty_identity_error = empty_identity_runtime
        .bootstrap(&declarations, &IdentityPolicy::new("", 1))
        .expect_err("empty policy identity");
    assert!(matches!(
        empty_identity_error,
        super::RuntimeBootstrapError::PolicyIdentity(PolicyIdentityError::EmptyName)
    ));
    assert!(!empty_identity_runtime.is_bootstrapped());
    let invalid_identity_doctor =
        empty_identity_runtime.doctor_report(&declarations, &IdentityPolicy::new("", 1));
    assert!(matches!(
        invalid_identity_doctor.tested_policy_identity,
        Err(crate::DiagnosticFailure {
            code: DiagnosticCode::PolicyIdentity,
            ..
        })
    ));
    assert!(matches!(
        invalid_identity_doctor.validation,
        DiagnosticCheck::NotRun {
            code: DiagnosticCode::PolicyIdentity,
            ..
        }
    ));

    let mut configured_runtime = empty_runtime();
    configured_runtime
        .bootstrap(
            &declarations,
            &IdentityPolicy::configured("runtime-tests.configured-policy", 1, [1; 32]),
        )
        .expect("configured policy bootstrap");
    let digest_mismatch = configured_runtime
        .bootstrap(
            &declarations,
            &IdentityPolicy::configured("runtime-tests.configured-policy", 1, [2; 32]),
        )
        .expect_err("configuration digest is part of identity");
    assert!(matches!(
        digest_mismatch,
        super::RuntimeBootstrapError::PolicyIdentityMismatch { .. }
    ));
}

struct RejectPolicy;

impl AllocationPolicy for RejectPolicy {
    type Error = &'static str;

    fn validate_key(&self, _key: &StableKey) -> Result<(), Self::Error> {
        Err("rejected")
    }

    fn validate_slot(
        &self,
        _key: &StableKey,
        _slot: &AllocationSlotDescriptor,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn validate_reserved_slot(
        &self,
        _key: &StableKey,
        _slot: &AllocationSlotDescriptor,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl RuntimeBootstrapPolicy for RejectPolicy {
    fn runtime_bootstrap_identity(&self) -> Result<PolicyIdentity, PolicyIdentityError> {
        PolicyIdentity::new("runtime-tests.reject-policy", 1)
    }
}

#[test]
fn open_errors_and_failed_bootstrap_do_not_publish_authority() {
    let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock");
    let declarations = declarations();
    let mut runtime = empty_runtime();

    let Err(open_before_bootstrap) = runtime.open_memory("runtime_tests.rows.v1", 120) else {
        panic!("open before bootstrap must fail");
    };
    assert_eq!(open_before_bootstrap, RuntimeOpenError::NotBootstrapped);
    assert!(runtime.bootstrap(&declarations, &RejectPolicy).is_err());
    assert!(!runtime.is_bootstrapped());
    let rejected_doctor = runtime.doctor_report(&declarations, &RejectPolicy);
    assert!(!rejected_doctor.bootstrapped);
    assert!(matches!(
        rejected_doctor.validation,
        DiagnosticCheck::Failed {
            code: DiagnosticCode::AllocationValidation,
            ..
        }
    ));
    assert!(matches!(
        runtime.diagnostic_export(),
        Err(RuntimeDiagnosticError::NotBootstrapped)
    ));
    assert_eq!(
        runtime.committed_allocations().expect_err("no capability"),
        RuntimeOpenError::NotBootstrapped
    );

    runtime
        .bootstrap(&declarations, &NoopPolicy)
        .expect("successful retry");
    let Err(wrong_key) = runtime.open_memory("runtime_tests.missing.v1", 120) else {
        panic!("wrong key must fail");
    };
    assert!(matches!(
        wrong_key,
        RuntimeOpenError::StableKeyNotCommitted(_)
    ));
    let Err(wrong_id) = runtime.open_memory("runtime_tests.rows.v1", 121) else {
        panic!("wrong ID must fail");
    };
    assert!(matches!(
        wrong_id,
        RuntimeOpenError::MemoryIdMismatch { .. }
    ));
}

#[test]
fn doctor_and_diagnostics_report_the_same_runtime_lifecycle() {
    let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock");
    let declarations = declarations();
    let mut runtime = empty_runtime();

    assert!(
        !runtime
            .doctor_report(&declarations, &NoopPolicy)
            .bootstrapped
    );
    assert!(matches!(
        runtime.diagnostic_export(),
        Err(RuntimeDiagnosticError::NotBootstrapped)
    ));

    runtime
        .bootstrap(&declarations, &NoopPolicy)
        .expect("bootstrap");
    let memory = runtime
        .open_memory("runtime_tests.rows.v1", 120)
        .expect("open");
    memory.grow(2);
    let doctor = runtime.doctor_report(&declarations, &NoopPolicy);
    let export = runtime.diagnostic_export().expect("diagnostic export");
    assert!(doctor.bootstrapped);
    assert_eq!(
        doctor
            .ledger
            .as_ref()
            .expect("doctor ledger")
            .current_generation,
        export.current_generation
    );
    assert_eq!(
        export
            .records
            .iter()
            .find(|record| { record.allocation.stable_key().as_str() == "runtime_tests.rows.v1" })
            .expect("runtime record")
            .memory_size,
        Some(DiagnosticMemorySizeOutcome::Measured(
            DiagnosticMemorySize::from_wasm_pages(2)
        ))
    );
    assert!(matches!(&doctor.bootstrap_binding, DiagnosticCheck::Passed));
    assert_eq!(
        doctor
            .tested_policy_identity
            .as_ref()
            .expect("valid tested identity"),
        &PolicyIdentity::new("ic-memory.noop-policy", 1).expect("valid identity")
    );
    let established = doctor
        .established_bootstrap_binding
        .as_ref()
        .expect("established runtime binding");
    assert_eq!(
        established.policy_identity,
        PolicyIdentity::new("ic-memory.noop-policy", 1).expect("valid identity")
    );
    assert_eq!(
        established.declaration_fingerprint,
        declarations.fingerprint()
    );
    let doctor_bytes = crate::test_cbor::to_vec(&doctor).expect("doctor diagnostic bytes");
    let decoded_doctor: crate::MemoryRuntimeDoctorReport =
        crate::test_cbor::from_slice(&doctor_bytes).expect("doctor diagnostic round trip");
    assert_eq!(decoded_doctor, doctor);

    let mismatched = runtime.doctor_report(
        &declarations,
        &IdentityPolicy::new("runtime-tests.identity-policy", 2),
    );
    assert!(matches!(
        mismatched.bootstrap_binding,
        DiagnosticCheck::Failed {
            code: DiagnosticCode::RuntimeBinding,
            ..
        }
    ));
    assert!(matches!(mismatched.validation, DiagnosticCheck::Passed));
}

#[test]
fn memory_size_diagnostics_preserve_success_when_another_slot_fails() {
    let users = crate::AllocationDeclaration::memory_manager("size.users.v1", 100, "users")
        .expect("users declaration");
    let orders = crate::AllocationDeclaration::memory_manager("size.orders.v1", 101, "orders")
        .expect("orders declaration");
    let mut ledger = AllocationLedger {
        current_generation: 1,
        allocation_history: AllocationHistory::from_parts(
            vec![
                AllocationRecord::active(1, users).expect("users record"),
                AllocationRecord::active(1, orders).expect("orders record"),
            ],
            Vec::new(),
        ),
    };
    ledger.allocation_history.records_mut()[1].slot =
        AllocationSlotDescriptor::memory_manager_unchecked(crate::MEMORY_MANAGER_INVALID_ID);

    let outcomes = empty_runtime().memory_size_outcomes(&ledger);

    assert!(matches!(
        &outcomes[0].1,
        DiagnosticMemorySizeOutcome::Measured(_)
    ));
    assert!(matches!(
        &outcomes[1].1,
        DiagnosticMemorySizeOutcome::Failed(crate::DiagnosticFailure {
            code: DiagnosticCode::MemorySize,
            ..
        })
    ));
}

#[test]
fn default_runtime_reentry_is_a_typed_state_error() {
    with_default_runtime_borrowed(|| {
        assert_eq!(
            is_default_memory_manager_bootstrapped().expect_err("re-entry"),
            RuntimeStateError::ReentrantAccess
        );
        Ok(())
    })
    .expect("test borrow");
}

#[test]
fn validation_diagnostic_preserves_unsupported_format_code() {
    let recovered = Err(LedgerCommitError::PayloadEnvelope(
        LedgerPayloadEnvelopeError::UnsupportedFormat {
            marker: *b"ICMF",
            version: Some(crate::LEDGER_PAYLOAD_FORMAT_VERSION + 1),
        },
    ));

    let failure = diagnostic_validation_ledger(None, Some(&recovered))
        .expect_err("unsupported format must block validation");

    assert_eq!(failure.code, DiagnosticCode::UnsupportedFormat);
    assert!(
        failure
            .message
            .contains("unsupported ic-memory ledger payload format")
    );
}
