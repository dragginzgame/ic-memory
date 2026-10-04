use crate::{
    declaration::{AllocationDeclaration, DeclarationSnapshot},
    schema::SchemaMetadata,
    slot::{
        IC_MEMORY_AUTHORITY_OWNER, IC_MEMORY_AUTHORITY_PURPOSE, IC_MEMORY_LEDGER_LABEL,
        IC_MEMORY_LEDGER_STABLE_KEY, MEMORY_MANAGER_LEDGER_ID, MemoryManagerAuthorityRecord,
        MemoryManagerIdRange, MemoryManagerRangeAuthority, MemoryManagerRangeAuthorityError,
        MemoryManagerRangeMode, is_ic_memory_stable_key, memory_manager_governance_range,
    },
    text::validate_diagnostic_text,
};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
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
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
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
/// MemoryRequest
///
/// Key-only request resolved after ledger recovery. New keys require an explicit
/// Allowed range owned by this authority; known keys retain their durable slot.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MemoryRequest {
    authority: String,
    stable_key: crate::StableKey,
    schema: SchemaMetadata,
}

impl MemoryRequest {
    /// Build a checked logical request before sealing.
    pub fn new(
        authority: impl Into<String>,
        stable_key: &str,
        schema: SchemaMetadata,
    ) -> Result<Self, StaticMemoryDeclarationError> {
        let authority = authority.into();
        validate_external_authority(&authority)?;
        let stable_key =
            crate::StableKey::parse(stable_key).map_err(crate::DeclarationSnapshotError::Key)?;
        schema
            .validate()
            .map_err(crate::DeclarationSnapshotError::SchemaMetadata)?;
        if is_ic_memory_stable_key(stable_key.as_str()) {
            return Err(StaticMemoryDeclarationError::ReservedStableKey {
                stable_key: stable_key.as_str().to_string(),
            });
        }
        Ok(Self {
            authority,
            stable_key,
            schema,
        })
    }

    /// Attach schema metadata from the immutable, integrity-checked recovered ledger.
    pub(crate) const fn with_schema(mut self, schema: SchemaMetadata) -> Self {
        self.schema = schema;
        self
    }

    /// Borrow the requested durable key.
    #[must_use]
    pub const fn stable_key(&self) -> &crate::StableKey {
        &self.stable_key
    }

    /// Borrow the requested diagnostic schema metadata.
    #[must_use]
    pub const fn schema(&self) -> &SchemaMetadata {
        &self.schema
    }

    /// Borrow the declaring authority.
    #[must_use]
    pub fn authority(&self) -> &str {
        &self.authority
    }
}

/// Register a key-only request before the linked snapshot seals.
pub fn register_memory_request(request: MemoryRequest) -> Result<(), StaticMemoryDeclarationError> {
    with_unsealed_registry(|registry| registry.requests.push(request))
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
    #[error("at most 254 external declarations and ranges are supported")]
    TooManyDeclarations,
    #[error("duplicate requested stable key {stable_key}")]
    DuplicateRequest { stable_key: crate::StableKey },
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
    requests: Vec<MemoryRequest>,
    registered_declarations: Vec<StaticMemoryDeclaration>,
    registered_ranges: Vec<StaticMemoryRangeDeclaration>,
    range_authority: MemoryManagerRangeAuthority,
    fingerprint: SealedDeclarationFingerprint,
}

impl SealedDeclarationSnapshot {
    /// Seal explicitly owned inputs with the same rules as the linked registry.
    pub fn new(
        declarations: &[StaticMemoryDeclaration],
        ranges: &[StaticMemoryRangeDeclaration],
        requests: &[MemoryRequest],
    ) -> Result<Self, StaticMemoryDeclarationError> {
        build_snapshot(
            Cow::Borrowed(declarations),
            Cow::Borrowed(ranges),
            Cow::Borrowed(requests),
        )
    }

    /// Borrow canonical unresolved key-only requests.
    #[must_use]
    pub fn requests(&self) -> &[MemoryRequest] {
        &self.inner.requests
    }

