mod admission;
#[cfg(test)]
mod admission_tests;
mod adoption;
#[cfg(test)]
mod adoption_tests;
mod allocations;
mod backing;
mod config;
mod default;
mod diagnostics;
mod error;
mod layout;
mod policy;

#[cfg(test)]
mod allocation_tests;
#[cfg(test)]
mod growth_tests;
#[cfg(test)]
#[expect(
    unsafe_code,
    reason = "exercise raw reads with valid uninitialized destinations"
)]
mod read_tests;
#[cfg(test)]
mod request_tests;
#[cfg(test)]
mod tests;

pub use admission::{BootstrapAdmission, BootstrapAdmissionError, RecoveredAllocationMetadata};
pub use adoption::RuntimeAdoptionError;

pub use allocations::{
    AllocationBinding, AllocationRangeClaim, MemoryAllocation, MemoryAllocationSummary,
    MemoryAllocations, MemoryBindingSummary,
};
pub use backing::RuntimeMemory;
pub use config::MemoryManagerConfig;
pub use default::{
    bootstrap_default_memory_manager, bootstrap_default_memory_manager_with_config,
    bootstrap_default_memory_manager_with_policy, committed_allocations,
    default_memory_manager_commit_recovery_diagnostic, default_memory_manager_diagnostic_export,
    default_memory_manager_doctor_report, default_memory_manager_doctor_report_with_policy,
    default_memory_manager_memory_allocation_summary, default_memory_manager_memory_allocations,
    default_memory_manager_memory_id, is_default_memory_manager_bootstrapped,
    open_default_memory_manager_memory, open_default_memory_manager_memory_by_key,
    verify_default_memory_manager_authority,
};
pub use error::{
    MemoryResolutionError, RuntimeBootstrapError, RuntimeConstructionError, RuntimeDiagnosticError,
    RuntimeGrowError, RuntimeOpenError, RuntimePolicyError, RuntimeStateError,
};
pub use layout::MemoryManagerLayoutError;
pub use policy::GenericRangePolicy;

use self::policy::{RuntimeMemoryManagerPolicy, runtime_bootstrap_error_from_bootstrap};
use crate::{
    AllocationBootstrap, AllocationLedger, CommittedAllocations, PolicyIdentity,
    RuntimeBootstrapPolicy, STABLE_CELL_VALUE_OFFSET, StableCellLedgerError,
    StableCellLedgerRecord, registry::SealedDeclarationSnapshot, slot::MEMORY_MANAGER_LEDGER_ID,
    stable_cell::decode_stable_cell_ledger_record_from_memory,
};
use ic_stable_structures::{
    Cell, Memory,
    memory_manager::{MemoryId, MemoryManager},
};

use std::rc::Rc;

enum RuntimeLifecycle {
    Unbootstrapped,
    Bootstrapped {
        committed_allocations: CommittedAllocations,
        binding: RuntimeBootstrapBinding,
    },
}

struct RuntimeBootstrapBinding {
    source: SealedDeclarationSnapshot,
    declarations: SealedDeclarationSnapshot,
    policy_identity: PolicyIdentity,
}

///
/// MemoryRuntime
///
/// Canonical owner of allocation bootstrap state for one backing memory.
///
/// The runtime owns its `MemoryManager`, allocation-ledger persistence, bootstrap
/// lifecycle, committed allocation capability, opens, and diagnostics. Static
/// linked-program declarations are supplied separately as one immutable
/// [`SealedDeclarationSnapshot`].
///
/// Each bootstrap attempt fallibly decodes its ledger record from persisted
/// memory. Capacity-checked writes use `Cell`; diagnostics also read persisted
/// memory directly.
///
/// `M` needs only [`Memory`]. The runtime does not require the backing memory
/// to be `Send`, `Sync`, `Clone`, or `'static`.
///

