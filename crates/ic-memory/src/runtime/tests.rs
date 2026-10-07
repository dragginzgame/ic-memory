use super::{
    MemoryRuntime, RuntimeConstructionError, RuntimeDiagnosticError, RuntimeOpenError,
    RuntimeStateError,
    default::{is_default_memory_manager_bootstrapped, with_default_runtime_borrowed},
    diagnostics::diagnostic_validation_ledger,
    policy::GenericRangePolicy,
};
use crate::{
    AllocationPolicy, DiagnosticCheck, DiagnosticCode, DiagnosticMemorySize, LedgerCommitError,
    LedgerPayloadEnvelopeError, MemoryManagerSlot, PolicyIdentity, PolicyIdentityError,
    RuntimeBootstrapPolicy, StableKey,
    registry::{
        SealedDeclarationSnapshot, TEST_REGISTRY_LOCK, register_static_memory_manager_declaration,
        register_static_memory_manager_range, reset_static_memory_declarations_for_tests,
        sealed_declaration_snapshot,
    },
};
use ic_stable_structures::{Cell, Memory, VectorMemory};
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
        _slot: &MemoryManagerSlot,
    ) -> Result<(), Self::Error> {
        self.0.set(self.0.get() + 1);
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

impl RuntimeBootstrapPolicy for CountingPolicy {
    fn runtime_bootstrap_identity(&self) -> Result<PolicyIdentity, PolicyIdentityError> {
        PolicyIdentity::new("runtime-tests.counting-policy", 1)
    }
}

#[test]
fn fixed_range_checks_preserve_custom_policy_admission_and_rejection_order() {
    use crate::{
        AllocationValidationError, MemoryManagerAuthorityRecord, MemoryManagerIdRange,
        MemoryManagerRangeAuthorityError, MemoryManagerRangeMode, RuntimeBootstrapError,
        RuntimePolicyError, StaticMemoryDeclaration, StaticMemoryRangeDeclaration,
    };

    let mismatch = |id, actual: &str| MemoryManagerRangeAuthorityError::AuthorityMismatch {
        id,
        expected_authority: "app".to_string(),
        actual_authority: actual.to_string(),
    };
    for (authority, id, expected) in [
        (None, 100, None),
        (Some("app"), 100, None),
        (None, 1, Some(mismatch(1, crate::IC_MEMORY_AUTHORITY_OWNER))),
        (Some("foreign"), 100, Some(mismatch(100, "foreign"))),
        (
            Some("app"),
            101,
            Some(MemoryManagerRangeAuthorityError::UnclaimedId { id: 101 }),
        ),
    ] {
        let ranges: Vec<_> = authority
            .map(|authority| {
                StaticMemoryRangeDeclaration::new(
                    MemoryManagerAuthorityRecord::new(
                        MemoryManagerIdRange::new(100, 100).unwrap(),
                        authority,
                        MemoryManagerRangeMode::Allowed,
                        None,
                    )
                    .unwrap(),
                )
                .unwrap()
            })
            .into_iter()
            .collect();
        let registration = StaticMemoryDeclaration::new(
            "app",
            crate::AllocationDeclaration::memory_manager_unlabeled("app.rows.v1", id).unwrap(),
        )
        .unwrap();
        let declarations = SealedDeclarationSnapshot::new(&[registration], &ranges, &[]).unwrap();
        let policy = CountingPolicy(std::cell::Cell::new(0));
        let mut runtime = empty_runtime();
        match (expected, runtime.bootstrap(&declarations, &policy)) {
            (None, Ok(_)) => {
                assert_eq!(runtime.memory_id("app.rows.v1").unwrap(), id);
                assert_eq!(policy.0.get(), 2);
            }
            (
                Some(expected),
                Err(RuntimeBootstrapError::Validation(AllocationValidationError::Policy(
                    RuntimePolicyError::Range(actual),
                ))),
            ) => {
                assert_eq!(actual, expected);
                assert!(!runtime.is_bootstrapped());
                // Key policy runs first; the range failure precedes slot policy.
                assert_eq!(policy.0.get(), 1);
            }
            outcome => panic!("unexpected range-policy outcome: {outcome:?}"),
        }
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
fn construction_rejects_caller_pregrown_blank_memory_without_writing() {
    for configured in [false, true] {
        let backing = VectorMemory::default();
        assert_eq!(backing.grow(1), 0);
        let original = read_one_page(&backing);
        let result = if configured {
            MemoryRuntime::new_with_config(
                backing.clone(),
                super::MemoryManagerConfig::new(16).unwrap(),
            )
        } else {
            MemoryRuntime::new(backing.clone())
        };
        let Err(error) = result else {
            panic!("caller-pregrown blank memory must remain foreign input");
        };
        assert_eq!(
            error,
            RuntimeConstructionError::ForeignMemory {
                observed_magic: [0; 3]
            }
        );
        assert_eq!(read_one_page(&backing), original);
    }
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
    memory_a.grow(1).unwrap();
    memory_a.write(0, b"runtime-a");

    runtime_b
        .bootstrap(&declarations, &policy)
        .expect("runtime B bootstrap");
    assert_eq!(policy.0.get(), 4);
    let memory_b = runtime_b
        .open_memory("runtime_tests.rows.v1", 120)
        .expect("runtime B memory");
    assert_eq!(memory_b.size(), 0);
    memory_b.grow(1).unwrap();
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
                .bootstrap(&first_declarations, &GenericRangePolicy)
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
                .bootstrap(&second_declarations, &GenericRangePolicy)
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
            .bootstrap(&declarations, &GenericRangePolicy)
            .expect("first bootstrap")
            .generation();
        let second = runtime
            .bootstrap(&declarations, &GenericRangePolicy)
            .expect("idempotent bootstrap")
            .generation();
        assert_eq!(first, second);
        let memory = runtime
            .open_memory("runtime_tests.rows.v1", 120)
            .expect("first runtime memory");
        memory.grow(1).unwrap();
        memory.write(0, b"persisted");
        first
    };

    let mut recovered_runtime =
        MemoryRuntime::new(backing).expect("existing MemoryManager backing memory");
    let recovered_generation = recovered_runtime
        .bootstrap(&declarations, &GenericRangePolicy)
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
fn repeated_bootstrap_accepts_independently_sealed_equivalent_declarations() {
    let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock");
    let declarations = declarations();
    let equivalent = SealedDeclarationSnapshot::new(
        declarations.registered_declarations(),
        declarations.registered_ranges(),
        declarations.requests(),
    )
    .expect("equivalent declarations");
    assert_eq!(equivalent, declarations);
    assert!(!declarations.shares_storage_with(&equivalent));

    let policy = CountingPolicy(std::cell::Cell::new(0));
    let mut runtime = empty_runtime();
    let generation = runtime
        .bootstrap(&declarations, &policy)
        .expect("bootstrap")
        .generation();
    assert!(matches!(
        runtime
            .doctor_report(&equivalent, &policy)
            .bootstrap_binding,
        DiagnosticCheck::Passed
    ));
    let policy_calls = policy.0.get();
    let ledger_before = runtime.ledger_record_from_memory().expect("ledger before");

    assert_eq!(
        runtime
            .bootstrap(&equivalent, &policy)
            .expect("equivalent bootstrap")
            .generation(),
        generation
    );
    assert_eq!(policy.0.get(), policy_calls);
    assert_eq!(
        runtime.ledger_record_from_memory().expect("ledger after"),
        ledger_before
    );
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
        .bootstrap(&declarations, &GenericRangePolicy)
        .expect("successful retry");
    let Err(wrong_key) = runtime.open_memory("runtime_tests.missing.v1", 120) else {
        panic!("wrong key must fail");
    };
    assert!(matches!(
        wrong_key,
        RuntimeOpenError::StableKeyNotCommitted(_)
    ));
    for requested_id in [121, crate::MEMORY_MANAGER_INVALID_ID] {
        let Err(wrong_id) = runtime.open_memory("runtime_tests.rows.v1", requested_id) else {
            panic!("wrong ID must fail");
        };
        assert_eq!(
            wrong_id,
            RuntimeOpenError::MemoryIdMismatch {
                stable_key: "runtime_tests.rows.v1".to_string(),
                committed_id: 120,
                requested_id,
            }
        );
    }
}

#[test]
fn doctor_and_diagnostics_report_the_same_runtime_lifecycle() {
    let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock");
    let declarations = declarations();
    let mut runtime = empty_runtime();

    assert!(
        !runtime
            .doctor_report(&declarations, &GenericRangePolicy)
            .bootstrapped
    );
    assert!(matches!(
        runtime.diagnostic_export(),
        Err(RuntimeDiagnosticError::NotBootstrapped)
    ));

    runtime
        .bootstrap(&declarations, &GenericRangePolicy)
        .expect("bootstrap");
    let memory = runtime
        .open_memory("runtime_tests.rows.v1", 120)
        .expect("open");
    memory.grow(2).unwrap();
    let doctor = runtime.doctor_report(&declarations, &GenericRangePolicy);
    let export = runtime.diagnostic_export().expect("diagnostic export");
    assert!(doctor.bootstrapped);
    assert_eq!(doctor.ledger.as_ref().expect("doctor ledger"), &export);
    assert!(doctor.commit_recovery.is_some());
    assert_eq!(doctor.commit_recovery, export.commit_recovery);
    assert_eq!(
        export
            .records
            .iter()
            .find(|record| { record.allocation.stable_key().as_str() == "runtime_tests.rows.v1" })
            .expect("runtime record")
            .memory_size,
        Some(DiagnosticMemorySize::from_wasm_pages(2))
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
fn diagnostics_reject_invalid_persisted_slots_before_measuring_sizes() {
    let declarations = SealedDeclarationSnapshot::new(&[], &[], &[]).unwrap();
    let backing = VectorMemory::default();
    let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
    runtime
        .bootstrap(&declarations, &GenericRangePolicy)
        .unwrap();
    let record = runtime.ledger_record_from_memory().unwrap();
    let ledger = record.store().recover().unwrap().into_ledger();
    let mut value = serde_json::to_value(&ledger).unwrap();
    value["records"][0]["slot"]["slot"]["MemoryManagerId"] = serde_json::json!(255);
    let payload = crate::LedgerPayloadEnvelope::current(crate::test_cbor::to_vec(&value).unwrap())
        .try_encode()
        .unwrap();
    // Keep physical framing/checksums valid so recovery reaches slot validation.
    let store = serde_json::from_value(serde_json::json!({
        "physical": {
            "slot0": crate::CommittedGenerationBytes::new(ledger.current_generation(), payload),
            "slot1": null,
        },
    }))
    .unwrap();
    let _cell = Cell::new(
        runtime.memory(crate::MEMORY_MANAGER_LEDGER_ID),
        crate::StableCellLedgerRecord::new(store),
    );
    let before = backing.borrow().clone();

    assert!(matches!(
        runtime.diagnostic_export(),
        Err(RuntimeDiagnosticError::LedgerCommit(
            LedgerCommitError::Codec(_)
        ))
    ));
    let doctor = runtime.doctor_report(&declarations, &GenericRangePolicy);
    assert!(doctor.ledger.is_none());
    assert!(matches!(
        doctor.validation,
        DiagnosticCheck::NotRun {
            code: DiagnosticCode::LedgerRecovery,
            ..
        }
    ));
    let mut reopened = MemoryRuntime::new(backing.clone()).unwrap();
    assert!(matches!(
        reopened.bootstrap(&declarations, &GenericRangePolicy),
        Err(super::RuntimeBootstrapError::LedgerCommit(
            LedgerCommitError::Codec(_)
        ))
    ));
    assert!(!reopened.is_bootstrapped());
    assert_eq!(*backing.borrow(), before);
}

#[test]
fn default_runtime_reentry_is_a_typed_state_error() {
    with_default_runtime_borrowed(|| {
        assert_eq!(
            is_default_memory_manager_bootstrapped().expect_err("re-entry"),
            RuntimeStateError::ReentrantAccess
        );
        assert_eq!(
            super::committed_allocations().expect_err("re-entry"),
            RuntimeOpenError::State(RuntimeStateError::ReentrantAccess)
        );
        Ok(())
    })
    .expect("test borrow");
}

#[test]
fn doctor_preserves_distinct_record_decode_causes_without_writes() {
    let declarations = SealedDeclarationSnapshot::new(&[], &[], &[]).unwrap();
    let mut messages = Vec::new();
    for bytes in [vec![0xff], vec![0xa0]] {
        let cause = crate::decode_stable_cell_ledger_record(&bytes)
            .unwrap_err()
            .to_string();
        let backing = VectorMemory::default();
        let runtime = MemoryRuntime::new(backing.clone()).unwrap();
        let memory = runtime.memory(crate::MEMORY_MANAGER_LEDGER_ID);
        memory.grow(1).unwrap();
        memory.write(0, crate::STABLE_CELL_MAGIC);
        memory.write(3, &[crate::STABLE_CELL_LAYOUT_VERSION]);
        memory.write(4, &u32::try_from(bytes.len()).unwrap().to_le_bytes());
        memory.write(crate::STABLE_CELL_VALUE_OFFSET, &bytes);
        let before = backing.borrow().clone();

        let report = runtime.doctor_report(&declarations, &GenericRangePolicy);
        assert!(report.commit_recovery.is_none());
        assert!(report.ledger.is_none());
        let crate::DiagnosticStableCellStatus::Corrupt { failure } = report.stable_cell.status
        else {
            panic!("malformed record must be reported as corrupt");
        };
        assert_eq!(failure.code, DiagnosticCode::StableCell);
        assert!(failure.message.contains(&cause));
        messages.push(failure.message);
        assert_eq!(*backing.borrow(), before);
    }
    assert_ne!(messages[0], messages[1]);
}

#[test]
fn doctor_uses_genesis_only_for_empty_commit_storage() {
    let declarations = SealedDeclarationSnapshot::new(&[], &[], &[]).unwrap();
    let backing = VectorMemory::default();
    let runtime = MemoryRuntime::new(backing.clone()).unwrap();
    for initialized in [false, true] {
        if initialized {
            let _cell = Cell::new(
                runtime.memory(crate::MEMORY_MANAGER_LEDGER_ID),
                crate::StableCellLedgerRecord::default(),
            );
        }
        let before = backing.borrow().clone();
        let report = runtime.doctor_report(&declarations, &GenericRangePolicy);
        assert!(!report.bootstrapped);
        assert_eq!(
            report.stable_cell.status,
            if initialized {
                crate::DiagnosticStableCellStatus::Readable
            } else {
                crate::DiagnosticStableCellStatus::Empty
            }
        );
        assert!(matches!(report.validation, DiagnosticCheck::Passed));
        assert!(report.commit_recovery.is_some());
        assert!(report.ledger.is_none());
        assert_eq!(*backing.borrow(), before);
    }

    // A present invalid physical slot must never be treated as empty storage.
    let mut record = runtime.ledger_record_from_memory().unwrap();
    let genesis = crate::AllocationLedger::new(0, Vec::new()).unwrap();
    record
        .store_mut()
        .write_corrupt_inactive_ledger(&genesis)
        .unwrap();
    let _cell = Cell::new(runtime.memory(crate::MEMORY_MANAGER_LEDGER_ID), record);
    let before = backing.borrow().clone();
    let report = runtime.doctor_report(&declarations, &GenericRangePolicy);
    assert!(matches!(
        report.stable_cell.status,
        crate::DiagnosticStableCellStatus::Readable
    ));
    assert!(matches!(
        report.validation,
        DiagnosticCheck::NotRun {
            code: DiagnosticCode::LedgerRecovery,
            ..
        }
    ));
    assert!(report.ledger.is_none());
    assert!(!runtime.is_bootstrapped());
    assert!(report.commit_recovery.is_some());
    assert_eq!(*backing.borrow(), before);
}

#[test]
fn validation_diagnostic_preserves_unsupported_format_code() {
    let recovered = Err(LedgerCommitError::PayloadEnvelope(
        LedgerPayloadEnvelopeError::UnsupportedFormat {
            marker: *b"ICMS",
            version: Some(crate::LEDGER_PAYLOAD_FORMAT_VERSION + 1),
        },
    ));

    let failure = diagnostic_validation_ledger(Some(&recovered))
        .expect_err("unsupported format must block validation");

    assert_eq!(failure.code, DiagnosticCode::UnsupportedFormat);
}
