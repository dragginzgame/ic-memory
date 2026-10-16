//! Explicit fixture-only ledger and cell setup; callers own codecs, bounds and lifecycle calls.

use crate::{
    GenericAllocationPolicy, MemoryAllocationPool, MemoryAuthority, MemoryManagerConfig,
    MemoryRequest, MemoryRuntime, RuntimeMemory, SchemaMetadata, SealedDeclarationSnapshot,
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
///

pub struct FixtureCell<M: Memory> {
    cell: Cell<Vec<u8>, RuntimeMemory<M>>,
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
        let request = MemoryRequest::new(owner, stable_key, SchemaMetadata::default())
            .expect("fixture memory request");
        let pool = MemoryAllocationPool::new(
            vec![MemoryAuthority::new(owner, namespace).expect("fixture namespace grant")],
            vec![],
        )
        .expect("fixture allocation pool");
        let declarations =
            SealedDeclarationSnapshot::new(vec![request]).expect("fixture declarations");
        let mut runtime = MemoryRuntime::new_with_config(memory, config).expect("fixture runtime");
        runtime
            .bootstrap(&declarations, &pool, &GenericAllocationPolicy)
            .expect("fixture bootstrap");
        let memory = runtime
            .open_memory(stable_key)
            .expect("fixture memory grant");
        let cell = Cell::init(memory, Vec::new());
        Self { cell }
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
