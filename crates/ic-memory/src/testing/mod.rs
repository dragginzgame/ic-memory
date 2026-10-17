//! Explicit fixture-only ledger and cell setup; callers own codecs, bounds and lifecycle calls.

mod runtime;
pub use runtime::{FixtureRuntimeError, fixture_runtime};

use crate::{
    MemoryManagerConfig, RuntimeMemory,
    ic_stable_structures::{Cell, Memory},
};

/// Construct a passive fixture record from checked declaration fields and a chosen state.
///
/// This performs no transition, generation advance, persistence or capability
/// publication. Compose records with [`crate::AllocationLedger::new`] to check
/// ledger-wide uniqueness and genesis rules before encoding fixture bytes.
#[must_use]
pub fn allocation_record(
    declaration: &crate::AllocationDeclaration,
    state: crate::AllocationState,
) -> crate::AllocationRecord {
    crate::AllocationRecord::from_declaration(declaration, state)
}

///
/// FixtureCell
///
/// One byte cell opened through an allocation runtime only by an explicit
/// caller. Namespace grants and the stable key are fixture inputs, not product
/// authority. Uses the ordinary runtime, ledger and Cell formats without a new
/// journal schema. The cell's memory handle retains the backing and manager;
/// bootstrap declarations and policy state are released after opening. No global
/// memory manager, endpoint or lifecycle is installed.
///
/// Fresh memory has an empty cell. Restored callers must reject empty/invalid
/// bytes where their journal contract requires retained state. Callers own
/// serialization, byte/work budgets and same-release recovery checks.
/// [`Self::create_bounded`] and [`Self::open_bounded`] enforce explicit fresh or
/// retained admission plus a caller-selected byte ceiling before payload loading.
/// [`Self::try_new`] is the fallible trusted-image constructor.
///

pub struct FixtureCell<M: Memory> {
    cell: Cell<Vec<u8>, RuntimeMemory<M>>,
    maximum: usize,
}