    pub(crate) fn resolve(
        &self,
        ledger: &crate::AllocationLedger,
        historical: Vec<MemoryRequest>,
    ) -> Result<Self, crate::MemoryResolutionError> {
        if self.requests().is_empty() && historical.is_empty() {
            return Ok(self.clone());
        }
        if self.registered_declarations().len() + self.requests().len() + historical.len() > 254 {
            return Err(StaticMemoryDeclarationError::TooManyDeclarations.into());
        }
        let mut declarations = self.registered_declarations().to_vec();
        let mut occupied = [false; 255];
        for record in ledger.allocation_history().records() {
            occupied[usize::from(record.slot().id())] = true;
        }
        for fixed in &declarations {
            occupied[usize::from(fixed.declaration().slot().id())] = true;
        }
        // Only the original requests can allocate new slots and they are already
        // canonical. Admission selections are known-only: all their slots are
        // occupied above regardless of selection order. Final declarations are
        // canonicalized and checked together below.
        for request in self
            .requests()
            .iter()
            .map(Cow::Borrowed)
            .chain(historical.into_iter().map(Cow::Owned))
        {
            let historical = ledger
                .allocation_history()
                .records()
                .iter()
                .find(|record| record.stable_key() == &request.stable_key);
            let id = if let Some(record) = historical {
                let id = record.slot().id();
                // Historical assignment is not current authorization. Fresh
                // placement below obtains its authorization from the grant
                // that supplies the ID.
                self.range_authority()
                    .validate_id_authority(id, &request.authority)
                    .map_err(crate::MemoryResolutionError::Range)?;
                id
            } else {
                // Validated ranges are disjoint and ascending, so walking only
                // this authority's Allowed grants preserves lowest-ID placement.
                self.range_authority()
                    .authorities()
                    .iter()
                    .filter(|range| {
                        range.authority() == request.authority
                            && range.mode() == MemoryManagerRangeMode::Allowed
                    })
                    .flat_map(|range| range.range().start()..=range.range().end())
                    .find(|id| !occupied[usize::from(*id)])
                    .ok_or_else(|| crate::MemoryResolutionError::Exhausted {
                        stable_key: request.stable_key.clone(),
                        authority: request.authority.clone(),
                    })?
            };
            let slot = crate::MemoryManagerSlot::new(id).expect("usable id");
            occupied[usize::from(id)] = true;
            // Request construction checked authority/key/schema, and recovery
            // checked historical schemas. Copy borrowed source requests only;
            // owned selections move their fields into the final declarations.
            // The final snapshot still validates all declarations together.
            let request = request.into_owned();
            declarations.push(StaticMemoryDeclaration {
                authority: request.authority,
                declaration: AllocationDeclaration {
                    stable_key: request.stable_key,
                    slot,
                    label: None,
                    schema: request.schema,
                },
            });
        }
        Ok(build_snapshot(
            Cow::Owned(declarations),
            Cow::Borrowed(self.registered_ranges()),
            Cow::Owned(Vec::new()),
        )?)
    }

    /// Borrow fixed declarations, including runtime governance. Key-only requests
    /// are resolved by the runtime after recovery; inspect committed allocations
    /// for the complete resolved set.
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

    pub(crate) fn registered_declaration(
        &self,
        key: &crate::StableKey,
    ) -> Option<&StaticMemoryDeclaration> {
        let declarations = self.registered_declarations();
        // Sealing establishes unique keys in ascending canonical order.
        declarations
            .binary_search_by(|registration| registration.declaration().stable_key().cmp(key))
            .ok()
            .map(|index| &declarations[index])
    }

    pub(crate) fn user_ranges_registered(&self) -> bool {
        !self.inner.registered_ranges.is_empty()
    }

