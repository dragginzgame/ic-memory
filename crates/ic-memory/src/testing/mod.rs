//! Explicit fixture-only cell setup; callers own codecs, ceilings and lifecycle calls.

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
/// Use [`Self::create_bounded`] and [`Self::open_bounded`] for an enforced byte
/// ceiling and explicit fresh/retained admission. The retained opener admits the
/// envelope before Cell allocates its value. Ordinary runtime bootstrap may
/// advance allocation-ledger metadata; journal refusal never resets cell bytes.
///

pub struct FixtureCell<M: Memory> {
    _runtime: MemoryRuntime<M>,
    cell: Cell<Vec<u8>, RuntimeMemory<M>>,
    maximum: usize,
}

///
/// FixtureCellError
///
/// Typed admission failure for an explicitly requested fixture byte cell.
/// The ordinary allocation and ledger errors retain their original sources.
///

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum FixtureCellError {
    /// The caller's request or declaration snapshot is invalid.
    #[error(transparent)]
    Declaration(#[from] crate::MemoryDeclarationError),
    /// The caller's authority grant or pool is invalid.
    #[error(transparent)]
    AllocationPool(#[from] crate::MemoryAllocationPoolError),
    /// Backing layout or manager configuration admission failed.
    #[error(transparent)]
    Construction(#[from] crate::RuntimeConstructionError),
    /// Current ledger recovery or allocation bootstrap failed.
    #[error(transparent)]
    Bootstrap(Box<crate::RuntimeBootstrapError<std::convert::Infallible>>),
    /// The committed allocation does not satisfy the requested authority.
    #[error(transparent)]
    Authority(#[from] crate::RuntimeAdoptionError),
    /// The selected key cannot be opened.
    #[error(transparent)]
    Open(#[from] crate::RuntimeOpenError),
    /// Existing cell metadata or the caller's byte ceiling is invalid.
    #[error(transparent)]
    Payload(#[from] crate::StableCellPayloadError),
    /// Retained opening was requested without backing memory.
    #[error("fixture retained backing is missing")]
    MissingBacking,
    /// Fresh creation was requested for an allocated key.
    #[error("fixture initialization cannot reset history")]
    Occupied,
}

impl<M: Memory> FixtureCell<M> {
    /// Bootstrap one caller-selected fixture key and open its existing byte cell.
    /// This unbounded constructor is only suitable for trusted fixture images;
    /// application decode limits do not bound Cell's initial payload allocation.
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
    /// This preserves the ordinary runtime's typed declaration, configuration,
    /// ledger, authority and cell-envelope refusal. It never replaces backing
    /// memory after failure. This constructor is unbounded; callers must supply
    /// a trusted image and enforce their journal's decode and write ceilings.
    ///
    /// # Errors
    /// Returns the original admission failure through [`FixtureCellError`].
    /// Bootstrap can advance ledger metadata before later cell refusal.
    ///
    /// # Panics
    /// Backing Memory operations and initial Cell publication retain their
    /// substrate panic behavior, as with the ordinary runtime and Cell APIs.
    pub fn try_new(
        memory: M,
        owner: &str,
        namespace: &str,
        stable_key: &str,
        config: MemoryManagerConfig,
    ) -> Result<Self, FixtureCellError> {
        Self::open(memory, owner, namespace, stable_key, config, None)
    }

    /// Create a bounded empty cell, refusing any previously allocated key.
    ///
    /// # Panics
    /// Panics on occupied memory or the same setup failures as [`Self::new`].
    pub fn create_bounded(
        memory: M,
        owner: &str,
        namespace: &str,
        stable_key: &str,
        config: MemoryManagerConfig,
        maximum: usize,
    ) -> Self {
        Self::open(
            memory,
            owner,
            namespace,
            stable_key,
            config,
            Some((maximum, true)),
        )
        .expect("fixture cell admission")
    }

    /// Open a retained cell after admitting its header and caller byte ceiling.
    ///
    /// The length is checked before `Cell::init` can allocate or read its value.
    /// No empty-memory default or incompatible-envelope reset is permitted.
    /// Callers still validate the payload's identity, codec and invariants.
    ///
    /// # Panics
    /// Panics on missing/corrupt/oversized cells or setup failures in [`Self::new`].
    pub fn open_bounded(
        memory: M,
        owner: &str,
        namespace: &str,
        stable_key: &str,
        config: MemoryManagerConfig,
        maximum: usize,
    ) -> Self {
        Self::open(
            memory,
            owner,
            namespace,
            stable_key,
            config,
            Some((maximum, false)),
        )
        .expect("fixture cell admission")
    }

    fn open(
        memory: M,
        owner: &str,
        namespace: &str,
        stable_key: &str,
        config: MemoryManagerConfig,
        admission: Option<(usize, bool)>,
    ) -> Result<Self, FixtureCellError> {
        if matches!(admission, Some((_, false))) && memory.size() == 0 {
            return Err(FixtureCellError::MissingBacking);
        }
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
        if let Some((maximum, fresh)) = admission {
            if fresh {
                if memory.size() != 0 {
                    return Err(FixtureCellError::Occupied);
                }
            } else {
                crate::stable_cell::stable_cell_payload_length(&memory, maximum)?;
            }
        }
        if admission.is_none() && memory.size() > 0 {
            crate::stable_cell::stable_cell_payload_length(&memory, usize::MAX)?;
        }
        let cell = Cell::init(memory, Vec::new());
        Ok(Self {
            _runtime: runtime,
            cell,
            maximum: admission.map_or(usize::MAX, |(maximum, _)| maximum),
        })
    }

    /// Consume this fixture and transfer its admitted virtual memory to the caller.
    ///
    /// The returned grant retains the backing manager. This performs no bootstrap,
    /// reset or cleanup; callers own any subsequent raw-memory inspection.
    #[must_use]
    pub fn into_memory(self) -> RuntimeMemory<M> {
        self.cell.into_memory()
    }

    /// Borrow the original cell bytes without decoding, defaulting or copying them.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        self.cell.get()
    }

    /// Persist caller-encoded bytes, refusing the selected ceiling before writing.
    ///
    /// # Panics
    /// Panics if bytes exceed the ceiling or Cell persistence fails.
    pub fn set(&mut self, bytes: Vec<u8>) {
        assert!(bytes.len() <= self.maximum, "fixture cell byte bound");
        self.cell.set(bytes);
    }
}

#[cfg(test)]
mod tests;
