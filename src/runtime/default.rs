use super::{
    MemoryManagerConfig, MemoryRuntime, RuntimeBootstrapError, RuntimeConstructionError,
    RuntimeDiagnosticError, RuntimeMemory, RuntimeOpenError, RuntimeStateError,
    policy::GenericRangePolicy,
};
use crate::{
    CommittedAllocations, DiagnosticExport, MemoryRuntimeDoctorReport, RuntimeBootstrapPolicy,
    physical::CommitStoreDiagnostic, registry::sealed_declaration_snapshot,
};
use ic_stable_structures::DefaultMemoryImpl;
use std::{cell::RefCell, convert::Infallible, fmt::Display};

thread_local! {
    static DEFAULT_RUNTIME:
        RefCell<Option<Result<MemoryRuntime<DefaultMemoryImpl>, RuntimeConstructionError>>> =
        const { RefCell::new(None) };
}

fn with_default_runtime_mut<T, E>(
    config: Option<MemoryManagerConfig>,
    operation: impl FnOnce(&mut MemoryRuntime<DefaultMemoryImpl>) -> Result<T, E>,
) -> Result<T, E>
where
    E: From<RuntimeStateError>,
{
    match DEFAULT_RUNTIME.try_with(|runtime| {
        let mut runtime = runtime
            .try_borrow_mut()
            .map_err(|_| E::from(RuntimeStateError::ReentrantAccess))?;
        let runtime = runtime
            .get_or_insert_with(|| match config {
                Some(config) => {
                    MemoryRuntime::new_with_config(DefaultMemoryImpl::default(), config)
                }
                None => MemoryRuntime::new(DefaultMemoryImpl::default()),
            })
            .as_mut()
            .map_err(|error| E::from(RuntimeStateError::Construction(*error)))?;
        if let Some(config) = config {
            super::check_bucket_size(runtime.bucket_size_pages, config)
                .map_err(|error| E::from(RuntimeStateError::Construction(error)))?;
        }
        operation(runtime)
    }) {
        Ok(result) => result,
        Err(_) => Err(E::from(RuntimeStateError::Unavailable)),
    }
}

// Observation must not choose a bucket configuration or initialize backing
// memory. Keep absence distinct from a cached construction failure.
fn with_existing_default_runtime<T, E>(
    operation: impl FnOnce(Option<&MemoryRuntime<DefaultMemoryImpl>>) -> Result<T, E>,
) -> Result<T, E>
where
    E: From<RuntimeStateError>,
{
    DEFAULT_RUNTIME
        .try_with(|runtime| {
            let runtime = runtime
                .try_borrow()
                .map_err(|_| E::from(RuntimeStateError::ReentrantAccess))?;
            let existing = runtime
                .as_ref()
                .map(|runtime| {
                    runtime
                        .as_ref()
                        .map_err(|error| E::from(RuntimeStateError::Construction(*error)))
                })
                .transpose()?;
            operation(existing)
        })
        .map_err(|_| E::from(RuntimeStateError::Unavailable))?
}

/// Return whether this thread's default runtime has completed bootstrap.
///
/// Does not construct an absent runtime or initialize backing memory. Returns
/// `false` for an absent or unbootstrapped runtime, preserving construction and
/// TLS access failures as typed errors.
pub fn is_default_memory_manager_bootstrapped() -> Result<bool, RuntimeStateError> {
    with_existing_default_runtime(|runtime| Ok(runtime.is_some_and(MemoryRuntime::is_bootstrapped)))
}

/// Return this thread's default runtime committed allocation capability.
///
/// Does not construct an absent runtime or initialize backing memory. Returns
/// `NotBootstrapped` for an absent or unbootstrapped runtime. This lookup can
/// precede configured bootstrap without selecting the default bucket size.
pub fn committed_allocations() -> Result<CommittedAllocations, RuntimeOpenError> {
    with_existing_default_runtime(|runtime| {
        runtime
            .ok_or(RuntimeOpenError::NotBootstrapped)?
            .committed_allocations()
            .cloned()
    })
}

/// Resolve an application key's committed ID in the existing default runtime.
/// Does not construct a manager, open memory, or choose a bucket configuration.
pub fn default_memory_manager_memory_id(stable_key: &str) -> Result<u8, RuntimeOpenError> {
    with_existing_default_runtime(|runtime| {
        runtime
            .ok_or(RuntimeOpenError::NotBootstrapped)?
            .memory_id(stable_key)
    })
}

