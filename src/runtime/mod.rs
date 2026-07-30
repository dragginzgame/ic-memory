mod default;
mod diagnostics;
mod error;
mod policy;

#[cfg(test)]
mod tests;

pub use default::{
    bootstrap_default_memory_manager, bootstrap_default_memory_manager_with_policy,
    committed_allocations, default_memory_manager_commit_recovery_diagnostic,
    default_memory_manager_diagnostic_export, default_memory_manager_doctor_report,
    default_memory_manager_doctor_report_with_policy, is_default_memory_manager_bootstrapped,
    open_default_memory_manager_memory,
};
pub use error::{
    RuntimeBootstrapError, RuntimeConstructionError, RuntimeDiagnosticError, RuntimeOpenError,
    RuntimePolicyError, RuntimeStateError,
};

use self::policy::{RuntimeMemoryManagerPolicy, runtime_bootstrap_error_from_bootstrap};
use crate::{
    AllocationBootstrap, AllocationHistory, AllocationLedger, AllocationPolicy,
    CommittedAllocations, PolicyIdentity, RuntimeBootstrapPolicy, STABLE_CELL_VALUE_OFFSET,
    StableCellLedgerError, StableCellLedgerRecord, StableKey, registry::SealedDeclarationSnapshot,
    slot::MEMORY_MANAGER_LEDGER_ID, stable_cell::decode_stable_cell_ledger_record_from_memory,
};
use ic_stable_structures::{
    Cell, Memory, Storable,
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
};

type LedgerCell<M> = Cell<StableCellLedgerRecord, VirtualMemory<M>>;

// `ic-stable-structures` 0.7.2 documents this four-byte prefix for its V1
// `MemoryManager` layout but does not expose a fallible constructor or these
// constants. Keep this preflight coupled to the pinned dependency version.
const MEMORY_MANAGER_MAGIC: [u8; 3] = *b"MGR";
const MEMORY_MANAGER_LAYOUT_VERSION: u8 = 1;

enum RuntimeLifecycle {
    Unbootstrapped,
    Bootstrapped {
        committed_allocations: CommittedAllocations,
        binding: RuntimeBootstrapBinding,
    },
}

struct RuntimeBootstrapBinding {
    declarations: SealedDeclarationSnapshot,
    policy_identity: PolicyIdentity,
}

///
/// MemoryRuntime
///
/// Canonical owner of allocation bootstrap state for one backing memory.
///
/// The runtime owns its `MemoryManager`, allocation-ledger cell, bootstrap
/// lifecycle, committed allocation capability, opens, and diagnostics. Static
/// linked-program declarations are supplied separately as one immutable
/// [`SealedDeclarationSnapshot`].
///
/// `M` needs only [`Memory`]. The runtime does not require the backing memory
/// to be `Send`, `Sync`, `Clone`, or `'static`.
///

pub struct MemoryRuntime<M: Memory> {
    memory_manager: MemoryManager<M>,
    ledger_cell: Option<LedgerCell<M>>,
    lifecycle: RuntimeLifecycle,
}

impl<M: Memory> MemoryRuntime<M> {
    /// Construct an unbootstrapped runtime without overwriting foreign memory.
    ///
    /// Empty backing memory is initialized as an
    /// `ic_stable_structures::MemoryManager`. Nonempty memory must already
    /// contain the current `MemoryManager` magic and layout version; otherwise
    /// construction returns a typed error before `MemoryManager::init` can
    /// write its header or allocation table. A pre-grown blank memory is
    /// nonempty and is therefore rejected rather than assumed disposable.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeConstructionError::ForeignMemory`] for nonempty memory
    /// without `MemoryManager` magic, or
    /// [`RuntimeConstructionError::UnsupportedMemoryManagerVersion`] when the
    /// magic is recognized but the layout version is not current.
    pub fn new(memory: M) -> Result<Self, RuntimeConstructionError> {
        preflight_memory_manager_backing(&memory)?;
        Ok(Self {
            memory_manager: MemoryManager::init(memory),
            ledger_cell: None,
            lifecycle: RuntimeLifecycle::Unbootstrapped,
        })
    }

    /// Return whether this runtime has published committed allocation authority.
    #[must_use]
    pub const fn is_bootstrapped(&self) -> bool {
        matches!(self.lifecycle, RuntimeLifecycle::Bootstrapped { .. })
    }

