use super::request_tests::snapshot;
use super::*;
use crate::{
    AllocationPolicy, AllocationSlotDescriptor, BootstrapAdmissionError as AdmissionError,
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

pub(super) fn seeded() -> VectorMemory {
    let backing = VectorMemory::default();
    let mut runtime =
        MemoryRuntime::new_with_config(backing.clone(), MemoryManagerConfig::new(1).unwrap())
            .unwrap();
    runtime
        .bootstrap(
            &snapshot(&[CONTROL, JOURNAL], "app", 100, 110, &[]),
            &GenericRangePolicy,
        )
        .unwrap();
    let journal = runtime.open_memory_by_key(JOURNAL).unwrap();
    journal.grow(1);
    journal.write(0, b"pending/debt");
    backing
}

#[test]
fn discovers_omitted_journal_before_one_commit_and_skips_warm_admission() {
    let backing = seeded();
    let current = snapshot(&[CONTROL, "app.new.journal.v1"], "app", 100, 110, &[]);
    let policy = AdmissionPolicy {
        discover: true,
        ..Default::default()
    };
    let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
    assert!(matches!(
        runtime.open_memory_by_key(JOURNAL),
        Err(RuntimeOpenError::NotBootstrapped)
    ));
    assert_eq!(
        runtime.bootstrap(&current, &policy).unwrap().generation(),
        2
    );
    let mut marker = [0; 12];
    runtime
        .open_memory_by_key(JOURNAL)
        .unwrap()
        .read(0, &mut marker);
    assert_eq!(&marker, b"pending/debt");
    assert_eq!(
        runtime
            .committed_allocations()
            .unwrap()
            .slot_for(&StableKey::parse(JOURNAL).unwrap())
            .unwrap()
            .memory_manager_id()
            .unwrap(),
        101
    );
    let before = backing.borrow().clone();
    assert_eq!(
        runtime.bootstrap(&current, &policy).unwrap().generation(),
        2
    );
    let _ = runtime.doctor_report(&current, &policy);
    assert_eq!(policy.calls.get(), 1);
    assert_eq!(runtime.memory_manager_config().bucket_size_pages(), 1);
    assert_eq!(*backing.borrow(), before);
    assert!(matches!(
        runtime.bootstrap(&current, &GenericRangePolicy),
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
            .bootstrap(&snapshot(&[CONTROL], "app", 100, 110, &[]), &policy)
            .unwrap_err();
        match (kind, err) {
            ("unknown", RuntimeBootstrapError::Admission(AdmissionError::Unknown(_)))
            | ("foreign", RuntimeBootstrapError::Admission(AdmissionError::Range { .. }))
            | ("duplicate", RuntimeBootstrapError::Admission(AdmissionError::Duplicate(_))) => (),
            (_, error) => panic!("unexpected {error:?}"),
        }
        assert!(!runtime.is_bootstrapped());
        assert_eq!(*backing.borrow(), before);
    }
    let backing = seeded();
    let before = backing.borrow().clone();
    let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
    let current = snapshot(&["replacement.control.v1"], "app", 100, 110, &[]);
    let policy = AdmissionPolicy {
        discover: true,
        ..Default::default()
    };
    assert!(matches!(
        runtime.bootstrap(&current, &policy),
        Err(RuntimeBootstrapError::AdmissionPolicy("identity rejected"))
    ));
    assert_eq!(*backing.borrow(), before);
    assert!(matches!(
        runtime.bootstrap(
            &snapshot(&["other.control.v1"], "other_owner", 100, 110, &[]),
            &policy
        ),
        Err(RuntimeBootstrapError::AdmissionPolicy("identity rejected"))
    ));
    assert_eq!(*backing.borrow(), before);
    // Retry uses unchanged evidence; an allowed addition succeeds in one generation.
    assert_eq!(
        runtime
            .bootstrap(
                &snapshot(&[CONTROL, "app.added.v1"], "app", 100, 110, &[]),
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
        runtime.bootstrap(&snapshot(&[CONTROL], "app", 100, 110, &[]), &policy),
        Err(RuntimeBootstrapError::Validation(_))
    ));
    assert_eq!(*backing.borrow(), before);
    let policy = AdmissionPolicy {
        selections: vec![("app", JOURNAL)],
        ..Default::default()
    };
    assert!(matches!(
        runtime.bootstrap(
            &snapshot(&[CONTROL], "app", 100, 110, &[("app.conflict.v1", 101)]),
            &policy
        ),
        Err(RuntimeBootstrapError::Resolution(_))
    ));
    assert_eq!(*backing.borrow(), before);
    assert!(matches!(
        runtime.bootstrap(&snapshot(&[], "app", 102, 110, &[]), &policy),
        Err(RuntimeBootstrapError::Admission(
            AdmissionError::Range { .. }
        ))
    ));
    assert_eq!(*backing.borrow(), before);
    let mut record = runtime.ledger_record_from_memory().unwrap();
    AllocationBootstrap::new(record.store_mut())
        .retire_and_commit(
            &crate::AllocationRetirement::new(
                JOURNAL,
                AllocationSlotDescriptor::memory_manager(101).unwrap(),
            )
            .unwrap(),
            None,
        )
        .unwrap();
    runtime.persist_ledger_record::<&str>(record).unwrap();
    drop(runtime);
    let before = backing.borrow().clone();
    let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
    assert!(matches!(
        runtime.bootstrap(&snapshot(&[CONTROL], "app", 100, 110, &[]), &policy),
        Err(RuntimeBootstrapError::Admission(AdmissionError::Retired(_)))
    ));
    assert_eq!(*backing.borrow(), before);
}

#[test]
fn corruption_precedes_admission_and_exhaustion_follows_it_without_commit() {
    let backing = seeded();
    let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
    let current = snapshot(&["app.new.v1"], "app", 100, 101, &[]);
    let policy = AdmissionPolicy {
        selections: vec![("app", JOURNAL)],
        ..Default::default()
    };
    let before = backing.borrow().clone();
    assert!(matches!(
        runtime.bootstrap(&current, &policy),
        Err(RuntimeBootstrapError::Resolution(
            MemoryResolutionError::Exhausted { .. }
        ))
    ));
    assert_eq!(policy.calls.get(), 1);
    assert_eq!(*backing.borrow(), before);
    runtime.memory(0).write(STABLE_CELL_VALUE_OFFSET, &[0xff]);
    drop(runtime);
    let before = backing.borrow().clone();
    let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
    assert!(matches!(
        runtime.bootstrap(&current, &policy),
        Err(RuntimeBootstrapError::StableCellLedger(_))
    ));
    assert_eq!(policy.calls.get(), 1);
    assert_eq!(*backing.borrow(), before);
    assert!(!runtime.is_bootstrapped());
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
    let before = backing.borrow().clone();
    let limit = std::rc::Rc::new(Counter::new(backing.size()));
    let keys: Vec<_> = (0..200).map(|i| format!("app.new{i}.v1")).collect();
    let refs: Vec<_> = keys.iter().map(String::as_str).collect();
    let current = snapshot(&refs, "app", 10, 254, &[]);
    let policy = AdmissionPolicy {
        selections: vec![("app", JOURNAL)],
        ..Default::default()
    };
    let mut runtime = MemoryRuntime::new(Limited {
        backing: backing.clone(),
        limit: limit.clone(),
    })
    .unwrap();
    assert!(matches!(
        runtime.bootstrap(&current, &policy),
        Err(RuntimeBootstrapError::StableCellLedgerWriteTooLarge { .. })
    ));
    assert!(!runtime.is_bootstrapped());
    assert_eq!(*backing.borrow(), before);
    limit.set(100);
    assert_eq!(
        runtime.bootstrap(&current, &policy).unwrap().generation(),
        2
    );
    assert_eq!(policy.calls.get(), 2);
    let mut marker = [0; 12];
    runtime
        .open_memory_by_key(JOURNAL)
        .unwrap()
        .read(0, &mut marker);
    assert_eq!(&marker, b"pending/debt");
}

#[test]
fn fresh_rejection_can_acquire_root_but_cannot_commit_genesis() {
    let mut runtime = MemoryRuntime::new(VectorMemory::default()).unwrap();
    let current = snapshot(&[CONTROL], "app", 100, 110, &[]);
    let policy = AdmissionPolicy {
        reject: true,
        ..Default::default()
    };
    assert!(matches!(
        runtime.bootstrap(&current, &policy),
        Err(RuntimeBootstrapError::AdmissionPolicy(_))
    ));
    assert!(!runtime.is_bootstrapped());
    assert!(
        runtime
            .ledger_record_from_memory()
            .unwrap()
            .store()
            .physical()
            .is_uninitialized()
    );
    let policy = AdmissionPolicy::default();
    assert_eq!(
        runtime.bootstrap(&current, &policy).unwrap().generation(),
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
        runtime.bootstrap(&snapshot(&refs, "app", 10, 254, &[]), &policy),
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
                    102,
                    crate::SchemaMetadata::new(Some(5)).unwrap(),
                )
                .unwrap(),
            ],
            &GenericRangePolicy,
            None,
        )
        .unwrap();
    runtime.persist_ledger_record::<&str>(record).unwrap();
    drop(runtime);
    let mut runtime = MemoryRuntime::new(backing).unwrap();
    let policy = AdmissionPolicy {
        selections: vec![("app", "app.reserved.journal.v1")],
        ..Default::default()
    };
    let committed = runtime
        .bootstrap(&snapshot(&[CONTROL], "app", 100, 110, &[]), &policy)
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
    assert_eq!(declaration.slot().memory_manager_id().unwrap(), 102);
}