pub struct MemoryRuntime<M: Memory> {
    memory_manager: MemoryManager<Rc<M>>,
    // Shared backing, immutable geometry and live accounting belong to growth.
    // Handles reserve physical capacity; the manager owns bucket metadata.
    growth: Rc<backing::GrowthState<M>>,
    lifecycle: RuntimeLifecycle,
}

impl<M: Memory> MemoryRuntime<M> {
    /// Construct an unbootstrapped runtime without overwriting foreign memory.
    ///
    /// Empty backing memory is initialized as an
    /// `ic_stable_structures::MemoryManager`. Nonempty memory must pass bounded
    /// validation of the current manager header, bucket table, and extents; otherwise
    /// construction returns a typed error before the manager can
    /// write its header or allocation table. A pre-grown blank memory is
    /// nonempty and is therefore rejected rather than assumed disposable.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeConstructionError::ForeignMemory`] for nonempty memory
    /// without `MemoryManager` magic, or
    /// [`RuntimeConstructionError::UnsupportedMemoryManagerVersion`] when the
    /// magic is recognized but the layout version is not current. Invalid
    /// metadata returns [`RuntimeConstructionError::Layout`]. Refused fresh
    /// metadata growth returns [`RuntimeConstructionError::Growth`] without
    /// writes, allowing the same backing memory to be retried. Reopening honors
    /// the actual persisted bucket size; only fresh memory uses 128 pages.
    pub fn new(memory: M) -> Result<Self, RuntimeConstructionError> {
        Self::construct(memory, None)
    }

    /// Construct with an explicit immutable bucket policy. Existing memory must
    /// match exactly; mismatches fail before manager initialization or writes.
    ///
    /// # Errors
    ///
    /// Returns the construction errors described by [`Self::new`], or
    /// [`RuntimeConstructionError::BucketSizeMismatch`] when existing geometry
    /// differs from `config`.
    pub fn new_with_config(
        memory: M,
        config: MemoryManagerConfig,
    ) -> Result<Self, RuntimeConstructionError> {
        Self::construct(memory, Some(config))
    }

    fn construct(
        memory: M,
        requested: Option<MemoryManagerConfig>,
    ) -> Result<Self, RuntimeConstructionError> {
        if cfg!(target_endian = "big") {
            return Err(MemoryManagerLayoutError::UnsupportedByteOrder.into());
        }
        let (bucket_size_pages, allocated_buckets) = if memory.size() == 0 {
            // Reserve the metadata page before the dependency writes its header.
            // This fresh allocation is owned here; caller-supplied nonempty
            // memory still passes the layout checks below before any writes.
            if memory.grow(1) == -1 {
                return Err(RuntimeGrowError::BackingRefused {
                    additional_pages: 1,
                }
                .into());
            }
            (requested.unwrap_or_default().bucket_size_pages(), 0)
        } else {
            let measured = layout::read(&memory)?;
            let actual = measured.bucket_pages;
            if let Some(config) = requested {
                check_bucket_size(actual, config)?;
            }
            (actual, measured.allocated_buckets)
        };
        let backing = Rc::new(memory);
        let growth = Rc::new(backing::GrowthState {
            backing: Rc::clone(&backing),
            bucket_size_pages,
            allocated_buckets: std::cell::RefCell::new(allocated_buckets),
        });
        Ok(Self {
            memory_manager: MemoryManager::init_with_bucket_size(
                Rc::clone(&backing),
                bucket_size_pages,
            ),
            growth,
            lifecycle: RuntimeLifecycle::Unbootstrapped,
        })
    }

    /// Return the immutable bucket configuration bound to this runtime's manager.
    #[must_use]
    pub fn memory_manager_config(&self) -> MemoryManagerConfig {
        // Construction has already validated the nonzero persisted setting.
        MemoryManagerConfig::from_validated(self.growth.bucket_size_pages)
    }

