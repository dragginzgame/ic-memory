use super::request_tests::{pool, snapshot};
use super::*;
use crate::{
    BootstrapAdmissionError as AdmissionError, MemoryDeclarationError as DeclarationError,
    MemoryManagerSlot, RuntimeBootstrapPolicy, StableKey,
};
use ic_stable_structures::VectorMemory;
use std::cell::Cell as Counter;

#[derive(Default)]
pub(super) struct AdmissionPolicy {
    pub calls: Counter<usize>,
    pub selections: Vec<(&'static str, &'static str)>,
    pub discover: bool,
    pub reject: bool,
    pub deny_journals: bool,
}

impl RuntimeBootstrapPolicy for AdmissionPolicy {
    type Error = &'static str;
    fn validate_allocation(
        &self,
        key: &StableKey,
        _slot: &MemoryManagerSlot,
    ) -> Result<(), Self::Error> {
        if self.deny_journals && key.as_str().ends_with(".journal.v1") {
            Err("revoked")
        } else {
            Ok(())
        }
    }

    fn runtime_bootstrap_identity(&self) -> PolicyIdentity {
        PolicyIdentity::new(*b"test.admission..................")
    }
    fn prepare_bootstrap(&self, admission: &mut BootstrapAdmission<'_>) -> Result<(), Self::Error> {
        self.calls.set(self.calls.get() + 1);
        if self.reject {
            return Err("identity rejected");
        }
        let mut selected = Vec::new();
        if self.discover {
            for record in admission.recovered_allocations() {
                if record.stable_key().as_str().ends_with(".control.v1")
                    && !admission.is_declared(record.stable_key())
                {
                    return Err("identity rejected");
                }
                if record.stable_key().as_str().ends_with(".journal.v1")
                    && !admission.is_declared(record.stable_key())
                {
                    selected.push(record.stable_key().clone());
                }
            }
        }
        for key in selected {
            let _ = admission.include_historical("app", key.as_str());
        }
        // Deliberately ignore returned errors: the attempt must still fail closed.
        for (authority, key) in &self.selections {
            let _ = admission.include_historical(authority, key);
        }
        Ok(())
    }
}

const CONTROL: &str = "app.main.control.v1";
const JOURNAL: &str = "app.old.journal.v1";

#[test]
fn full_pool_resolution_combines_current_and_selected_history_without_reassignment() {
    let keys: Vec<_> = (0..245)
        .map(|i| format!("app.store{i:03}.journal.v1"))
        .collect();
    let backing = VectorMemory::default();
    let reversed: Vec<_> = keys.iter().rev().map(String::as_str).collect();
    let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
    runtime
        .bootstrap(
            &snapshot(&reversed, "app"),
            &pool(),
            &GenericAllocationPolicy,
        )
        .unwrap();
    drop(runtime);

    // Alternate direct requirements with historical completion across the entire
    // application pool. Both paths must retain the same recovered placement.
    let current: Vec<_> = keys.iter().step_by(2).map(String::as_str).collect();
    let policy = AdmissionPolicy {
        discover: true,
        ..Default::default()
    };
    let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
    let committed = runtime
        .bootstrap(&snapshot(&current, "app"), &pool(), &policy)
        .unwrap();
    assert_eq!(committed, 2);
    assert_eq!(runtime.committed_allocations().unwrap().len(), 245);
    for (index, key) in keys.iter().enumerate() {
        assert_eq!(
            runtime.memory_id(key),
            Ok(u8::try_from(index + 10).unwrap())
        );
    }
    drop(runtime);

    let before = backing.borrow().clone();
    let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
    assert!(matches!(
        runtime.bootstrap(&snapshot(&["app.new.v1"], "app"), &pool(), &policy),
        Err(RuntimeBootstrapError::Resolution(
            super::MemoryResolutionError::Exhausted { .. }
        ))
    ));
    assert_eq!(*backing.borrow(), before);
    assert!(!runtime.is_bootstrapped());
}