///
/// FixtureCellError
///
/// Typed admission failure for an explicitly requested fixture byte cell.
/// Runtime sources remain available; payload refusal does not reset cell bytes.
///

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum FixtureCellError {
    /// Invalid request or declaration snapshot.
    #[error(transparent)]
    Declaration(#[from] crate::MemoryDeclarationError),
    /// Invalid namespace grant or allocation pool.
    #[error(transparent)]
    AllocationPool(#[from] crate::MemoryAllocationPoolError),
    /// Backing layout or bucket configuration was refused.
    #[error(transparent)]
    Construction(#[from] crate::RuntimeConstructionError),
    /// Runtime bootstrap or protected ledger recovery failed.
    #[error(transparent)]
    Bootstrap(Box<crate::RuntimeBootstrapError<std::convert::Infallible>>),
    /// The committed key cannot be opened.
    #[error(transparent)]
    Open(#[from] crate::RuntimeOpenError),
    /// Missing, malformed or oversized retained Cell metadata.
    #[error(transparent)]
    Payload(#[from] crate::StableCellPayloadError),
    /// Fresh-cell creation cannot replace an occupied allocation.
    #[error("fixture allocation is already occupied")]
    Occupied,
    /// Candidate bytes exceed the selected write ceiling.
    #[error("fixture value length {size} exceeds byte limit {maximum}")]
    ValueTooLarge { size: usize, maximum: usize },
}

impl<M: Memory> FixtureCell<M> {
    /// Bootstrap a trusted fixture key and open its existing byte cell.
    /// This constructor is unbounded; use [`Self::open_bounded`] for untrusted images.
    ///
    /// # Panics
    /// Panics on the setup refusals and substrate failures in [`Self::try_new`].
    /// It never substitutes fresh backing memory after a failure.
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
    /// Nonempty Cell headers are admitted before initialization; fresh allocations
    /// receive an empty cell. Payload allocation is unbounded, so callers must
    /// supply trusted images and enforce their own decode/write ceilings.
    /// Bootstrap may advance ledger metadata before a later Cell refusal.
    ///
    /// # Panics
    /// Backing IO and Cell publication retain their substrate panic behavior.
    pub fn try_new(
        memory: M,
        owner: &str,
        namespace: &str,
        stable_key: &str,
        config: MemoryManagerConfig,
    ) -> Result<Self, FixtureCellError> {
        let memory = Self::open_memory(memory, owner, namespace, stable_key, config)?;
        if memory.size() > 0 {
            crate::stable_cell::stable_cell_payload_length(&memory, usize::MAX)?;
        }
        Ok(Self::from_admitted_memory(memory, usize::MAX))
    }

    /// Create a bounded empty cell, refusing any previously occupied allocation.
    ///
    /// Bootstrap may update ledger metadata before occupied-cell refusal; existing
    /// cell bytes are never cleared. Callers own payload codecs and initialization.
    ///
    /// # Panics
    /// Backing IO and initial Cell publication can panic.
    pub fn create_bounded(
        memory: M,
        owner: &str,
        namespace: &str,
        stable_key: &str,
        config: MemoryManagerConfig,
        maximum: usize,
    ) -> Result<Self, FixtureCellError> {
        let memory = Self::open_memory(memory, owner, namespace, stable_key, config)?;
        if memory.size() > 0 {
            return Err(FixtureCellError::Occupied);
        }
        Ok(Self::from_admitted_memory(memory, maximum))
    }

    /// Open retained state, checking its header, capacity and byte ceiling first.
    ///
    /// Refuses missing backing without initializing it. The selected Cell must
    /// already exist; its payload is not allocated or read until header admission
    /// succeeds. Bootstrap may advance ledger metadata before later Cell refusal.
    /// Callers still validate payload identity, codec and lifecycle invariants.
    ///
    /// # Panics
    /// Backing IO and Cell loading retain their substrate panic behavior.
    pub fn open_bounded(
        memory: M,
        owner: &str,
        namespace: &str,
        stable_key: &str,
        config: MemoryManagerConfig,
        maximum: usize,
    ) -> Result<Self, FixtureCellError> {
        if memory.size() == 0 {
            return Err(crate::StableCellPayloadError::NotStableCell.into());
        }
        let memory = Self::open_memory(memory, owner, namespace, stable_key, config)?;
        crate::stable_cell::stable_cell_payload_length(&memory, maximum)?;
        Ok(Self::from_admitted_memory(memory, maximum))
    }

    fn open_memory(
        memory: M,
        owner: &str,
        namespace: &str,
        stable_key: &str,
        config: MemoryManagerConfig,
    ) -> Result<RuntimeMemory<M>, FixtureCellError> {
        let runtime = fixture_runtime(memory, owner, namespace, &[stable_key], config)?;
        Ok(runtime.open_memory(stable_key)?)
    }

    fn from_admitted_memory(memory: RuntimeMemory<M>, maximum: usize) -> Self {
        Self {
            cell: Cell::init(memory, Vec::new()),
            maximum,
        }
    }

    /// Transfer the admitted grant for caller-owned raw inspection.
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

    /// Persist caller-encoded bytes after checking the selected byte ceiling.
    ///
    /// # Panics
    /// Cell persistence and backing IO can panic after successful length admission.
    pub fn try_set(&mut self, bytes: Vec<u8>) -> Result<(), FixtureCellError> {
        if bytes.len() > self.maximum {
            return Err(FixtureCellError::ValueTooLarge {
                size: bytes.len(),
                maximum: self.maximum,
            });
        }
        self.cell.set(bytes);
        Ok(())
    }

    /// Persist caller-encoded bytes, preserving the fixture's panic convention.
    ///
    /// # Panics
    /// Panics if bytes exceed the selected ceiling or Cell persistence fails.
    pub fn set(&mut self, bytes: Vec<u8>) {
        self.try_set(bytes).expect("fixture cell write");
    }
}

#[cfg(test)]
mod tests;
