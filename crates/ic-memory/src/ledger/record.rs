#[cfg(any(test, feature = "test-support"))]
use super::LedgerIntegrityError;
use crate::{declaration::AllocationDeclaration, key::StableKey, slot::MemoryManagerSlot};
use serde::{Deserialize, Serialize};

///
/// AllocationLedger
///
/// Durable ownership, bounded by the usable memory-ID
/// domain. Omitted and retired identities retain their slots; no per-upgrade or
/// application schema state is stored.
///
/// The counter binds logical generations to physical commits. Decoded DTOs remain
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
/// Durable ownership and lifecycle state for one stable
/// key. Retaining this record prevents slot reuse after omission or retirement.
/// It is metadata, not a live memory handle or an upgrade audit log.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AllocationRecord {
    pub(crate) stable_key: StableKey,
    pub(crate) slot: MemoryManagerSlot,
    pub(crate) state: AllocationState,
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
/// RecoveredLedger
///
/// Validated base for declaration checks: either a recovered ledger that passed
/// current-format decoding, integrity checks and physical/logical counter binding,
/// or empty logical genesis admitted only by explicit snapshot absence. Genesis
/// is not a persisted generation. A decoded ledger alone is a passive DTO.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveredLedger {
    ledger: AllocationLedger,
}

impl RecoveredLedger {
    // Protected recovery alone establishes absence. All present-snapshot
    // failures remain failures for bootstrap and diagnostic validation alike.
    pub(crate) fn genesis_for_absent_snapshot(error: &super::LedgerCommitError) -> Option<Self> {
        matches!(
            error,
            super::LedgerCommitError::Recovery(crate::CommitRecoveryError::NoValidGeneration)
        )
        .then(|| Self::from_trusted_ledger(AllocationLedger::empty_genesis()))
    }

    pub(crate) const fn from_trusted_ledger(ledger: AllocationLedger) -> Self {
        Self { ledger }
    }

    /// Borrow the recovered metadata, without granting open authority.
    #[must_use]
    pub const fn ledger(&self) -> &AllocationLedger {
        &self.ledger
    }

    /// Return the bound recovered counter, or zero for uncommitted logical genesis.
    #[must_use]
    #[cfg(test)]
    pub const fn current_generation(&self) -> u64 {
        self.ledger.current_generation
    }

    pub(crate) fn into_ledger(self) -> AllocationLedger {
        self.ledger
    }
}

impl AllocationRecord {
    pub(crate) fn from_declaration(
        declaration: &AllocationDeclaration,
        state: AllocationState,
    ) -> Self {
        Self {
            stable_key: declaration.stable_key.clone(),
            slot: declaration.slot.clone(),
            state,
        }
    }

    pub(crate) fn active(declaration: &AllocationDeclaration) -> Self {
        Self::from_declaration(declaration, AllocationState::Active)
    }

    #[cfg(test)]
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
    #[cfg(any(test, feature = "test-support"))]
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

    // Every retained lifecycle state owns its ID, including omitted/retired keys.
    pub(crate) fn occupied_slots(&self) -> [bool; crate::constants::MAX_ALLOCATIONS] {
        let mut occupied = [false; crate::constants::MAX_ALLOCATIONS];
        for record in &self.records {
            occupied[usize::from(record.slot.id())] = true;
        }
        occupied
    }
}