#[test]
fn declared_membership_covers_original_inputs_and_retains_poisoned_selections() {
    let runtime = MemoryRuntime::new(seeded()).unwrap();
    let recovered = runtime
        .ledger_record_from_memory()
        .unwrap()
        .recover()
        .unwrap();
    // Requests arrive in deliberately noncanonical key order.
    let original = snapshot(
        &["app.z.v1", "app.a.v1", "app.fixed.v1", "zoo.fixed.v1"],
        "app",
    );
    let host_pool = pool();
    let mut admission = BootstrapAdmission::new(recovered.ledger(), &original, &host_pool);
    for name in [
        crate::IC_MEMORY_LEDGER_STABLE_KEY,
        "app.a.v1",
        "app.z.v1",
        "app.fixed.v1",
        "zoo.fixed.v1",
    ] {
        assert!(admission.is_declared(&StableKey::parse(name).unwrap()));
    }
    for name in [CONTROL, JOURNAL, "ic_memory.other.v1", "app.unknown.v1"] {
        assert!(!admission.is_declared(&StableKey::parse(name).unwrap()));
    }
    // Selection order is deliberately descending; it is independent of sealing.
    admission.include_historical("app", JOURNAL).unwrap();
    admission.include_historical("app", CONTROL).unwrap();
    let failure = admission
        .include_historical("app", "app.unknown.v1")
        .unwrap_err();
    assert!(matches!(failure, AdmissionError::Unknown(_)));
    for name in [JOURNAL, CONTROL] {
        assert!(admission.is_declared(&StableKey::parse(name).unwrap()));
    }
    assert!(!admission.is_declared(&StableKey::parse("app.unknown.v1").unwrap()));
    assert_eq!(admission.complete().unwrap_err(), failure);
}

pub(super) fn seeded() -> VectorMemory {
    let backing = VectorMemory::default();
    let mut runtime =
        MemoryRuntime::new_with_config(backing.clone(), MemoryManagerConfig::new(1).unwrap())
            .unwrap();
    runtime
        .bootstrap(
            &snapshot(&[CONTROL, JOURNAL], "app"),
            &pool(),
            &GenericAllocationPolicy,
        )
        .unwrap();
    let journal = runtime.open_memory(JOURNAL).unwrap();
    journal.grow(1).unwrap();
    journal.write(0, b"pending/debt");
    backing
}

#[test]
fn discovers_omitted_journal_before_one_commit_and_skips_warm_admission() {
    let backing = seeded();
    let current = snapshot(&[CONTROL, "app.new.journal.v1"], "app");
    let policy = AdmissionPolicy {
        discover: true,
        ..Default::default()
    };
    let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
    assert!(matches!(
        runtime.open_memory(JOURNAL),
        Err(RuntimeOpenError::NotBootstrapped)
    ));
    assert_eq!(runtime.bootstrap(&current, &pool(), &policy).unwrap(), 2);
    let mut marker = [0; 12];
    runtime.open_memory(JOURNAL).unwrap().read(0, &mut marker);
    assert_eq!(&marker, b"pending/debt");
    assert_eq!(runtime.memory_id(JOURNAL).unwrap(), 11);
    let before = backing.borrow().clone();
    assert_eq!(runtime.bootstrap(&current, &pool(), &policy).unwrap(), 2);
    let _ = runtime.doctor_report(&current, &pool(), &policy);
    assert_eq!(policy.calls.get(), 1);
    assert_eq!(runtime.memory_manager_config().bucket_size_pages(), 1);
    assert_eq!(*backing.borrow(), before);
    assert!(matches!(
        runtime.bootstrap(&current, &pool(), &GenericAllocationPolicy),
        Err(RuntimeBootstrapError::PolicyIdentityMismatch { .. })
    ));
}

