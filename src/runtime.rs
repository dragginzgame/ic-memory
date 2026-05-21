use crate::{
    AllocationBootstrap, AllocationDeclaration, AllocationHistory, AllocationLedger,
    AllocationPolicy, AllocationSlotDescriptor, DeclarationSnapshot, StableCellLedgerError,
    StableCellLedgerRecord, StableKey, ValidatedAllocations,
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
};
use ic_stable_structures::{
    Cell, DefaultMemoryImpl,
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
static VALIDATED_ALLOCATIONS: Mutex<Option<ValidatedAllocations>> = Mutex::new(None);
static BOOTSTRAPPED: AtomicBool = AtomicBool::new(false);

///
/// RuntimeBootstrapError
///
/// Failure to bootstrap the generic `ic-memory` runtime layer.
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
/// Failure to open a validated allocation through the default runtime substrate.
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum RuntimeOpenError {
    /// Runtime bootstrap has not published validated allocations.
    #[error("ic-memory runtime has not completed bootstrap validation")]
    NotBootstrapped,
    /// Runtime state lock was poisoned.
    #[error("ic-memory runtime lock poisoned")]
    RuntimeLockPoisoned,
    /// Stable-key grammar failure.
    #[error(transparent)]
    StableKey(#[from] crate::StableKeyError),
    /// The stable key was not present in the validated declaration set.
    #[error("stable key '{0}' was not validated by ic-memory runtime bootstrap")]
    StableKeyNotValidated(String),
    /// The validated slot is not a usable `MemoryManager` ID.
    #[error(transparent)]
    MemoryManagerSlot(#[from] MemoryManagerSlotError),
    /// The requested memory ID does not match the validated stable-key binding.
    #[error(
        "stable key '{stable_key}' is validated for MemoryManager ID {validated_id}, not requested ID {requested_id}"
    )]
    MemoryIdMismatch {
        /// Stable key being opened.
        stable_key: String,
        /// Validated MemoryManager ID.
        validated_id: u8,
        /// Requested MemoryManager ID.
        requested_id: u8,
    },
}

///
/// RuntimePolicyError
///
/// Failure in generic runtime range policy or caller-supplied policy.
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

/// Return the published validated allocations for the default runtime substrate.
pub fn validated_allocations() -> Result<ValidatedAllocations, RuntimeOpenError> {
    if !is_default_memory_manager_bootstrapped() {
        return Err(RuntimeOpenError::NotBootstrapped);
    }
    VALIDATED_ALLOCATIONS
        .lock()
        .map_err(|_| RuntimeOpenError::RuntimeLockPoisoned)?
        .clone()
        .ok_or(RuntimeOpenError::NotBootstrapped)
}

/// Bootstrap the default `MemoryManager<DefaultMemoryImpl>` runtime using generic policy.
pub fn bootstrap_default_memory_manager()
-> Result<ValidatedAllocations, RuntimeBootstrapError<Infallible>> {
    bootstrap_default_memory_manager_with_policy(&NoopPolicy)
}

/// Bootstrap the default runtime and layer caller-supplied policy over generic range checks.
///
/// Authority order is explicit:
///
/// 1. `ic-memory` always owns its governance range.
/// 2. If any user range is registered, all `MemoryManager` declarations must
///    belong to the range claimed by their declaring crate.
/// 3. The caller-supplied [`AllocationPolicy`] then applies framework-specific
///    namespace and lifecycle rules.
///
/// Framework adapters such as Canic should register only the ranges they want
/// this generic runtime to enforce. If a framework wants its own policy to be
/// authoritative for application space, it should omit user range registrations
/// for that space and enforce the rule in its [`AllocationPolicy`].
pub fn bootstrap_default_memory_manager_with_policy<P: AllocationPolicy>(
    policy: &P,
) -> Result<ValidatedAllocations, RuntimeBootstrapError<P::Error>> {
    if let Ok(validated) = validated_allocations() {
        return Ok(validated);
    }

    run_eager_init_hooks();

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
    let genesis = AllocationLedger::new(
        crate::CURRENT_LEDGER_SCHEMA_VERSION,
        crate::CURRENT_PHYSICAL_FORMAT_ID,
        0,
        AllocationHistory::default(),
    )?;

    let validated = with_default_ledger_cell(
        |cell| -> Result<ValidatedAllocations, RuntimeBootstrapError<P::Error>> {
            let mut record = cell.get().clone();
            let mut bootstrap = AllocationBootstrap::new(record.store_mut());
            let commit = bootstrap
                .initialize_validate_and_commit(&genesis, snapshot, &policy, None)
                .map_err(runtime_bootstrap_error_from_bootstrap)?;
            cell.set(record);
            Ok(commit.validated)
        },
    )?;

    publish_validated_allocations(validated.clone())?;
    BOOTSTRAPPED.store(true, Ordering::SeqCst);
    Ok(validated)
}

/// Open a validated `MemoryManager` memory by stable key and expected ID.
pub fn open_default_memory_manager_memory(
    stable_key: &str,
    id: u8,
) -> Result<VirtualMemory<DefaultMemoryImpl>, RuntimeOpenError> {
    let key = StableKey::parse(stable_key)?;
    let validated = validated_allocations()?;
    let slot = validated
        .slot_for(&key)
        .ok_or_else(|| RuntimeOpenError::StableKeyNotValidated(stable_key.to_string()))?;
    let validated_id = slot.memory_manager_id()?;
    if validated_id != id {
        return Err(RuntimeOpenError::MemoryIdMismatch {
            stable_key: stable_key.to_string(),
            validated_id,
            requested_id: id,
        });
    }
    Ok(default_memory_manager_memory(id))
}

fn run_eager_init_hooks() {
    let hooks = {
        let mut hooks = EAGER_INIT_HOOKS
            .lock()
            .expect("ic-memory eager-init queue poisoned");
        std::mem::take(&mut *hooks)
    };

    for hook in hooks {
        hook();
    }
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
        op(cell.as_mut().expect("default ledger cell initialized"))
    })
}

