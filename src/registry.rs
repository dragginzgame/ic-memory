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
use serde::{Deserialize, Serialize};
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
    /// Internal declaration-registry lifecycle state was inconsistent.
    #[error("static memory declaration registry lifecycle is internally inconsistent")]
    InconsistentLifecycle,
    /// A deferred eager initialization hook panicked while declarations were sealing.
    #[error("static memory declaration eager-init hook panicked")]
    EagerInitPanicked,
    /// Declaration validation failed.
    #[error(transparent)]
    Declaration(#[from] crate::DeclarationSnapshotError),
    /// Range authority validation failed.
    #[error(transparent)]
    Range(#[from] MemoryManagerRangeAuthorityError),
    /// Canonical sealed-snapshot diagnostic fingerprint encoding failed.
    #[error("failed to encode canonical sealed declaration fingerprint material: {message}")]
    SnapshotFingerprintEncoding {
        /// Encoder failure.
        message: String,
    },
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

///
/// SealedDeclarationFingerprint
///
/// Deterministic non-cryptographic fingerprint of one canonical sealed
/// declaration snapshot.
///
/// The fingerprint covers canonical allocation declarations, their linked-code
/// authorities, and the effective range-authority table. It is diagnostic
/// metadata for comparing in-memory bootstrap bindings, not persisted
/// allocation authority or an adversarial integrity proof.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SealedDeclarationFingerprint {
    algorithm_version: u8,
    value: u64,
}

impl SealedDeclarationFingerprint {
    /// Return the diagnostic fingerprint algorithm version.
    #[must_use]
    pub const fn algorithm_version(&self) -> u8 {
        self.algorithm_version
    }

    /// Return the non-cryptographic fingerprint value.
    #[must_use]
    pub const fn value(&self) -> u64 {
        self.value
    }
}

#[derive(Debug, Eq, PartialEq)]
struct SealedDeclarationSnapshotInner {
    allocation_snapshot: DeclarationSnapshot,
    registered_declarations: Vec<StaticMemoryDeclaration>,
    registered_ranges: Vec<StaticMemoryRangeDeclaration>,
    range_authority: MemoryManagerRangeAuthority,
    declaration_authority: BTreeMap<String, RuntimeDeclarationAuthority>,
    fingerprint: SealedDeclarationFingerprint,
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

    /// Return the deterministic fingerprint of this sealed declaration meaning.
    #[must_use]
    pub fn fingerprint(&self) -> SealedDeclarationFingerprint {
        self.inner.fingerprint
    }

    pub(crate) fn declaration_authority(&self) -> &BTreeMap<String, RuntimeDeclarationAuthority> {
        &self.inner.declaration_authority
    }

    pub(crate) fn user_ranges_registered(&self) -> bool {
        !self.inner.registered_ranges.is_empty()
    }

    pub(crate) fn shares_storage_with(&self, other: &Self) -> bool {
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
    Sealing {
        owner: ThreadId,
        deferred_error: Option<StaticMemoryDeclarationError>,
    },
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
        StaticRegistryLifecycle::Sealing { owner, .. } if *owner == std::thread::current().id() => {
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

/// Queue a generated registration hook for the fallible sealing phase.
///
/// Static constructors cannot return an error. A late deferral is therefore
/// retained in registry state and returned by snapshot sealing.
#[doc(hidden)]
pub fn defer_static_memory_registration(hook: StaticRegistrationHook) {
    defer_constructor_registration(|registry| {
        registry.registration_hooks.push(hook);
    });
}

/// Queue a declaration-only hook to run immediately before snapshot sealing.
///
/// Static constructors cannot return an error. A late deferral is therefore
/// retained in registry state and returned by snapshot sealing.
#[doc(hidden)]
pub fn defer_eager_init(hook: fn()) {
    defer_constructor_registration(|registry| {
        registry.eager_init_hooks.push(hook);
    });
}

fn defer_constructor_registration(op: impl FnOnce(&mut StaticMemoryDeclarationRegistry)) {
    let Ok(mut registry) = STATIC_MEMORY_DECLARATIONS.lock() else {
        // Mutex poisoning is itself durable evidence of the registration
        // failure and is reported by the next snapshot request.
        return;
    };
    if matches!(registry.lifecycle, StaticRegistryLifecycle::Open) {
        op(&mut registry);
        return;
    }
    match &mut registry.lifecycle {
        StaticRegistryLifecycle::Sealing { deferred_error, .. } => {
            if deferred_error.is_none() {
                *deferred_error = Some(StaticMemoryDeclarationError::RegistrySealed);
            }
        }
        StaticRegistryLifecycle::Sealed(_) => {
            registry.lifecycle =
                StaticRegistryLifecycle::Failed(StaticMemoryDeclarationError::RegistrySealed);
        }
        StaticRegistryLifecycle::Failed(_) | StaticRegistryLifecycle::Open => {}
    }
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
            StaticRegistryLifecycle::Sealing { owner, .. }
                if *owner == std::thread::current().id() =>
            {
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
            deferred_error: None,
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

    let mut registry = lock_registry()?;
    let deferred_error = match &registry.lifecycle {
        StaticRegistryLifecycle::Sealing { deferred_error, .. } => deferred_error.clone(),
        StaticRegistryLifecycle::Failed(err) => return Err(err.clone()),
        StaticRegistryLifecycle::Open | StaticRegistryLifecycle::Sealed(_) => {
            return Err(StaticMemoryDeclarationError::InconsistentLifecycle);
        }
    };
    if let Some(err) = deferred_error {
        registry.lifecycle = StaticRegistryLifecycle::Failed(err.clone());
        return Err(err);
    }
    let snapshot = match build_sealed_snapshot(&registry.declarations, &registry.ranges) {
        Ok(snapshot) => snapshot,
        Err(err) => {
            registry.lifecycle = StaticRegistryLifecycle::Failed(err.clone());
            return Err(err);
        }
    };
    registry.lifecycle = StaticRegistryLifecycle::Sealed(snapshot.clone());
    Ok(snapshot)
}

fn fail_sealing<T>(err: StaticMemoryDeclarationError) -> Result<T, StaticMemoryDeclarationError> {
    let mut registry = lock_registry()?;
    let failure = match &registry.lifecycle {
        StaticRegistryLifecycle::Sealing {
            deferred_error: Some(deferred_error),
            ..
        } => deferred_error.clone(),
        StaticRegistryLifecycle::Open
        | StaticRegistryLifecycle::Sealing {
            deferred_error: None,
            ..
        }
        | StaticRegistryLifecycle::Sealed(_) => err,
        StaticRegistryLifecycle::Failed(failure) => failure.clone(),
    };
    registry.lifecycle = StaticRegistryLifecycle::Failed(failure.clone());
    Err(failure)
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
    let fingerprint = sealed_declaration_fingerprint(
        &allocation_snapshot,
        &registered_declarations,
        range_authority.authorities(),
    )?;

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
            fingerprint,
        }),
    })
}

#[derive(Serialize)]
struct FingerprintDeclaration<'a> {
    authority: &'a str,
    declaration: &'a AllocationDeclaration,
}