#[test]
fn historical_request_validation_precedes_selection_and_poisons_the_attempt() {
    let backing = seeded();
    let before = backing.borrow().clone();
    let runtime = MemoryRuntime::new(backing.clone()).unwrap();
    let recovered = runtime
        .ledger_record_from_memory()
        .unwrap()
        .recover()
        .unwrap();
    let original = snapshot(&[CONTROL], "app");
    let host_pool = pool();
    for (authority, key, kind) in [
        ("", "not a key", "authority"),
        ("ic-memory", "not a key", "reserved authority"),
        ("app", "not a key", "key"),
        ("app", crate::IC_MEMORY_LEDGER_STABLE_KEY, "reserved key"),
    ] {
        let mut admission = BootstrapAdmission::new(recovered.ledger(), &original, &host_pool);
        let error = admission.include_historical(authority, key).unwrap_err();
        let AdmissionError::Declarations(ref declaration_error) = error else {
            panic!("unexpected selection error: {error:?}");
        };
        assert!(matches!(
            (kind, declaration_error),
            ("authority", DeclarationError::InvalidAuthority { .. })
                | (
                    "reserved authority",
                    DeclarationError::ReservedAuthority { .. }
                )
                | ("key", DeclarationError::Key(_))
                | ("reserved key", DeclarationError::ReservedStableKey { .. })
        ));
        assert_eq!(
            *declaration_error,
            crate::MemoryRequest::new(authority, key).unwrap_err()
        );
        assert_eq!(
            admission.include_historical("app", JOURNAL).unwrap_err(),
            error
        );
        assert_eq!(admission.complete().unwrap_err(), error);
        assert_eq!(*backing.borrow(), before);
    }
}

#[test]
fn bad_selections_and_consumer_rejections_leave_mapping_unchanged() {
    for (selections, kind) in [
        (vec![("app", "app.unknown.v1")], "unknown"),
        (vec![("foreign", JOURNAL)], "foreign"),
        (vec![("app", JOURNAL), ("app", JOURNAL)], "duplicate"),
    ] {
        let backing = seeded();
        let before = backing.borrow().clone();
        let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
        let policy = AdmissionPolicy {
            selections,
            ..Default::default()
        };
        let err = runtime
            .bootstrap(&snapshot(&[CONTROL], "app"), &pool(), &policy)
            .unwrap_err();
        match (kind, err) {
            ("unknown", RuntimeBootstrapError::Admission(AdmissionError::Unknown(_)))
            | ("foreign", RuntimeBootstrapError::Admission(AdmissionError::Pool(_)))
            | ("duplicate", RuntimeBootstrapError::Admission(AdmissionError::Duplicate(_))) => (),
            (_, error) => panic!("unexpected {error:?}"),
        }
        assert!(!runtime.is_bootstrapped());
        assert_eq!(*backing.borrow(), before);
    }
    let backing = seeded();
    let before = backing.borrow().clone();
    let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
    let current = snapshot(&["replacement.control.v1"], "app");
    let policy = AdmissionPolicy {
        discover: true,
        ..Default::default()
    };
    assert!(matches!(
        runtime.bootstrap(&current, &pool(), &policy),
        Err(RuntimeBootstrapError::AdmissionPolicy("identity rejected"))
    ));
    assert_eq!(*backing.borrow(), before);
    assert!(matches!(
        runtime.bootstrap(
            &snapshot(&["other.control.v1"], "other_owner"),
            &pool(),
            &policy
        ),
        Err(RuntimeBootstrapError::AdmissionPolicy("identity rejected"))
    ));
    assert_eq!(*backing.borrow(), before);
    // Retry uses unchanged evidence; an allowed addition succeeds in one generation.
    assert_eq!(
        runtime
            .bootstrap(
                &snapshot(&[CONTROL, "app.added.v1"], "app"),
                &pool(),
                &policy
            )
            .unwrap(),
        2
    );
    assert_eq!(policy.calls.get(), 3);
}

