use super::request_tests::{pool, snapshot};
use super::*;
use crate::{
    AllocationPolicy, BootstrapAdmissionError as AdmissionError, MemoryManagerSlot, StableKey,
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

impl AllocationPolicy for AdmissionPolicy {
    type Error = &'static str;
    fn validate_key(&self, key: &StableKey) -> Result<(), Self::Error> {
        if self.deny_journals && key.as_str().ends_with(".journal.v1") {
            Err("revoked")
        } else {
            Ok(())
        }
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
impl RuntimeBootstrapPolicy for AdmissionPolicy {
    fn runtime_bootstrap_identity(&self) -> Result<PolicyIdentity, crate::PolicyIdentityError> {
        PolicyIdentity::new("test.admission", 1)
    }
    fn prepare_bootstrap(&self, admission: &mut BootstrapAdmission<'_>) -> Result<(), Self::Error> {
        self.calls.set(self.calls.get() + 1);
        if self.reject {
            return Err("identity rejected");
        }
        let mut selected = Vec::new();
        if self.discover {
            for record in admission.recovered_allocations() {
                if record.stable_key.as_str().ends_with(".control.v1")
                    && !admission.is_declared(record.stable_key)
                {
                    return Err("identity rejected");
                }
                if record.stable_key.as_str().ends_with(".journal.v1")
                    && !admission.is_declared(record.stable_key)
                {
                    selected.push(record.stable_key.clone());
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
fn declared_membership_covers_original_inputs_and_retains_poisoned_selections() {
    let runtime = MemoryRuntime::new(seeded()).unwrap();
    let recovered = runtime
        .ledger_record_from_memory()
        .unwrap()
        .store()
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
    assert_eq!(
        runtime
            .bootstrap(&current, &pool(), &policy)
            .unwrap()
            .generation(),
        2
    );
    let mut marker = [0; 12];
    runtime.open_memory(JOURNAL).unwrap().read(0, &mut marker);
    assert_eq!(&marker, b"pending/debt");
    assert_eq!(
        runtime
            .committed_allocations()
            .unwrap()
            .slot_for(&StableKey::parse(JOURNAL).unwrap())
            .unwrap()
            .id(),
        11
    );
    let before = backing.borrow().clone();
    assert_eq!(
        runtime
            .bootstrap(&current, &pool(), &policy)
            .unwrap()
            .generation(),
        2
    );
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
            .unwrap()
            .generation(),
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
    let excluded = crate::MemoryAllocationPool::new(
        pool().authorities().to_vec(),
        vec![crate::MemoryManagerIdRange::new(11, 11).unwrap()],
    )
    .unwrap();
    assert!(matches!(
        runtime.bootstrap(&snapshot(&[CONTROL], "app"), &excluded, &policy),
        Err(RuntimeBootstrapError::Admission(AdmissionError::Pool(_)))
    ));
    assert_eq!(*backing.borrow(), before);
    let mut record = runtime.ledger_record_from_memory().unwrap();
    AllocationBootstrap::new(record.store_mut())
        .retire_and_commit(
            &crate::AllocationRetirement::new(JOURNAL, MemoryManagerSlot::new(11).unwrap())
                .unwrap(),
        )
        .unwrap();
    let _cell = Cell::new(runtime.memory(MEMORY_MANAGER_LEDGER_ID), record);
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
    let bounded_pool = crate::MemoryAllocationPool::new(
        pool().authorities().to_vec(),
        vec![crate::MemoryManagerIdRange::new(12, 254).unwrap()],
    )
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
    runtime.memory(0).write(STABLE_CELL_VALUE_OFFSET, &[0xff]);
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
    struct Limited {
        backing: VectorMemory,
        limit: std::rc::Rc<Counter<u64>>,
    }
    impl Memory for Limited {
        fn size(&self) -> u64 {
            self.backing.size()
        }
        fn grow(&self, pages: u64) -> i64 {
            if self.size() + pages > self.limit.get() {
                -1
            } else {
                self.backing.grow(pages)
            }
        }
        fn read(&self, offset: u64, bytes: &mut [u8]) {
            self.backing.read(offset, bytes);
        }
        fn write(&self, offset: u64, bytes: &[u8]) {
            self.backing.write(offset, bytes);
        }
    }
    let backing = seeded();
    // Two current snapshots with many retained keys require a second ledger page.
    let keys: Vec<_> = (0..240)
        .map(|i| format!("app.new{i}.{}.v1", "x".repeat(110)))
        .collect();
    let refs: Vec<_> = keys.iter().map(String::as_str).collect();
    let mut seeded_refs = refs.clone();
    seeded_refs.extend([CONTROL, JOURNAL]);
    MemoryRuntime::new(backing.clone())
        .unwrap()
        .bootstrap(
            &snapshot(&seeded_refs, "app"),
            &pool(),
            &GenericAllocationPolicy,
        )
        .unwrap();
    let before = backing.borrow().clone();
    let limit = std::rc::Rc::new(Counter::new(backing.size()));
    let current = snapshot(&refs, "app");
    let policy = AdmissionPolicy {
        selections: vec![("app", JOURNAL)],
        ..Default::default()
    };
    let mut runtime = MemoryRuntime::new(Limited {
        backing: backing.clone(),
        limit: limit.clone(),
    })
    .unwrap();
    let result = runtime.bootstrap(&current, &pool(), &policy);
    assert!(
        matches!(
            result,
            Err(RuntimeBootstrapError::LedgerGrowth(
                super::RuntimeGrowError::BackingRefused { .. }
            ))
        ),
        "{:?}",
        result.err()
    );
    assert!(!runtime.is_bootstrapped());
    assert_eq!(*backing.borrow(), before);
    limit.set(100);
    assert_eq!(
        runtime
            .bootstrap(&current, &pool(), &policy)
            .unwrap()
            .generation(),
        3
    );
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
    assert_eq!(runtime.memory(MEMORY_MANAGER_LEDGER_ID).size(), 1);
    assert!(
        runtime
            .ledger_record_from_memory()
            .unwrap()
            .store()
            .physical()
            .is_uninitialized()
    );
    let before = backing.borrow().clone();
    assert!(matches!(
        runtime.bootstrap(&current, &pool(), &policy),
        Err(RuntimeBootstrapError::AdmissionPolicy(_))
    ));
    assert_eq!(*backing.borrow(), before);
    assert!(!runtime.is_bootstrapped());
    let policy = AdmissionPolicy::default();
    assert_eq!(
        runtime
            .bootstrap(&current, &pool(), &policy)
            .unwrap()
            .generation(),
        1
    );
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
    AllocationBootstrap::new(record.store_mut())
        .reserve_and_commit(
            &[
                crate::AllocationDeclaration::memory_manager_unlabeled_with_schema(
                    "app.reserved.journal.v1",
                    12,
                    crate::SchemaMetadata::new(Some(5)).unwrap(),
                )
                .unwrap(),
            ],
            &GenericAllocationPolicy,
        )
        .unwrap();
    let _cell = Cell::new(runtime.memory(MEMORY_MANAGER_LEDGER_ID), record);
    drop(runtime);
    let mut runtime = MemoryRuntime::new(backing).unwrap();
    let policy = AdmissionPolicy {
        selections: vec![("app", "app.reserved.journal.v1")],
        ..Default::default()
    };
    let committed = runtime
        .bootstrap(&snapshot(&[CONTROL], "app"), &pool(), &policy)
        .unwrap();
    assert_eq!(committed.generation(), 3);
    let declaration = committed
        .declarations()
        .iter()
        .find(|d| d.stable_key().as_str() == "app.reserved.journal.v1")
        .unwrap();
    assert_eq!(
        declaration.schema(),
        &crate::SchemaMetadata::new(Some(5)).unwrap()
    );
    assert_eq!(declaration.slot().id(), 12);
}
