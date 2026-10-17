use super::{
    GenericAllocationPolicy, MemoryManagerConfig, MemoryRuntime, RuntimeAdoptionError,
    RuntimeBootstrapError, RuntimeOpenError,
};
use crate::{
    BootstrapAdmission, MemoryAllocationPool, MemoryAuthority, MemoryManagerSlot, MemoryRequest,
    PolicyIdentity, RuntimeBootstrapPolicy, SealedDeclarationSnapshot, StableKey,
};
use ic_stable_structures::VectorMemory;
use std::{cell::Cell, convert::Infallible};
const KEY: &str = "db.rows.v1";
const CONTROL: &str = "db.control.v1";
struct HostPolicy(Cell<usize>);
impl RuntimeBootstrapPolicy for HostPolicy {
    type Error = Infallible;
    fn validate_allocation(
        &self,
        _key: &StableKey,
        _slot: &MemoryManagerSlot,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn prepare_bootstrap(&self, _: &mut BootstrapAdmission<'_>) -> Result<(), Self::Error> {
        self.0.set(self.0.get() + 1);
        Ok(())
    }
    fn runtime_bootstrap_identity(&self) -> PolicyIdentity {
        PolicyIdentity::new(*b"tests.adoption.host.............")
    }
}

fn pool() -> MemoryAllocationPool {
    MemoryAllocationPool::new(vec![MemoryAuthority::new("db", "db.").unwrap()], vec![]).unwrap()
}
fn requirements(authority: &str, key: &str) -> SealedDeclarationSnapshot {
    SealedDeclarationSnapshot::new(vec![MemoryRequest::new(authority, key).unwrap()]).unwrap()
}
fn source() -> SealedDeclarationSnapshot {
    super::request_tests::snapshot(&[KEY, CONTROL], "db")
}

#[test]
fn key_lookup_preserves_refusal_order_without_memory_effects() {
    let backing = VectorMemory::default();
    let mut host = MemoryRuntime::new(backing.clone()).unwrap();
    for bootstrapped in [false, true] {
        if bootstrapped {
            host.bootstrap(&source(), &pool(), &GenericAllocationPolicy)
                .unwrap();
        }
        let before = backing.borrow().clone();
        for key in ["", "INVALID", "db.rows.v0", "ic_memory.ledger.v0"] {
            let expected = RuntimeOpenError::StableKey(StableKey::parse(key).unwrap_err());
            assert_eq!(host.memory_id(key), Err(expected.clone()));
            assert_eq!(host.open_memory(key).err(), Some(expected));
        }
        assert!(matches!(
            host.memory_id(crate::IC_MEMORY_LEDGER_STABLE_KEY),
            Err(RuntimeOpenError::ReservedStableKey { .. })
        ));
        assert_eq!(
            host.memory_id(KEY),
            if bootstrapped {
                Ok(11)
            } else {
                Err(RuntimeOpenError::NotBootstrapped)
            }
        );
        assert_eq!(*backing.borrow(), before);
    }
}

#[test]
fn adoption_checks_logical_requirements_without_replaying_host() {
    let backing = VectorMemory::default();
    let policy = HostPolicy(Cell::new(0));
    let mut host =
        MemoryRuntime::new_with_config(backing.clone(), MemoryManagerConfig::new(16).unwrap())
            .unwrap();
    assert_eq!(
        host.verify_authority(&source(), "db"),
        Err(RuntimeAdoptionError::Open(
            RuntimeOpenError::NotBootstrapped
        ))
    );
    host.bootstrap(&source(), &pool(), &policy).unwrap();
    let capability = host.committed_allocations().unwrap().clone();
    let before = backing.borrow().clone();
    let report = host.memory_allocations().unwrap();
    for _ in 0..2 {
        host.verify_authority(&requirements("db", KEY), "db")
            .unwrap();
        assert_eq!(host.memory_id(KEY), Ok(11));
    }
    let mut requests = source().requests().to_vec();
    requests.push(MemoryRequest::new("other", "other.missing.v1").unwrap());
    host.verify_authority(
        &SealedDeclarationSnapshot::new(requests.clone()).unwrap(),
        "db",
    )
    .unwrap();
    assert_eq!(policy.0.get(), 1);
    assert_eq!(host.committed_allocations().unwrap(), &capability);
    assert_eq!(host.memory_allocations().unwrap(), report);
    assert_eq!(*backing.borrow(), before);
    assert!(matches!(
        host.bootstrap(&source(), &pool(), &GenericAllocationPolicy),
        Err(RuntimeBootstrapError::PolicyIdentityMismatch { .. })
    ));
    let replaced =
        MemoryAllocationPool::new(pool().authorities().to_vec(), (200..=200).collect()).unwrap();
    assert!(matches!(
        host.bootstrap(&source(), &replaced, &policy),
        Err(RuntimeBootstrapError::AllocationPoolMismatch)
    ));
    assert!(matches!(
        host.doctor_report(&source(), &replaced, &policy)
            .bootstrap_binding,
        crate::DiagnosticCheck::Failed {
            code: crate::DiagnosticCode::RuntimeBinding,
            ..
        }
    ));
    assert_eq!(policy.0.get(), 1);
    assert_eq!(*backing.borrow(), before);
}

#[test]
fn adoption_rejects_missing_keys_and_wrong_authorities() {
    let mut host = MemoryRuntime::new(VectorMemory::default()).unwrap();
    host.bootstrap(&source(), &pool(), &GenericAllocationPolicy)
        .unwrap();
    assert!(matches!(
        host.verify_authority(&requirements("foreign", KEY), "foreign"),
        Err(RuntimeAdoptionError::AuthorityMismatch { .. })
    ));
    assert!(matches!(
        host.verify_authority(&requirements("db", "db.missing.v1"), "db"),
        Err(RuntimeAdoptionError::Open(
            RuntimeOpenError::StableKeyNotCommitted(_)
        ))
    ));
    assert!(matches!(
        host.verify_authority(&source(), "missing"),
        Err(RuntimeAdoptionError::UnknownAuthority { .. })
    ));
}
