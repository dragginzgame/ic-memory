use crate::{
    AllocationBootstrap, AllocationHistory, AllocationLedger, AllocationPolicy,
    AllocationSlotDescriptor, CommittedAllocations, DiagnosticCheck, DiagnosticCode,
    DiagnosticDeclaration, DiagnosticExport, DiagnosticFailure, DiagnosticMemorySize,
    DiagnosticRangeAuthority, DiagnosticStableCell, DiagnosticStableCellStatus, LedgerCommitError,
    LedgerPayloadEnvelopeError, MemoryRuntimeDoctorReport, STABLE_CELL_VALUE_OFFSET,
    StableCellLedgerError, StableCellLedgerRecord, StableKey,
    physical::CommitStoreDiagnostic,
    registry::{
        RuntimeDeclarationAuthority, SealedDeclarationSnapshot, StaticMemoryDeclarationError,
        sealed_declaration_snapshot,
    },
    slot::{
        IC_MEMORY_AUTHORITY_OWNER, MEMORY_MANAGER_LEDGER_ID, MemoryManagerRangeAuthorityError,
        MemoryManagerSlotError,
    },
    stable_cell::decode_stable_cell_ledger_record_from_memory,
};
use ic_stable_structures::{
    Cell, DefaultMemoryImpl, Memory, Storable,
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
};
use std::{cell::RefCell, convert::Infallible};

type LedgerCell<M> = Cell<StableCellLedgerRecord, VirtualMemory<M>>;

enum RuntimeLifecycle {
    Unbootstrapped,
    Bootstrapped {
        committed_allocations: CommittedAllocations,
    },
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

///
/// RuntimeStateError
///
/// Failure to enter or maintain one memory runtime's in-memory lifecycle.
///

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, thiserror::Error, PartialEq)]
pub enum RuntimeStateError {
    /// A default-runtime operation re-entered while that TLS runtime was borrowed.
    #[error("ic-memory default runtime is already borrowed by an active operation")]
    ReentrantAccess,
    /// The thread-local default runtime is being destroyed and cannot be entered.
    #[error("ic-memory default runtime is unavailable during thread-local destruction")]
    Unavailable,
    /// Internal runtime lifecycle state was inconsistent.
    #[error("ic-memory runtime lifecycle is internally inconsistent")]
    InconsistentLifecycle,
}

