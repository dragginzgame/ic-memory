use super::{
    MemoryRuntime, RuntimeBootstrapError, RuntimeConstructionError, RuntimeDiagnosticError,
    RuntimeMemory, RuntimeOpenError, RuntimeStateError, policy::GenericRangePolicy,
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

fn with_default_runtime<T, E>(
    operation: impl FnOnce(&MemoryRuntime<DefaultMemoryImpl>) -> Result<T, E>,
) -> Result<T, E>
where
    E: From<RuntimeStateError>,
{
    match DEFAULT_RUNTIME.try_with(|runtime| {
        let mut runtime = runtime
            .try_borrow_mut()
            .map_err(|_| E::from(RuntimeStateError::ReentrantAccess))?;
        let runtime = runtime
            .get_or_insert_with(|| MemoryRuntime::new(DefaultMemoryImpl::default()))
            .as_ref()
            .map_err(|error| E::from(RuntimeStateError::Construction(*error)))?;
        operation(runtime)
    }) {
        Ok(result) => result,
        Err(_) => Err(E::from(RuntimeStateError::Unavailable)),
    }
}

fn with_default_runtime_mut<T, E>(
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
            .get_or_insert_with(|| MemoryRuntime::new(DefaultMemoryImpl::default()))
            .as_mut()
            .map_err(|error| E::from(RuntimeStateError::Construction(*error)))?;
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
    with_default_runtime_mut(|runtime| runtime.bootstrap(&declarations, policy).cloned())
}

/// Open a committed memory from this thread's default runtime.
pub fn open_default_memory_manager_memory(
    stable_key: &str,
    id: u8,
) -> Result<RuntimeMemory<DefaultMemoryImpl>, RuntimeOpenError> {
    with_default_runtime(|runtime| runtime.open_memory(stable_key, id))
}

/// Open a key already committed by the host's default runtime without changing policy.
pub fn open_default_memory_manager_memory_by_key(
    stable_key: &str,
) -> Result<RuntimeMemory<DefaultMemoryImpl>, RuntimeOpenError> {
    with_default_runtime(|runtime| runtime.open_memory_by_key(stable_key))
}

/// Export this thread's default runtime ledger and live memory sizes.
pub fn default_memory_manager_diagnostic_export() -> Result<DiagnosticExport, RuntimeDiagnosticError>
{
    with_default_runtime(MemoryRuntime::diagnostic_export)
}

/// Diagnose protected commit recovery for this thread's default runtime.
pub fn default_memory_manager_commit_recovery_diagnostic()
-> Result<CommitStoreDiagnostic, RuntimeDiagnosticError> {
    with_default_runtime(MemoryRuntime::commit_recovery_diagnostic)
}

/// Build preflight and lifecycle diagnostics for this thread's default runtime.
pub fn default_memory_manager_doctor_report()
-> Result<MemoryRuntimeDoctorReport, RuntimeDiagnosticError> {
    default_memory_manager_doctor_report_with_policy(&GenericRangePolicy)
}

/// Build diagnostics for this thread's default runtime under one explicit policy.
pub fn default_memory_manager_doctor_report_with_policy<P>(
    policy: &P,
) -> Result<MemoryRuntimeDoctorReport, RuntimeDiagnosticError>
where
    P: RuntimeBootstrapPolicy,
    P::Error: Display,
{
    let declarations = sealed_declaration_snapshot()?;
    with_default_runtime(|runtime| Ok(runtime.doctor_report(&declarations, policy)))
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

/// Bootstrap the default runtime with an explicit bucket setting and allocation
/// policy.
///
/// The first construction uses this setting; repeated calls and reopened
/// memory must match it exactly before bootstrap effects. Call this during
/// bootstrap before any operation that would construct the default runtime.
/// Use [`super::GenericRangePolicy`] to select the built-in policy, or pass the
/// host's custom policy. This operation does not adopt a different bound policy.
pub fn bootstrap_default_memory_manager_with_config<P: RuntimeBootstrapPolicy>(
    config: super::MemoryManagerConfig,
    policy: &P,
) -> Result<CommittedAllocations, RuntimeBootstrapError<P::Error>> {
    DEFAULT_RUNTIME
        .try_with(|runtime| {
            let mut runtime = runtime
                .try_borrow_mut()
                .map_err(|_| RuntimeStateError::ReentrantAccess)?;
            let runtime = runtime
                .get_or_insert_with(|| {
                    MemoryRuntime::new_with_config(DefaultMemoryImpl::default(), config)
                })
                .as_mut()
                .map_err(|error| RuntimeStateError::Construction(*error))?;
            super::check_bucket_size(runtime.bucket_size_pages, config)
                .map_err(RuntimeStateError::Construction)?;
            let declarations = sealed_declaration_snapshot()?;
            runtime.bootstrap(&declarations, policy).cloned()
        })
        .map_err(|_| RuntimeStateError::Unavailable)?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observations_leave_an_absent_runtime_absent() {
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
                DEFAULT_RUNTIME.with(|runtime| assert!(runtime.borrow().is_none()));
            }
        })
        .join()
        .unwrap();
    }

    #[test]
    fn observations_preserve_unbootstrapped_configuration() {
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
            assert_eq!(default_memory_manager_memory_allocations().unwrap(), before);
        })
        .join()
        .unwrap();
    }

    #[test]
    fn observations_preserve_cached_construction_failure() {
        std::thread::spawn(|| {
            let error = RuntimeConstructionError::ForeignMemory {
                observed_magic: *b"BAD",
            };
            DEFAULT_RUNTIME.with(|runtime| *runtime.borrow_mut() = Some(Err(error)));
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
            DEFAULT_RUNTIME.with(|runtime| {
                assert!(matches!(runtime.borrow().as_ref(), Some(Err(cause)) if *cause == error));
            });
        })
        .join()
        .unwrap();
    }
}
