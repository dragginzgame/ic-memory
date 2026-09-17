use super::*;
use crate::{
    AllocationDeclaration, AllocationPolicy, AllocationRetirement, AllocationSlotDescriptor,
    MemoryManagerAuthorityRecord, MemoryManagerIdRange, MemoryManagerRangeMode, MemoryRequest,
    SchemaMetadata, StaticMemoryDeclaration, StaticMemoryRangeDeclaration,
};
use ic_stable_structures::VectorMemory;

pub(super) fn snapshot(
    keys: &[&str],
    owner: &str,
    start: u8,
    end: u8,
    fixed: &[(&str, u8)],
) -> SealedDeclarationSnapshot {
    let range = StaticMemoryRangeDeclaration::new(
        MemoryManagerAuthorityRecord::new(
            MemoryManagerIdRange::new(start, end).unwrap(),
            owner,
            MemoryManagerRangeMode::Allowed,
            None,
        )
        .unwrap(),
    )
    .unwrap();
    let requests: Vec<_> = keys
        .iter()
        .map(|key| MemoryRequest::new(owner, key, SchemaMetadata::default()).unwrap())
        .collect();
    let declarations: Vec<_> = fixed
        .iter()
        .map(|(key, id)| {
            StaticMemoryDeclaration::new(
                owner,
                AllocationDeclaration::memory_manager_unlabeled(key, *id).unwrap(),
            )
            .unwrap()
        })
        .collect();
    SealedDeclarationSnapshot::new(&declarations, &[range], &requests).unwrap()
}

fn id(runtime: &MemoryRuntime<VectorMemory>, key: &str) -> u8 {
    runtime
        .committed_allocations()
        .unwrap()
        .slot_for(&StableKey::parse(key).unwrap())
        .unwrap()
        .memory_manager_id()
        .unwrap()
}

#[test]
fn deterministic_requests_preserve_history_and_explicit_inspection() {
    let backing = VectorMemory::default();
    let first = snapshot(
        &["app.b.v1", "app.a.v1"],
        "app",
        100,
        103,
        &[("app.fixed.v1", 100)],
    );
    let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
    assert!(matches!(
        runtime.open_memory_by_key("app.b.v1"),
        Err(RuntimeOpenError::NotBootstrapped)
    ));
    runtime.bootstrap(&first, &GenericRangePolicy).unwrap();
    assert_eq!(
        (id(&runtime, "app.a.v1"), id(&runtime, "app.b.v1")),
        (101, 102)
    );
    let b = runtime.open_memory_by_key("app.b.v1").unwrap();
    assert_eq!(b.grow(1), 0);
    b.write(0, b"journal debt");
    drop(b);
    let fresh = VectorMemory::default();
    let mut other = MemoryRuntime::new(fresh).unwrap();
    other
        .bootstrap(
            &snapshot(
                &["app.a.v1", "app.b.v1"],
                "app",
                100,
                103,
                &[("app.fixed.v1", 100)],
            ),
            &GenericRangePolicy,
        )
        .unwrap();
    assert_eq!(id(&other, "app.b.v1"), 102);
    drop(runtime);

    // B is absent from current open authority, but still owns its slot.
    let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
    runtime
        .bootstrap(
            &snapshot(
                &["app.c.v1", "app.a.v1"],
                "app",
                100,
                103,
                &[("app.fixed.v1", 100)],
            ),
            &GenericRangePolicy,
        )
        .unwrap();
    assert_eq!(
        (id(&runtime, "app.a.v1"), id(&runtime, "app.c.v1")),
        (101, 103)
    );
    assert!(matches!(
        runtime.open_memory_by_key("app.b.v1"),
        Err(RuntimeOpenError::StableKeyNotCommitted(_))
    ));
    drop(runtime);

    // The generated reconciliation manifest explicitly includes B before sealing.
    let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
    runtime
        .bootstrap(
            &snapshot(
                &["app.a.v1", "app.b.v1", "app.c.v1"],
                "app",
                100,
                103,
                &[("app.fixed.v1", 100)],
            ),
            &GenericRangePolicy,
        )
        .unwrap();
    let mut marker = [0; 12];
    runtime
        .open_memory_by_key("app.b.v1")
        .unwrap()
        .read(0, &mut marker);
    assert_eq!(&marker, b"journal debt");
    assert_eq!(id(&runtime, "app.b.v1"), 102);
    assert!(matches!(
        runtime.open_memory_by_key("app.unknown.v1"),
        Err(RuntimeOpenError::StableKeyNotCommitted(_))
    ));
    drop(runtime);

    for denied in [
        snapshot(&["app.b.v1"], "foreign", 110, 115, &[]),
        snapshot(&["app.b.v1"], "app", 103, 115, &[]),
    ] {
        let before = backing.borrow().clone();
        let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
        assert!(matches!(
            runtime.bootstrap(&denied, &GenericRangePolicy),
            Err(RuntimeBootstrapError::Resolution(_))
        ));
        assert!(!runtime.is_bootstrapped());
        assert_eq!(*backing.borrow(), before);
    }
}