    /// Return whether this runtime has published committed allocation authority.
    #[must_use]
    pub const fn is_bootstrapped(&self) -> bool {
        matches!(self.lifecycle, RuntimeLifecycle::Bootstrapped { .. })
    }

    /// Bootstrap this backing memory from one immutable declaration snapshot.
    ///
    /// Recovery, metadata admission, logical resolution, policy evaluation,
    /// staging, persistence and capability publication are local to this runtime.
    /// A repeated call is idempotent only when the sealed declaration snapshot and
    /// [`RuntimeBootstrapPolicy::runtime_bootstrap_identity`] match the
    /// successful bootstrap. A mismatch returns a typed error without
    /// advancing the durable generation or re-evaluating policy.
    /// Independently sealed snapshots match when their canonical contents are equal.
    ///
    /// # Panics
    ///
    /// Panics if a private runtime or encoding invariant is broken, or backing
    /// memory or a policy callback panics.
    pub fn bootstrap<P: RuntimeBootstrapPolicy>(
        &mut self,
        declarations: &SealedDeclarationSnapshot,
        policy: &P,
    ) -> Result<&CommittedAllocations, RuntimeBootstrapError<P::Error>> {
        let policy_identity = policy.runtime_bootstrap_identity()?;
        match &self.lifecycle {
            RuntimeLifecycle::Unbootstrapped => {
                self.bootstrap_unbootstrapped(declarations, policy, policy_identity)?;
            }
            RuntimeLifecycle::Bootstrapped { binding, .. } => {
                binding.validate(declarations, &policy_identity)?;
            }
        }
        match &self.lifecycle {
            RuntimeLifecycle::Bootstrapped {
                committed_allocations,
                ..
            } => Ok(committed_allocations),
            RuntimeLifecycle::Unbootstrapped => {
                unreachable!("successful bootstrap publishes committed allocations")
            }
        }
    }

