use crate::{
    declaration::{AllocationDeclaration, DeclarationCollector, DeclarationSnapshot},
    schema::SchemaMetadata,
    slot::{
        MemoryManagerAuthorityRecord, MemoryManagerIdRange, MemoryManagerRangeAuthority,
        MemoryManagerRangeAuthorityError, MemoryManagerRangeMode,
    },
};
use std::sync::{Mutex, MutexGuard};

#[cfg(test)]
pub(crate) static TEST_REGISTRY_LOCK: Mutex<()> = Mutex::new(());

///
/// StaticMemoryDeclaration
///
/// One allocation declaration registered by crate-level generated or macro
/// code before bootstrap seals the declaration snapshot.
///
/// The `declaring_crate` field is policy metadata for integration layers such
/// as Canic or IcyDB. The default runtime uses it to match declarations against
/// registered range claims before it calls the caller's
/// [`crate::AllocationPolicy`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaticMemoryDeclaration {
    declaring_crate: String,
    declaration: AllocationDeclaration,
}

impl StaticMemoryDeclaration {
    /// Build one static declaration from raw parts.
    pub fn new(declaring_crate: impl Into<String>, declaration: AllocationDeclaration) -> Self {
        Self {
            declaring_crate: declaring_crate.into(),
            declaration,
        }
    }

    /// Return the crate that registered this declaration.
    #[must_use]
    pub fn declaring_crate(&self) -> &str {
        &self.declaring_crate
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
/// declarations must stay inside the declaring crate's claimed range before
/// caller-supplied policy runs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaticMemoryRangeDeclaration {
    record: MemoryManagerAuthorityRecord,
}

impl StaticMemoryRangeDeclaration {
    /// Build one static range declaration from a validated authority record.
    #[must_use]
    pub const fn new(record: MemoryManagerAuthorityRecord) -> Self {
        Self { record }
    }

    /// Return the crate that registered this range.
    #[must_use]
    pub fn declaring_crate(&self) -> &str {
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
    declaring_crate: impl Into<String>,
    declaration: AllocationDeclaration,
) -> Result<(), StaticMemoryDeclarationError> {
    with_unsealed_registry(|registry| {
        registry
            .declarations
            .push(StaticMemoryDeclaration::new(declaring_crate, declaration));
    })
}

/// Register one `MemoryManager` authority range before bootstrap seals the snapshot.
pub fn register_static_memory_manager_range(
    start: u8,
    end: u8,
    declaring_crate: impl Into<String>,
    mode: MemoryManagerRangeMode,
    purpose: Option<String>,
) -> Result<(), StaticMemoryDeclarationError> {
    let declaring_crate = declaring_crate.into();
    let record = MemoryManagerAuthorityRecord::new(
        MemoryManagerIdRange::new(start, end).map_err(MemoryManagerRangeAuthorityError::Range)?,
        declaring_crate,
        mode,
        purpose,
    )?;
    register_static_memory_range_declaration(StaticMemoryRangeDeclaration::new(record))
}

/// Register one authority range declaration before bootstrap seals the snapshot.
pub fn register_static_memory_range_declaration(
    declaration: StaticMemoryRangeDeclaration,
) -> Result<(), StaticMemoryDeclarationError> {
    with_unsealed_registry(|registry| {
        registry.ranges.push(declaration);
    })
}

/// Register one `MemoryManager` declaration before bootstrap seals the snapshot.
pub fn register_static_memory_manager_declaration(
    id: u8,
    declaring_crate: impl Into<String>,
    label: impl Into<String>,
    stable_key: impl AsRef<str>,
) -> Result<(), StaticMemoryDeclarationError> {
    register_static_memory_manager_declaration_with_schema(
        id,
        declaring_crate,
        label,
        stable_key,
        SchemaMetadata::default(),
    )
}

/// Register one `MemoryManager` declaration with schema metadata.
pub fn register_static_memory_manager_declaration_with_schema(
    id: u8,
    declaring_crate: impl Into<String>,
    label: impl Into<String>,
    stable_key: impl AsRef<str>,
    schema: SchemaMetadata,
) -> Result<(), StaticMemoryDeclarationError> {
    let declaration =
        AllocationDeclaration::memory_manager_with_schema(stable_key, id, label, schema)?;
    register_static_memory_declaration(declaring_crate, declaration)
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
pub(crate) fn seal_static_memory_registry() -> Result<(), StaticMemoryDeclarationError> {
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
pub(crate) fn reset_static_memory_declarations_for_tests() {
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
        assert_eq!(registrations[0].declaring_crate(), "icydb");
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
        assert_eq!(ranges[0].declaring_crate(), "crate_a");
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

        let range = StaticMemoryRangeDeclaration::new(record);

        assert_eq!(range.declaring_crate(), "record_authority");
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
}