#[derive(Serialize)]
struct SealedDeclarationFingerprintMaterial<'a> {
    format: &'static str,
    allocation_snapshot: &'a DeclarationSnapshot,
    registered_declarations: Vec<FingerprintDeclaration<'a>>,
    effective_ranges: &'a [MemoryManagerAuthorityRecord],
}

fn sealed_declaration_fingerprint(
    allocation_snapshot: &DeclarationSnapshot,
    registered_declarations: &[StaticMemoryDeclaration],
    effective_ranges: &[MemoryManagerAuthorityRecord],
) -> Result<SealedDeclarationFingerprint, StaticMemoryDeclarationError> {
    let material = SealedDeclarationFingerprintMaterial {
        format: "ic-memory.sealed-declaration-fingerprint.v1",
        allocation_snapshot,
        registered_declarations: registered_declarations
            .iter()
            .map(|registration| FingerprintDeclaration {
                authority: registration.authority(),
                declaration: registration.declaration(),
            })
            .collect(),
        effective_ranges,
    };
    let mut bytes = Vec::new();
    ciborium::into_writer(&material, &mut bytes).map_err(|err| {
        StaticMemoryDeclarationError::SnapshotFingerprintEncoding {
            message: err.to_string(),
        }
    })?;

    let value = bytes
        .into_iter()
        .fold(FINGERPRINT_FNV_OFFSET, fingerprint_hash_byte);
    Ok(SealedDeclarationFingerprint {
        algorithm_version: SEALED_DECLARATION_FINGERPRINT_VERSION,
        value,
    })
}

const SEALED_DECLARATION_FINGERPRINT_VERSION: u8 = 1;
const FINGERPRINT_FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FINGERPRINT_FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

const fn fingerprint_hash_byte(hash: u64, byte: u8) -> u64 {
    (hash ^ byte as u64).wrapping_mul(FINGERPRINT_FNV_PRIME)
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
mod tests;
