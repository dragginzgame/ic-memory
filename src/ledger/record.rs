use super::{AllocationRetirementError, LedgerCompatibilityError, LedgerIntegrityError};
use crate::{
    declaration::{AllocationDeclaration, DeclarationSnapshotError, validate_runtime_fingerprint},
    key::StableKey,
    schema::{SchemaMetadata, SchemaMetadataError},
    slot::AllocationSlotDescriptor,
};
use serde::{Deserialize, Serialize};

/// Current allocation ledger schema version.
pub const CURRENT_LEDGER_SCHEMA_VERSION: u32 = 1;

/// Current protected physical ledger format identifier.
pub const CURRENT_PHYSICAL_FORMAT_ID: u32 = 1;

///
/// AllocationLedger
///
/// Durable root of allocation history.
///
/// Decoded ledgers are input from persistent storage and should be treated as
/// untrusted until compatibility and integrity validation pass. Public
/// construction goes through [`AllocationLedger::new`], which validates
/// structural history invariants before returning a value. Use
/// [`AllocationLedger::new_committed`] when the value should also satisfy the
/// strict committed-generation chain required by recovery and commit.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AllocationLedger {
    /// Ledger schema version.
    pub(crate) ledger_schema_version: u32,
    /// Physical encoding format identifier.
    pub(crate) physical_format_id: u32,
    /// Current committed generation selected by recovery.
    pub(crate) current_generation: u64,
    /// Historical allocation facts.
    pub(crate) allocation_history: AllocationHistory,
}

///
/// AllocationHistory
///
/// Durable allocation records and generation history.
///
/// This is the durable DTO embedded in an [`AllocationLedger`]. It records
/// allocation facts and generation diagnostics; callers should prefer ledger
/// staging/validation methods over mutating histories directly.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct AllocationHistory {
    /// Stable-key allocation records.
    pub(crate) records: Vec<AllocationRecord>,
    /// Committed generation records.
    pub(crate) generations: Vec<GenerationRecord>,
}

///
/// AllocationRecord
///
/// Durable ownership record for one stable key.
///
/// Records are historical facts, not live handles. Fields are private so stale
/// or invalid ownership state cannot be assembled through public struct
/// literals; use accessors for diagnostics and ledger methods for mutation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AllocationRecord {
    /// Stable key that owns the slot.
    pub(crate) stable_key: StableKey,
    /// Durable allocation slot owned by the key.
    pub(crate) slot: AllocationSlotDescriptor,
    /// Current allocation lifecycle state.
    pub(crate) state: AllocationState,
    /// First committed generation that recorded this allocation.
    pub(crate) first_generation: u64,
    /// Latest committed generation that observed this allocation declaration.
    pub(crate) last_seen_generation: u64,
    /// Generation that explicitly retired this allocation.
    pub(crate) retired_generation: Option<u64>,
    /// Per-generation schema metadata history.
    pub(crate) schema_history: Vec<SchemaMetadataRecord>,
}

///
/// AllocationRetirement
///
/// Explicit request to tombstone one historical allocation identity.
///
/// Retirement prevents a stable key from being redeclared. It does not make the
/// physical slot safe for another active stable key.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AllocationRetirement {
    /// Stable key being retired.
    pub stable_key: StableKey,
    /// Allocation slot historically owned by the stable key.
    pub slot: AllocationSlotDescriptor,
}

impl AllocationRetirement {
    /// Build an explicit retirement request from raw parts.
    pub fn new(
        stable_key: impl AsRef<str>,
        slot: AllocationSlotDescriptor,
    ) -> Result<Self, AllocationRetirementError> {
        Ok(Self {
            stable_key: StableKey::parse(stable_key).map_err(AllocationRetirementError::Key)?,
            slot,
        })
    }
}

///
/// AllocationState
///
/// Allocation lifecycle state.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AllocationState {
    /// Slot is reserved for a future allocation identity.
    Reserved,
    /// Slot is active and may be opened after validation.
    Active,
    /// Slot was explicitly retired and remains tombstoned.
    Retired,
}

///
/// SchemaMetadataRecord
///
/// Schema metadata observed in one committed generation.
///
/// Schema metadata is diagnostic ledger history. It is validated for bounded
/// durable encoding, but `ic-memory` does not prove application schema
/// compatibility or data migration correctness.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SchemaMetadataRecord {
    /// Generation that declared this schema metadata.
    pub(crate) generation: u64,
    /// Schema metadata declared by that generation.
    pub(crate) schema: SchemaMetadata,
}

///
/// GenerationRecord
///
/// Diagnostic metadata for one committed ledger generation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GenerationRecord {
    /// Committed generation number.
    pub(crate) generation: u64,
    /// Parent generation, if recorded.
    pub(crate) parent_generation: Option<u64>,
    /// Optional binary/runtime fingerprint.
    pub(crate) runtime_fingerprint: Option<String>,
    /// Number of declarations in the generation.
    pub(crate) declaration_count: u32,
    /// Optional commit timestamp supplied by the integration layer.
    pub(crate) committed_at: Option<u64>,
}

