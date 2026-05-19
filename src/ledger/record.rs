use super::{AllocationRetirementError, LedgerCompatibilityError, LedgerIntegrityError};
use crate::{
    declaration::AllocationDeclaration, key::StableKey, schema::SchemaMetadata,
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
/// construction goes through [`AllocationLedger::new`], which validates the
/// invariant-bearing history before returning a value.
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
    pub records: Vec<AllocationRecord>,
    /// Committed generation records.
    pub generations: Vec<GenerationRecord>,
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
    pub generation: u64,
    /// Schema metadata declared by that generation.
    pub schema: SchemaMetadata,
}

///
/// GenerationRecord
///
/// Diagnostic metadata for one committed ledger generation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GenerationRecord {
    /// Committed generation number.
    pub generation: u64,
    /// Parent generation, if recorded.
    pub parent_generation: Option<u64>,
    /// Optional binary/runtime fingerprint.
    pub runtime_fingerprint: Option<String>,
    /// Number of declarations in the generation.
    pub declaration_count: u32,
    /// Optional commit timestamp supplied by the integration layer.
    pub committed_at: Option<u64>,
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
            schema_history: vec![SchemaMetadataRecord {
                generation,
                schema: declaration.schema,
            }],
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
            self.schema_history.push(SchemaMetadataRecord {
                generation,
                schema: declaration.schema.clone(),
            });
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
            self.schema_history.push(SchemaMetadataRecord {
                generation,
                schema: reservation.schema.clone(),
            });
        }
    }
}

impl AllocationLedger {
    /// Build a ledger DTO and validate structural ledger invariants.
    ///
    /// This constructor is intended for recovered durable records and tests. It
    /// validates committed allocation history, including schema metadata
    /// records, but it does not open storage or allocate slots.
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
