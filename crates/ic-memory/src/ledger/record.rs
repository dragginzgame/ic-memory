use super::{AllocationRetirementError, LedgerIntegrityError};
use crate::{
    declaration::AllocationDeclaration, key::StableKey, schema::SchemaMetadata,
    slot::MemoryManagerSlot,
};
use serde::{Deserialize, Serialize};

///
/// AllocationLedger
///
/// Durable ownership and current metadata, bounded by the usable memory-ID
/// domain. Omitted and retired identities retain their slots; no per-upgrade or
/// schema audit trail is stored.
///
/// The counter binds validation proofs to physical commits. Decoded DTOs remain
/// untrusted until integrity validation and protected recovery succeed.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AllocationLedger {
    pub(crate) current_generation: u64,
    #[serde(deserialize_with = "crate::cbor::deserialize_records")]
    pub(crate) records: Vec<AllocationRecord>,
}

///
/// AllocationRecord
///
/// Durable ownership, lifecycle state and latest schema metadata for one stable
/// key. Retaining this record prevents slot reuse after omission or retirement.
/// It is metadata, not a live memory handle or an upgrade audit log.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AllocationRecord {
    pub(crate) stable_key: StableKey,
    pub(crate) slot: MemoryManagerSlot,
    pub(crate) state: AllocationState,
    pub(crate) schema: SchemaMetadata,
}

///
/// AllocationState
///
/// Current allocation lifecycle state. Retirement permanently retains the
/// identity and slot rather than adding a historical event or freeing the ID.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub enum AllocationState {
    /// Slot is reserved for a future allocation identity.
    Reserved,
    /// Slot is active and may be opened after validation.
    Active,
    /// Identity and slot are permanently tombstoned.
    Retired,
}

///
/// AllocationRetirement
///
/// Explicit request to tombstone one retained allocation identity. Retirement
/// prevents redeclaration and never frees the physical slot for another key.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AllocationRetirement {
    pub(crate) stable_key: StableKey,
    pub(crate) slot: MemoryManagerSlot,
}

impl AllocationRetirement {
    /// Build an explicit retirement request from checked raw parts.
    pub fn new(
        stable_key: impl AsRef<str>,
        slot: MemoryManagerSlot,
    ) -> Result<Self, AllocationRetirementError> {
        let stable_key = StableKey::parse(stable_key).map_err(AllocationRetirementError::Key)?;
        Ok(Self { stable_key, slot })
    }

    /// Return the stable key being retired.
    #[must_use]
    pub const fn stable_key(&self) -> &StableKey {
        &self.stable_key
    }

    /// Return the allocation slot named by the request.
    #[must_use]
    pub const fn slot(&self) -> &MemoryManagerSlot {
        &self.slot
    }
}

///
/// RecoveredLedger
///
/// Proof that a ledger crossed physical recovery, current-format decoding,
/// integrity validation and physical/logical commit-counter binding. Only this
/// proof can feed declaration validation; a decoded ledger is a passive DTO.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveredLedger {
    ledger: AllocationLedger,
}

impl RecoveredLedger {
    pub(crate) const fn from_trusted_ledger(ledger: AllocationLedger) -> Self {
        Self { ledger }
    }

    /// Borrow the recovered metadata, without granting open authority.
    #[must_use]
    pub const fn ledger(&self) -> &AllocationLedger {
        &self.ledger
    }

    /// Return the checked physical commit counter.
    #[must_use]
    pub const fn physical_generation(&self) -> u64 {
        self.ledger.current_generation
    }

    /// Return the matching logical commit counter.
    #[must_use]
    pub const fn current_generation(&self) -> u64 {
        self.ledger.current_generation
    }

    pub(crate) fn into_ledger(self) -> AllocationLedger {
        self.ledger
    }
}

impl AllocationRecord {
    fn from_declaration(declaration: &AllocationDeclaration, state: AllocationState) -> Self {
        Self {
            stable_key: declaration.stable_key.clone(),
            slot: declaration.slot.clone(),
            state,
            schema: declaration.schema.clone(),
        }
    }

    pub(crate) fn active(declaration: &AllocationDeclaration) -> Self {
        Self::from_declaration(declaration, AllocationState::Active)
    }

    pub(crate) fn reserved(declaration: &AllocationDeclaration) -> Self {
        Self::from_declaration(declaration, AllocationState::Reserved)
    }

    /// Return the permanent store identity.
    #[must_use]
    pub const fn stable_key(&self) -> &StableKey {
        &self.stable_key
    }

    /// Return its permanently claimed memory ID.
    #[must_use]
    pub const fn slot(&self) -> &MemoryManagerSlot {
        &self.slot
    }

    /// Return the current lifecycle state.
    #[must_use]
    pub const fn state(&self) -> AllocationState {
        self.state
    }

    /// Borrow the latest declared schema metadata; no schema history is kept.
    #[must_use]
    pub const fn schema(&self) -> &SchemaMetadata {
        &self.schema
    }
}

impl AllocationLedger {
    pub(crate) const fn empty_genesis() -> Self {
        Self {
            current_generation: 0,
            records: Vec::new(),
        }
    }

    /// Build a ledger DTO after validating count, ownership and genesis rules.
    /// Recovery must still establish the persisted format and physical binding.
    pub fn new(
        current_generation: u64,
        records: Vec<AllocationRecord>,
    ) -> Result<Self, LedgerIntegrityError> {
        let ledger = Self {
            current_generation,
            records,
        };
        ledger.validate_integrity()?;
        Ok(ledger)
    }

    /// Return the current commit counter; this is not retained upgrade history.
    #[must_use]
    pub const fn current_generation(&self) -> u64 {
        self.current_generation
    }

    /// Borrow retained ownership records, including omitted and retired stores.
    #[must_use]
    pub fn records(&self) -> &[AllocationRecord] {
        &self.records
    }
}