    fn bootstrap_unbootstrapped<P: RuntimeBootstrapPolicy>(
        &mut self,
        declarations: &SealedDeclarationSnapshot,
        policy: &P,
        policy_identity: PolicyIdentity,
    ) -> Result<(), RuntimeBootstrapError<P::Error>> {
        let memory = self.memory(MEMORY_MANAGER_LEDGER_ID);
        let mut record = decode_stable_cell_ledger_record_from_memory(&memory)?;
        if memory.size() == 0 {
            // Empty memory decodes to an uninitialized record. Preserve fresh
            // cell acquisition before admission without persisting genesis.
            ensure_ledger_cell_capacity(&memory, &record)?;
            drop(Cell::new(memory.clone(), StableCellLedgerRecord::default()));
        }
        let genesis = AllocationLedger::empty_genesis();
        let recovered = record.store_mut().recover_or_initialize(&genesis)?;
        let mut admission = BootstrapAdmission::new(recovered.ledger(), declarations);
        let preparation = policy.prepare_bootstrap(&mut admission);
        let historical = admission.complete()?;
        preparation.map_err(RuntimeBootstrapError::AdmissionPolicy)?;
        let resolved = declarations.resolve(recovered.ledger(), historical)?;
        let runtime_policy = RuntimeMemoryManagerPolicy {
            declarations: &resolved,
            custom_policy: policy,
        };
        let commit = AllocationBootstrap::new(record.store_mut())
            .validate_against(
                recovered,
                resolved.allocation_snapshot().clone(),
                &runtime_policy,
                None,
            )
            .map_err(runtime_bootstrap_error_from_bootstrap)?;
        ensure_ledger_cell_capacity(&memory, &record)?;
        drop(Cell::new(memory, record));
        let committed = commit.confirm_persisted().into_application_allocations();
        self.lifecycle = RuntimeLifecycle::Bootstrapped {
            committed_allocations: committed,
            binding: RuntimeBootstrapBinding {
                source: declarations.clone(),
                declarations: resolved,
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

    /// Open by durable key using only this runtime's persisted current capability.
    pub fn open_memory_by_key(
        &self,
        stable_key: &str,
    ) -> Result<RuntimeMemory<M>, RuntimeOpenError> {
        Ok(self.memory(self.memory_id(stable_key)?))
    }

    /// Open this runtime's committed memory by stable key and expected ID.
    pub fn open_memory(
        &self,
        stable_key: &str,
        expected_id: u8,
    ) -> Result<RuntimeMemory<M>, RuntimeOpenError> {
        let committed_id = self.memory_id(stable_key)?;
        if committed_id != expected_id {
            return Err(RuntimeOpenError::MemoryIdMismatch {
                stable_key: stable_key.to_string(),
                committed_id,
                requested_id: expected_id,
            });
        }
        Ok(self.memory(committed_id))
    }

    /// Resolve an application key's committed ID without opening memory,
    /// reading history, or changing the host's policy or bucket configuration.
    pub fn memory_id(&self, stable_key: &str) -> Result<u8, RuntimeOpenError> {
        crate::key::validate(stable_key)?;
        if crate::is_ic_memory_stable_key(stable_key) {
            return Err(RuntimeOpenError::ReservedStableKey {
                stable_key: stable_key.to_string(),
            });
        }
        let slot = crate::capability::slot_for_key(
            self.committed_allocations()?.declarations(),
            stable_key,
        )
        .ok_or_else(|| RuntimeOpenError::StableKeyNotCommitted(stable_key.to_string()))?;
        Ok(slot.id())
    }

    fn memory(&self, id: u8) -> RuntimeMemory<M> {
        RuntimeMemory {
            memory: self.memory_manager.get(MemoryId::new(id)),
            growth: Rc::clone(&self.growth),
        }
    }

    // Size-only observations need no handle carrying runtime growth authority.
    fn memory_size_pages(&self, id: u8) -> u64 {
        self.memory_manager.get(MemoryId::new(id)).size()
    }

    fn ledger_record_from_memory(&self) -> Result<StableCellLedgerRecord, StableCellLedgerError> {
        // Decoding only reads; the manager handle retains backing lifetime
        // without carrying the growth owner needed by writable runtime handles.
        let memory = self
            .memory_manager
            .get(MemoryId::new(MEMORY_MANAGER_LEDGER_ID));
        decode_stable_cell_ledger_record_from_memory(&memory)
    }
}

impl RuntimeBootstrapBinding {
    fn validate<P>(
        &self,
        declarations: &SealedDeclarationSnapshot,
        policy_identity: &PolicyIdentity,
    ) -> Result<(), RuntimeBootstrapError<P>> {
        if &self.source != declarations {
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
    memory: &RuntimeMemory<M>,
    record: &StableCellLedgerRecord,
) -> Result<(), RuntimeBootstrapError<P>> {
    let value_size = record.encoded_size();
    if value_size > crate::constants::MAX_LEDGER_RECORD_BYTES {
        return Err(RuntimeBootstrapError::StableCellLedgerWriteTooLarge { value_size });
    }
    // The record limit is below u32::MAX, including the eight-byte cell header.
    let required_bytes = STABLE_CELL_VALUE_OFFSET + value_size as u64;
    let available_bytes = memory.size().saturating_mul(crate::WASM_PAGE_SIZE_BYTES);
    if required_bytes <= available_bytes {
        return Ok(());
    }
    let grow_by = (required_bytes - available_bytes).div_ceil(crate::WASM_PAGE_SIZE_BYTES);
    memory.grow(grow_by)?;
    Ok(())
}

const fn check_bucket_size(
    actual: u16,
    requested: MemoryManagerConfig,
) -> Result<(), RuntimeConstructionError> {
    if actual != requested.bucket_size_pages() {
        return Err(RuntimeConstructionError::BucketSizeMismatch {
            persisted: actual,
            requested: requested.bucket_size_pages(),
        });
    }
    Ok(())
}
