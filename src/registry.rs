use crate::{
    constants::DIAGNOSTIC_STRING_MAX_BYTES,
    declaration::{AllocationDeclaration, DeclarationSnapshot},
    schema::SchemaMetadata,
    slot::{
        IC_MEMORY_AUTHORITY_OWNER, IC_MEMORY_AUTHORITY_PURPOSE, IC_MEMORY_LEDGER_LABEL,
        IC_MEMORY_LEDGER_STABLE_KEY, MEMORY_MANAGER_GOVERNANCE_MAX_ID, MEMORY_MANAGER_LEDGER_ID,
        MemoryManagerAuthorityRecord, MemoryManagerIdRange, MemoryManagerRangeAuthority,
        MemoryManagerRangeAuthorityError, MemoryManagerRangeMode, is_ic_memory_stable_key,
    },
};
use std::{
    collections::BTreeMap,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Arc, Mutex, MutexGuard},
    thread::ThreadId,
};

#[cfg(test)]
pub static TEST_REGISTRY_LOCK: Mutex<()> = Mutex::new(());

///
/// StaticMemoryDeclaration
///
/// One allocation declaration registered by crate-level generated or macro
/// code before the linked declaration registry seals its snapshot.
///
/// The `authority` field is policy metadata for integration layers such as
/// Canic or IcyDB. Each `MemoryRuntime` uses it to match declarations against
/// registered range claims before it calls the caller's
/// [`crate::AllocationPolicy`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaticMemoryDeclaration {
    authority: String,
    declaration: AllocationDeclaration,
}

impl StaticMemoryDeclaration {
    /// Build one static declaration from raw parts.
    pub fn new(
        authority: impl Into<String>,
        declaration: AllocationDeclaration,
    ) -> Result<Self, StaticMemoryDeclarationError> {
        let authority = authority.into();
        validate_external_authority(&authority)?;
        declaration.validate()?;
        if is_ic_memory_stable_key(declaration.stable_key().as_str()) {
            return Err(StaticMemoryDeclarationError::ReservedStableKey {
                stable_key: declaration.stable_key().as_str().to_string(),
            });
        }
        Ok(Self {
            authority,
            declaration,
        })
    }

    /// Return the authority that registered this declaration.
    #[must_use]
    pub fn authority(&self) -> &str {
        &self.authority
    }

    /// Borrow the allocation declaration.
    #[must_use]
    pub const fn declaration(&self) -> &AllocationDeclaration {
        &self.declaration
    }

    /// Consume this registration and return the allocation declaration.
    #[must_use]
    pub fn into_declaration(self) -> AllocationDeclaration {
        self.declaration
    }
}

///
/// StaticMemoryRangeDeclaration
///
/// One `MemoryManager` authority range registered by crate-level generated or
/// macro code before the linked registry seals the declaration snapshot. In a
/// `MemoryRuntime`, registered user ranges are authoritative generic range policy:
/// declarations must stay inside the authority's claimed range before
/// caller-supplied policy runs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaticMemoryRangeDeclaration {
    record: MemoryManagerAuthorityRecord,
}

impl StaticMemoryRangeDeclaration {
    /// Build one static range declaration from a validated authority record.
    pub fn new(record: MemoryManagerAuthorityRecord) -> Result<Self, StaticMemoryDeclarationError> {
        validate_external_authority(record.authority())?;
        record.validate()?;
        Ok(Self { record })
    }

    /// Return the authority that registered this range.
    #[must_use]
    pub fn authority(&self) -> &str {
        self.record.authority()
    }

    /// Borrow the authority record.
    #[must_use]
    pub const fn record(&self) -> &MemoryManagerAuthorityRecord {
        &self.record
    }

    /// Consume this registration and return the authority record.
    #[must_use]
    pub fn into_record(self) -> MemoryManagerAuthorityRecord {
        self.record
    }
}