    /// Bootstrap this backing memory from one immutable declaration snapshot.
    ///
    /// Recovery, policy evaluation, staging, persistence, and capability
    /// publication are local to this runtime. A repeated call is idempotent
    /// only when the sealed declaration snapshot and
    /// [`RuntimeBootstrapPolicy::runtime_bootstrap_identity`] match the
    /// successful bootstrap. A mismatch returns a typed error without
    /// advancing the durable generation or re-evaluating policy.
    pub fn bootstrap<P: RuntimeBootstrapPolicy>(
        &mut self,
        declarations: &SealedDeclarationSnapshot,
        policy: &P,
    ) -> Result<&CommittedAllocations, RuntimeBootstrapError<P::Error>> {
        let policy_identity = policy.runtime_bootstrap_identity()?;
        let already_bootstrapped = match &self.lifecycle {
            RuntimeLifecycle::Unbootstrapped => false,
            RuntimeLifecycle::Bootstrapped { binding, .. } => {
                binding.validate(declarations, &policy_identity)?;
                true
            }
        };
        if !already_bootstrapped {
            self.bootstrap_unbootstrapped(declarations, policy, policy_identity)?;
        }
        match &self.lifecycle {
            RuntimeLifecycle::Bootstrapped {
                committed_allocations,
                ..
            } => Ok(committed_allocations),
            RuntimeLifecycle::Unbootstrapped => Err(RuntimeBootstrapError::State(
                RuntimeStateError::InconsistentLifecycle,
            )),
        }
    }

    fn bootstrap_unbootstrapped<P: AllocationPolicy>(
        &mut self,
        declarations: &SealedDeclarationSnapshot,
        policy: &P,
        policy_identity: PolicyIdentity,
    ) -> Result<(), RuntimeBootstrapError<P::Error>> {
        self.initialize_ledger_cell()?;
        let mut record = self
            .ledger_cell
            .as_ref()
            .map(|cell| cell.get().clone())
            .ok_or(RuntimeStateError::InconsistentLifecycle)?;
        let runtime_policy = RuntimeMemoryManagerPolicy {
            declarations,
            custom_policy: policy,
        };
        let genesis = AllocationLedger::new(0, AllocationHistory::default())?;
        let commit = AllocationBootstrap::new(record.store_mut())
            .initialize_validate_and_commit(
                &genesis,
                declarations.allocation_snapshot().clone(),
                &runtime_policy,
                None,
            )
            .map_err(runtime_bootstrap_error_from_bootstrap)?;
        let (ledger, validated) = commit.into_parts();

        self.persist_ledger_record(record)?;
        let committed =
            external_runtime_allocations(validated.confirm_persisted(ledger.current_generation()));
        self.lifecycle = RuntimeLifecycle::Bootstrapped {
            committed_allocations: committed,
            binding: RuntimeBootstrapBinding {
                declarations: declarations.clone(),
                policy_identity,
            },
        };
        Ok(())
    }

    /// Borrow this runtime's committed allocation-open capability.
    pub const fn committed_allocations(&self) -> Result<&CommittedAllocations, RuntimeOpenError> {
        match &self.lifecycle {
            RuntimeLifecycle::Unbootstrapped => Err(RuntimeOpenError::NotBootstrapped),
            RuntimeLifecycle::Bootstrapped {
                committed_allocations,
                ..
            } => Ok(committed_allocations),
        }
    }

    /// Open this runtime's committed memory by stable key and expected ID.
    pub fn open_memory(
        &self,
        stable_key: &str,
        expected_id: u8,
    ) -> Result<VirtualMemory<M>, RuntimeOpenError> {
        let key = StableKey::parse(stable_key)?;
        if crate::is_ic_memory_stable_key(key.as_str()) {
            return Err(RuntimeOpenError::ReservedStableKey {
                stable_key: stable_key.to_string(),
            });
        }
        let committed = self.committed_allocations()?;
        let slot = committed
            .slot_for(&key)
            .ok_or_else(|| RuntimeOpenError::StableKeyNotCommitted(stable_key.to_string()))?;
        let committed_id = slot.memory_manager_id()?;
        if committed_id != expected_id {
            return Err(RuntimeOpenError::MemoryIdMismatch {
                stable_key: stable_key.to_string(),
                committed_id,
                requested_id: expected_id,
            });
        }
        Ok(self.memory(expected_id))
    }