    #[cfg(test)]
    pub(crate) fn shares_storage_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

type StaticRegistrationHook = fn() -> Result<(), StaticMemoryDeclarationError>;

#[derive(Debug)]
struct StaticMemoryDeclarationRegistry {
    declarations: Vec<StaticMemoryDeclaration>,
    requests: Vec<MemoryRequest>,
    ranges: Vec<StaticMemoryRangeDeclaration>,
    registration_hooks: Vec<StaticRegistrationHook>,
    eager_init_hooks: Vec<fn()>,
    lifecycle: StaticRegistryLifecycle,
}

impl StaticMemoryDeclarationRegistry {
    fn finish_sealing(
        &mut self,
        result: Result<SealedDeclarationSnapshot, StaticMemoryDeclarationError>,
    ) -> Result<SealedDeclarationSnapshot, StaticMemoryDeclarationError> {
        // Only the immutable snapshot or terminal error remains useful.
        self.declarations = Vec::new();
        self.requests = Vec::new();
        self.ranges = Vec::new();
        self.registration_hooks = Vec::new();
        self.eager_init_hooks = Vec::new();
        self.lifecycle = match &result {
            Ok(snapshot) => StaticRegistryLifecycle::Sealed(snapshot.clone()),
            Err(error) => StaticRegistryLifecycle::Failed(error.clone()),
        };
        result
    }
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
        requests: Vec::new(),
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
    validate_diagnostic_text(value).map_err(|error| {
        StaticMemoryDeclarationError::InvalidAuthority {
            reason: error.reason(),
        }
    })
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
///
/// # Panics
///
/// Panics only if a private governance-metadata, sealing or fingerprint-encoding
/// invariant is broken.
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
            unreachable!("seal lock preserves the in-progress registry lifecycle");
        }
    };
    let result = match deferred_error {
        Some(error) => Err(error),
        None => build_snapshot(
            Cow::Owned(std::mem::take(&mut registry.declarations)),
            Cow::Owned(std::mem::take(&mut registry.ranges)),
            Cow::Owned(std::mem::take(&mut registry.requests)),
        ),
    };
    registry.finish_sealing(result)
}

fn fail_sealing(
    err: StaticMemoryDeclarationError,
) -> Result<SealedDeclarationSnapshot, StaticMemoryDeclarationError> {
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
    registry.finish_sealing(Err(failure))
}

fn build_snapshot(
    declarations: Cow<'_, [StaticMemoryDeclaration]>,
    ranges: Cow<'_, [StaticMemoryRangeDeclaration]>,
    requests: Cow<'_, [MemoryRequest]>,
) -> Result<SealedDeclarationSnapshot, StaticMemoryDeclarationError> {
    if declarations.len().saturating_add(requests.len()) > 254 || ranges.len() > 254 {
        return Err(StaticMemoryDeclarationError::TooManyDeclarations);
    }
    // Borrowed public inputs stay untouched. Registry sealing and resolution
    // transfer vectors they would otherwise discard after this build.
    let mut requests = requests.into_owned();
    // Accepted keys are unique; equal keys reject below, so stability adds no meaning.
    requests.sort_unstable_by(|a, b| a.stable_key.cmp(&b.stable_key));
    let mut registered_declarations = declarations.into_owned();
    registered_declarations.sort_by(|left, right| {
        left.declaration()
            .stable_key()
            .cmp(right.declaration().stable_key())
            .then_with(|| left.declaration().slot().cmp(right.declaration().slot()))
            .then_with(|| left.authority().cmp(right.authority()))
    });

    // Canonical vectors already supply both membership and adjacency. Check
    // each request in key order so fixed/request and request/request conflicts
    // preserve their shared duplicate-error precedence.
    for (index, request) in requests.iter().enumerate() {
        if (index > 0 && requests[index - 1].stable_key == request.stable_key)
            || registered_declarations
                .binary_search_by(|d| d.declaration().stable_key().cmp(&request.stable_key))
                .is_ok()
        {
            return Err(StaticMemoryDeclarationError::DuplicateRequest {
                stable_key: request.stable_key.clone(),
            });
        }
    }

    let mut registered_ranges = ranges.into_owned();
    // Equal bounds reject as overlaps, so metadata cannot distinguish accepted
    // ranges. Keep bound ordering for deterministic overlap diagnostics.
    registered_ranges.sort_by(|left, right| {
        let left = left.record();
        let right = right.record();
        left.range()
            .start()
            .cmp(&right.range().start())
            .then_with(|| left.range().end().cmp(&right.range().end()))
    });

    let mut allocation_declarations = Vec::with_capacity(registered_declarations.len() + 1);
    allocation_declarations.push(internal_ledger_declaration());
    allocation_declarations.extend(
        registered_declarations
            .iter()
            .map(|registration| registration.declaration().clone()),
    );
    let allocation_snapshot = DeclarationSnapshot::new(allocation_declarations)?;

    let mut authority_records = Vec::with_capacity(registered_ranges.len() + 1);
    authority_records.push(internal_ledger_range());
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
        &requests,
    );

    Ok(SealedDeclarationSnapshot {
        inner: Arc::new(SealedDeclarationSnapshotInner {
            allocation_snapshot,
            requests,
            registered_declarations,
            registered_ranges,
            range_authority,
            fingerprint,
        }),
    })
}