///
/// StaticMemoryDeclarationError
///
/// Failure to register or collect static allocation declarations.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum StaticMemoryDeclarationError {
    /// Static declaration registry lock was poisoned.
    #[error("static memory declaration registry lock poisoned")]
    RegistryPoisoned,
    /// Bootstrap already sealed the declaration snapshot.
    #[error("static memory declaration registry is already sealed")]
    RegistrySealed,
    /// Snapshot sealing was called recursively from an eager hook.
    #[error("static memory declaration snapshot sealing is already active on this thread")]
    ReentrantSealing,
    /// A deferred eager initialization hook panicked while declarations were sealing.
    #[error("static memory declaration eager-init hook panicked")]
    EagerInitPanicked,
    /// Declaration validation failed.
    #[error(transparent)]
    Declaration(#[from] crate::DeclarationSnapshotError),
    /// Range authority validation failed.
    #[error(transparent)]
    Range(#[from] MemoryManagerRangeAuthorityError),
    /// External registration attempted to use an invalid authority identifier.
    #[error("authority {reason}")]
    InvalidAuthority {
        /// Validation failure.
        reason: &'static str,
    },
    /// External registration attempted to impersonate the internal authority.
    #[error("authority '{authority}' is reserved for ic-memory runtime internals")]
    ReservedAuthority {
        /// Reserved authority identifier.
        authority: String,
    },
    /// External registration attempted to claim the internal stable-key namespace.
    #[error("stable key '{stable_key}' is reserved for ic-memory runtime internals")]
    ReservedStableKey {
        /// Reserved stable key.
        stable_key: String,
    },
}

///
/// SealedDeclarationSnapshot
///
/// Immutable, canonical linked-program allocation declarations and range
/// authority supplied to each concrete [`crate::MemoryRuntime`].
///
/// Sealing runs generated registration hooks and eager declaration hooks
/// exactly once. Clones share the same immutable snapshot. This value contains
/// declaration authority only; it contains no memory handles, recovery state,
/// bootstrap lifecycle, or committed allocation capability.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SealedDeclarationSnapshot {
    inner: Arc<SealedDeclarationSnapshotInner>,
}

#[derive(Debug, Eq, PartialEq)]
struct SealedDeclarationSnapshotInner {
    allocation_snapshot: DeclarationSnapshot,
    registered_declarations: Vec<StaticMemoryDeclaration>,
    registered_ranges: Vec<StaticMemoryRangeDeclaration>,
    range_authority: MemoryManagerRangeAuthority,
    declaration_authority: BTreeMap<String, RuntimeDeclarationAuthority>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeDeclarationAuthority {
    Internal,
    External(String),
}

impl SealedDeclarationSnapshot {
    /// Borrow the canonical allocation snapshot, including runtime governance.
    #[must_use]
    pub fn allocation_snapshot(&self) -> &DeclarationSnapshot {
        &self.inner.allocation_snapshot
    }

    /// Borrow canonical external declarations registered by linked code.
    #[must_use]
    pub fn registered_declarations(&self) -> &[StaticMemoryDeclaration] {
        &self.inner.registered_declarations
    }

    /// Borrow canonical external range declarations registered by linked code.
    #[must_use]
    pub fn registered_ranges(&self) -> &[StaticMemoryRangeDeclaration] {
        &self.inner.registered_ranges
    }

    /// Borrow the effective range authority, including runtime governance.
    #[must_use]
    pub fn range_authority(&self) -> &MemoryManagerRangeAuthority {
        &self.inner.range_authority
    }

    pub(crate) fn declaration_authority(&self) -> &BTreeMap<String, RuntimeDeclarationAuthority> {
        &self.inner.declaration_authority
    }

    pub(crate) fn user_ranges_registered(&self) -> bool {
        !self.inner.registered_ranges.is_empty()
    }

