//! Fixture-only runtime bootstrap shared by cells and collection qualification.
use crate::{
    GenericAllocationPolicy, MemoryAllocationPool, MemoryAuthority, MemoryManagerConfig,
    MemoryRequest, MemoryRuntime, SealedDeclarationSnapshot, ic_stable_structures::Memory,
};

/// Failure to construct and admit an explicitly configured fixture runtime.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum FixtureRuntimeError {
    /// Invalid owner, stable key or declaration snapshot.
    #[error(transparent)]
    Declaration(#[from] crate::MemoryDeclarationError),
    /// Invalid namespace grant or allocation pool.
    #[error(transparent)]
    AllocationPool(#[from] crate::MemoryAllocationPoolError),
    /// Backing layout or configured geometry was refused.
    #[error(transparent)]
    Construction(#[from] crate::RuntimeConstructionError),
    /// Runtime bootstrap or protected ledger recovery failed.
    #[error(transparent)]
    Bootstrap(Box<crate::RuntimeBootstrapError<std::convert::Infallible>>),
}

/// Bootstrap explicitly named fixture allocations using the ordinary runtime.
///
/// The caller supplies backing memory, one owner/namespace, stable keys and bucket
/// geometry. No Cell or collection is initialized. Returns the runtime so callers
/// can open multiple committed allocations or exercise rejected opens. Retained
/// backing is admitted by normal bootstrap; errors never trigger a reset/fallback.
/// This fixture helper uses the generic allocation policy and no exclusions;
/// production policy/composition callers must configure their own runtime.
///
/// # Errors
/// Preserves typed declaration, pool, construction and bootstrap failures.
/// Bootstrap retains its ordinary ledger publication and recovery behavior.
/// # Panics
/// Backing memory IO retains its substrate panic behavior.
pub fn fixture_runtime<M: Memory>(
    backing: M,
    owner: &str,
    namespace: &str,
    stable_keys: &[&str],
    config: MemoryManagerConfig,
) -> Result<MemoryRuntime<M>, FixtureRuntimeError> {
    let requests = stable_keys
        .iter()
        .map(|key| MemoryRequest::new(owner, key))
        .collect::<Result<Vec<_>, _>>()?;
    let pool = MemoryAllocationPool::new(vec![MemoryAuthority::new(owner, namespace)?], vec![])?;
    let declarations = SealedDeclarationSnapshot::new(requests)?;
    let mut runtime = MemoryRuntime::new_with_config(backing, config)?;
    runtime
        .bootstrap(&declarations, &pool, &GenericAllocationPolicy)
        .map_err(|error| FixtureRuntimeError::Bootstrap(Box::new(error)))?;
    Ok(runtime)
}

// Preserve the existing cell API and its error variants while sharing setup.
impl From<FixtureRuntimeError> for super::FixtureCellError {
    fn from(error: FixtureRuntimeError) -> Self {
        match error {
            FixtureRuntimeError::Declaration(error) => Self::Declaration(error),
            FixtureRuntimeError::AllocationPool(error) => Self::AllocationPool(error),
            FixtureRuntimeError::Construction(error) => Self::Construction(error),
            FixtureRuntimeError::Bootstrap(error) => Self::Bootstrap(error),
        }
    }
}