/// Verify one consumer's allocation requirements against the existing host
/// runtime. Does not construct, bootstrap, replay admission or change configuration.
pub fn verify_default_memory_manager_authority(
    requirements: &crate::SealedDeclarationSnapshot,
    authority: &str,
) -> Result<(), super::RuntimeAdoptionError> {
    with_existing_default_runtime(|runtime| {
        runtime
            .ok_or(RuntimeOpenError::NotBootstrapped)?
            .verify_authority(requirements, authority)
    })
}

/// Bootstrap this thread's default runtime using generic range policy.
pub fn bootstrap_default_memory_manager()
-> Result<CommittedAllocations, RuntimeBootstrapError<Infallible>> {
    bootstrap_default_memory_manager_with_policy(&GenericRangePolicy)
}

/// Bootstrap this thread's default runtime with caller-supplied policy.
///
/// Static declarations are sealed once per linked program. Recovery, policy
/// evaluation, persistence, and capability publication occur once for this
/// concrete TLS runtime. Repeated calls must supply the policy identity bound
/// by the successful bootstrap.
pub fn bootstrap_default_memory_manager_with_policy<P: RuntimeBootstrapPolicy>(
    policy: &P,
) -> Result<CommittedAllocations, RuntimeBootstrapError<P::Error>> {
    let declarations = sealed_declaration_snapshot()?;
    with_default_runtime_mut(None, |runtime| {
        runtime.bootstrap(&declarations, policy).cloned()
    })
}

/// Open a committed memory from this thread's default runtime.
/// Does not construct an absent runtime or select its bucket configuration.
pub fn open_default_memory_manager_memory(
    stable_key: &str,
    id: u8,
) -> Result<RuntimeMemory<DefaultMemoryImpl>, RuntimeOpenError> {
    with_existing_default_runtime(|runtime| {
        runtime
            .ok_or(RuntimeOpenError::NotBootstrapped)?
            .open_memory(stable_key, id)
    })
}

/// Open a key already committed by the host's default runtime without changing policy.
/// Does not construct an absent runtime or select its bucket configuration.
pub fn open_default_memory_manager_memory_by_key(
    stable_key: &str,
) -> Result<RuntimeMemory<DefaultMemoryImpl>, RuntimeOpenError> {
    with_existing_default_runtime(|runtime| {
        runtime
            .ok_or(RuntimeOpenError::NotBootstrapped)?
            .open_memory_by_key(stable_key)
    })
}

/// Export this thread's default runtime ledger and live memory sizes.
///
/// Returns `NotBootstrapped` for an absent or unbootstrapped runtime without
/// initializing backing memory or choosing a bucket configuration.
pub fn default_memory_manager_diagnostic_export() -> Result<DiagnosticExport, RuntimeDiagnosticError>
{
    with_existing_default_runtime(|runtime| {
        runtime
            .ok_or(RuntimeDiagnosticError::NotBootstrapped)?
            .diagnostic_export()
    })
}

/// Diagnose protected commit recovery for this thread's default runtime.
///
/// Returns `NotBootstrapped` if no runtime exists without initializing memory
/// or choosing configuration. An existing runtime can be inspected before bootstrap.
pub fn default_memory_manager_commit_recovery_diagnostic()
-> Result<CommitStoreDiagnostic, RuntimeDiagnosticError> {
    with_existing_default_runtime(|runtime| {
        runtime
            .ok_or(RuntimeDiagnosticError::NotBootstrapped)?
            .commit_recovery_diagnostic()
    })
}

/// Build preflight and lifecycle diagnostics for this thread's default runtime.
///
/// Returns `NotBootstrapped` if no runtime exists without initializing memory
/// or choosing configuration. An existing runtime can be inspected before bootstrap.
pub fn default_memory_manager_doctor_report()
-> Result<MemoryRuntimeDoctorReport, RuntimeDiagnosticError> {
    default_memory_manager_doctor_report_with_policy(&GenericRangePolicy)
}