    #[cfg(test)]
    fn shares_storage_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

type StaticRegistrationHook = fn() -> Result<(), StaticMemoryDeclarationError>;

#[derive(Debug)]
struct StaticMemoryDeclarationRegistry {
    declarations: Vec<StaticMemoryDeclaration>,
    ranges: Vec<StaticMemoryRangeDeclaration>,
    registration_hooks: Vec<StaticRegistrationHook>,
    eager_init_hooks: Vec<fn()>,
    lifecycle: StaticRegistryLifecycle,
}

#[derive(Debug)]
enum StaticRegistryLifecycle {
    Open,
    Sealing { owner: ThreadId },
    Sealed(SealedDeclarationSnapshot),
    Failed(StaticMemoryDeclarationError),
}

static STATIC_MEMORY_DECLARATIONS: Mutex<StaticMemoryDeclarationRegistry> =
    Mutex::new(StaticMemoryDeclarationRegistry {
        declarations: Vec::new(),
        ranges: Vec::new(),
        registration_hooks: Vec::new(),
        eager_init_hooks: Vec::new(),
        lifecycle: StaticRegistryLifecycle::Open,
    });

static STATIC_MEMORY_SEAL: Mutex<()> = Mutex::new(());

fn lock_registry()
-> Result<MutexGuard<'static, StaticMemoryDeclarationRegistry>, StaticMemoryDeclarationError> {
    STATIC_MEMORY_DECLARATIONS
        .lock()
        .map_err(|_| StaticMemoryDeclarationError::RegistryPoisoned)
}

fn ensure_registration_open(
    registry: &StaticMemoryDeclarationRegistry,
) -> Result<(), StaticMemoryDeclarationError> {
    match &registry.lifecycle {
        StaticRegistryLifecycle::Open => Ok(()),
        StaticRegistryLifecycle::Sealing { owner } if *owner == std::thread::current().id() => {
            Ok(())
        }
        StaticRegistryLifecycle::Sealing { .. }
        | StaticRegistryLifecycle::Sealed(_)
        | StaticRegistryLifecycle::Failed(_) => Err(StaticMemoryDeclarationError::RegistrySealed),
    }
}

fn with_unsealed_registry(
    op: impl FnOnce(&mut StaticMemoryDeclarationRegistry),
) -> Result<(), StaticMemoryDeclarationError> {
    let mut registry = lock_registry()?;
    ensure_registration_open(&registry)?;
    op(&mut registry);
    Ok(())
}

#[doc(hidden)]
pub fn defer_static_memory_registration(
    hook: StaticRegistrationHook,
) -> Result<(), StaticMemoryDeclarationError> {
    let mut registry = lock_registry()?;
    if !matches!(registry.lifecycle, StaticRegistryLifecycle::Open) {
        return Err(StaticMemoryDeclarationError::RegistrySealed);
    }
    registry.registration_hooks.push(hook);
    Ok(())
}

/// Register a declaration-only hook to run immediately before snapshot sealing.
#[doc(hidden)]
pub fn defer_eager_init(hook: fn()) -> Result<(), StaticMemoryDeclarationError> {
    let mut registry = lock_registry()?;
    if !matches!(registry.lifecycle, StaticRegistryLifecycle::Open) {
        return Err(StaticMemoryDeclarationError::RegistrySealed);
    }
    registry.eager_init_hooks.push(hook);
    Ok(())
}

/// Register one allocation declaration before bootstrap seals the snapshot.
pub fn register_static_memory_declaration(
    authority: impl Into<String>,
    declaration: AllocationDeclaration,
) -> Result<(), StaticMemoryDeclarationError> {
    let registration = StaticMemoryDeclaration::new(authority, declaration)?;
    with_unsealed_registry(|registry| {
        registry.declarations.push(registration);
    })
}

/// Register one `MemoryManager` authority range before bootstrap seals the snapshot.
pub fn register_static_memory_manager_range(
    start: u8,
    end: u8,
    authority: impl Into<String>,
    mode: MemoryManagerRangeMode,
    purpose: Option<String>,
) -> Result<(), StaticMemoryDeclarationError> {
    let authority = authority.into();
    let record = MemoryManagerAuthorityRecord::new(
        MemoryManagerIdRange::new(start, end).map_err(MemoryManagerRangeAuthorityError::Range)?,
        authority,
        mode,
        purpose,
    )?;
    register_static_memory_range_declaration(StaticMemoryRangeDeclaration::new(record)?)
}

/// Register one authority range declaration before bootstrap seals the snapshot.
pub fn register_static_memory_range_declaration(
    declaration: StaticMemoryRangeDeclaration,
) -> Result<(), StaticMemoryDeclarationError> {
    validate_external_authority(declaration.authority())?;
    with_unsealed_registry(|registry| {
        registry.ranges.push(declaration);
    })
}

fn validate_external_authority(value: &str) -> Result<(), StaticMemoryDeclarationError> {
    if value == IC_MEMORY_AUTHORITY_OWNER {
        return Err(StaticMemoryDeclarationError::ReservedAuthority {
            authority: value.to_string(),
        });
    }
    if value.is_empty() {
        return Err(StaticMemoryDeclarationError::InvalidAuthority {
            reason: "must not be empty",
        });
    }
    if value.len() > DIAGNOSTIC_STRING_MAX_BYTES {
        return Err(StaticMemoryDeclarationError::InvalidAuthority {
            reason: "must be at most 256 bytes",
        });
    }
    if !value.is_ascii() {
        return Err(StaticMemoryDeclarationError::InvalidAuthority {
            reason: "must be ASCII",
        });
    }
    if value.bytes().any(|byte| byte.is_ascii_control()) {
        return Err(StaticMemoryDeclarationError::InvalidAuthority {
            reason: "must not contain ASCII control characters",
        });
    }
    Ok(())
}

/// Register one `MemoryManager` declaration before bootstrap seals the snapshot.
pub fn register_static_memory_manager_declaration(
    id: u8,
    authority: impl Into<String>,
    label: impl Into<String>,
    stable_key: impl AsRef<str>,
) -> Result<(), StaticMemoryDeclarationError> {
    register_static_memory_manager_declaration_with_schema(
        id,
        authority,
        label,
        stable_key,
        SchemaMetadata::default(),
    )
}

/// Register one `MemoryManager` declaration with schema metadata.
pub fn register_static_memory_manager_declaration_with_schema(
    id: u8,
    authority: impl Into<String>,
    label: impl Into<String>,
    stable_key: impl AsRef<str>,
    schema: SchemaMetadata,
) -> Result<(), StaticMemoryDeclarationError> {
    let declaration =
        AllocationDeclaration::memory_manager_with_schema(stable_key, id, label, schema)?;
    register_static_memory_declaration(authority, declaration)
}

/// Seal and return the canonical linked-program declaration snapshot.
///
/// The first caller runs deferred generated registrations and eager hooks,
/// canonicalizes declarations and ranges, validates duplicates and range
/// authority, and publishes one immutable snapshot. Concurrent and subsequent
/// callers receive clones backed by that same snapshot.
pub fn sealed_declaration_snapshot()
-> Result<SealedDeclarationSnapshot, StaticMemoryDeclarationError> {
    {
        let registry = lock_registry()?;
        match &registry.lifecycle {
            StaticRegistryLifecycle::Sealed(snapshot) => return Ok(snapshot.clone()),
            StaticRegistryLifecycle::Failed(err) => return Err(err.clone()),
            StaticRegistryLifecycle::Sealing { owner } if *owner == std::thread::current().id() => {
                return Err(StaticMemoryDeclarationError::ReentrantSealing);
            }
            StaticRegistryLifecycle::Open | StaticRegistryLifecycle::Sealing { .. } => {}
        }
    }

    let _seal = STATIC_MEMORY_SEAL
        .lock()
        .map_err(|_| StaticMemoryDeclarationError::RegistryPoisoned)?;
    let (registration_hooks, eager_init_hooks) = {
        let mut registry = lock_registry()?;
        match &registry.lifecycle {
            StaticRegistryLifecycle::Sealed(snapshot) => return Ok(snapshot.clone()),
            StaticRegistryLifecycle::Failed(err) => return Err(err.clone()),
            StaticRegistryLifecycle::Sealing { .. } => {
                return Err(StaticMemoryDeclarationError::ReentrantSealing);
            }
            StaticRegistryLifecycle::Open => {}
        }
        registry.lifecycle = StaticRegistryLifecycle::Sealing {
            owner: std::thread::current().id(),
        };
        (
            std::mem::take(&mut registry.registration_hooks),
            std::mem::take(&mut registry.eager_init_hooks),
        )
    };

    for hook in registration_hooks {
        let result = catch_unwind(AssertUnwindSafe(hook))
            .map_err(|_| StaticMemoryDeclarationError::EagerInitPanicked)
            .and_then(std::convert::identity);
        if let Err(err) = result {
            return fail_sealing(err);
        }
    }
    for hook in eager_init_hooks {
        if catch_unwind(AssertUnwindSafe(hook)).is_err() {
            return fail_sealing(StaticMemoryDeclarationError::EagerInitPanicked);
        }
    }

    let snapshot = {
        let registry = lock_registry()?;
        build_sealed_snapshot(&registry.declarations, &registry.ranges)
    };
    let snapshot = match snapshot {
        Ok(snapshot) => snapshot,
        Err(err) => return fail_sealing(err),
    };
    lock_registry()?.lifecycle = StaticRegistryLifecycle::Sealed(snapshot.clone());
    Ok(snapshot)
}

fn fail_sealing<T>(err: StaticMemoryDeclarationError) -> Result<T, StaticMemoryDeclarationError> {
    let mut registry = lock_registry()?;
    registry.lifecycle = StaticRegistryLifecycle::Failed(err.clone());
    Err(err)
}

fn build_sealed_snapshot(
    declarations: &[StaticMemoryDeclaration],
    ranges: &[StaticMemoryRangeDeclaration],
) -> Result<SealedDeclarationSnapshot, StaticMemoryDeclarationError> {
    let mut registered_declarations = declarations.to_vec();
    registered_declarations.sort_by(|left, right| {
        left.declaration()
            .stable_key()
            .cmp(right.declaration().stable_key())
            .then_with(|| left.declaration().slot().cmp(right.declaration().slot()))
            .then_with(|| left.authority().cmp(right.authority()))
    });

    let mut registered_ranges = ranges.to_vec();
    registered_ranges.sort_by(|left, right| {
        let left = left.record();
        let right = right.record();
        left.range()
            .start()
            .cmp(&right.range().start())
            .then_with(|| left.range().end().cmp(&right.range().end()))
            .then_with(|| left.authority().cmp(right.authority()))
            .then_with(|| range_mode_order(left.mode()).cmp(&range_mode_order(right.mode())))
            .then_with(|| left.purpose().cmp(&right.purpose()))
    });

    let mut allocation_declarations = Vec::with_capacity(registered_declarations.len() + 1);
    allocation_declarations.push(internal_ledger_declaration()?);
    allocation_declarations.extend(
        registered_declarations
            .iter()
            .map(|registration| registration.declaration().clone()),
    );
    let allocation_snapshot = DeclarationSnapshot::new(allocation_declarations)?;

    let mut authority_records = Vec::with_capacity(registered_ranges.len() + 1);
    authority_records.push(internal_ledger_range()?);
    authority_records.extend(
        registered_ranges
            .iter()
            .map(|registration| registration.record().clone()),
    );
    let range_authority = MemoryManagerRangeAuthority::from_records(authority_records)?;

    let mut declaration_authority = BTreeMap::new();
    declaration_authority.insert(
        IC_MEMORY_LEDGER_STABLE_KEY.to_string(),
        RuntimeDeclarationAuthority::Internal,
    );
    for registration in &registered_declarations {
        declaration_authority.insert(
            registration.declaration().stable_key().as_str().to_string(),
            RuntimeDeclarationAuthority::External(registration.authority().to_string()),
        );
    }

    Ok(SealedDeclarationSnapshot {
        inner: Arc::new(SealedDeclarationSnapshotInner {
            allocation_snapshot,
            registered_declarations,
            registered_ranges,
            range_authority,
            declaration_authority,
        }),
    })
}

const fn range_mode_order(mode: MemoryManagerRangeMode) -> u8 {
    match mode {
        MemoryManagerRangeMode::Reserved => 0,
        MemoryManagerRangeMode::Allowed => 1,
    }
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
        MemoryManagerIdRange::new(MEMORY_MANAGER_LEDGER_ID, MEMORY_MANAGER_GOVERNANCE_MAX_ID)?,
        IC_MEMORY_AUTHORITY_OWNER,
        MemoryManagerRangeMode::Reserved,
        Some(IC_MEMORY_AUTHORITY_PURPOSE.to_string()),
    )
}