#[test]
fn current_policy_revoked_grants_and_retirement_still_reject() {
    let backing = seeded();
    let before = backing.borrow().clone();
    let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
    let policy = AdmissionPolicy {
        selections: vec![("app", JOURNAL)],
        deny_journals: true,
        ..Default::default()
    };
    assert!(matches!(
        runtime.bootstrap(&snapshot(&[CONTROL], "app"), &pool(), &policy),
        Err(RuntimeBootstrapError::Validation(_))
    ));
    assert_eq!(*backing.borrow(), before);
    let policy = AdmissionPolicy {
        selections: vec![("app", JOURNAL)],
        ..Default::default()
    };
    let revoked = crate::MemoryAllocationPool::new(vec![], vec![]).unwrap();
    assert!(matches!(
        runtime.bootstrap(&snapshot(&[CONTROL], "app"), &revoked, &policy),
        Err(RuntimeBootstrapError::Admission(AdmissionError::Pool(_)))
    ));
    assert_eq!(*backing.borrow(), before);
    let excluded =
        crate::MemoryAllocationPool::new(pool().authorities().to_vec(), (11..=11).collect())
            .unwrap();
    assert!(matches!(
        runtime.bootstrap(&snapshot(&[CONTROL], "app"), &excluded, &policy),
        Err(RuntimeBootstrapError::Admission(AdmissionError::Pool(_)))
    ));
    assert_eq!(*backing.borrow(), before);
    let mut record = runtime.ledger_record_from_memory().unwrap();
    let recovered = record.recover().unwrap();
    let mut rows = recovered.ledger().records().to_vec();
    rows.iter_mut()
        .find(|row| row.stable_key().as_str() == JOURNAL)
        .unwrap()
        .state = crate::AllocationState::Retired;
    let retired = crate::AllocationLedger::new(recovered.current_generation() + 1, rows).unwrap();
    record.commit_generation(&retired).unwrap();
    let _cell = Cell::new(
        runtime.memory(MEMORY_MANAGER_LEDGER_ID).unwrap(),
        EncodedLedgerRecord::new(&record).unwrap(),
    );
    drop(runtime);
    let before = backing.borrow().clone();
    let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
    assert!(matches!(
        runtime.bootstrap(&snapshot(&[CONTROL], "app"), &pool(), &policy),
        Err(RuntimeBootstrapError::Admission(AdmissionError::Retired(_)))
    ));
    assert_eq!(*backing.borrow(), before);
}

#[test]
fn corruption_precedes_admission_and_exhaustion_follows_it_without_commit() {
    let backing = seeded();
    let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
    let current = snapshot(&["app.new.v1"], "app");
    let bounded_pool =
        crate::MemoryAllocationPool::new(pool().authorities().to_vec(), (12..=254).collect())
            .unwrap();
    let policy = AdmissionPolicy {
        selections: vec![("app", JOURNAL)],
        ..Default::default()
    };
    let before = backing.borrow().clone();
    assert!(matches!(
        runtime.bootstrap(&current, &bounded_pool, &policy),
        Err(RuntimeBootstrapError::Resolution(
            MemoryResolutionError::Exhausted { .. }
        ))
    ));
    assert_eq!(policy.calls.get(), 1);
    assert_eq!(*backing.borrow(), before);
    runtime
        .memory(0)
        .unwrap()
        .write(STABLE_CELL_VALUE_OFFSET, &[0xff]);
    let before = backing.borrow().clone();
    for reopen in [false, true] {
        if reopen {
            runtime = MemoryRuntime::new(backing.clone()).unwrap();
        }
        assert!(matches!(
            runtime.bootstrap(&current, &pool(), &policy),
            Err(RuntimeBootstrapError::StableCellLedger(_))
        ));
        assert_eq!(policy.calls.get(), 1);
        assert_eq!(*backing.borrow(), before);
        assert!(!runtime.is_bootstrapped());
    }
}

#[test]
fn failed_persistence_retries_admission_without_partial_publication() {
    struct RefuseFirstWrite<'a> {
        backing: VectorMemory,
        calls: &'a Counter<usize>,
        refused: Counter<bool>,
    }
    impl Memory for RefuseFirstWrite<'_> {
        fn size(&self) -> u64 {
            self.backing.size()
        }
        fn grow(&self, pages: u64) -> i64 {
            self.backing.grow(pages)
        }
        fn read(&self, offset: u64, bytes: &mut [u8]) {
            self.backing.read(offset, bytes);
        }
        fn write(&self, offset: u64, bytes: &[u8]) {
            // Fail after admission, before the first persistence mutation.
            // Native Memory does not promise rollback of arbitrary partial writes.
            assert!(
                self.calls.get() != 1 || self.refused.replace(true),
                "injected write failure"
            );
            self.backing.write(offset, bytes);
        }
    }
    let backing = seeded();
    let before = backing.borrow().clone();
    let current = snapshot(&[CONTROL], "app");
    let policy = AdmissionPolicy {
        selections: vec![("app", JOURNAL)],
        ..Default::default()
    };
    let mut runtime = MemoryRuntime::new(RefuseFirstWrite {
        backing: backing.clone(),
        calls: &policy.calls,
        refused: Counter::new(false),
    })
    .unwrap();
    let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = runtime.bootstrap(&current, &pool(), &policy);
    }));
    assert!(failed.is_err());
    assert!(!runtime.is_bootstrapped());
    assert_eq!(*backing.borrow(), before);
    assert_eq!(runtime.bootstrap(&current, &pool(), &policy).unwrap(), 2);
    assert_eq!(policy.calls.get(), 2);
    let mut marker = [0; 12];
    runtime.open_memory(JOURNAL).unwrap().read(0, &mut marker);
    assert_eq!(&marker, b"pending/debt");
}

