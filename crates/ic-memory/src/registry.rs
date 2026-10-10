use crate::{
    declaration::{AllocationDeclaration, DeclarationSnapshot},
    schema::SchemaMetadata,
    slot::{
        IC_MEMORY_AUTHORITY_OWNER, IC_MEMORY_LEDGER_LABEL, IC_MEMORY_LEDGER_STABLE_KEY,
        MEMORY_MANAGER_LEDGER_ID, is_ic_memory_stable_key,
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
/// MemoryRequest
///
/// Key-only request resolved after ledger recovery. All admitted owners share the
/// host's free application pool; known keys retain their durable slot.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MemoryRequest {
    authority: String,
    stable_key: crate::StableKey,
    schema: SchemaMetadata,
}

impl<'de> Deserialize<'de> for MemoryRequest {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error as _;
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Request {
            authority: String,
            stable_key: crate::StableKey,
            schema: SchemaMetadata,
        }
        let request = Request::deserialize(deserializer)?;
        Self::new(
            request.authority,
            request.stable_key.as_str(),
            request.schema,
        )
        .map_err(D::Error::custom)
    }
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
/// StaticMemoryDeclarationError
///
/// Failure to register or collect static allocation declarations.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum StaticMemoryDeclarationError {
    #[error("at most 254 external requests are supported")]
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
/// Immutable, canonical linked-program allocation requests supplied to each concrete [`crate::MemoryRuntime`].
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
/// The fingerprint covers canonical source keys, owner labels and schema metadata. It is diagnostic
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
    requests: Vec<MemoryRequest>,
    fingerprint: SealedDeclarationFingerprint,
}

impl SealedDeclarationSnapshot {
    /// Canonicalize explicit requests with the same rules as the linked registry.
    pub fn new(requests: &[MemoryRequest]) -> Result<Self, StaticMemoryDeclarationError> {
        build_snapshot(Cow::Borrowed(requests))
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
        pool: &crate::MemoryAllocationPool,
    ) -> Result<DeclarationSnapshot, crate::MemoryResolutionError> {
        if self.requests().len() + historical.len() > 254 {
            return Err(StaticMemoryDeclarationError::TooManyDeclarations.into());
        }
        let mut declarations = Vec::with_capacity(self.requests().len() + historical.len() + 1);
        declarations.push(internal_ledger_declaration());
        let mut occupied = [false; 255];
        for record in ledger.records() {
            occupied[usize::from(record.slot().id())] = true;
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
                .records()
                .iter()
                .find(|record| record.stable_key() == &request.stable_key);
            pool.validate_authority(&request.stable_key, &request.authority)?;
            let id = if let Some(record) = historical {
                // Durable identity selects placement; the host separately grants
                // current key ownership and retains explicit physical exclusions.
                pool.validate_id(record.slot().id())?;
                record.slot().id()
            } else {
                (0..crate::MEMORY_MANAGER_INVALID_ID)
                    .find(|id| pool.contains(*id) && !occupied[usize::from(*id)])
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
            declarations.push(AllocationDeclaration {
                stable_key: request.stable_key,
                slot,
                label: None,
                schema: request.schema,
            });
        }
        declarations.sort_unstable_by(|a, b| a.stable_key().cmp(b.stable_key()));
        Ok(DeclarationSnapshot::new(declarations).map_err(StaticMemoryDeclarationError::from)?)
    }

    /// Return the deterministic fingerprint of this sealed declaration meaning.
    #[must_use]
    pub fn fingerprint(&self) -> SealedDeclarationFingerprint {
        self.inner.fingerprint
    }

    #[cfg(test)]
    pub(crate) fn shares_storage_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

type StaticRegistrationHook = fn() -> Result<(), StaticMemoryDeclarationError>;

#[derive(Debug)]
struct StaticMemoryDeclarationRegistry {
    requests: Vec<MemoryRequest>,
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
        self.requests = Vec::new();
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
        requests: Vec::new(),
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

fn validate_external_authority(value: &str) -> Result<(), StaticMemoryDeclarationError> {
    reject_internal_authority(value)?;
    validate_diagnostic_text(value).map_err(|error| {
        StaticMemoryDeclarationError::InvalidAuthority {
            reason: error.reason(),
        }
    })
}

fn reject_internal_authority(value: &str) -> Result<(), StaticMemoryDeclarationError> {
    if value == IC_MEMORY_AUTHORITY_OWNER {
        return Err(StaticMemoryDeclarationError::ReservedAuthority {
            authority: value.to_string(),
        });
    }
    Ok(())
}

/// Seal and return the canonical linked-program declaration snapshot.
///
/// The first caller runs deferred generated registrations and eager hooks,
/// canonicalizes requests, validates duplicates and requests, and publishes one immutable snapshot. Concurrent and subsequent
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
        None => build_snapshot(Cow::Owned(std::mem::take(&mut registry.requests))),
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
    requests: Cow<'_, [MemoryRequest]>,
) -> Result<SealedDeclarationSnapshot, StaticMemoryDeclarationError> {
    if requests.len() > 254 {
        return Err(StaticMemoryDeclarationError::TooManyDeclarations);
    }
    let mut requests = requests.into_owned();
    requests.sort_unstable_by(|a, b| a.stable_key.cmp(&b.stable_key));
    for pair in requests.windows(2) {
        if pair[0].stable_key == pair[1].stable_key {
            return Err(StaticMemoryDeclarationError::DuplicateRequest {
                stable_key: pair[1].stable_key.clone(),
            });
        }
    }
    let fingerprint = sealed_declaration_fingerprint(&requests);
    Ok(SealedDeclarationSnapshot {
        inner: Arc::new(SealedDeclarationSnapshotInner {
            requests,
            fingerprint,
        }),
    })
}

#[derive(Serialize)]
struct SealedDeclarationFingerprintMaterial<'a> {
    format: &'static str,
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

fn sealed_declaration_fingerprint(requests: &[MemoryRequest]) -> SealedDeclarationFingerprint {
    let material = SealedDeclarationFingerprintMaterial {
        format: "ic-memory.sealed-declaration-fingerprint.v1",
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

#[cfg(test)]
pub fn reset_static_memory_declarations_for_tests() {
    let mut registry = STATIC_MEMORY_DECLARATIONS
        .lock()
        .expect("static memory declaration registry poisoned");
    registry.requests.clear();
    registry.registration_hooks.clear();
    registry.eager_init_hooks.clear();
    registry.lifecycle = StaticRegistryLifecycle::Open;
}

#[cfg(test)]
mod tests;