///
/// LedgerCompatibility
///
/// Supported logical and physical ledger format versions.
///
/// Run this check on recovered ledgers before treating them as authoritative
/// state. Integrity validation then checks allocation history invariants.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LedgerCompatibility {
    /// Minimum supported ledger schema version.
    pub min_ledger_schema_version: u32,
    /// Maximum supported ledger schema version.
    pub max_ledger_schema_version: u32,
    /// Required physical encoding format identifier.
    pub physical_format_id: u32,
}

impl LedgerCompatibility {
    /// Return the compatibility supported by this crate version.
    #[must_use]
    pub const fn current() -> Self {
        Self {
            min_ledger_schema_version: CURRENT_LEDGER_SCHEMA_VERSION,
            max_ledger_schema_version: CURRENT_LEDGER_SCHEMA_VERSION,
            physical_format_id: CURRENT_PHYSICAL_FORMAT_ID,
        }
    }

    /// Validate a decoded ledger before it is used as authoritative state.
    pub const fn validate(
        &self,
        ledger: &AllocationLedger,
    ) -> Result<(), LedgerCompatibilityError> {
        if ledger.ledger_schema_version < self.min_ledger_schema_version {
            return Err(LedgerCompatibilityError::UnsupportedLedgerSchemaVersion {
                found: ledger.ledger_schema_version,
                min_supported: self.min_ledger_schema_version,
                max_supported: self.max_ledger_schema_version,
            });
        }
        if ledger.ledger_schema_version > self.max_ledger_schema_version {
            return Err(LedgerCompatibilityError::UnsupportedLedgerSchemaVersion {
                found: ledger.ledger_schema_version,
                min_supported: self.min_ledger_schema_version,
                max_supported: self.max_ledger_schema_version,
            });
        }
        if ledger.physical_format_id != self.physical_format_id {
            return Err(LedgerCompatibilityError::UnsupportedPhysicalFormat {
                found: ledger.physical_format_id,
                supported: self.physical_format_id,
            });
        }
        Ok(())
    }
}

impl Default for LedgerCompatibility {
    fn default() -> Self {
        Self::current()
    }
}

impl AllocationHistory {
    #[cfg(test)]
    pub(crate) const fn from_parts(
        records: Vec<AllocationRecord>,
        generations: Vec<GenerationRecord>,
    ) -> Self {
        Self {
            records,
            generations,
        }
    }

    /// Borrow stable-key allocation records in durable order.
    #[must_use]
    pub fn records(&self) -> &[AllocationRecord] {
        &self.records
    }

    /// Borrow committed generation records in durable order.
    #[must_use]
    pub fn generations(&self) -> &[GenerationRecord] {
        &self.generations
    }

    /// Return true when the history has no allocation records and no generation records.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty() && self.generations.is_empty()
    }

    pub(crate) const fn records_mut(&mut self) -> &mut Vec<AllocationRecord> {
        &mut self.records
    }

    #[cfg(test)]
    pub(crate) const fn generations_mut(&mut self) -> &mut Vec<GenerationRecord> {
        &mut self.generations
    }

    pub(crate) fn push_record(&mut self, record: AllocationRecord) {
        self.records.push(record);
    }

    pub(crate) fn push_generation(&mut self, generation: GenerationRecord) {
        self.generations.push(generation);
    }
}

impl SchemaMetadataRecord {
    /// Build a schema metadata history record after validating the metadata.
    pub fn new(generation: u64, schema: SchemaMetadata) -> Result<Self, SchemaMetadataError> {
        schema.validate()?;
        Ok(Self { generation, schema })
    }

    /// Return the generation that declared this schema metadata.
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    /// Return the schema metadata declared by that generation.
    #[must_use]
    pub const fn schema(&self) -> &SchemaMetadata {
        &self.schema
    }
}

impl GenerationRecord {
    /// Build a committed generation diagnostic record after validating metadata.
    pub fn new(
        generation: u64,
        parent_generation: Option<u64>,
        runtime_fingerprint: Option<String>,
        declaration_count: u32,
        committed_at: Option<u64>,
    ) -> Result<Self, DeclarationSnapshotError> {
        validate_runtime_fingerprint(runtime_fingerprint.as_deref())?;
        Ok(Self {
            generation,
            parent_generation,
            runtime_fingerprint,
            declaration_count,
            committed_at,
        })
    }

    /// Return the committed generation number.
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    /// Return the parent generation, if recorded.
    #[must_use]
    pub const fn parent_generation(&self) -> Option<u64> {
        self.parent_generation
    }

    /// Borrow the optional binary/runtime fingerprint.
    #[must_use]
    pub fn runtime_fingerprint(&self) -> Option<&str> {
        self.runtime_fingerprint.as_deref()
    }

    /// Return the number of declarations in the generation.
    #[must_use]
    pub const fn declaration_count(&self) -> u32 {
        self.declaration_count
    }

    /// Return the optional commit timestamp supplied by the integration layer.
    #[must_use]
    pub const fn committed_at(&self) -> Option<u64> {
        self.committed_at
    }
}