    fn initialize_ledger_cell<P>(&mut self) -> Result<(), RuntimeBootstrapError<P>> {
        if self.ledger_cell.is_some() {
            return Ok(());
        }
        let memory = self.memory(MEMORY_MANAGER_LEDGER_ID);
        crate::validate_stable_cell_ledger_memory(&memory)?;
        ensure_ledger_cell_capacity(&memory, &StableCellLedgerRecord::default())?;
        self.ledger_cell = Some(Cell::init(memory, StableCellLedgerRecord::default()));
        Ok(())
    }

    fn persist_ledger_record<P>(
        &mut self,
        record: StableCellLedgerRecord,
    ) -> Result<(), RuntimeBootstrapError<P>> {
        let memory = self.memory(MEMORY_MANAGER_LEDGER_ID);
        ensure_ledger_cell_capacity(&memory, &record)?;
        let cell = self
            .ledger_cell
            .as_mut()
            .ok_or(RuntimeStateError::InconsistentLifecycle)?;
        let _previous = cell.set(record);
        Ok(())
    }

    fn memory(&self, id: u8) -> VirtualMemory<M> {
        self.memory_manager.get(MemoryId::new(id))
    }

    fn ledger_record_from_memory(&self) -> Result<StableCellLedgerRecord, StableCellLedgerError> {
        decode_stable_cell_ledger_record_from_memory(&self.memory(MEMORY_MANAGER_LEDGER_ID))
    }
}

fn preflight_memory_manager_backing<M: Memory>(memory: &M) -> Result<(), RuntimeConstructionError> {
    if memory.size() == 0 {
        return Ok(());
    }

    let mut prefix = [0_u8; 4];
    memory.read(0, &mut prefix);
    let observed_magic = [prefix[0], prefix[1], prefix[2]];
    if observed_magic != MEMORY_MANAGER_MAGIC {
        return Err(RuntimeConstructionError::ForeignMemory { observed_magic });
    }
    let observed = prefix[3];
    if observed != MEMORY_MANAGER_LAYOUT_VERSION {
        return Err(RuntimeConstructionError::UnsupportedMemoryManagerVersion {
            observed,
            supported: MEMORY_MANAGER_LAYOUT_VERSION,
        });
    }
    Ok(())
}

impl RuntimeBootstrapBinding {
    fn validate<P>(
        &self,
        declarations: &SealedDeclarationSnapshot,
        policy_identity: &PolicyIdentity,
    ) -> Result<(), RuntimeBootstrapError<P>> {
        if !self.declarations.shares_storage_with(declarations) {
            return Err(RuntimeBootstrapError::DeclarationSnapshotMismatch);
        }
        if &self.policy_identity != policy_identity {
            return Err(RuntimeBootstrapError::PolicyIdentityMismatch {
                established: self.policy_identity.clone(),
                requested: policy_identity.clone(),
            });
        }
        Ok(())
    }
}

fn ensure_ledger_cell_capacity<M: Memory, P>(
    memory: &VirtualMemory<M>,
    record: &StableCellLedgerRecord,
) -> Result<(), RuntimeBootstrapError<P>> {
    let value_size = record.to_bytes().len();
    let value_size_u32 = u32::try_from(value_size)
        .map_err(|_| RuntimeBootstrapError::StableCellLedgerWriteTooLarge { value_size })?;
    let required_bytes = STABLE_CELL_VALUE_OFFSET
        .checked_add(u64::from(value_size_u32))
        .ok_or(RuntimeBootstrapError::StableCellLedgerWriteTooLarge { value_size })?;
    let available_bytes = memory.size().saturating_mul(crate::WASM_PAGE_SIZE_BYTES);
    if required_bytes <= available_bytes {
        return Ok(());
    }
    let grow_by = required_bytes
        .saturating_sub(available_bytes)
        .div_ceil(crate::WASM_PAGE_SIZE_BYTES);
    if memory.grow(grow_by) < 0 {
        return Err(RuntimeBootstrapError::StableCellLedgerWriteTooLarge { value_size });
    }
    Ok(())
}

fn external_runtime_allocations(committed: CommittedAllocations) -> CommittedAllocations {
    committed.without_stable_key_prefix(crate::IC_MEMORY_STABLE_KEY_PREFIX)
}