#[test]
fn fresh_rejection_can_acquire_root_but_cannot_commit_genesis() {
    let backing = VectorMemory::default();
    let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
    let current = snapshot(&[CONTROL], "app");
    let policy = AdmissionPolicy {
        reject: true,
        ..Default::default()
    };
    assert!(matches!(
        runtime.bootstrap(&current, &pool(), &policy),
        Err(RuntimeBootstrapError::AdmissionPolicy(_))
    ));
    assert!(!runtime.is_bootstrapped());
    // A fresh rejection still initializes a readable cell, without committing
    // genesis. Retrying the rejection must leave that initialized root unchanged.
    assert_eq!(runtime.memory(MEMORY_MANAGER_LEDGER_ID).unwrap().size(), 1);
    assert!(
        runtime
            .ledger_record_from_memory()
            .unwrap()
            .current()
            .is_none()
    );
    let before = backing.borrow().clone();
    assert!(matches!(
        runtime.bootstrap(&current, &pool(), &policy),
        Err(RuntimeBootstrapError::AdmissionPolicy(_))
    ));
    assert_eq!(*backing.borrow(), before);
    assert!(!runtime.is_bootstrapped());
    let policy = AdmissionPolicy::default();
    assert_eq!(runtime.bootstrap(&current, &pool(), &policy).unwrap(), 1);
}

#[test]
fn completion_bound_and_reservation_activation_preserve_evidence() {
    let backing = seeded();
    let before = backing.borrow().clone();
    let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
    let keys: Vec<_> = (0..254).map(|i| format!("app.request{i}.v1")).collect();
    let refs: Vec<_> = keys.iter().map(String::as_str).collect();
    let policy = AdmissionPolicy {
        selections: vec![("app", JOURNAL)],
        ..Default::default()
    };
    assert!(matches!(
        runtime.bootstrap(&snapshot(&refs, "app"), &pool(), &policy),
        Err(RuntimeBootstrapError::Admission(
            AdmissionError::TooManyDeclarations
        ))
    ));
    assert_eq!(*backing.borrow(), before);
    let mut record = runtime.ledger_record_from_memory().unwrap();
    let recovered = record.recover().unwrap();
    let mut rows = recovered.ledger().records().to_vec();
    rows.push(crate::testing::allocation_record(
        &crate::AllocationDeclaration::memory_manager("app.reserved.journal.v1", 12).unwrap(),
        crate::AllocationState::Reserved,
    ));
    let reserved = crate::AllocationLedger::new(recovered.current_generation() + 1, rows).unwrap();
    record.commit_generation(&reserved).unwrap();
    let _cell = Cell::new(
        runtime.memory(MEMORY_MANAGER_LEDGER_ID).unwrap(),
        EncodedLedgerRecord::new(&record).unwrap(),
    );
    drop(runtime);
    let mut runtime = MemoryRuntime::new(backing).unwrap();
    let policy = AdmissionPolicy {
        selections: vec![("app", "app.reserved.journal.v1"), ("app", JOURNAL)],
        ..Default::default()
    };
    let committed = runtime
        .bootstrap(&snapshot(&[CONTROL], "app"), &pool(), &policy)
        .unwrap();
    assert_eq!(committed, 3);
    let declaration = runtime
        .committed_allocations()
        .unwrap()
        .iter()
        .find(|d| d.stable_key().as_str() == "app.reserved.journal.v1")
        .unwrap();
    assert_eq!(declaration.slot().id(), 12);
    assert!(
        runtime
            .committed_allocations()
            .unwrap()
            .windows(2)
            .all(|pair| { pair[0].stable_key() < pair[1].stable_key() })
    );
    assert_eq!(runtime.memory_id(JOURNAL).unwrap(), 11);
}