#[test]
fn exhaustion_and_fixed_conflicts_do_not_publish_or_change_committed_state() {
    let backing = VectorMemory::default();
    let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
    runtime
        .bootstrap(
            &snapshot(&["app.a.v1"], "app", 100, 100, &[]),
            &GenericRangePolicy,
        )
        .unwrap();
    drop(runtime);
    for invalid in [
        snapshot(&["app.b.v1"], "app", 100, 100, &[]),
        snapshot(&["app.a.v1"], "app", 100, 101, &[("app.fixed.v1", 100)]),
    ] {
        let before = backing.borrow().clone();
        let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
        assert!(runtime.bootstrap(&invalid, &GenericRangePolicy).is_err());
        assert!(!runtime.is_bootstrapped());
        assert_eq!(*backing.borrow(), before);
    }
    assert!(MemoryRequest::new("app", "BAD KEY", SchemaMetadata::default()).is_err());
    let request = MemoryRequest::new("app", "app.a.v1", SchemaMetadata::default()).unwrap();
    assert!(SealedDeclarationSnapshot::new(&[], &[], &[request.clone(), request]).is_err());
    let mut runtime = MemoryRuntime::new(VectorMemory::default()).unwrap();
    let no_grant = SealedDeclarationSnapshot::new(
        &[],
        &[],
        &[MemoryRequest::new("app", "app.a.v1", SchemaMetadata::default()).unwrap()],
    )
    .unwrap();
    assert!(matches!(
        runtime.bootstrap(&no_grant, &GenericRangePolicy),
        Err(RuntimeBootstrapError::Resolution(
            MemoryResolutionError::Exhausted { .. }
        ))
    ));
}

#[test]
fn reservation_activation_and_retirement_use_existing_claim_rules() {
    let backing = VectorMemory::default();
    let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
    let genesis = AllocationLedger::new(0, AllocationHistory::default()).unwrap();
    let reservation = AllocationDeclaration::memory_manager_unlabeled("app.b.v1", 101).unwrap();
    let mut record = StableCellLedgerRecord::default();
    record.store_mut().commit(&genesis).unwrap();
    AllocationBootstrap::new(record.store_mut())
        .reserve_and_commit(&[reservation], &GenericRangePolicy, None)
        .unwrap();
    let _cell = Cell::init(runtime.memory(MEMORY_MANAGER_LEDGER_ID), record);
    runtime
        .bootstrap(
            &snapshot(&["app.a.v1", "app.b.v1"], "app", 100, 103, &[]),
            &GenericRangePolicy,
        )
        .unwrap();
    assert_eq!(id(&runtime, "app.b.v1"), 101);
    let memory = runtime.open_memory_by_key("app.b.v1").unwrap();
    memory.grow(1);
    memory.write(0, b"pending");
    let mut record = runtime.ledger_record_from_memory().unwrap();
    AllocationBootstrap::new(record.store_mut())
        .retire_and_commit(
            &AllocationRetirement::new(
                "app.b.v1",
                AllocationSlotDescriptor::memory_manager(101).unwrap(),
            )
            .unwrap(),
            None,
        )
        .unwrap();
    runtime
        .persist_ledger_record::<std::convert::Infallible>(record)
        .unwrap();
    drop(runtime);
    let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
    assert!(matches!(
        runtime.bootstrap(
            &snapshot(&["app.b.v1"], "app", 100, 103, &[]),
            &GenericRangePolicy
        ),
        Err(RuntimeBootstrapError::Validation(_))
    ));
    assert!(!runtime.is_bootstrapped());
    drop(runtime);
    let mut runtime = MemoryRuntime::new(backing).unwrap();
    runtime
        .bootstrap(
            &snapshot(&["app.c.v1"], "app", 100, 103, &[]),
            &GenericRangePolicy,
        )
        .unwrap();
    assert_eq!(id(&runtime, "app.c.v1"), 102); // A and retired B remain claimed.
}

