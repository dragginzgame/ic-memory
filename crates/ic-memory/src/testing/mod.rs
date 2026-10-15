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
    }

    fn open(
        memory: M,
        owner: &str,
        namespace: &str,
        stable_key: &str,
        config: MemoryManagerConfig,
        admission: Option<(usize, bool)>,
    ) -> Self {
        if matches!(admission, Some((_, false))) {
            assert!(memory.size() > 0, "fixture retained backing is missing");
        }
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
        if let Some((maximum, fresh)) = admission {
            if fresh {
                assert_eq!(
                    memory.size(),
                    0,
                    "fixture initialization cannot reset history"
                );
            } else {
                crate::stable_cell::stable_cell_payload_length(&memory, maximum)
                    .expect("fixture retained cell envelope and byte bound");
            }
        }
        let cell = Cell::init(memory, Vec::new());
        Self {
            _runtime: runtime,
            cell,
            maximum: admission.map_or(usize::MAX, |(maximum, _)| maximum),
        }
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