#[cfg(test)]
pub fn reset_static_memory_declarations_for_tests() {
    let mut registry = STATIC_MEMORY_DECLARATIONS
        .lock()
        .expect("static memory declaration registry poisoned");
    registry.declarations.clear();
    registry.ranges.clear();
    registry.registration_hooks.clear();
    registry.eager_init_hooks.clear();
    registry.lifecycle = StaticRegistryLifecycle::Open;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static EAGER_INIT_RUNS: AtomicUsize = AtomicUsize::new(0);
    static BLOCKING_HOOK_STARTED: AtomicUsize = AtomicUsize::new(0);
    static RELEASE_BLOCKING_HOOK: AtomicUsize = AtomicUsize::new(0);

    fn register_from_eager_init() {
        EAGER_INIT_RUNS.fetch_add(1, Ordering::SeqCst);
        register_static_memory_manager_declaration(101, "eager", "audit", "eager.audit.v1")
            .expect("eager declaration");
    }

    fn block_during_sealing() {
        BLOCKING_HOOK_STARTED.store(1, Ordering::SeqCst);
        while RELEASE_BLOCKING_HOOK.load(Ordering::SeqCst) == 0 {
            std::thread::yield_now();
        }
    }

    fn record_reentrant_seal_error() {
        let error = sealed_declaration_snapshot().expect_err("recursive seal must fail");
        if error == StaticMemoryDeclarationError::ReentrantSealing {
            EAGER_INIT_RUNS.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn registers_and_seals_static_memory_declarations() {
        let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
        reset_static_memory_declarations_for_tests();

        register_static_memory_manager_declaration(100, "icydb", "users", "icydb.users.data.v1")
            .expect("register declaration");

        let snapshot = sealed_declaration_snapshot().expect("snapshot");
        let registrations = snapshot.registered_declarations();
        assert_eq!(registrations.len(), 1);
        assert_eq!(registrations[0].authority(), "icydb");
        assert_eq!(
            registrations[0].declaration().stable_key().as_str(),
            "icydb.users.data.v1"
        );

        assert_eq!(snapshot.allocation_snapshot().len(), 2);

        let err =
            register_static_memory_manager_declaration(101, "icydb", "orders", "icydb.orders.v1")
                .expect_err("late registration must fail");
        assert_eq!(err, StaticMemoryDeclarationError::RegistrySealed);
    }

    #[test]
    fn registers_static_memory_ranges() {
        let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
        reset_static_memory_declarations_for_tests();

        register_static_memory_manager_range(
            100,
            109,
            "crate_a",
            MemoryManagerRangeMode::Reserved,
            Some("crate A stores".to_string()),
        )
        .expect("register range");

        let snapshot = sealed_declaration_snapshot().expect("snapshot");
        let ranges = snapshot.registered_ranges();
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].authority(), "crate_a");
        assert_eq!(ranges[0].record().range().start(), 100);
        assert_eq!(ranges[0].record().range().end(), 109);
    }

    #[test]
    fn static_range_declaration_uses_record_authority() {
        let record = MemoryManagerAuthorityRecord::new(
            MemoryManagerIdRange::new(100, 109).expect("range"),
            "record_authority",
            MemoryManagerRangeMode::Reserved,
            None,
        )
        .expect("record");

        let range = StaticMemoryRangeDeclaration::new(record).expect("external range");

        assert_eq!(range.authority(), "record_authority");
    }

    #[test]
    fn static_declaration_rejects_invalid_decoded_declaration() {
        let mut declaration = AllocationDeclaration::memory_manager("app.users.v1", 100, "users")
            .expect("declaration");
        declaration.slot = crate::AllocationSlotDescriptor::memory_manager_unchecked(
            crate::MEMORY_MANAGER_INVALID_ID,
        );

        let err = StaticMemoryDeclaration::new("app", declaration)
            .expect_err("decoded invalid declaration must fail at the registry boundary");

        assert!(matches!(
            err,
            StaticMemoryDeclarationError::Declaration(
                crate::DeclarationSnapshotError::MemoryManagerSlot(
                    crate::MemoryManagerSlotError::InvalidMemoryManagerId { id }
                )
            ) if id == crate::MEMORY_MANAGER_INVALID_ID
        ));
    }

    #[test]
    fn static_range_declaration_rejects_invalid_decoded_record() {
        let mut record = MemoryManagerAuthorityRecord::new(
            MemoryManagerIdRange::new(100, 109).expect("range"),
            "app",
            MemoryManagerRangeMode::Reserved,
            None,
        )
        .expect("record");
        record.range = MemoryManagerIdRange {
            start: 109,
            end: 100,
        };

        let err = StaticMemoryRangeDeclaration::new(record)
            .expect_err("decoded invalid range record must fail at the registry boundary");

        assert!(matches!(
            err,
            StaticMemoryDeclarationError::Range(MemoryManagerRangeAuthorityError::Range(_))
        ));
    }

    #[test]
    fn snapshot_rejects_duplicate_static_memory_declarations() {
        let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
        reset_static_memory_declarations_for_tests();

        register_static_memory_manager_declaration(100, "icydb", "users", "icydb.users.data.v1")
            .expect("register first declaration");
        register_static_memory_manager_declaration(100, "icydb", "orders", "icydb.orders.v1")
            .expect("register duplicate slot declaration");

        let err = sealed_declaration_snapshot().expect_err("duplicate slot must fail");
        let repeated = sealed_declaration_snapshot().expect_err("seal failure is stable");
        assert!(matches!(
            err,
            StaticMemoryDeclarationError::Declaration(
                crate::DeclarationSnapshotError::DuplicateSlot(_)
            )
        ));
        assert_eq!(repeated, err);
    }

    #[test]
    fn external_registration_rejects_internal_stable_key_namespace() {
        let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
        reset_static_memory_declarations_for_tests();

        let err = register_static_memory_manager_declaration(
            1,
            "external",
            "governance",
            "ic_memory.spoof.v1",
        )
        .expect_err("internal stable key must be unavailable externally");

        assert!(matches!(
            err,
            StaticMemoryDeclarationError::ReservedStableKey { stable_key }
                if stable_key == "ic_memory.spoof.v1"
        ));
    }

    #[test]
    fn external_registration_rejects_internal_authority_identity() {
        let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
        reset_static_memory_declarations_for_tests();

        let declaration_err = register_static_memory_manager_declaration(
            100,
            IC_MEMORY_AUTHORITY_OWNER,
            "users",
            "app.users.v1",
        )
        .expect_err("internal declaration authority must be unavailable externally");
        let range_err = register_static_memory_manager_range(
            100,
            109,
            IC_MEMORY_AUTHORITY_OWNER,
            MemoryManagerRangeMode::Reserved,
            None,
        )
        .expect_err("internal range authority must be unavailable externally");

        assert!(matches!(
            declaration_err,
            StaticMemoryDeclarationError::ReservedAuthority { .. }
        ));
        assert!(matches!(
            range_err,
            StaticMemoryDeclarationError::ReservedAuthority { .. }
        ));
    }

    #[test]
    fn eager_hooks_run_once_before_the_canonical_snapshot_is_published() {
        let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
        reset_static_memory_declarations_for_tests();
        EAGER_INIT_RUNS.store(0, Ordering::SeqCst);
        defer_eager_init(register_from_eager_init).expect("defer eager hook");

        let first = sealed_declaration_snapshot().expect("first snapshot");
        let second = sealed_declaration_snapshot().expect("second snapshot");

        assert_eq!(EAGER_INIT_RUNS.load(Ordering::SeqCst), 1);
        assert!(first.shares_storage_with(&second));
        assert_eq!(
            first.registered_declarations()[0]
                .declaration()
                .stable_key()
                .as_str(),
            "eager.audit.v1"
        );
    }

    #[test]
    fn snapshot_order_is_independent_of_registration_order() {
        let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
        reset_static_memory_declarations_for_tests();
        register_static_memory_manager_declaration(102, "order", "z", "order.z.v1")
            .expect("z declaration");
        register_static_memory_manager_declaration(101, "order", "a", "order.a.v1")
            .expect("a declaration");
        let first = sealed_declaration_snapshot().expect("first snapshot");

        reset_static_memory_declarations_for_tests();
        register_static_memory_manager_declaration(101, "order", "a", "order.a.v1")
            .expect("a declaration");
        register_static_memory_manager_declaration(102, "order", "z", "order.z.v1")
            .expect("z declaration");
        let second = sealed_declaration_snapshot().expect("second snapshot");

        assert_eq!(first, second);
        assert_eq!(
            crate::test_cbor::to_vec(first.allocation_snapshot()).expect("first bytes"),
            crate::test_cbor::to_vec(second.allocation_snapshot()).expect("second bytes")
        );
    }

    #[test]
    fn concurrent_snapshot_requests_share_one_complete_seal() {
        let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
        reset_static_memory_declarations_for_tests();
        defer_eager_init(register_from_eager_init).expect("defer eager hook");

        let (first, second) = std::thread::scope(|scope| {
            let first = scope.spawn(sealed_declaration_snapshot);
            let second = scope.spawn(sealed_declaration_snapshot);
            (
                first.join().expect("first thread").expect("first seal"),
                second.join().expect("second thread").expect("second seal"),
            )
        });

        assert!(first.shares_storage_with(&second));
        assert_eq!(first.registered_declarations().len(), 1);
    }

    #[test]
    fn registration_from_another_thread_fails_after_sealing_begins() {
        let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
        reset_static_memory_declarations_for_tests();
        BLOCKING_HOOK_STARTED.store(0, Ordering::SeqCst);
        RELEASE_BLOCKING_HOOK.store(0, Ordering::SeqCst);
        defer_eager_init(block_during_sealing).expect("defer blocking hook");

        std::thread::scope(|scope| {
            let sealing = scope.spawn(sealed_declaration_snapshot);
            while BLOCKING_HOOK_STARTED.load(Ordering::SeqCst) == 0 {
                std::thread::yield_now();
            }
            let late =
                register_static_memory_manager_declaration(101, "late", "late", "late.rows.v1")
                    .expect_err("concurrent late registration");
            assert_eq!(late, StaticMemoryDeclarationError::RegistrySealed);
            RELEASE_BLOCKING_HOOK.store(1, Ordering::SeqCst);
            sealing.join().expect("sealing thread").expect("snapshot");
        });
    }

    #[test]
    fn recursive_snapshot_request_from_eager_hook_is_typed() {
        let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
        reset_static_memory_declarations_for_tests();
        EAGER_INIT_RUNS.store(0, Ordering::SeqCst);
        defer_eager_init(record_reentrant_seal_error).expect("defer recursive hook");

        sealed_declaration_snapshot().expect("outer snapshot");

        assert_eq!(EAGER_INIT_RUNS.load(Ordering::SeqCst), 1);
    }
}