/// Build diagnostics for this thread's default runtime under one explicit policy.
///
/// Returns `NotBootstrapped` if no runtime exists without sealing declarations,
/// initializing memory or choosing configuration. The policy is evaluated only;
/// it does not construct or bootstrap the runtime.
pub fn default_memory_manager_doctor_report_with_policy<P>(
    policy: &P,
) -> Result<MemoryRuntimeDoctorReport, RuntimeDiagnosticError>
where
    P: RuntimeBootstrapPolicy,
    P::Error: Display,
{
    // Check presence before running registration hooks, but release the TLS
    // borrow so hooks can inspect the existing runtime during snapshot sealing.
    with_existing_default_runtime(|runtime| {
        runtime
            .ok_or(RuntimeDiagnosticError::NotBootstrapped)
            .map(|_| ())
    })?;
    let declarations = sealed_declaration_snapshot()?;
    with_existing_default_runtime(|runtime| {
        Ok(runtime
            .ok_or(RuntimeDiagnosticError::NotBootstrapped)?
            .doctor_report(&declarations, policy))
    })
}

#[cfg(test)]
pub(super) fn with_default_runtime_borrowed(
    operation: impl FnOnce() -> Result<(), RuntimeStateError>,
) -> Result<(), RuntimeStateError> {
    DEFAULT_RUNTIME.with(|runtime| {
        let _borrow = runtime.borrow_mut();
        operation()
    })
}

/// Measure the existing default runtime without constructing a manager or
/// initializing backing memory.
///
/// Returns `NotBootstrapped` if no runtime exists.
/// A constructed runtime may be measured before bootstrap with unknown bindings.
pub fn default_memory_manager_memory_allocations()
-> Result<super::MemoryAllocations, RuntimeDiagnosticError> {
    with_existing_default_runtime(|runtime| {
        runtime
            .ok_or(RuntimeDiagnosticError::NotBootstrapped)?
            .memory_allocations()
    })
}

/// Measure numeric allocation totals in the existing default runtime.
///
/// Does not copy binding names or construct per-ID rows. Absence returns
/// `NotBootstrapped` without initializing memory or choosing configuration.
pub fn default_memory_manager_memory_allocation_summary()
-> Result<super::MemoryAllocationSummary, RuntimeDiagnosticError> {
    with_existing_default_runtime(|runtime| {
        runtime
            .ok_or(RuntimeDiagnosticError::NotBootstrapped)?
            .memory_allocation_summary()
    })
}