fn default_memory_manager_memory(id: u8) -> VirtualMemory<DefaultMemoryImpl> {
    DEFAULT_MEMORY_MANAGER.with(|manager| manager.get(MemoryId::new(id)))
}

fn publish_validated_allocations<P>(
    validated: ValidatedAllocations,
) -> Result<(), RuntimeBootstrapError<P>> {
    *VALIDATED_ALLOCATIONS
        .lock()
        .map_err(|_| RuntimeBootstrapError::RuntimeLockPoisoned)? = Some(validated);
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

fn declaration_metadata(registrations: &[StaticMemoryDeclaration]) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    metadata.insert(
        IC_MEMORY_LEDGER_STABLE_KEY.to_string(),
        IC_MEMORY_AUTHORITY_OWNER.to_string(),
    );
    for registration in registrations {
        metadata.insert(
            registration.declaration().stable_key().as_str().to_string(),
            registration.declaring_crate().to_string(),
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
    declaration_metadata: BTreeMap<String, String>,
    custom_policy: &'a P,
}

impl<P: AllocationPolicy> AllocationPolicy for RuntimeMemoryManagerPolicy<'_, P> {
    type Error = RuntimePolicyError<P::Error>;

    fn validate_key(&self, key: &StableKey) -> Result<(), Self::Error> {
        let declaring_crate = self.declaring_crate(key)?;
        if crate::is_ic_memory_stable_key(key.as_str())
            && declaring_crate != IC_MEMORY_AUTHORITY_OWNER
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
    fn declaring_crate(&self, key: &StableKey) -> Result<&str, RuntimePolicyError<P::Error>> {
        self.declaration_metadata
            .get(key.as_str())
            .map(String::as_str)
            .ok_or_else(|| RuntimePolicyError::MissingDeclarationMetadata(key.as_str().to_string()))
    }

    fn validate_runtime_range(
        &self,
        key: &StableKey,
        slot: &AllocationSlotDescriptor,
    ) -> Result<(), RuntimePolicyError<P::Error>> {
        let declaring_crate = self.declaring_crate(key)?;
        // Range claims are authoritative generic policy in the default runtime.
        // Once any user range is registered, every user declaration must fit
        // the declaring crate's claimed range. With no user ranges, only the
        // internal ic-memory governance range is enforced here and custom
        // policy may decide application-space ownership.
        if declaring_crate == IC_MEMORY_AUTHORITY_OWNER || self.user_ranges_registered {
            self.range_authority
                .validate_slot_authority(slot, declaring_crate)?;
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
                .validate_slot_authority(slot, declaring_crate)?;
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
pub(crate) fn reset_for_tests() {
    crate::registry::reset_static_memory_declarations_for_tests();
    EAGER_INIT_HOOKS
        .lock()
        .expect("ic-memory eager-init queue poisoned")
        .clear();
    *VALIDATED_ALLOCATIONS
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

        assert_eq!(validated.declarations().len(), 3);
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
}