#[test]
fn failed_persistence_publishes_no_mapping_and_retries_deterministically() {
    struct Limited {
        bytes: VectorMemory,
        limit: std::rc::Rc<std::cell::Cell<u64>>,
    }
    impl Memory for Limited {
        fn size(&self) -> u64 {
            self.bytes.size()
        }
        fn grow(&self, pages: u64) -> i64 {
            if self.size() + pages > self.limit.get() {
                -1
            } else {
                self.bytes.grow(pages)
            }
        }
        fn read(&self, offset: u64, bytes: &mut [u8]) {
            self.bytes.read(offset, bytes);
        }
        fn write(&self, offset: u64, bytes: &[u8]) {
            self.bytes.write(offset, bytes);
        }
    }
    let bytes = VectorMemory::default();
    let limit = std::rc::Rc::new(std::cell::Cell::new(2));
    // Long valid keys force capacity growth even with compact byte-string payloads.
    let keys: Vec<_> = (0..239)
        .map(|i| format!("app.store{i:03}.{}.v1", "x".repeat(110)))
        .collect();
    let refs: Vec<_> = keys.iter().map(String::as_str).collect();
    let declarations = snapshot(&refs, "app", 16, 254, &[]);
    let mut runtime = MemoryRuntime::new_with_config(
        Limited {
            bytes: bytes.clone(),
            limit: limit.clone(),
        },
        MemoryManagerConfig::new(1).unwrap(),
    )
    .unwrap();
    let result = runtime.bootstrap(&declarations, &GenericRangePolicy);
    assert!(
        matches!(
            result,
            Err(RuntimeBootstrapError::StableCellLedgerWriteTooLarge { .. })
        ),
        "{:?}",
        result.err()
    );
    assert!(!runtime.is_bootstrapped());
    assert!(matches!(
        runtime.open_memory_by_key(&keys[0]),
        Err(RuntimeOpenError::NotBootstrapped)
    ));
    drop(runtime);
    limit.set(100);
    let mut runtime = MemoryRuntime::new(Limited { bytes, limit }).unwrap();
    runtime
        .bootstrap(&declarations, &GenericRangePolicy)
        .unwrap();
    assert_eq!(runtime.committed_allocations().unwrap().generation(), 1);
    assert_eq!(
        runtime
            .committed_allocations()
            .unwrap()
            .slot_for(&StableKey::parse(&keys[0]).unwrap())
            .unwrap()
            .memory_manager_id()
            .unwrap(),
        16
    );
}

#[test]
fn current_custom_policy_can_reject_a_recovered_logical_key() {
    struct Revoked;
    impl AllocationPolicy for Revoked {
        type Error = &'static str;
        fn validate_key(&self, _: &StableKey) -> Result<(), Self::Error> {
            Err("revoked")
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
    impl RuntimeBootstrapPolicy for Revoked {
        fn runtime_bootstrap_identity(&self) -> Result<PolicyIdentity, crate::PolicyIdentityError> {
            PolicyIdentity::new("revoked", 1)
        }
    }
    let backing = VectorMemory::default();
    let declarations = snapshot(&["app.a.v1"], "app", 100, 110, &[]);
    let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
    runtime
        .bootstrap(&declarations, &GenericRangePolicy)
        .unwrap();
    // Logical diagnostics use the same resolution, and the original input binding.
    assert_eq!(
        runtime
            .doctor_report(&declarations, &GenericRangePolicy)
            .bootstrap_binding,
        crate::DiagnosticCheck::passed()
    );
    drop(runtime);
    let before = backing.borrow().clone();
    let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
    assert!(matches!(
        runtime.bootstrap(&declarations, &Revoked),
        Err(RuntimeBootstrapError::Validation(
            crate::AllocationValidationError::Policy(RuntimePolicyError::Custom("revoked"))
        ))
    ));
    assert_eq!(*backing.borrow(), before);
    assert!(!runtime.is_bootstrapped());
}