/// Bootstrap the default runtime with an explicit bucket setting and allocation
/// policy.
///
/// The first construction uses this setting; repeated calls and reopened
/// memory must match it exactly before bootstrap effects. Select this setting
/// on the first bootstrap; observation and open helpers leave an absent runtime
/// untouched.
/// Registration hooks run without a TLS borrow and may observe the configured,
/// unbootstrapped runtime.
/// Use [`super::GenericRangePolicy`] to select the built-in policy, or pass the
/// host's custom policy. This operation does not adopt a different bound policy.
pub fn bootstrap_default_memory_manager_with_config<P: RuntimeBootstrapPolicy>(
    config: MemoryManagerConfig,
    policy: &P,
) -> Result<CommittedAllocations, RuntimeBootstrapError<P::Error>> {
    // Reject construction/configuration failures before sealing, but release
    // the TLS borrow while registration hooks inspect the existing runtime.
    with_default_runtime_mut(Some(config), |_| Ok::<_, RuntimeStateError>(()))?;
    let declarations = sealed_declaration_snapshot()?;
    with_default_runtime_mut(Some(config), |runtime| {
        runtime.bootstrap(&declarations, policy).cloned()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configured_registration_hooks_can_observe_unbootstrapped_runtime() {
        use crate::registry::{
            TEST_REGISTRY_LOCK, defer_eager_init, reset_static_memory_declarations_for_tests,
        };
        let _guard = TEST_REGISTRY_LOCK.lock().unwrap();
        reset_static_memory_declarations_for_tests();
        defer_eager_init(|| {
            assert_eq!(is_default_memory_manager_bootstrapped(), Ok(false));
            assert_eq!(
                committed_allocations(),
                Err(RuntimeOpenError::NotBootstrapped)
            );
            let summary = default_memory_manager_memory_allocation_summary().unwrap();
            assert_eq!(summary.bucket_size_pages, 16);
            assert_eq!(summary.current_generation, None);
        });
        std::thread::spawn(|| {
            let config = super::super::MemoryManagerConfig::new(16).unwrap();
            bootstrap_default_memory_manager_with_config(config, &GenericRangePolicy).unwrap();
            assert_eq!(is_default_memory_manager_bootstrapped(), Ok(true));
        })
        .join()
        .unwrap();
        reset_static_memory_declarations_for_tests();
    }

    fn diagnostic_observations() -> [Result<(), RuntimeDiagnosticError>; 4] {
        [
            default_memory_manager_diagnostic_export().map(|_| ()),
            default_memory_manager_commit_recovery_diagnostic().map(|_| ()),
            default_memory_manager_doctor_report().map(|_| ()),
            default_memory_manager_doctor_report_with_policy(&GenericRangePolicy).map(|_| ()),
        ]
    }

    #[test]
    fn observations_leave_an_absent_runtime_absent() {
        use crate::registry::{
            TEST_REGISTRY_LOCK, defer_eager_init, reset_static_memory_declarations_for_tests,
        };
        use std::sync::atomic::{AtomicBool, Ordering};
        static REGISTRATION_RAN: AtomicBool = AtomicBool::new(false);
        let _guard = TEST_REGISTRY_LOCK.lock().unwrap();
        reset_static_memory_declarations_for_tests();
        defer_eager_init(|| REGISTRATION_RAN.store(true, Ordering::SeqCst));
        std::thread::spawn(|| {
            for _ in 0..2 {
                assert!(!is_default_memory_manager_bootstrapped().unwrap());
                assert_eq!(
                    committed_allocations(),
                    Err(RuntimeOpenError::NotBootstrapped)
                );
                assert!(matches!(
                    default_memory_manager_memory_allocations(),
                    Err(RuntimeDiagnosticError::NotBootstrapped)
                ));
                for result in diagnostic_observations() {
                    assert!(matches!(
                        result,
                        Err(RuntimeDiagnosticError::NotBootstrapped)
                    ));
                }
                DEFAULT_RUNTIME.with(|runtime| assert!(runtime.borrow().is_none()));
            }
        })
        .join()
        .unwrap();
        assert!(!REGISTRATION_RAN.load(Ordering::SeqCst));
        reset_static_memory_declarations_for_tests();
    }

    #[test]
    fn observations_preserve_unbootstrapped_configuration() {
        use crate::registry::{TEST_REGISTRY_LOCK, reset_static_memory_declarations_for_tests};
        let _guard = TEST_REGISTRY_LOCK.lock().unwrap();
        reset_static_memory_declarations_for_tests();
        std::thread::spawn(|| {
            let config = super::super::MemoryManagerConfig::new(16).unwrap();
            DEFAULT_RUNTIME.with(|runtime| {
                *runtime.borrow_mut() = Some(MemoryRuntime::new_with_config(
                    DefaultMemoryImpl::default(),
                    config,
                ));
            });
            let before = default_memory_manager_memory_allocations().unwrap();
            assert!(!is_default_memory_manager_bootstrapped().unwrap());
            assert_eq!(
                committed_allocations(),
                Err(RuntimeOpenError::NotBootstrapped)
            );
            assert_eq!(before.bucket_size_pages, 16);
            assert!(matches!(
                default_memory_manager_diagnostic_export(),
                Err(RuntimeDiagnosticError::NotBootstrapped)
            ));
            default_memory_manager_commit_recovery_diagnostic().unwrap();
            assert!(!default_memory_manager_doctor_report().unwrap().bootstrapped);
            assert!(
                !default_memory_manager_doctor_report_with_policy(&GenericRangePolicy)
                    .unwrap()
                    .bootstrapped
            );
            assert_eq!(default_memory_manager_memory_allocations().unwrap(), before);
        })
        .join()
        .unwrap();
        reset_static_memory_declarations_for_tests();
    }

    #[test]
    fn observations_preserve_cached_construction_failure() {
        std::thread::spawn(|| {
            let error = RuntimeConstructionError::ForeignMemory {
                observed_magic: *b"BAD",
            };
            DEFAULT_RUNTIME.with(|runtime| *runtime.borrow_mut() = Some(Err(error)));
            for result in diagnostic_observations() {
                assert!(matches!(result, Err(RuntimeDiagnosticError::State(RuntimeStateError::Construction(cause))) if cause == error));
            }
            assert_eq!(
                is_default_memory_manager_bootstrapped(),
                Err(RuntimeStateError::Construction(error))
            );
            assert_eq!(
                committed_allocations(),
                Err(RuntimeOpenError::State(RuntimeStateError::Construction(
                    error
                )))
            );
            assert!(matches!(
                default_memory_manager_memory_allocations(),
                Err(RuntimeDiagnosticError::State(RuntimeStateError::Construction(cause)))
                    if cause == error
            ));
            for result in [
                open_default_memory_manager_memory_by_key("app.rows.v1").err(),
                open_default_memory_manager_memory("app.rows.v1", 100).err(),
                default_memory_manager_memory_id("app.rows.v1").err(),
            ] {
                assert_eq!(result, Some(RuntimeOpenError::State(RuntimeStateError::Construction(error))));
            }
            let requirements = crate::SealedDeclarationSnapshot::new(&[], &[], &[]).unwrap();
            assert_eq!(verify_default_memory_manager_authority(&requirements, "app"), Err(super::super::RuntimeAdoptionError::Open(RuntimeOpenError::State(RuntimeStateError::Construction(error)))));
            assert!(matches!(default_memory_manager_memory_allocation_summary(), Err(RuntimeDiagnosticError::State(RuntimeStateError::Construction(cause))) if cause == error));
            DEFAULT_RUNTIME.with(|runtime| {
                assert!(matches!(runtime.borrow().as_ref(), Some(Err(cause)) if *cause == error));
            });
        })
        .join()
        .unwrap();
    }

    #[test]
    fn diagnostic_observations_preserve_reentrant_access_errors() {
        with_default_runtime_borrowed(|| {
            for result in diagnostic_observations() {
                assert!(matches!(
                    result,
                    Err(RuntimeDiagnosticError::State(
                        RuntimeStateError::ReentrantAccess
                    ))
                ));
            }
            Ok(())
        })
        .unwrap();
    }
    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn configured_default_prepares_once_and_warm_library_adoption_only_opens() {
        use crate::registry::{TEST_REGISTRY_LOCK, reset_static_memory_declarations_for_tests};
        use ic_stable_structures::Memory;
        let _guard = TEST_REGISTRY_LOCK.lock().unwrap();
        reset_static_memory_declarations_for_tests();
        crate::register_static_memory_manager_range(
            100,
            110,
            "app",
            crate::MemoryManagerRangeMode::Allowed,
            None,
        )
        .unwrap();
        crate::register_memory_request(
            crate::MemoryRequest::new(
                "app",
                "app.main.control.v1",
                crate::SchemaMetadata::default(),
            )
            .unwrap(),
        )
        .unwrap();
        let backing = super::super::admission_tests::seeded();
        DEFAULT_RUNTIME
            .with(|runtime| *runtime.borrow_mut() = Some(MemoryRuntime::new(backing.clone())));
        let policy = super::super::admission_tests::AdmissionPolicy {
            discover: true,
            ..Default::default()
        };
        let config = super::super::MemoryManagerConfig::new(1).unwrap();
        let committed = bootstrap_default_memory_manager_with_config(config, &policy).unwrap();
        assert_eq!(committed.generation(), 2);
        let before = backing.borrow().clone();
        let mut marker = [0; 12];
        open_default_memory_manager_memory_by_key("app.old.journal.v1")
            .unwrap()
            .read(0, &mut marker);
        assert_eq!(&marker, b"pending/debt");
        assert_eq!(committed_allocations().unwrap(), committed);
        assert_eq!(
            bootstrap_default_memory_manager_with_config(config, &policy).unwrap(),
            committed
        );
        assert!(
            bootstrap_default_memory_manager_with_config(
                super::super::MemoryManagerConfig::new(2).unwrap(),
                &policy
            )
            .is_err()
        );
        assert_eq!(policy.calls.get(), 1);
        assert_eq!(*backing.borrow(), before);
        assert_eq!(
            default_memory_manager_memory_allocations()
                .unwrap()
                .bucket_size_pages,
            1
        );
        DEFAULT_RUNTIME.with(|runtime| *runtime.borrow_mut() = None);
        reset_static_memory_declarations_for_tests();
    }
}