///
/// RuntimeBootstrapError
///
/// Failure to bootstrap one `MemoryRuntime`.
///

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum RuntimeBootstrapError<P> {
    /// Linked-program declaration snapshot sealing failed.
    #[error(transparent)]
    Registry(#[from] StaticMemoryDeclarationError),
    /// Runtime ledger genesis construction failed.
    #[error(transparent)]
    LedgerIntegrity(#[from] crate::LedgerIntegrityError),
    /// Protected ledger recovery or commit failed.
    #[error(transparent)]
    LedgerCommit(#[from] crate::LedgerCommitError),
    /// Stable-cell ledger storage is corrupt before protected recovery can run.
    #[error(transparent)]
    StableCellLedger(#[from] StableCellLedgerError),
    /// Stable-cell ledger storage cannot fit the next protected ledger record.
    #[error("stable-cell ledger record size {value_size} cannot be written to stable memory")]
    StableCellLedgerWriteTooLarge {
        /// Encoded stable-cell ledger record size in bytes.
        value_size: usize,
    },
    /// Declaration validation failed.
    #[error(transparent)]
    Validation(#[from] crate::AllocationValidationError<RuntimePolicyError<P>>),
    /// Validated declarations could not be staged.
    #[error(transparent)]
    Staging(#[from] crate::AllocationStageError),
    /// Runtime lifecycle or default TLS access failed.
    #[error(transparent)]
    State(#[from] RuntimeStateError),
}

///
/// RuntimeOpenError
///
/// Failure to open an allocation through one memory runtime.
///

#[non_exhaustive]
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum RuntimeOpenError {
    /// This runtime has not published committed allocations.
    #[error("ic-memory runtime has not completed bootstrap validation")]
    NotBootstrapped,
    /// Runtime lifecycle or default TLS access failed.
    #[error(transparent)]
    State(#[from] RuntimeStateError),
    /// Stable-key grammar failure.
    #[error(transparent)]
    StableKey(#[from] crate::StableKeyError),
    /// The stable key was not present in this runtime's committed declaration set.
    #[error("stable key '{0}' was not committed by ic-memory runtime bootstrap")]
    StableKeyNotCommitted(String),
    /// Runtime governance stable keys are internal and cannot be opened publicly.
    #[error("stable key '{stable_key}' is reserved for ic-memory runtime governance")]
    ReservedStableKey {
        /// Reserved stable key.
        stable_key: String,
    },
    /// The committed slot is not a usable `MemoryManager` ID.
    #[error(transparent)]
    MemoryManagerSlot(#[from] MemoryManagerSlotError),
    /// The requested memory ID does not match the committed stable-key binding.
    #[error(
        "stable key '{stable_key}' is committed for MemoryManager ID {committed_id}, not requested ID {requested_id}"
    )]
    MemoryIdMismatch {
        /// Stable key being opened.
        stable_key: String,
        /// Committed MemoryManager ID.
        committed_id: u8,
        /// Requested MemoryManager ID.
        requested_id: u8,
    },
}

///
/// RuntimeDiagnosticError
///
/// Failure to build diagnostics for one memory runtime.
///

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum RuntimeDiagnosticError {
    /// This runtime has not opened and validated its ledger cell.
    #[error("ic-memory runtime has not completed bootstrap validation")]
    NotBootstrapped,
    /// Linked-program declaration snapshot sealing failed.
    #[error(transparent)]
    Registry(#[from] StaticMemoryDeclarationError),
    /// Runtime lifecycle or default TLS access failed.
    #[error(transparent)]
    State(#[from] RuntimeStateError),
    /// The recovered allocation ledger failed protected commit validation.
    #[error(transparent)]
    LedgerCommit(#[from] LedgerCommitError),
    /// Stable-cell ledger storage is corrupt before protected recovery can run.
    #[error(transparent)]
    StableCellLedger(#[from] StableCellLedgerError),
    /// A committed allocation slot was not a usable `MemoryManager` ID.
    #[error(transparent)]
    MemoryManagerSlot(#[from] MemoryManagerSlotError),
}

///
/// RuntimePolicyError
///
/// Failure in generic runtime range policy or caller-supplied policy.
///

#[non_exhaustive]
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum RuntimePolicyError<P> {
    /// Runtime range authority rejected the declaration.
    #[error(transparent)]
    Range(#[from] MemoryManagerRangeAuthorityError),
    /// Runtime metadata is internally inconsistent.
    #[error("runtime declaration metadata is missing for stable key '{0}'")]
    MissingDeclarationMetadata(String),
    /// `ic_memory.*` stable keys are reserved to the `ic-memory` authority.
    #[error("stable key '{stable_key}' is reserved to authority '{expected_authority}'")]
    ReservedStableKeyAuthority {
        /// Stable key being declared.
        stable_key: String,
        /// Required declaring authority.
        expected_authority: &'static str,
    },
    /// Caller-supplied policy rejected the declaration.
    #[error(transparent)]
    Custom(P),
}

impl<M: Memory> MemoryRuntime<M> {
    /// Construct an unbootstrapped runtime over one backing memory.
    #[must_use]
    pub fn new(memory: M) -> Self {
        Self {
            memory_manager: MemoryManager::init(memory),
            ledger_cell: None,
            lifecycle: RuntimeLifecycle::Unbootstrapped,
        }
    }

    /// Return whether this runtime has published committed allocation authority.
    #[must_use]
    pub const fn is_bootstrapped(&self) -> bool {
        matches!(self.lifecycle, RuntimeLifecycle::Bootstrapped { .. })
    }

    /// Bootstrap this backing memory from one immutable declaration snapshot.
    ///
    /// Recovery, policy evaluation, staging, persistence, and capability
    /// publication are local to this runtime. A repeated call on the same
    /// successfully bootstrapped object is idempotent and does not advance the
    /// durable generation or re-evaluate policy.
    pub fn bootstrap<P: AllocationPolicy>(
        &mut self,
        declarations: &SealedDeclarationSnapshot,
        policy: &P,
    ) -> Result<&CommittedAllocations, RuntimeBootstrapError<P::Error>> {
        if !self.is_bootstrapped() {
            self.bootstrap_unbootstrapped(declarations, policy)?;
        }
        match &self.lifecycle {
            RuntimeLifecycle::Bootstrapped {
                committed_allocations,
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
        };
        Ok(())
    }

    /// Borrow this runtime's committed allocation-open capability.
    pub const fn committed_allocations(&self) -> Result<&CommittedAllocations, RuntimeOpenError> {
        match &self.lifecycle {
            RuntimeLifecycle::Unbootstrapped => Err(RuntimeOpenError::NotBootstrapped),
            RuntimeLifecycle::Bootstrapped {
                committed_allocations,
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

    /// Export this runtime's recovered ledger and live virtual-memory sizes.
    pub fn diagnostic_export(&self) -> Result<DiagnosticExport, RuntimeDiagnosticError> {
        if !self.is_bootstrapped() {
            return Err(RuntimeDiagnosticError::NotBootstrapped);
        }
        let record = self.ledger_record_from_memory()?;
        let recovered = record.store().recover()?;
        let ledger = recovered.ledger();
        Ok(
            DiagnosticExport::from_ledger_with_commit_recovery_and_memory_sizes(
                ledger,
                ledger_anchor_descriptor(),
                Some(record.store().physical().diagnostic()),
                self.memory_sizes(ledger)?,
            ),
        )
    }

    /// Diagnose protected commit recovery from this runtime's ledger memory.
    ///
    /// This operation is available before bootstrap when the stable-cell
    /// envelope is readable or the ledger memory is empty.
    pub fn commit_recovery_diagnostic(
        &self,
    ) -> Result<CommitStoreDiagnostic, RuntimeDiagnosticError> {
        let record = self.ledger_record_from_memory()?;
        Ok(record.store().physical().diagnostic())
    }

    /// Build preflight and lifecycle diagnostics for this runtime.
    #[must_use]
    pub fn doctor_report(
        &self,
        declarations: &SealedDeclarationSnapshot,
    ) -> MemoryRuntimeDoctorReport {
        let stable_cell = self.stable_cell_diagnostic();
        let commit_recovery = stable_cell
            .record
            .as_ref()
            .map(|record| record.store().physical().diagnostic());
        let recovered = stable_cell
            .record
            .as_ref()
            .map(|record| record.store().recover());
        let recovered_for_export = recovered.as_ref().and_then(|result| result.as_ref().ok());
        let ledger = recovered_for_export.map(|recovered| {
            DiagnosticExport::from_ledger_with_commit_recovery_and_memory_sizes(
                recovered.ledger(),
                ledger_anchor_descriptor(),
                commit_recovery,
                self.memory_sizes_lossy(recovered.ledger()),
            )
        });
        let diagnostic_declarations = declarations
            .registered_declarations()
            .iter()
            .map(|registration| {
                DiagnosticDeclaration::new(
                    registration.authority(),
                    registration.declaration().clone(),
                )
            })
            .collect();
        let registered_records = declarations
            .registered_ranges()
            .iter()
            .map(|registration| registration.record().clone())
            .collect();
        let range_authority = DiagnosticRangeAuthority::new(
            registered_records,
            Ok(declarations.range_authority().clone()),
        );
        let validation = diagnostic_validation(
            declarations,
            stable_cell.record.as_ref(),
            recovered.as_ref(),
        );

        MemoryRuntimeDoctorReport {
            bootstrapped: self.is_bootstrapped(),
            ledger_anchor: ledger_anchor_descriptor(),
            stable_cell: stable_cell.diagnostic,
            commit_recovery,
            ledger,
            registered_declarations: diagnostic_declarations,
            range_authority,
            validation,
        }
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

    fn memory_sizes(
        &self,
        ledger: &AllocationLedger,
    ) -> Result<Vec<(AllocationSlotDescriptor, DiagnosticMemorySize)>, RuntimeDiagnosticError> {
        ledger
            .allocation_history()
            .records()
            .iter()
            .map(|record| {
                let id = record.slot().memory_manager_id()?;
                Ok((
                    record.slot().clone(),
                    DiagnosticMemorySize::from_wasm_pages(self.memory(id).size()),
                ))
            })
            .collect()
    }

    fn memory_sizes_lossy(
        &self,
        ledger: &AllocationLedger,
    ) -> Vec<(AllocationSlotDescriptor, DiagnosticMemorySize)> {
        self.memory_sizes(ledger).unwrap_or_default()
    }

    fn stable_cell_diagnostic(&self) -> StableCellDiagnostic {
        let memory = self.memory(MEMORY_MANAGER_LEDGER_ID);
        let memory_size = DiagnosticMemorySize::from_wasm_pages(memory.size());
        if memory.size() == 0 {
            return StableCellDiagnostic {
                diagnostic: DiagnosticStableCell::new(
                    DiagnosticStableCellStatus::Empty,
                    memory_size,
                ),
                record: Some(StableCellLedgerRecord::default()),
            };
        }

        match decode_stable_cell_ledger_record_from_memory(&memory) {
            Ok(record) => StableCellDiagnostic {
                diagnostic: DiagnosticStableCell::new(
                    DiagnosticStableCellStatus::Readable,
                    memory_size,
                ),
                record: Some(record),
            },
            Err(err) => StableCellDiagnostic {
                diagnostic: DiagnosticStableCell::new(
                    DiagnosticStableCellStatus::Corrupt {
                        failure: DiagnosticFailure::new(
                            DiagnosticCode::StableCell,
                            err.to_string(),
                        ),
                    },
                    memory_size,
                ),
                record: None,
            },
        }
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

struct StableCellDiagnostic {
    diagnostic: DiagnosticStableCell,
    record: Option<StableCellLedgerRecord>,
}

const fn ledger_anchor_descriptor() -> AllocationSlotDescriptor {
    AllocationSlotDescriptor::memory_manager_unchecked(MEMORY_MANAGER_LEDGER_ID)
}

fn external_runtime_allocations(committed: CommittedAllocations) -> CommittedAllocations {
    committed.without_stable_key_prefix(crate::IC_MEMORY_STABLE_KEY_PREFIX)
}

fn diagnostic_validation(
    declarations: &SealedDeclarationSnapshot,
    stable_cell_record: Option<&StableCellLedgerRecord>,
    recovered: Option<&Result<crate::RecoveredLedger, LedgerCommitError>>,
) -> DiagnosticCheck {
    let recovered = match diagnostic_validation_ledger(stable_cell_record, recovered) {
        Ok(recovered) => recovered,
        Err(failure) => return DiagnosticCheck::not_run(failure.code, failure.message),
    };
    let policy = RuntimeMemoryManagerPolicy {
        declarations,
        custom_policy: &NoopPolicy,
    };
    match crate::validate_allocations(
        &recovered,
        declarations.allocation_snapshot().clone(),
        &policy,
    ) {
        Ok(_) => DiagnosticCheck::passed(),
        Err(err) => DiagnosticCheck::failed(DiagnosticCode::AllocationValidation, err.to_string()),
    }
}

fn diagnostic_validation_ledger(
    stable_cell_record: Option<&StableCellLedgerRecord>,
    recovered: Option<&Result<crate::RecoveredLedger, LedgerCommitError>>,
) -> Result<crate::RecoveredLedger, DiagnosticFailure> {
    if let Some(Ok(recovered)) = recovered {
        return Ok(recovered.clone());
    }
    if let Some(Err(err)) = recovered {
        if stable_cell_record.is_some_and(|record| record.store().physical().is_uninitialized()) {
            return diagnostic_genesis_recovered_ledger();
        }
        let code = if matches!(
            err,
            LedgerCommitError::PayloadEnvelope(
                LedgerPayloadEnvelopeError::UnsupportedFormat { .. }
            )
        ) {
            DiagnosticCode::UnsupportedFormat
        } else {
            DiagnosticCode::LedgerRecovery
        };
        return Err(DiagnosticFailure::new(
            code,
            format!("protected ledger recovery: {err}"),
        ));
    }
    if stable_cell_record.is_some() {
        return diagnostic_genesis_recovered_ledger();
    }
    Err(DiagnosticFailure::new(
        DiagnosticCode::StableCell,
        "stable-cell ledger record is not readable",
    ))
}

fn diagnostic_genesis_recovered_ledger() -> Result<crate::RecoveredLedger, DiagnosticFailure> {
    AllocationLedger::new(0, AllocationHistory::default())
        .map(|ledger| crate::RecoveredLedger::from_trusted_parts(ledger, 0))
        .map_err(|err| {
            DiagnosticFailure::new(
                DiagnosticCode::GenesisLedger,
                format!("genesis ledger: {err}"),
            )
        })
}

fn runtime_bootstrap_error_from_bootstrap<P>(
    err: crate::BootstrapError<RuntimePolicyError<P>>,
) -> RuntimeBootstrapError<P> {
    match err {
        crate::BootstrapError::Ledger(err) => RuntimeBootstrapError::LedgerCommit(err),
        crate::BootstrapError::Validation(err) => RuntimeBootstrapError::Validation(err),
        crate::BootstrapError::Staging(err) => RuntimeBootstrapError::Staging(err),
    }
}

struct RuntimeMemoryManagerPolicy<'a, P> {
    declarations: &'a SealedDeclarationSnapshot,
    custom_policy: &'a P,
}

impl<P: AllocationPolicy> AllocationPolicy for RuntimeMemoryManagerPolicy<'_, P> {
    type Error = RuntimePolicyError<P::Error>;

    fn validate_key(&self, key: &StableKey) -> Result<(), Self::Error> {
        let authority = self.declaration_authority(key)?;
        if matches!(authority, RuntimeDeclarationAuthority::Internal) {
            return Ok(());
        }
        if crate::is_ic_memory_stable_key(key.as_str()) {
            return Err(RuntimePolicyError::ReservedStableKeyAuthority {
                stable_key: key.as_str().to_string(),
                expected_authority: IC_MEMORY_AUTHORITY_OWNER,
            });
        }
        self.custom_policy
            .validate_key(key)
            .map_err(RuntimePolicyError::Custom)
    }

    fn validate_slot(
        &self,
        key: &StableKey,
        slot: &AllocationSlotDescriptor,
    ) -> Result<(), Self::Error> {
        self.validate_runtime_range(key, slot)?;
        if matches!(
            self.declaration_authority(key)?,
            RuntimeDeclarationAuthority::Internal
        ) {
            return Ok(());
        }
        self.custom_policy
            .validate_slot(key, slot)
            .map_err(RuntimePolicyError::Custom)
    }

    fn validate_reserved_slot(
        &self,
        key: &StableKey,
        slot: &AllocationSlotDescriptor,
    ) -> Result<(), Self::Error> {
        self.validate_runtime_range(key, slot)?;
        if matches!(
            self.declaration_authority(key)?,
            RuntimeDeclarationAuthority::Internal
        ) {
            return Ok(());
        }
        self.custom_policy
            .validate_reserved_slot(key, slot)
            .map_err(RuntimePolicyError::Custom)
    }
}

impl<P: AllocationPolicy> RuntimeMemoryManagerPolicy<'_, P> {
    fn declaration_authority(
        &self,
        key: &StableKey,
    ) -> Result<&RuntimeDeclarationAuthority, RuntimePolicyError<P::Error>> {
        self.declarations
            .declaration_authority()
            .get(key.as_str())
            .ok_or_else(|| RuntimePolicyError::MissingDeclarationMetadata(key.as_str().to_string()))
    }

    fn validate_runtime_range(
        &self,
        key: &StableKey,
        slot: &AllocationSlotDescriptor,
    ) -> Result<(), RuntimePolicyError<P::Error>> {
        let authority = self.declaration_authority(key)?;
        if matches!(authority, RuntimeDeclarationAuthority::Internal) {
            self.declarations
                .range_authority()
                .validate_slot_authority(slot, IC_MEMORY_AUTHORITY_OWNER)?;
            return Ok(());
        }

        let RuntimeDeclarationAuthority::External(authority) = authority else {
            return Err(RuntimePolicyError::MissingDeclarationMetadata(
                key.as_str().to_string(),
            ));
        };
        if self.declarations.user_ranges_registered() {
            self.declarations
                .range_authority()
                .validate_slot_authority(slot, authority)?;
            return Ok(());
        }

        let id = slot
            .memory_manager_id()
            .map_err(MemoryManagerRangeAuthorityError::Slot)?;
        if self
            .declarations
            .range_authority()
            .authority_for_id(id)?
            .is_some()
        {
            self.declarations
                .range_authority()
                .validate_slot_authority(slot, authority)?;
        }
        Ok(())
    }
}

struct NoopPolicy;

impl AllocationPolicy for NoopPolicy {
    type Error = Infallible;

    fn validate_key(&self, _key: &StableKey) -> Result<(), Self::Error> {
        Ok(())
    }

    fn validate_slot(
        &self,
        _key: &StableKey,
        _slot: &AllocationSlotDescriptor,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn validate_reserved_slot(
        &self,
        _key: &StableKey,
        _slot: &AllocationSlotDescriptor,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

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
/// concrete TLS runtime.
pub fn bootstrap_default_memory_manager_with_policy<P: AllocationPolicy>(
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
mod tests {
    use super::*;
    use crate::registry::{
        TEST_REGISTRY_LOCK, register_static_memory_manager_declaration,
        register_static_memory_manager_range, reset_static_memory_declarations_for_tests,
    };
    use ic_stable_structures::VectorMemory;

    fn declarations() -> SealedDeclarationSnapshot {
        reset_static_memory_declarations_for_tests();
        register_static_memory_manager_range(
            120,
            120,
            "runtime_tests",
            crate::MemoryManagerRangeMode::Reserved,
            None,
        )
        .expect("test range");
        register_static_memory_manager_declaration(
            120,
            "runtime_tests",
            "rows",
            "runtime_tests.rows.v1",
        )
        .expect("test declaration");
        sealed_declaration_snapshot().expect("sealed declarations")
    }

    struct CountingPolicy(std::cell::Cell<usize>);

    impl AllocationPolicy for CountingPolicy {
        type Error = Infallible;

        fn validate_key(&self, _key: &StableKey) -> Result<(), Self::Error> {
            self.0.set(self.0.get() + 1);
            Ok(())
        }

        fn validate_slot(
            &self,
            _key: &StableKey,
            _slot: &AllocationSlotDescriptor,
        ) -> Result<(), Self::Error> {
            self.0.set(self.0.get() + 1);
            Ok(())
        }

        fn validate_reserved_slot(
            &self,
            _key: &StableKey,
            _slot: &AllocationSlotDescriptor,
        ) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    #[test]
    fn separate_runtimes_have_independent_bootstrap_authority_and_memory() {
        let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock");
        let declarations = declarations();
        let mut runtime_a = MemoryRuntime::new(VectorMemory::default());
        let mut runtime_b = MemoryRuntime::new(VectorMemory::default());
        let policy = CountingPolicy(std::cell::Cell::new(0));

        runtime_a
            .bootstrap(&declarations, &policy)
            .expect("runtime A bootstrap");
        assert_eq!(policy.0.get(), 2);
        assert!(runtime_a.is_bootstrapped());
        assert!(!runtime_b.is_bootstrapped());
        assert_eq!(
            runtime_b.committed_allocations().expect_err("runtime B"),
            RuntimeOpenError::NotBootstrapped
        );

        let memory_a = runtime_a
            .open_memory("runtime_tests.rows.v1", 120)
            .expect("runtime A memory");
        memory_a.grow(1);
        memory_a.write(0, b"runtime-a");

        runtime_b
            .bootstrap(&declarations, &policy)
            .expect("runtime B bootstrap");
        assert_eq!(policy.0.get(), 4);
        let memory_b = runtime_b
            .open_memory("runtime_tests.rows.v1", 120)
            .expect("runtime B memory");
        assert_eq!(memory_b.size(), 0);
        memory_b.grow(1);
        let mut bytes = [0; 9];
        memory_b.read(0, &mut bytes);
        assert_eq!(&bytes, &[0; 9]);
        assert_eq!(
            runtime_a
                .diagnostic_export()
                .expect("runtime A diagnostics")
                .current_generation,
            1
        );
        assert_eq!(
            runtime_b
                .diagnostic_export()
                .expect("runtime B diagnostics")
                .current_generation,
            1
        );
    }

    #[test]
    fn concurrent_independent_runtimes_do_not_share_bootstrap_state() {
        let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock");
        let declarations = declarations();
        let first_declarations = declarations.clone();
        let second_declarations = declarations;

        let (first, second) = std::thread::scope(|scope| {
            let first = scope.spawn(move || {
                let mut runtime = MemoryRuntime::new(VectorMemory::default());
                let generation = runtime
                    .bootstrap(&first_declarations, &NoopPolicy)
                    .expect("first bootstrap")
                    .generation();
                let diagnostic_generation = runtime
                    .diagnostic_export()
                    .expect("first diagnostics")
                    .current_generation;
                (generation, diagnostic_generation)
            });
            let second = scope.spawn(move || {
                let mut runtime = MemoryRuntime::new(VectorMemory::default());
                let generation = runtime
                    .bootstrap(&second_declarations, &NoopPolicy)
                    .expect("second bootstrap")
                    .generation();
                let diagnostic_generation = runtime
                    .diagnostic_export()
                    .expect("second diagnostics")
                    .current_generation;
                (generation, diagnostic_generation)
            });
            (
                first.join().expect("first runtime thread"),
                second.join().expect("second runtime thread"),
            )
        });

        assert_eq!(first, (1, 1));
        assert_eq!(second, (1, 1));
    }

    #[test]
    fn repeated_bootstrap_is_idempotent_and_existing_memory_recovers() {
        let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock");
        let declarations = declarations();
        let backing = VectorMemory::default();
        let generation = {
            let mut runtime = MemoryRuntime::new(backing.clone());
            let first = runtime
                .bootstrap(&declarations, &NoopPolicy)
                .expect("first bootstrap")
                .generation();
            let second = runtime
                .bootstrap(&declarations, &NoopPolicy)
                .expect("idempotent bootstrap")
                .generation();
            assert_eq!(first, second);
            let memory = runtime
                .open_memory("runtime_tests.rows.v1", 120)
                .expect("first runtime memory");
            memory.grow(1);
            memory.write(0, b"persisted");
            first
        };

        let mut recovered_runtime = MemoryRuntime::new(backing);
        let recovered_generation = recovered_runtime
            .bootstrap(&declarations, &NoopPolicy)
            .expect("recover existing backing memory")
            .generation();
        assert!(recovered_generation >= generation);
        assert_eq!(
            recovered_runtime
                .diagnostic_export()
                .expect("runtime diagnostic")
                .current_generation,
            recovered_generation
        );
        let memory = recovered_runtime
            .open_memory("runtime_tests.rows.v1", 120)
            .expect("recovered runtime memory");
        let mut bytes = [0; 9];
        memory.read(0, &mut bytes);
        assert_eq!(&bytes, b"persisted");
    }

    struct RejectPolicy;

    impl AllocationPolicy for RejectPolicy {
        type Error = &'static str;

        fn validate_key(&self, _key: &StableKey) -> Result<(), Self::Error> {
            Err("rejected")
        }

        fn validate_slot(
            &self,
            _key: &StableKey,
            _slot: &AllocationSlotDescriptor,
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        fn validate_reserved_slot(
            &self,
            _key: &StableKey,
            _slot: &AllocationSlotDescriptor,
        ) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    #[test]
    fn open_errors_and_failed_bootstrap_do_not_publish_authority() {
        let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock");
        let declarations = declarations();
        let mut runtime = MemoryRuntime::new(VectorMemory::default());

        let Err(open_before_bootstrap) = runtime.open_memory("runtime_tests.rows.v1", 120) else {
            panic!("open before bootstrap must fail");
        };
        assert_eq!(open_before_bootstrap, RuntimeOpenError::NotBootstrapped);
        assert!(runtime.bootstrap(&declarations, &RejectPolicy).is_err());
        assert!(!runtime.is_bootstrapped());
        assert!(!runtime.doctor_report(&declarations).bootstrapped);
        assert!(matches!(
            runtime.diagnostic_export(),
            Err(RuntimeDiagnosticError::NotBootstrapped)
        ));
        assert_eq!(
            runtime.committed_allocations().expect_err("no capability"),
            RuntimeOpenError::NotBootstrapped
        );

        runtime
            .bootstrap(&declarations, &NoopPolicy)
            .expect("successful retry");
        let Err(wrong_key) = runtime.open_memory("runtime_tests.missing.v1", 120) else {
            panic!("wrong key must fail");
        };
        assert!(matches!(
            wrong_key,
            RuntimeOpenError::StableKeyNotCommitted(_)
        ));
        let Err(wrong_id) = runtime.open_memory("runtime_tests.rows.v1", 121) else {
            panic!("wrong ID must fail");
        };
        assert!(matches!(
            wrong_id,
            RuntimeOpenError::MemoryIdMismatch { .. }
        ));
    }

    #[test]
    fn doctor_and_diagnostics_report_the_same_runtime_lifecycle() {
        let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock");
        let declarations = declarations();
        let mut runtime = MemoryRuntime::new(VectorMemory::default());

        assert!(!runtime.doctor_report(&declarations).bootstrapped);
        assert!(matches!(
            runtime.diagnostic_export(),
            Err(RuntimeDiagnosticError::NotBootstrapped)
        ));

        runtime
            .bootstrap(&declarations, &NoopPolicy)
            .expect("bootstrap");
        let memory = runtime
            .open_memory("runtime_tests.rows.v1", 120)
            .expect("open");
        memory.grow(2);
        let doctor = runtime.doctor_report(&declarations);
        let export = runtime.diagnostic_export().expect("diagnostic export");
        assert!(doctor.bootstrapped);
        assert_eq!(
            doctor
                .ledger
                .as_ref()
                .expect("doctor ledger")
                .current_generation,
            export.current_generation
        );
        assert_eq!(
            export
                .records
                .iter()
                .find(|record| {
                    record.allocation.stable_key().as_str() == "runtime_tests.rows.v1"
                })
                .expect("runtime record")
                .memory_size,
            Some(DiagnosticMemorySize::from_wasm_pages(2))
        );
    }

    #[test]
    fn default_runtime_reentry_is_a_typed_state_error() {
        DEFAULT_RUNTIME.with(|runtime| {
            let _borrow = runtime.borrow_mut();
            assert_eq!(
                is_default_memory_manager_bootstrapped().expect_err("re-entry"),
                RuntimeStateError::ReentrantAccess
            );
        });
    }

    #[test]
    fn validation_diagnostic_preserves_unsupported_format_code() {
        let recovered = Err(LedgerCommitError::PayloadEnvelope(
            LedgerPayloadEnvelopeError::UnsupportedFormat {
                marker: *b"ICMF",
                version: Some(crate::LEDGER_PAYLOAD_FORMAT_VERSION + 1),
            },
        ));

        let failure = diagnostic_validation_ledger(None, Some(&recovered))
            .expect_err("unsupported format must block validation");

        assert_eq!(failure.code, DiagnosticCode::UnsupportedFormat);
        assert!(
            failure
                .message
                .contains("unsupported ic-memory ledger payload format")
        );
    }
}
