//! Explicit fixture-only cell setup; callers own codecs, bounds and lifecycle calls.

use crate::{
    GenericAllocationPolicy, MemoryAllocationPool, MemoryAuthority, MemoryManagerConfig,
    MemoryRequest, MemoryRuntime, RuntimeMemory, SchemaMetadata, SealedDeclarationSnapshot,
    ic_stable_structures::{Cell, Memory},
};

///
/// FixtureCell
///
/// One byte cell and its retained allocation runtime, opened only by an explicit
/// caller. Namespace grants and the stable key are fixture inputs, not product
/// authority. Uses the ordinary runtime, ledger and Cell formats without a new
/// journal schema. No global memory manager, endpoint or lifecycle is installed.
///
/// Fresh memory has an empty cell. Restored callers must reject empty/invalid
/// bytes where their journal contract requires retained state. Callers own
/// serialization, byte/work budgets and same-release recovery checks.
///

pub struct FixtureCell<M: Memory> {
    _runtime: MemoryRuntime<M>,
    cell: Cell<Vec<u8>, RuntimeMemory<M>>,
}

///
/// FixtureCellError
///
/// Typed admission failure for an explicitly requested fixture byte cell.
/// Ordinary runtime errors retain their original sources.
///

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum FixtureCellError {
    /// The request or declaration snapshot is invalid.
    #[error(transparent)]
    Declaration(#[from] crate::MemoryDeclarationError),
    /// The authority grant or pool is invalid.
    #[error(transparent)]
    AllocationPool(#[from] crate::MemoryAllocationPoolError),
    /// Backing layout or manager configuration admission failed.
    #[error(transparent)]
    Construction(#[from] crate::RuntimeConstructionError),
    /// Allocation bootstrap or current-ledger recovery failed.
    #[error(transparent)]
    Bootstrap(Box<crate::RuntimeBootstrapError<std::convert::Infallible>>),
    /// The committed allocation does not satisfy the requested authority.
    #[error(transparent)]
    Authority(#[from] crate::RuntimeAdoptionError),
    /// The selected key cannot be opened.
    #[error(transparent)]
    Open(#[from] crate::RuntimeOpenError),
    /// Existing cell metadata is invalid.
    #[error(transparent)]
    Payload(#[from] crate::StableCellPayloadError),
}

impl<M: Memory> FixtureCell<M> {
    /// Bootstrap one caller-selected fixture key and open its existing byte cell.
    ///
    /// # Panics
    /// Panics on invalid fixture declarations/grants, incompatible persisted
    /// configuration, failed runtime bootstrap/open or invalid Cell metadata.
    /// It never substitutes fresh backing memory after such a failure.
    pub fn new(
        memory: M,
        owner: &str,
        namespace: &str,
        stable_key: &str,
        config: MemoryManagerConfig,
    ) -> Self {
        Self::try_new(memory, owner, namespace, stable_key, config).expect("fixture cell setup")
    }

    /// Fallibly bootstrap and open a trusted fixture byte cell.
    ///
    /// Existing Cell metadata is validated before initialization can replace it.
    /// This opener is unbounded: callers must supply trusted images and enforce
    /// their own payload decode/write ceilings and initialization policy.
    ///
    /// # Errors
    /// Returns typed request, configuration, bootstrap, authority, opening and
    /// envelope failures. Bootstrap may advance ledger metadata before a later
    /// envelope refusal; no failure substitutes fresh backing memory.
    ///
    /// # Panics
    /// Raw Memory operations and initial Cell publication retain their ordinary
    /// substrate panic behavior.
    pub fn try_new(
        memory: M,
        owner: &str,
        namespace: &str,
        stable_key: &str,
        config: MemoryManagerConfig,
    ) -> Result<Self, FixtureCellError> {
        let request = MemoryRequest::new(owner, stable_key, SchemaMetadata::default())?;
        let pool =
            MemoryAllocationPool::new(vec![MemoryAuthority::new(owner, namespace)?], vec![])?;
        let declarations = SealedDeclarationSnapshot::new(vec![request])?;
        let mut runtime = MemoryRuntime::new_with_config(memory, config)?;
        runtime
            .bootstrap(&declarations, &pool, &GenericAllocationPolicy)
            .map_err(|error| FixtureCellError::Bootstrap(Box::new(error)))?;
        runtime.verify_authority(&declarations, owner)?;
        let memory = runtime.open_memory(stable_key)?;
        if memory.size() > 0 {
            crate::stable_cell::stable_cell_payload_length(&memory, usize::MAX)?;
        }
        let cell = Cell::init(memory, Vec::new());
        Ok(Self {
            _runtime: runtime,
            cell,
        })
    }

    /// Transfer the admitted virtual memory for caller-owned raw inspection.
    /// This consumes the fixture without another bootstrap, reset or cleanup.
    #[must_use]
    pub fn into_memory(self) -> RuntimeMemory<M> {
        self.cell.into_memory()
    }

    /// Borrow the original cell bytes without decoding, defaulting or copying them.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        self.cell.get()
    }

    /// Persist caller-encoded bytes; the caller must enforce its journal budget first.
    pub fn set(&mut self, bytes: Vec<u8>) {
        self.cell.set(bytes);
    }
}

#[cfg(test)]
mod tests;
