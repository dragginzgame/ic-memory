use super::{
    MemoryRuntime, RuntimeBootstrapError, RuntimeDiagnosticError, RuntimeOpenError,
    RuntimeStateError, policy::NoopPolicy,
};
use crate::{
    CommittedAllocations, DiagnosticExport, MemoryRuntimeDoctorReport, RuntimeBootstrapPolicy,
    physical::CommitStoreDiagnostic, registry::sealed_declaration_snapshot,
};
use ic_stable_structures::{DefaultMemoryImpl, memory_manager::VirtualMemory};
use std::{cell::RefCell, convert::Infallible};

thread_local! {
    static DEFAULT_RUNTIME: RefCell<MemoryRuntime<DefaultMemoryImpl>> =
        RefCell::new(MemoryRuntime::new(DefaultMemoryImpl::default()));
}

fn with_default_runtime<T, E>(
    operation: impl FnOnce(&MemoryRuntime<DefaultMemoryImpl>) -> Result<T, E>,
) -> Result<T, E>
where
    E: From<RuntimeStateError>,
{
    match DEFAULT_RUNTIME.try_with(|runtime| {
        let runtime = runtime
            .try_borrow()
            .map_err(|_| E::from(RuntimeStateError::ReentrantAccess))?;
        operation(&runtime)
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
        operation(&mut runtime)
    }) {
        Ok(result) => result,
        Err(_) => Err(E::from(RuntimeStateError::Unavailable)),
    }
}

/// Return whether this thread's default runtime has completed bootstrap.
pub fn is_default_memory_manager_bootstrapped() -> Result<bool, RuntimeStateError> {
    with_default_runtime(|runtime| Ok(runtime.is_bootstrapped()))
}

/// Return this thread's default runtime committed allocation capability.
pub fn committed_allocations() -> Result<CommittedAllocations, RuntimeOpenError> {
    with_default_runtime(|runtime| runtime.committed_allocations().cloned())
}

/// Bootstrap this thread's default runtime using generic range policy.
pub fn bootstrap_default_memory_manager()
-> Result<CommittedAllocations, RuntimeBootstrapError<Infallible>> {
    bootstrap_default_memory_manager_with_policy(&NoopPolicy)
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
) -> Result<VirtualMemory<DefaultMemoryImpl>, RuntimeOpenError> {
    with_default_runtime(|runtime| runtime.open_memory(stable_key, id))
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
    let declarations = sealed_declaration_snapshot()?;
    with_default_runtime(|runtime| Ok(runtime.doctor_report(&declarations)))
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