impl AllocationRecord {
    /// Create a new allocation record from a declaration.
    #[must_use]
    pub(crate) fn from_declaration(
        generation: u64,
        declaration: AllocationDeclaration,
        state: AllocationState,
    ) -> Self {
        Self {
            stable_key: declaration.stable_key,
            slot: declaration.slot,
            state,
            first_generation: generation,
            last_seen_generation: generation,
            retired_generation: None,
            schema_history: vec![
                SchemaMetadataRecord::new(generation, declaration.schema)
                    .expect("declarations validate schema metadata"),
            ],
        }
    }

    /// Create a new reserved allocation record from a declaration.
    #[must_use]
    pub(crate) fn reserved(generation: u64, declaration: AllocationDeclaration) -> Self {
        Self::from_declaration(generation, declaration, AllocationState::Reserved)
    }

    /// Return the stable key that owns this allocation record.
    #[must_use]
    pub const fn stable_key(&self) -> &StableKey {
        &self.stable_key
    }

    /// Return the durable allocation slot owned by this record.
    #[must_use]
    pub const fn slot(&self) -> &AllocationSlotDescriptor {
        &self.slot
    }

    /// Return the current allocation lifecycle state.
    #[must_use]
    pub const fn state(&self) -> AllocationState {
        self.state
    }

    /// Return the first committed generation that recorded this allocation.
    #[must_use]
    pub const fn first_generation(&self) -> u64 {
        self.first_generation
    }

    /// Return the latest committed generation that observed this allocation.
    #[must_use]
    pub const fn last_seen_generation(&self) -> u64 {
        self.last_seen_generation
    }

    /// Return the generation that explicitly retired this allocation, if any.
    #[must_use]
    pub const fn retired_generation(&self) -> Option<u64> {
        self.retired_generation
    }

    /// Return the per-generation schema metadata history.
    #[must_use]
    pub fn schema_history(&self) -> &[SchemaMetadataRecord] {
        &self.schema_history
    }

    pub(crate) fn observe_declaration(
        &mut self,
        generation: u64,
        declaration: &AllocationDeclaration,
    ) {
        self.last_seen_generation = generation;
        if self.state == AllocationState::Reserved {
            self.state = AllocationState::Active;
        }

        let latest_schema = self.schema_history.last().map(|record| &record.schema);
        if latest_schema != Some(&declaration.schema) {
            self.schema_history.push(
                SchemaMetadataRecord::new(generation, declaration.schema.clone())
                    .expect("declarations validate schema metadata"),
            );
        }
    }

    pub(crate) fn observe_reservation(
        &mut self,
        generation: u64,
        reservation: &AllocationDeclaration,
    ) {
        self.last_seen_generation = generation;

        let latest_schema = self.schema_history.last().map(|record| &record.schema);
        if latest_schema != Some(&reservation.schema) {
            self.schema_history.push(
                SchemaMetadataRecord::new(generation, reservation.schema.clone())
                    .expect("reservations validate schema metadata"),
            );
        }
    }
}

impl AllocationLedger {
    /// Build a ledger DTO and validate structural ledger invariants.
    ///
    /// This constructor validates duplicate records, lifecycle state, record
    /// generation bounds, and schema metadata records. It does not require a
    /// complete committed-generation chain. Use
    /// [`AllocationLedger::new_committed`] when constructing an authoritative
    /// committed ledger DTO.
    pub fn new(
        ledger_schema_version: u32,
        physical_format_id: u32,
        current_generation: u64,
        allocation_history: AllocationHistory,
    ) -> Result<Self, LedgerIntegrityError> {
        let ledger = Self {
            ledger_schema_version,
            physical_format_id,
            current_generation,
            allocation_history,
        };
        ledger.validate_integrity()?;
        Ok(ledger)
    }

    /// Build a committed ledger DTO and validate strict committed-history invariants.
    ///
    /// This constructor runs the same committed-integrity checks used by
    /// recovery and commit. Use it when the value should be treated as an
    /// authoritative committed ledger, not merely as a structurally valid DTO.
    pub fn new_committed(
        ledger_schema_version: u32,
        physical_format_id: u32,
        current_generation: u64,
        allocation_history: AllocationHistory,
    ) -> Result<Self, LedgerIntegrityError> {
        let ledger = Self::new(
            ledger_schema_version,
            physical_format_id,
            current_generation,
            allocation_history,
        )?;
        ledger.validate_committed_integrity()?;
        Ok(ledger)
    }

    /// Return the ledger schema version.
    #[must_use]
    pub const fn ledger_schema_version(&self) -> u32 {
        self.ledger_schema_version
    }

    /// Return the protected physical format identifier.
    #[must_use]
    pub const fn physical_format_id(&self) -> u32 {
        self.physical_format_id
    }

    /// Return the current committed generation selected by recovery.
    #[must_use]
    pub const fn current_generation(&self) -> u64 {
        self.current_generation
    }

    /// Return the historical allocation facts.
    #[must_use]
    pub const fn allocation_history(&self) -> &AllocationHistory {
        &self.allocation_history
    }
}
