use super::{
    GenericRangePolicy, MemoryManagerConfig, MemoryRuntime, RuntimeAdoptionError, RuntimeOpenError,
};
use crate::{
    AllocationDeclaration, AllocationPolicy, BootstrapAdmission, MemoryManagerSlot, MemoryRequest,
    PolicyIdentity, PolicyIdentityError, RuntimeBootstrapPolicy, SchemaMetadata,
    SealedDeclarationSnapshot, StableKey, StaticMemoryDeclaration,
};
use ic_stable_structures::VectorMemory;
use std::{cell::Cell, convert::Infallible};

const KEY: &str = "db.rows.v1";
const FIXED: &str = "db.control.v1";

struct HostPolicy(Cell<usize>);
impl AllocationPolicy for HostPolicy {
    type Error = Infallible;
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
    fn prepare_bootstrap(&self, _: &mut BootstrapAdmission<'_>) -> Result<(), Self::Error> {
        self.0.set(self.0.get() + 1);
        Ok(())
    }
    fn runtime_bootstrap_identity(&self) -> Result<PolicyIdentity, PolicyIdentityError> {
        PolicyIdentity::new("tests.adoption.host", 1)
    }
}

fn requirements(
    authority: &str,
    key: &str,
    schema: Option<u32>,
    fixed: Option<(u8, &str)>,
) -> SealedDeclarationSnapshot {
    let declaration = fixed.map(|(id, label)| {
        StaticMemoryDeclaration::new(
            authority,
            AllocationDeclaration::memory_manager_with_schema(
                key,
                id,
                label,
                SchemaMetadata::new(schema).unwrap(),
            )
            .unwrap(),
        )
        .unwrap()
    });
    let request = if fixed.is_none() {
        vec![MemoryRequest::new(authority, key, SchemaMetadata::new(schema).unwrap()).unwrap()]
    } else {
        vec![]
    };
    SealedDeclarationSnapshot::new(&declaration.into_iter().collect::<Vec<_>>(), &[], &request)
        .unwrap()
}

#[test]
fn key_lookup_preserves_refusal_order_without_memory_effects() {
    let backing = VectorMemory::default();
    let source = super::request_tests::snapshot(&[KEY], "db", 100, 110, &[(FIXED, 100)]);
    let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
    for bootstrapped in [false, true] {
        if bootstrapped {
            runtime.bootstrap(&source, &GenericRangePolicy).unwrap();
        }
        let before = backing.borrow().clone();
        for key in ["", "INVALID", "db.rows.v0", "ic_memory.ledger.v0"] {
            let expected = RuntimeOpenError::StableKey(StableKey::parse(key).unwrap_err());
            assert_eq!(runtime.memory_id(key), Err(expected.clone()));
            assert_eq!(
                runtime.open_memory_by_key(key).err(),
                Some(expected.clone())
            );
            assert_eq!(runtime.open_memory(key, 100).err(), Some(expected));
        }
        let reserved = crate::IC_MEMORY_LEDGER_STABLE_KEY;
        assert_eq!(
            runtime.memory_id(reserved),
            Err(RuntimeOpenError::ReservedStableKey {
                stable_key: reserved.into(),
            })
        );
        for key in [KEY, "db.missing.v1"] {
            let expected = if !bootstrapped {
                Err(RuntimeOpenError::NotBootstrapped)
            } else if key == KEY {
                Ok(101)
            } else {
                Err(RuntimeOpenError::StableKeyNotCommitted(key.into()))
            };
            assert_eq!(runtime.memory_id(key), expected);
        }
        assert_eq!(*backing.borrow(), before);
    }
}

