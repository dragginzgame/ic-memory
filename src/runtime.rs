use crate::{
    AllocationBootstrap, AllocationDeclaration, AllocationHistory, AllocationLedger,
    AllocationPolicy, AllocationSlotDescriptor, CommittedAllocations, DeclarationSnapshot,
    DefaultMemoryManagerDoctorReport, DiagnosticCheck, DiagnosticDeclaration, DiagnosticExport,
    DiagnosticMemorySize, DiagnosticRangeAuthority, DiagnosticStableCell,
    DiagnosticStableCellStatus, LedgerCommitError, STABLE_CELL_VALUE_OFFSET, StableCellLedgerError,
    StableCellLedgerRecord, StableKey,
    physical::CommitStoreDiagnostic,
    registry::{
        StaticMemoryDeclaration, StaticMemoryDeclarationError, StaticMemoryRangeDeclaration,
        seal_static_memory_registry, static_memory_declarations, static_memory_range_declarations,
    },
    slot::{
        IC_MEMORY_AUTHORITY_OWNER, IC_MEMORY_AUTHORITY_PURPOSE, IC_MEMORY_LEDGER_LABEL,
        IC_MEMORY_LEDGER_STABLE_KEY, MEMORY_MANAGER_LEDGER_ID, MemoryManagerAuthorityRecord,
        MemoryManagerIdRange, MemoryManagerRangeAuthority, MemoryManagerRangeAuthorityError,
        MemoryManagerRangeMode, MemoryManagerSlotError,
    },
    stable_cell::decode_stable_cell_ledger_record_from_memory,
};
use ic_stable_structures::{
    Cell, DefaultMemoryImpl, Memory, Storable,
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    convert::Infallible,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

type DefaultLedgerCell = Cell<StableCellLedgerRecord, VirtualMemory<DefaultMemoryImpl>>;

thread_local! {
    static DEFAULT_MEMORY_MANAGER: MemoryManager<DefaultMemoryImpl> =
        MemoryManager::init(DefaultMemoryImpl::default());
    static DEFAULT_LEDGER_CELL: RefCell<Option<DefaultLedgerCell>> = const {
        RefCell::new(None)
    };
}

static EAGER_INIT_HOOKS: Mutex<Vec<fn()>> = Mutex::new(Vec::new());
static COMMITTED_ALLOCATIONS: Mutex<Option<CommittedAllocations>> = Mutex::new(None);
static BOOTSTRAPPED: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RuntimeLockPoisoned;

impl RuntimeLockPoisoned {
    const MESSAGE: &'static str = "ic-memory runtime lock poisoned";
}

///
/// RuntimeBootstrapError
///
/// Failure to bootstrap the generic `ic-memory` runtime layer.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum RuntimeBootstrapError<P> {
    /// Runtime registration or snapshot collection failed.
    #[error(transparent)]
    Registry(#[from] StaticMemoryDeclarationError),
    /// Runtime range authority table is invalid.
    #[error(transparent)]
    Range(#[from] MemoryManagerRangeAuthorityError),
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
    /// Runtime state lock was poisoned.
    #[error("ic-memory runtime lock poisoned")]
    RuntimeLockPoisoned,
}

///
/// RuntimeOpenError
///
/// Failure to open a committed allocation through the default runtime substrate.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum RuntimeOpenError {
    /// Runtime bootstrap has not published committed allocations.
    #[error("ic-memory runtime has not completed bootstrap validation")]
    NotBootstrapped,
    /// Runtime state lock was poisoned.
    #[error("ic-memory runtime lock poisoned")]
    RuntimeLockPoisoned,
    /// Stable-key grammar failure.
    #[error(transparent)]
    StableKey(#[from] crate::StableKeyError),
    /// The stable key was not present in the committed declaration set.
    #[error("stable key '{0}' was not committed by ic-memory runtime bootstrap")]
    StableKeyNotCommitted(String),
    /// Runtime governance stable keys are internal and cannot be opened through the public runtime.
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
/// Failure to build diagnostics for the default `MemoryManager` runtime.
///

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum RuntimeDiagnosticError {
    /// Runtime bootstrap has not opened and validated the ledger cell.
    #[error("ic-memory runtime has not completed bootstrap validation")]
    NotBootstrapped,
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

/// Register a pre-bootstrap declaration hook.
#[doc(hidden)]
pub fn defer_eager_init(f: fn()) {
    assert!(
        !is_default_memory_manager_bootstrapped(),
        "ic-memory eager-init registration attempted after runtime bootstrap"
    );
    EAGER_INIT_HOOKS
        .lock()
        .expect("ic-memory eager-init queue poisoned")
        .push(f);
}

/// Return true once default runtime bootstrap has completed.
#[must_use]
pub fn is_default_memory_manager_bootstrapped() -> bool {
    BOOTSTRAPPED.load(Ordering::SeqCst)
}

/// Return the published committed allocations for the default runtime substrate.
pub fn committed_allocations() -> Result<CommittedAllocations, RuntimeOpenError> {
    if !is_default_memory_manager_bootstrapped() {
        return Err(RuntimeOpenError::NotBootstrapped);
    }
    COMMITTED_ALLOCATIONS
        .lock()
        .map_err(|_| RuntimeOpenError::RuntimeLockPoisoned)?
        .clone()
        .ok_or(RuntimeOpenError::NotBootstrapped)
}

/// Bootstrap the default `MemoryManager<DefaultMemoryImpl>` runtime using generic policy.
pub fn bootstrap_default_memory_manager()
-> Result<CommittedAllocations, RuntimeBootstrapError<Infallible>> {
    bootstrap_default_memory_manager_with_policy(&NoopPolicy)
}

/// Bootstrap the default runtime and layer caller-supplied policy over generic range checks.
///
/// Authority order is explicit:
///
/// 1. `ic-memory` always owns its governance range.
/// 2. If any user range is registered, all `MemoryManager` declarations must
///    belong to the range claimed by their authority.
/// 3. The caller-supplied [`AllocationPolicy`] then applies framework-specific
///    namespace and lifecycle rules.
///
/// Framework adapters such as Canic should register only the ranges they want
/// this generic runtime to enforce. If a framework wants its own policy to be
/// authoritative for application space, it should omit user range registrations
/// for that space and enforce the rule in its [`AllocationPolicy`].
pub fn bootstrap_default_memory_manager_with_policy<P: AllocationPolicy>(
    policy: &P,
) -> Result<CommittedAllocations, RuntimeBootstrapError<P::Error>> {
    if let Ok(committed) = committed_allocations() {
        return Ok(committed);
    }

    run_eager_init_hooks().map_err(|_err| RuntimeBootstrapError::RuntimeLockPoisoned)?;

    let registered_declarations = static_memory_declarations()?;
    let registered_ranges = static_memory_range_declarations()?;
    let user_ranges_registered = !registered_ranges.is_empty();
    let declaration_metadata = declaration_metadata(&registered_declarations);
    let range_authority = range_authority(registered_ranges)?;
    let snapshot = declaration_snapshot(registered_declarations)?;
    seal_static_memory_registry()?;
    let policy = RuntimeMemoryManagerPolicy {
        range_authority,
        user_ranges_registered,
        declaration_metadata,
        custom_policy: policy,
    };
    let genesis = AllocationLedger::new(0, AllocationHistory::default())?;

    let committed = with_default_ledger_cell(
        |cell| -> Result<CommittedAllocations, RuntimeBootstrapError<P::Error>> {
            let mut record = cell.get().clone();
            let mut bootstrap = AllocationBootstrap::new(record.store_mut());
            let commit = bootstrap
                .initialize_validate_and_commit(&genesis, snapshot, &policy, None)
                .map_err(runtime_bootstrap_error_from_bootstrap)?;
            let (ledger, validated) = commit.into_parts();
            set_default_ledger_cell(cell, record)?;
            Ok(external_runtime_allocations(
                validated.confirm_persisted(ledger.current_generation()),
            ))
        },
    )?;

    publish_committed_allocations(committed.clone())?;
    BOOTSTRAPPED.store(true, Ordering::SeqCst);
    Ok(committed)
}

/// Open a committed `MemoryManager` memory by stable key and expected ID.
pub fn open_default_memory_manager_memory(
    stable_key: &str,
    id: u8,
) -> Result<VirtualMemory<DefaultMemoryImpl>, RuntimeOpenError> {
    let key = StableKey::parse(stable_key)?;
    if crate::is_ic_memory_stable_key(key.as_str()) {
        return Err(RuntimeOpenError::ReservedStableKey {
            stable_key: stable_key.to_string(),
        });
    }
    let committed = committed_allocations()?;
    let slot = committed
        .slot_for(&key)
        .ok_or_else(|| RuntimeOpenError::StableKeyNotCommitted(stable_key.to_string()))?;
    let committed_id = slot.memory_manager_id()?;
    if committed_id != id {
        return Err(RuntimeOpenError::MemoryIdMismatch {
            stable_key: stable_key.to_string(),
            committed_id,
            requested_id: id,
        });
    }
    Ok(default_memory_manager_memory(id))
}

/// Build a diagnostic export for the default `MemoryManager` runtime.
///
/// Each allocation record includes the live `VirtualMemory::size()` for its
/// slot when the committed ledger can be recovered. The reported size is the
/// virtual memory size in WebAssembly pages and bytes, not logical data bytes
/// stored by a particular stable-structure collection.
pub fn default_memory_manager_diagnostic_export() -> Result<DiagnosticExport, RuntimeDiagnosticError>
{
    let record = default_ledger_record_for_diagnostics()?;
    let recovered = record.store().recover()?;
    let ledger = recovered.ledger();
    let memory_sizes = default_memory_manager_memory_sizes(ledger)?;

    Ok(
        DiagnosticExport::from_ledger_with_commit_recovery_and_memory_sizes(
            ledger,
            AllocationSlotDescriptor::memory_manager(MEMORY_MANAGER_LEDGER_ID)?,
            Some(record.store().physical().diagnostic()),
            memory_sizes,
        ),
    )
}

/// Build a protected commit recovery diagnostic for the default ledger store.
///
/// Unlike [`default_memory_manager_diagnostic_export`], this helper does not
/// require successful bootstrap. It can diagnose empty or partially corrupt
/// dual-slot commit state as long as the enclosing stable-cell ledger record is
/// readable.
pub fn default_memory_manager_commit_recovery_diagnostic()
-> Result<CommitStoreDiagnostic, RuntimeDiagnosticError> {
    let record = default_ledger_record_from_memory()?;
    Ok(record.store().physical().diagnostic())
}

/// Build a preflight and runtime diagnostic report for the default runtime.
///
/// The doctor report can be collected before bootstrap, after bootstrap, or
/// after a failed bootstrap attempt. Stable-cell, commit-recovery, declaration,
/// range-authority, validation, ledger, and live memory-size status are
/// collected into one serializable report. Recoverable problems are reported in
/// fields rather than returned as errors.
#[must_use]
pub fn default_memory_manager_doctor_report() -> DefaultMemoryManagerDoctorReport {
    let bootstrapped = is_default_memory_manager_bootstrapped();
    let eager_init_error = if bootstrapped {
        None
    } else {
        run_eager_init_hooks()
            .err()
            .map(|_err| format!("eager-init hooks: {}", RuntimeLockPoisoned::MESSAGE))
    };

    let stable_cell = default_memory_manager_stable_cell_diagnostic();
    let commit_recovery = stable_cell
        .record
        .as_ref()
        .map(|record| record.store().physical().diagnostic());
    let recovered = stable_cell
        .record
        .as_ref()
        .map(|record| record.store().recover());
    let recovered_for_export = recovered.as_ref().and_then(|result| result.as_ref().ok());
    let ledger_anchor = default_ledger_anchor_descriptor();
    let ledger = recovered_for_export.map(|recovered| {
        DiagnosticExport::from_ledger_with_commit_recovery_and_memory_sizes(
            recovered.ledger(),
            ledger_anchor.clone(),
            commit_recovery,
            default_memory_manager_memory_sizes_lossy(recovered.ledger()),
        )
    });

    let registered_declarations = static_memory_declarations();
    let registered_ranges = static_memory_range_declarations();
    let diagnostic_declarations = registered_declarations
        .as_ref()
        .map(|declarations| {
            declarations
                .iter()
                .map(|registration| {
                    DiagnosticDeclaration::new(
                        registration.authority(),
                        registration.declaration().clone(),
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    let range_authority = diagnostic_range_authority(&registered_ranges);
    let validation = eager_init_error.map_or_else(
        || {
            diagnostic_validation(
                &registered_declarations,
                &registered_ranges,
                stable_cell.record.as_ref(),
                recovered.as_ref(),
            )
        },
        DiagnosticCheck::failed,
    );

    DefaultMemoryManagerDoctorReport {
        bootstrapped: BOOTSTRAPPED.load(Ordering::SeqCst),
        ledger_anchor,
        stable_cell: stable_cell.diagnostic,
        commit_recovery,
        ledger,
        registered_declarations: diagnostic_declarations,
        range_authority,
        validation,
    }
}

fn run_eager_init_hooks() -> Result<(), RuntimeLockPoisoned> {
    let hooks = {
        let mut hooks = EAGER_INIT_HOOKS.lock().map_err(|_| RuntimeLockPoisoned)?;
        std::mem::take(&mut *hooks)
    };

    for hook in hooks {
        hook();
    }
    Ok(())
}

fn with_default_ledger_cell<P, T>(
    op: impl FnOnce(&mut DefaultLedgerCell) -> Result<T, RuntimeBootstrapError<P>>,
) -> Result<T, RuntimeBootstrapError<P>> {
    DEFAULT_LEDGER_CELL.with(|cell| {
        let mut cell = cell.borrow_mut();
        if cell.is_none() {
            let memory = default_memory_manager_memory(MEMORY_MANAGER_LEDGER_ID);
            crate::validate_stable_cell_ledger_memory(&memory)?;
            *cell = Some(Cell::init(memory, StableCellLedgerRecord::default()));
        }
        let Some(cell) = cell.as_mut() else {
            return Err(RuntimeBootstrapError::RuntimeLockPoisoned);
        };
        op(cell)
    })
}

fn set_default_ledger_cell<P>(
    cell: &mut DefaultLedgerCell,
    record: StableCellLedgerRecord,
) -> Result<(), RuntimeBootstrapError<P>> {
    ensure_default_ledger_cell_capacity(&record)?;
    let _previous = cell.set(record);
    Ok(())
}

fn ensure_default_ledger_cell_capacity<P>(
    record: &StableCellLedgerRecord,
) -> Result<(), RuntimeBootstrapError<P>> {
    let encoded = record.to_bytes();
    let value_size = encoded.len();
    if value_size > u32::MAX as usize {
        return Err(RuntimeBootstrapError::StableCellLedgerWriteTooLarge { value_size });
    }

    let value_size_u32 = u32::try_from(value_size)
        .map_err(|_| RuntimeBootstrapError::StableCellLedgerWriteTooLarge { value_size })?;
    let value_size_u64 = u64::from(value_size_u32);
    let required_bytes = STABLE_CELL_VALUE_OFFSET
        .checked_add(value_size_u64)
        .ok_or(RuntimeBootstrapError::StableCellLedgerWriteTooLarge { value_size })?;
    let memory = default_memory_manager_memory(MEMORY_MANAGER_LEDGER_ID);
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

fn default_memory_manager_memory(id: u8) -> VirtualMemory<DefaultMemoryImpl> {
    DEFAULT_MEMORY_MANAGER.with(|manager| manager.get(MemoryId::new(id)))
}

const fn default_ledger_anchor_descriptor() -> AllocationSlotDescriptor {
    AllocationSlotDescriptor::memory_manager_unchecked(MEMORY_MANAGER_LEDGER_ID)
}

fn default_ledger_record_for_diagnostics() -> Result<StableCellLedgerRecord, RuntimeDiagnosticError>
{
    if !is_default_memory_manager_bootstrapped() {
        return Err(RuntimeDiagnosticError::NotBootstrapped);
    }

    default_ledger_record_from_memory().map_err(RuntimeDiagnosticError::StableCellLedger)
}

fn default_ledger_record_from_memory() -> Result<StableCellLedgerRecord, StableCellLedgerError> {
    let memory = default_memory_manager_memory(MEMORY_MANAGER_LEDGER_ID);
    decode_stable_cell_ledger_record_from_memory(&memory)
}

fn default_memory_manager_memory_sizes(
    ledger: &AllocationLedger,
) -> Result<Vec<(AllocationSlotDescriptor, DiagnosticMemorySize)>, RuntimeDiagnosticError> {
    ledger
        .allocation_history()
        .records()
        .iter()
        .map(|record| {
            let id = record.slot().memory_manager_id()?;
            let memory = default_memory_manager_memory(id);
            Ok((
                record.slot().clone(),
                DiagnosticMemorySize::from_wasm_pages(memory.size()),
            ))
        })
        .collect()
}

fn default_memory_manager_memory_sizes_lossy(
    ledger: &AllocationLedger,
) -> Vec<(AllocationSlotDescriptor, DiagnosticMemorySize)> {
    default_memory_manager_memory_sizes(ledger).unwrap_or_default()
}

struct DefaultStableCellDiagnostic {
    diagnostic: DiagnosticStableCell,
    record: Option<StableCellLedgerRecord>,
}

fn default_memory_manager_stable_cell_diagnostic() -> DefaultStableCellDiagnostic {
    let memory = default_memory_manager_memory(MEMORY_MANAGER_LEDGER_ID);
    let memory_size = DiagnosticMemorySize::from_wasm_pages(memory.size());
    if memory.size() == 0 {
        return DefaultStableCellDiagnostic {
            diagnostic: DiagnosticStableCell::new(
                DiagnosticStableCellStatus::Empty,
                memory_size,
                None,
            ),
            record: Some(StableCellLedgerRecord::default()),
        };
    }

    let record = decode_stable_cell_ledger_record_from_memory(&memory);
    match record {
        Ok(record) => DefaultStableCellDiagnostic {
            diagnostic: DiagnosticStableCell::new(
                DiagnosticStableCellStatus::Readable,
                memory_size,
                None,
            ),
            record: Some(record),
        },
        Err(err) => DefaultStableCellDiagnostic {
            diagnostic: DiagnosticStableCell::new(
                DiagnosticStableCellStatus::Corrupt,
                memory_size,
                Some(err.to_string()),
            ),
            record: None,
        },
    }
}

fn diagnostic_range_authority(
    registered_ranges: &Result<Vec<StaticMemoryRangeDeclaration>, StaticMemoryDeclarationError>,
) -> DiagnosticRangeAuthority {
    match registered_ranges {
        Ok(ranges) => {
            let registered_records = ranges
                .iter()
                .map(|registration| registration.record().clone())
                .collect();
            match range_authority(ranges.clone()) {
                Ok(authority) => {
                    DiagnosticRangeAuthority::new(registered_records, Some(authority), None)
                }
                Err(err) => {
                    DiagnosticRangeAuthority::new(registered_records, None, Some(err.to_string()))
                }
            }
        }
        Err(err) => DiagnosticRangeAuthority::new(Vec::new(), None, Some(err.to_string())),
    }
}

fn diagnostic_validation(
    registered_declarations: &Result<Vec<StaticMemoryDeclaration>, StaticMemoryDeclarationError>,
    registered_ranges: &Result<Vec<StaticMemoryRangeDeclaration>, StaticMemoryDeclarationError>,
    stable_cell_record: Option<&StableCellLedgerRecord>,
    recovered: Option<&Result<crate::RecoveredLedger, LedgerCommitError>>,
) -> DiagnosticCheck {
    let registered_declarations = match registered_declarations {
        Ok(declarations) => declarations.clone(),
        Err(err) => return DiagnosticCheck::failed(format!("declaration registry: {err}")),
    };
    let registered_ranges = match registered_ranges {
        Ok(ranges) => ranges.clone(),
        Err(err) => return DiagnosticCheck::failed(format!("range registry: {err}")),
    };
    let range_authority = match range_authority(registered_ranges.clone()) {
        Ok(authority) => authority,
        Err(err) => return DiagnosticCheck::failed(format!("range authority: {err}")),
    };
    let snapshot = match declaration_snapshot(registered_declarations.clone()) {
        Ok(snapshot) => snapshot,
        Err(err) => return DiagnosticCheck::failed(format!("declaration snapshot: {err}")),
    };
    let recovered = match diagnostic_validation_ledger(stable_cell_record, recovered) {
        Ok(recovered) => recovered,
        Err(reason) => return DiagnosticCheck::not_run(reason),
    };
    let policy = RuntimeMemoryManagerPolicy {
        range_authority,
        user_ranges_registered: !registered_ranges.is_empty(),
        declaration_metadata: declaration_metadata(&registered_declarations),
        custom_policy: &NoopPolicy,
    };

    match crate::validate_allocations(&recovered, snapshot, &policy) {
        Ok(_) => DiagnosticCheck::passed(),
        Err(err) => DiagnosticCheck::failed(err.to_string()),
    }
}

fn diagnostic_validation_ledger(
    stable_cell_record: Option<&StableCellLedgerRecord>,
    recovered: Option<&Result<crate::RecoveredLedger, LedgerCommitError>>,
) -> Result<crate::RecoveredLedger, String> {
    if let Some(Ok(recovered)) = recovered {
        return Ok(recovered.clone());
    }
    if let Some(Err(err)) = recovered {
        if stable_cell_record.is_some_and(|record| record.store().physical().is_uninitialized()) {
            return diagnostic_genesis_recovered_ledger();
        }
        return Err(format!("protected ledger recovery: {err}"));
    }
    if stable_cell_record.is_some() {
        return diagnostic_genesis_recovered_ledger();
    }
    Err("stable-cell ledger record is not readable".to_string())
}

fn diagnostic_genesis_recovered_ledger() -> Result<crate::RecoveredLedger, String> {
    AllocationLedger::new(0, AllocationHistory::default())
        .map(|ledger| crate::RecoveredLedger::from_trusted_parts(ledger, 0))
        .map_err(|err| format!("genesis ledger: {err}"))
}

fn publish_committed_allocations<P>(
    committed: CommittedAllocations,
) -> Result<(), RuntimeBootstrapError<P>> {
    *COMMITTED_ALLOCATIONS
        .lock()
        .map_err(|_| RuntimeBootstrapError::RuntimeLockPoisoned)? = Some(committed);
    Ok(())
}

fn declaration_snapshot(
    registrations: Vec<StaticMemoryDeclaration>,
) -> Result<DeclarationSnapshot, StaticMemoryDeclarationError> {
    let mut declarations = Vec::with_capacity(registrations.len() + 1);
    declarations.push(internal_ledger_declaration()?);
    declarations.extend(
        registrations
            .into_iter()
            .map(StaticMemoryDeclaration::into_declaration),
    );
    DeclarationSnapshot::new(declarations).map_err(StaticMemoryDeclarationError::Declaration)
}

fn declaration_metadata(
    registrations: &[StaticMemoryDeclaration],
) -> BTreeMap<String, RuntimeDeclarationAuthority> {
    let mut metadata = BTreeMap::new();
    metadata.insert(
        IC_MEMORY_LEDGER_STABLE_KEY.to_string(),
        RuntimeDeclarationAuthority::Internal,
    );
    for registration in registrations {
        metadata.insert(
            registration.declaration().stable_key().as_str().to_string(),
            RuntimeDeclarationAuthority::External(registration.authority().to_string()),
        );
    }
    metadata
}

fn range_authority(
    registrations: Vec<StaticMemoryRangeDeclaration>,
) -> Result<MemoryManagerRangeAuthority, MemoryManagerRangeAuthorityError> {
    let mut records = Vec::with_capacity(registrations.len() + 1);
    records.push(internal_ledger_range()?);
    records.extend(
        registrations
            .into_iter()
            .map(StaticMemoryRangeDeclaration::into_record),
    );
    MemoryManagerRangeAuthority::from_records(records)
}

fn internal_ledger_declaration() -> Result<AllocationDeclaration, crate::DeclarationSnapshotError> {
    AllocationDeclaration::memory_manager(
        IC_MEMORY_LEDGER_STABLE_KEY,
        MEMORY_MANAGER_LEDGER_ID,
        IC_MEMORY_LEDGER_LABEL,
    )
}

fn internal_ledger_range() -> Result<MemoryManagerAuthorityRecord, MemoryManagerRangeAuthorityError>
{
    MemoryManagerAuthorityRecord::new(
        MemoryManagerIdRange::new(
            MEMORY_MANAGER_LEDGER_ID,
            crate::MEMORY_MANAGER_GOVERNANCE_MAX_ID,
        )?,
        IC_MEMORY_AUTHORITY_OWNER,
        MemoryManagerRangeMode::Reserved,
        Some(IC_MEMORY_AUTHORITY_PURPOSE.to_string()),
    )
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
    range_authority: MemoryManagerRangeAuthority,
    user_ranges_registered: bool,
    declaration_metadata: BTreeMap<String, RuntimeDeclarationAuthority>,
    custom_policy: &'a P,
}

enum RuntimeDeclarationAuthority {
    Internal,
    External(String),
}

impl<P: AllocationPolicy> AllocationPolicy for RuntimeMemoryManagerPolicy<'_, P> {
    type Error = RuntimePolicyError<P::Error>;

    fn validate_key(&self, key: &StableKey) -> Result<(), Self::Error> {
        let authority = self.declaration_authority(key)?;
        if crate::is_ic_memory_stable_key(key.as_str())
            && !matches!(authority, RuntimeDeclarationAuthority::Internal)
        {
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
        self.declaration_metadata
            .get(key.as_str())
            .ok_or_else(|| RuntimePolicyError::MissingDeclarationMetadata(key.as_str().to_string()))
    }

    fn validate_runtime_range(
        &self,
        key: &StableKey,
        slot: &AllocationSlotDescriptor,
    ) -> Result<(), RuntimePolicyError<P::Error>> {
        let authority = self.declaration_authority(key)?;
        // Range claims are authoritative generic policy in the default runtime.
        // Once any user range is registered, every user declaration must fit
        // the authority's claimed range. With no user ranges, only the
        // internal ic-memory governance range is enforced here and custom
        // policy may decide application-space ownership.
        if matches!(authority, RuntimeDeclarationAuthority::Internal) {
            self.range_authority
                .validate_slot_authority(slot, IC_MEMORY_AUTHORITY_OWNER)?;
            return Ok(());
        }

        let RuntimeDeclarationAuthority::External(authority) = authority else {
            return Err(RuntimePolicyError::MissingDeclarationMetadata(
                key.as_str().to_string(),
            ));
        };
        if self.user_ranges_registered {
            self.range_authority
                .validate_slot_authority(slot, authority)?;
            return Ok(());
        }

        let id = slot
            .memory_manager_id()
            .map_err(MemoryManagerRangeAuthorityError::Slot)?;
        if self
            .range_authority
            .authority_for_id(id)
            .map_err(RuntimePolicyError::Range)?
            .is_some()
        {
            self.range_authority
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

#[cfg(test)]
pub fn reset_for_tests() {
    crate::registry::reset_static_memory_declarations_for_tests();
    EAGER_INIT_HOOKS
        .lock()
        .expect("ic-memory eager-init queue poisoned")
        .clear();
    *COMMITTED_ALLOCATIONS
        .lock()
        .expect("ic-memory runtime validation state poisoned") = None;
    BOOTSTRAPPED.store(false, Ordering::SeqCst);
    DEFAULT_LEDGER_CELL.with_borrow_mut(|cell| {
        *cell = None;
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::{
        TEST_REGISTRY_LOCK, register_static_memory_manager_declaration,
        register_static_memory_manager_range,
    };
    use std::sync::atomic::{AtomicBool, Ordering};

    static EAGER_INIT_RAN: AtomicBool = AtomicBool::new(false);

    fn register_crate_a() {
        register_static_memory_manager_range(
            100,
            109,
            "crate_a",
            MemoryManagerRangeMode::Reserved,
            None,
        )
        .expect("crate A range");
        register_static_memory_manager_declaration(100, "crate_a", "users", "crate_a.users.v1")
            .expect("crate A memory");
    }

    fn register_crate_b() {
        register_static_memory_manager_range(
            110,
            119,
            "crate_b",
            MemoryManagerRangeMode::Reserved,
            None,
        )
        .expect("crate B range");
        register_static_memory_manager_declaration(110, "crate_b", "orders", "crate_b.orders.v1")
            .expect("crate B memory");
    }

    fn mark_eager_init() {
        EAGER_INIT_RAN.store(true, Ordering::SeqCst);
        register_static_memory_manager_declaration(101, "crate_a", "audit", "crate_a.audit.v1")
            .expect("eager-init declaration");
    }

    #[test]
    fn multi_crate_declarations_compose_into_one_bootstrap() {
        let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
        reset_for_tests();
        register_crate_a();
        register_crate_b();

        let validated = bootstrap_default_memory_manager().expect("bootstrap");

        assert_eq!(validated.declarations().len(), 2);
        assert!(
            validated
                .declarations()
                .iter()
                .any(|declaration| declaration.stable_key().as_str() == "crate_a.users.v1")
        );
        assert!(
            validated
                .declarations()
                .iter()
                .any(|declaration| declaration.stable_key().as_str() == "crate_b.orders.v1")
        );
    }

    #[test]
    fn default_runtime_keeps_internal_ledger_slot_private() {
        let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
        reset_for_tests();

        let validated = bootstrap_default_memory_manager().expect("bootstrap");

        assert!(validated.declarations().is_empty());
        assert!(
            committed_allocations()
                .expect("published allocations")
                .declarations()
                .is_empty()
        );
        let Err(err) = open_default_memory_manager_memory(
            IC_MEMORY_LEDGER_STABLE_KEY,
            MEMORY_MANAGER_LEDGER_ID,
        ) else {
            panic!("internal ledger slot must stay private");
        };
        assert!(matches!(err, RuntimeOpenError::ReservedStableKey { .. }));
    }

    #[test]
    fn conflicting_ranges_fail() {
        let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
        reset_for_tests();
        register_static_memory_manager_range(
            100,
            110,
            "crate_a",
            MemoryManagerRangeMode::Reserved,
            None,
        )
        .expect("crate A range");
        register_static_memory_manager_range(
            105,
            119,
            "crate_b",
            MemoryManagerRangeMode::Reserved,
            None,
        )
        .expect("crate B range");

        let err = bootstrap_default_memory_manager().expect_err("overlap must fail");
        assert!(matches!(
            err,
            RuntimeBootstrapError::Range(
                MemoryManagerRangeAuthorityError::OverlappingRanges { .. }
            )
        ));
    }

    #[test]
    fn duplicate_stable_keys_fail() {
        let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
        reset_for_tests();
        register_static_memory_manager_declaration(100, "crate_a", "users", "app.users.v1")
            .expect("first declaration");
        register_static_memory_manager_declaration(101, "crate_b", "users", "app.users.v1")
            .expect("second declaration");

        let err = bootstrap_default_memory_manager().expect_err("duplicate key must fail");
        assert!(matches!(
            err,
            RuntimeBootstrapError::Registry(StaticMemoryDeclarationError::Declaration(
                crate::DeclarationSnapshotError::DuplicateStableKey(_)
            ))
        ));
    }

    #[test]
    fn duplicate_memory_manager_ids_fail() {
        let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
        reset_for_tests();
        register_static_memory_manager_declaration(100, "crate_a", "users", "crate_a.users.v1")
            .expect("first declaration");
        register_static_memory_manager_declaration(100, "crate_b", "orders", "crate_b.orders.v1")
            .expect("second declaration");

        let err = bootstrap_default_memory_manager().expect_err("duplicate slot must fail");
        assert!(matches!(
            err,
            RuntimeBootstrapError::Registry(StaticMemoryDeclarationError::Declaration(
                crate::DeclarationSnapshotError::DuplicateSlot(_)
            ))
        ));
    }

    #[test]
    fn out_of_range_memory_declaration_fails_when_ranges_are_declared() {
        let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
        reset_for_tests();
        register_static_memory_manager_range(
            100,
            109,
            "crate_a",
            MemoryManagerRangeMode::Reserved,
            None,
        )
        .expect("crate A range");
        register_static_memory_manager_declaration(120, "crate_a", "users", "crate_a.users.v1")
            .expect("out-of-range declaration");

        let err = bootstrap_default_memory_manager().expect_err("out of range must fail");
        assert!(matches!(
            err,
            RuntimeBootstrapError::Validation(crate::AllocationValidationError::Policy(
                RuntimePolicyError::Range(MemoryManagerRangeAuthorityError::UnclaimedId {
                    id: 120
                })
            ))
        ));
    }

    #[test]
    fn late_registration_after_bootstrap_fails() {
        let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
        reset_for_tests();
        register_static_memory_manager_declaration(100, "crate_a", "users", "crate_a.users.v1")
            .expect("declaration");
        bootstrap_default_memory_manager().expect("bootstrap");

        let err = register_static_memory_manager_declaration(
            101,
            "crate_a",
            "orders",
            "crate_a.orders.v1",
        )
        .expect_err("late registration must fail");
        assert_eq!(err, StaticMemoryDeclarationError::RegistrySealed);
    }

    #[test]
    fn late_eager_init_registration_after_bootstrap_fails() {
        let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
        reset_for_tests();
        register_static_memory_manager_declaration(100, "crate_a", "users", "crate_a.users.v1")
            .expect("declaration");
        bootstrap_default_memory_manager().expect("bootstrap");

        let err = std::panic::catch_unwind(|| defer_eager_init(mark_eager_init))
            .expect_err("late eager-init registration must fail");

        let message = err
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| err.downcast_ref::<&str>().copied())
            .expect("panic message");
        assert!(message.contains("after runtime bootstrap"));
    }

    #[test]
    fn eager_init_runs_before_snapshot_seal() {
        let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
        reset_for_tests();
        EAGER_INIT_RAN.store(false, Ordering::SeqCst);
        register_static_memory_manager_range(
            100,
            109,
            "crate_a",
            MemoryManagerRangeMode::Reserved,
            None,
        )
        .expect("crate A range");
        defer_eager_init(mark_eager_init);

        let validated = bootstrap_default_memory_manager().expect("bootstrap");

        assert!(EAGER_INIT_RAN.load(Ordering::SeqCst));
        assert!(
            validated
                .declarations()
                .iter()
                .any(|declaration| declaration.stable_key().as_str() == "crate_a.audit.v1")
        );
    }

    #[test]
    fn direct_user_can_bootstrap_and_open_without_canic() {
        let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
        reset_for_tests();
        register_static_memory_manager_range(
            120,
            129,
            "icydb",
            MemoryManagerRangeMode::Reserved,
            None,
        )
        .expect("icydb range");
        register_static_memory_manager_declaration(120, "icydb", "users", "icydb.users.data.v1")
            .expect("icydb declaration");

        bootstrap_default_memory_manager().expect("bootstrap");
        open_default_memory_manager_memory("icydb.users.data.v1", 120).expect("open memory");
    }

    #[test]
    fn diagnostic_export_reports_default_memory_manager_sizes() {
        let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
        reset_for_tests();
        register_static_memory_manager_range(
            130,
            139,
            "diagnostics",
            MemoryManagerRangeMode::Reserved,
            None,
        )
        .expect("diagnostics range");
        register_static_memory_manager_declaration(
            130,
            "diagnostics",
            "users",
            "diagnostics.users.v1",
        )
        .expect("diagnostics declaration");

        bootstrap_default_memory_manager().expect("bootstrap");
        let memory =
            open_default_memory_manager_memory("diagnostics.users.v1", 130).expect("open memory");
        let old_size = memory.size();
        memory.grow(2);

        let export = default_memory_manager_diagnostic_export().expect("diagnostic export");
        let recovery =
            default_memory_manager_commit_recovery_diagnostic().expect("recovery diagnostic");
        let record = export
            .records
            .iter()
            .find(|record| record.allocation.stable_key().as_str() == "diagnostics.users.v1")
            .expect("diagnostic allocation");

        assert_eq!(
            recovery.authoritative_generation,
            Some(export.current_generation)
        );
        assert_eq!(
            record.memory_size,
            Some(DiagnosticMemorySize::from_wasm_pages(old_size + 2))
        );
    }

    #[test]
    fn doctor_report_preflights_before_bootstrap() {
        let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
        reset_for_tests();
        register_static_memory_manager_range(
            240,
            240,
            "doctor_preflight",
            MemoryManagerRangeMode::Reserved,
            None,
        )
        .expect("doctor range");
        register_static_memory_manager_declaration(
            240,
            "doctor_preflight",
            "users",
            "doctor_preflight.users.v1",
        )
        .expect("doctor declaration");

        let report = default_memory_manager_doctor_report();

        assert!(!report.bootstrapped);
        assert_eq!(report.registered_declarations.len(), 1);
        assert!(report.range_authority.effective_authority.is_some());
        assert_eq!(
            report.validation.status,
            crate::DiagnosticCheckStatus::Passed
        );
        assert!(report.commit_recovery.is_some());
        assert!(matches!(
            report.stable_cell.status,
            crate::DiagnosticStableCellStatus::Empty | crate::DiagnosticStableCellStatus::Readable
        ));
    }

    #[test]
    fn doctor_report_includes_recovered_ledger_and_memory_sizes_after_bootstrap() {
        let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
        reset_for_tests();
        register_static_memory_manager_range(
            241,
            241,
            "doctor_runtime",
            MemoryManagerRangeMode::Reserved,
            None,
        )
        .expect("doctor range");
        register_static_memory_manager_declaration(
            241,
            "doctor_runtime",
            "orders",
            "doctor_runtime.orders.v1",
        )
        .expect("doctor declaration");

        bootstrap_default_memory_manager().expect("bootstrap");
        let memory = open_default_memory_manager_memory("doctor_runtime.orders.v1", 241)
            .expect("open memory");
        let old_size = memory.size();
        memory.grow(1);

        let report = default_memory_manager_doctor_report();
        let ledger = report.ledger.expect("recovered ledger export");
        let record = ledger
            .records
            .iter()
            .find(|record| record.allocation.stable_key().as_str() == "doctor_runtime.orders.v1")
            .expect("doctor allocation");

        assert!(report.bootstrapped);
        assert_eq!(
            report.stable_cell.status,
            crate::DiagnosticStableCellStatus::Readable
        );
        assert_eq!(
            report.validation.status,
            crate::DiagnosticCheckStatus::Passed
        );
        assert_eq!(
            record.memory_size,
            Some(DiagnosticMemorySize::from_wasm_pages(old_size + 1))
        );
    }

    #[test]
    fn doctor_report_captures_validation_failure() {
        let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
        reset_for_tests();
        register_static_memory_manager_declaration(
            242,
            "doctor_failure_a",
            "users",
            "doctor_failure.users.v1",
        )
        .expect("first declaration");
        register_static_memory_manager_declaration(
            243,
            "doctor_failure_b",
            "orders",
            "doctor_failure.users.v1",
        )
        .expect("second declaration");

        let report = default_memory_manager_doctor_report();

        assert_eq!(
            report.validation.status,
            crate::DiagnosticCheckStatus::Failed
        );
        assert!(
            report
                .validation
                .message
                .expect("validation failure message")
                .contains("declared more than once")
        );
    }
}
