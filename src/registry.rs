use crate::{
    constants::DIAGNOSTIC_STRING_MAX_BYTES,
    declaration::{AllocationDeclaration, DeclarationCollector, DeclarationSnapshot},
    schema::SchemaMetadata,
    slot::{
        IC_MEMORY_AUTHORITY_OWNER, MemoryManagerAuthorityRecord, MemoryManagerIdRange,
        MemoryManagerRangeAuthority, MemoryManagerRangeAuthorityError, MemoryManagerRangeMode,
        is_ic_memory_stable_key,
    },
};
use std::sync::{Mutex, MutexGuard};

#[cfg(test)]
pub static TEST_REGISTRY_LOCK: Mutex<()> = Mutex::new(());

///
/// StaticMemoryDeclaration
///
/// One allocation declaration registered by crate-level generated or macro
/// code before bootstrap seals the declaration snapshot.
///
/// The `authority` field is policy metadata for integration layers such as
/// Canic or IcyDB. The default runtime uses it to match declarations against
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
/// macro code before bootstrap seals the declaration snapshot. In the default
/// runtime, registered user ranges are authoritative generic range policy:
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

#[derive(Debug, Default)]
struct StaticMemoryDeclarationRegistry {
    declarations: Vec<StaticMemoryDeclaration>,
    ranges: Vec<StaticMemoryRangeDeclaration>,
    sealed: bool,
}

static STATIC_MEMORY_DECLARATIONS: Mutex<StaticMemoryDeclarationRegistry> =
    Mutex::new(StaticMemoryDeclarationRegistry {
        declarations: Vec::new(),
        ranges: Vec::new(),
        sealed: false,
    });

fn lock_registry()
-> Result<MutexGuard<'static, StaticMemoryDeclarationRegistry>, StaticMemoryDeclarationError> {
    STATIC_MEMORY_DECLARATIONS
        .lock()
        .map_err(|_| StaticMemoryDeclarationError::RegistryPoisoned)
}

const fn ensure_unsealed(
    registry: &StaticMemoryDeclarationRegistry,
) -> Result<(), StaticMemoryDeclarationError> {
    if registry.sealed {
        return Err(StaticMemoryDeclarationError::RegistrySealed);
    }
    Ok(())
}

fn with_unsealed_registry(
    op: impl FnOnce(&mut StaticMemoryDeclarationRegistry),
) -> Result<(), StaticMemoryDeclarationError> {
    let mut registry = lock_registry()?;
    ensure_unsealed(&registry)?;
    op(&mut registry);
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

/// Return the currently registered static allocation declarations.
pub fn static_memory_declarations()
-> Result<Vec<StaticMemoryDeclaration>, StaticMemoryDeclarationError> {
    Ok(lock_registry()?.declarations.clone())
}

/// Return the currently registered static range declarations.
pub fn static_memory_range_declarations()
-> Result<Vec<StaticMemoryRangeDeclaration>, StaticMemoryDeclarationError> {
    Ok(lock_registry()?.ranges.clone())
}

/// Return the currently registered static range declarations as an authority table.
pub fn static_memory_range_authority()
-> Result<MemoryManagerRangeAuthority, StaticMemoryDeclarationError> {
    MemoryManagerRangeAuthority::from_records(
        static_memory_range_declarations()?
            .into_iter()
            .map(StaticMemoryRangeDeclaration::into_record)
            .collect(),
    )
    .map_err(StaticMemoryDeclarationError::Range)
}

/// Seal the static memory registry so later registration attempts fail closed.
pub fn seal_static_memory_registry() -> Result<(), StaticMemoryDeclarationError> {
    let mut registry = lock_registry()?;
    registry.sealed = true;
    Ok(())
}

/// Add currently registered static allocation declarations to a collector.
pub fn collect_static_memory_declarations(
    collector: &mut DeclarationCollector,
) -> Result<(), StaticMemoryDeclarationError> {
    for registration in static_memory_declarations()? {
        collector.push(registration.into_declaration());
    }
    Ok(())
}

/// Seal currently registered static allocation declarations into a snapshot.
///
/// Sealing prevents later static registrations from being accepted. Callers may
/// still call this function again to rebuild the same snapshot for idempotent
/// bootstrap paths.
pub fn static_memory_declaration_snapshot()
-> Result<DeclarationSnapshot, StaticMemoryDeclarationError> {
    let declarations = {
        let mut registry = lock_registry()?;
        registry.sealed = true;
        registry
            .declarations
            .iter()
            .map(|registration| registration.declaration.clone())
            .collect()
    };
    DeclarationSnapshot::new(declarations).map_err(StaticMemoryDeclarationError::Declaration)
}

#[cfg(test)]
pub fn reset_static_memory_declarations_for_tests() {
    let mut registry = STATIC_MEMORY_DECLARATIONS
        .lock()
        .expect("static memory declaration registry poisoned");
    registry.declarations.clear();
    registry.ranges.clear();
    registry.sealed = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_and_seals_static_memory_declarations() {
        let _guard = TEST_REGISTRY_LOCK.lock().expect("test lock poisoned");
        reset_static_memory_declarations_for_tests();

        register_static_memory_manager_declaration(100, "icydb", "users", "icydb.users.data.v1")
            .expect("register declaration");

        let registrations = static_memory_declarations().expect("registrations");
        assert_eq!(registrations.len(), 1);
        assert_eq!(registrations[0].authority(), "icydb");
        assert_eq!(
            registrations[0].declaration().stable_key().as_str(),
            "icydb.users.data.v1"
        );

        let snapshot = static_memory_declaration_snapshot().expect("snapshot");
        assert_eq!(snapshot.len(), 1);

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

        let ranges = static_memory_range_declarations().expect("ranges");
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

        let err = static_memory_declaration_snapshot().expect_err("duplicate slot must fail");
        assert!(matches!(
            err,
            StaticMemoryDeclarationError::Declaration(
                crate::DeclarationSnapshotError::DuplicateSlot(_)
            )
        ));
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
}