#[derive(Serialize)]
struct SealedDeclarationFingerprintMaterial<'a> {
    format: &'static str,
    allocation_snapshot: &'a DeclarationSnapshot,
    registered_declarations: &'a [StaticMemoryDeclaration],
    effective_ranges: &'a [MemoryManagerAuthorityRecord],
    requests: &'a [MemoryRequest],
}

// Fingerprints need the canonical encoded bytes only as input to the hash;
// keep no payload buffer after serialization.
struct FingerprintWriter(u64);

impl std::io::Write for FingerprintWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 = crate::hash::fnv64(self.0, bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn sealed_declaration_fingerprint(
    allocation_snapshot: &DeclarationSnapshot,
    registered_declarations: &[StaticMemoryDeclaration],
    effective_ranges: &[MemoryManagerAuthorityRecord],
    requests: &[MemoryRequest],
) -> SealedDeclarationFingerprint {
    let material = SealedDeclarationFingerprintMaterial {
        format: "ic-memory.sealed-declaration-fingerprint.v1",
        allocation_snapshot,
        registered_declarations,
        effective_ranges,
        requests,
    };
    let mut writer = FingerprintWriter(crate::hash::FNV_OFFSET);
    // Concrete derived serializers and this hash writer have no recoverable failures.
    ciborium::into_writer(&material, &mut writer)
        .expect("sealed declaration fingerprint encodes into hash");

    SealedDeclarationFingerprint {
        algorithm_version: SEALED_DECLARATION_FINGERPRINT_VERSION,
        value: writer.0,
    }
}

const SEALED_DECLARATION_FINGERPRINT_VERSION: u8 = 1;

fn internal_ledger_declaration() -> AllocationDeclaration {
    AllocationDeclaration::memory_manager(
        IC_MEMORY_LEDGER_STABLE_KEY,
        MEMORY_MANAGER_LEDGER_ID,
        IC_MEMORY_LEDGER_LABEL,
    )
    .unwrap_or_else(|_| unreachable!("built-in ledger declaration constants are valid"))
}

fn internal_ledger_range() -> MemoryManagerAuthorityRecord {
    MemoryManagerAuthorityRecord::new(
        memory_manager_governance_range(),
        IC_MEMORY_AUTHORITY_OWNER,
        MemoryManagerRangeMode::Reserved,
        Some(IC_MEMORY_AUTHORITY_PURPOSE.to_string()),
    )
    .unwrap_or_else(|_| unreachable!("built-in governance range metadata constants are valid"))
}

#[cfg(test)]
pub fn reset_static_memory_declarations_for_tests() {
    let mut registry = STATIC_MEMORY_DECLARATIONS
        .lock()
        .expect("static memory declaration registry poisoned");
    registry.declarations.clear();
    registry.requests.clear();
    registry.ranges.clear();
    registry.registration_hooks.clear();
    registry.eager_init_hooks.clear();
    registry.lifecycle = StaticRegistryLifecycle::Open;
}

#[cfg(test)]
mod tests;