#[test]
fn adoption_checks_fixed_and_logical_requirements_without_replaying_host() {
    let backing = VectorMemory::default();
    let source = super::request_tests::snapshot(&[KEY], "db", 100, 110, &[(FIXED, 100)]);
    let policy = HostPolicy(Cell::new(0));
    let mut runtime =
        MemoryRuntime::new_with_config(backing.clone(), MemoryManagerConfig::new(16).unwrap())
            .unwrap();
    assert_eq!(
        runtime.verify_authority(&source, "db"),
        Err(RuntimeAdoptionError::Open(
            RuntimeOpenError::NotBootstrapped
        ))
    );
    runtime.bootstrap(&source, &policy).unwrap();
    let capability = runtime.committed_allocations().unwrap().clone();
    let before = backing.borrow().clone();
    let report = runtime.memory_allocations().unwrap();
    for _ in 0..2 {
        runtime.verify_authority(&source, "db").unwrap();
        // A library may supply just its requirements, without the host's grants.
        runtime
            .verify_authority(&requirements("db", KEY, None, None), "db")
            .unwrap();
        assert_eq!(runtime.memory_id(KEY), Ok(101));
        assert_eq!(runtime.memory_id(FIXED), Ok(100));
    }
    let mut requests = source.requests().to_vec();
    requests
        .push(MemoryRequest::new("other", "other.missing.v1", SchemaMetadata::default()).unwrap());
    let combined_requirements =
        SealedDeclarationSnapshot::new(source.registered_declarations(), &[], &requests).unwrap();
    runtime
        .verify_authority(&combined_requirements, "db")
        .unwrap();
    assert_eq!(policy.0.get(), 1);
    assert_eq!(runtime.committed_allocations().unwrap(), &capability);
    assert_eq!(runtime.memory_manager_config().bucket_size_pages(), 16);
    assert_eq!(*backing.borrow(), before);
    assert_eq!(runtime.memory_allocations().unwrap(), report);
    // Adoption does not replace the policy bound by the host.
    assert!(matches!(
        runtime.bootstrap(&source, &GenericRangePolicy),
        Err(super::RuntimeBootstrapError::PolicyIdentityMismatch { .. })
    ));
}

#[test]
fn adoption_rejects_missing_keys_wrong_authorities_ids_and_metadata() {
    let backing = VectorMemory::default();
    let source = super::request_tests::snapshot(&[KEY], "db", 100, 110, &[(FIXED, 100)]);
    let mut runtime = MemoryRuntime::new(backing.clone()).unwrap();
    runtime.bootstrap(&source, &GenericRangePolicy).unwrap();
    let before = backing.borrow().clone();
    assert_eq!(
        runtime.verify_authority(&source, "absent"),
        Err(RuntimeAdoptionError::UnknownAuthority {
            authority: "absent".into()
        })
    );
    assert_eq!(
        runtime.verify_authority(&requirements("db", "db.missing.v1", None, None), "db"),
        Err(RuntimeAdoptionError::Open(
            RuntimeOpenError::StableKeyNotCommitted("db.missing.v1".into())
        ))
    );
    assert_eq!(
        runtime.verify_authority(&requirements("foreign", KEY, None, None), "foreign"),
        Err(RuntimeAdoptionError::AuthorityMismatch {
            stable_key: KEY.into(),
            committed_authority: "db".into(),
            requested_authority: "foreign".into(),
        })
    );
    assert_eq!(
        runtime.verify_authority(
            &requirements("db", FIXED, None, Some((110, "control"))),
            "db"
        ),
        Err(RuntimeAdoptionError::Open(
            RuntimeOpenError::MemoryIdMismatch {
                stable_key: FIXED.into(),
                committed_id: 100,
                requested_id: 110,
            }
        ))
    );
    for (key, schema, fixed) in [(KEY, Some(2), None), (FIXED, None, Some((100, "changed")))] {
        assert_eq!(
            runtime.verify_authority(&requirements("db", key, schema, fixed), "db"),
            Err(RuntimeAdoptionError::DeclarationMetadataMismatch {
                stable_key: key.into()
            })
        );
    }
    assert!(matches!(
        runtime.memory_id("ic_memory.ledger.v1"),
        Err(RuntimeOpenError::ReservedStableKey { .. })
    ));
    assert!(matches!(
        runtime.memory_id("INVALID"),
        Err(RuntimeOpenError::StableKey(_))
    ));
    assert_eq!(*backing.borrow(), before);
}
