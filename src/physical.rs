use serde::{Deserialize, Serialize};

const COMMIT_MARKER: u64 = 0x4943_4D45_4D43_4F4D;
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

///
/// ProtectedGenerationSlot
///
/// One physical generation slot that can participate in protected recovery.
///
/// This is an advanced low-level API for framework or stable-IO owners. Most
/// callers should use the ledger commit/recovery flow instead of implementing
/// physical slot recovery directly.
pub trait ProtectedGenerationSlot: Eq {
    /// Generation encoded by this slot.
    fn generation(&self) -> u64;

    /// Return whether the slot passed its marker/checksum validation.
    fn validates(&self) -> bool;
}

///
/// DualProtectedCommitStore
///
/// Physical store with two protected generation slots.
///
/// This is an advanced low-level API for framework or stable-IO owners. Normal
/// allocation flows recover and commit ledgers through the higher-level ledger
/// commit APIs.
pub trait DualProtectedCommitStore {
    /// Protected slot record type.
    type Slot: ProtectedGenerationSlot;

    /// Borrow the first physical slot.
    fn slot0(&self) -> Option<&Self::Slot>;

    /// Borrow the second physical slot.
    fn slot1(&self) -> Option<&Self::Slot>;

    /// Return true when no commit slot has ever been written.
    fn is_uninitialized(&self) -> bool {
        self.slot0().is_none() && self.slot1().is_none()
    }

    /// Return the highest-generation valid physical slot.
    fn authoritative_slot(&self) -> Result<AuthoritativeSlot<'_, Self::Slot>, CommitRecoveryError> {
        select_authoritative_slot(self.slot0(), self.slot1())
    }

    /// Return the slot that should receive the next staged generation write.
    ///
    /// The result is derived from validated recovery state. It does not trust a
    /// separate current-pointer/header field.
    fn inactive_slot_index(&self) -> CommitSlotIndex {
        match self.authoritative_slot() {
            Ok(authoritative) => authoritative.index.opposite(),
            Err(_) => CommitSlotIndex::Slot0,
        }
    }
}

///
/// CommitSlotIndex
///
/// Physical dual-slot index selected by protected recovery.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum CommitSlotIndex {
    /// First physical commit slot.
    Slot0,
    /// Second physical commit slot.
    Slot1,
}

impl CommitSlotIndex {
    /// Return the opposite physical slot.
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::Slot0 => Self::Slot1,
            Self::Slot1 => Self::Slot0,
        }
    }
}

///
/// AuthoritativeSlot
///
/// Highest-generation valid slot selected by protected recovery.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthoritativeSlot<'slot, T> {
    /// Physical slot index.
    pub index: CommitSlotIndex,
    /// Valid committed generation in that slot.
    pub record: &'slot T,
}

/// Select the highest-generation valid physical slot.
///
/// This is an advanced recovery helper for framework or stable-IO owners. It
/// only selects among supplied protected slots; it does not decode or validate
/// the allocation ledger payload.
pub fn select_authoritative_slot<'slot, T: ProtectedGenerationSlot>(
    slot0: Option<&'slot T>,
    slot1: Option<&'slot T>,
) -> Result<AuthoritativeSlot<'slot, T>, CommitRecoveryError> {
    let slot0 = slot0
        .filter(|slot| slot.validates())
        .map(|record| AuthoritativeSlot {
            index: CommitSlotIndex::Slot0,
            record,
        });
    let slot1 = slot1
        .filter(|slot| slot.validates())
        .map(|record| AuthoritativeSlot {
            index: CommitSlotIndex::Slot1,
            record,
        });

    match (slot0, slot1) {
        (Some(left), Some(right))
            if left.record.generation() == right.record.generation()
                && left.record != right.record =>
        {
            Err(CommitRecoveryError::AmbiguousGeneration {
                generation: left.record.generation(),
            })
        }
        (Some(left), Some(right)) if right.record.generation() > left.record.generation() => {
            Ok(right)
        }
        (Some(left), Some(_) | None) => Ok(left),
        (None, Some(right)) => Ok(right),
        (None, None) => Err(CommitRecoveryError::NoValidGeneration),
    }
}

///
/// CommittedGenerationBytes
///
/// Physically committed ledger generation payload protected by a checksum.
///
/// This is an advanced low-level DTO for framework or stable-IO owners. Its
/// recovered bytes are untrusted until marker/checksum validation and ledger
/// decoding/integrity validation have both succeeded.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CommittedGenerationBytes {
    /// Generation number represented by this payload.
    pub(crate) generation: u64,
    /// Physical commit marker. Readers reject records with an invalid marker.
    pub(crate) commit_marker: u64,
    /// Checksum over the generation, marker, and payload bytes.
    pub(crate) checksum: u64,
    /// Encoded ledger generation payload.
    pub(crate) payload: Vec<u8>,
}

impl CommittedGenerationBytes {
    /// Build a committed generation record.
    #[must_use]
    pub fn new(generation: u64, payload: Vec<u8>) -> Self {
        let mut record = Self {
            generation,
            commit_marker: COMMIT_MARKER,
            checksum: 0,
            payload,
        };
        record.checksum = generation_checksum(&record);
        record
    }

    /// Return the generation number represented by this payload.
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    /// Return the physical commit marker.
    ///
    /// This is diagnostic data from a recovered record. Callers should use
    /// [`CommittedGenerationBytes::validates`] before treating the record as
    /// authoritative.
    #[must_use]
    pub const fn commit_marker(&self) -> u64 {
        self.commit_marker
    }

    /// Return the checksum over the generation, marker, and payload bytes.
    ///
    /// The checksum is non-cryptographic and detects torn writes or accidental
    /// corruption only.
    #[must_use]
    pub const fn checksum(&self) -> u64 {
        self.checksum
    }

    /// Borrow the encoded ledger generation payload.
    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// Return whether the marker and checksum validate.
    #[must_use]
    pub fn validates(&self) -> bool {
        self.commit_marker == COMMIT_MARKER && self.checksum == generation_checksum(self)
    }
}

impl ProtectedGenerationSlot for CommittedGenerationBytes {
    fn generation(&self) -> u64 {
        self.generation
    }

    fn validates(&self) -> bool {
        self.validates()
    }
}

///
/// DualCommitStore
///
/// Dual-slot protected commit protocol for encoded ledger generations.
///
/// This is an advanced low-level API for framework or stable-IO owners. Most
/// applications should recover, validate, and commit through the allocation
/// ledger flow rather than manipulating encoded physical commit slots directly.
///
/// Writers stage a complete generation record into the inactive slot. Readers
/// recover by selecting the highest-generation valid slot. A torn or partial
/// write cannot become authoritative unless its marker and checksum validate.
///
/// The checksum is for torn-write and accidental-corruption detection only. It
/// is not a cryptographic hash and does not provide adversarial tamper
/// resistance.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DualCommitStore {
    /// First physical commit slot.
    pub(crate) slot0: Option<CommittedGenerationBytes>,
    /// Second physical commit slot.
    pub(crate) slot1: Option<CommittedGenerationBytes>,
}

impl DualCommitStore {
    /// Return true when no commit slot has ever been written.
    #[must_use]
    pub const fn is_uninitialized(&self) -> bool {
        self.slot0.is_none() && self.slot1.is_none()
    }

    /// Borrow the first physical commit slot.
    ///
    /// Slot records are untrusted recovered state until recovery selects an
    /// authoritative generation.
    #[must_use]
    pub const fn slot0(&self) -> Option<&CommittedGenerationBytes> {
        self.slot0.as_ref()
    }

    /// Borrow the second physical commit slot.
    ///
    /// Slot records are untrusted recovered state until recovery selects an
    /// authoritative generation.
    #[must_use]
    pub const fn slot1(&self) -> Option<&CommittedGenerationBytes> {
        self.slot1.as_ref()
    }

    /// Return the highest-generation valid committed record.
    pub fn authoritative(&self) -> Result<&CommittedGenerationBytes, CommitRecoveryError> {
        self.authoritative_slot()
            .map(|authoritative| authoritative.record)
    }

    /// Build a read-only recovery diagnostic for the protected commit slots.
    #[must_use]
    pub fn diagnostic(&self) -> CommitStoreDiagnostic {
        CommitStoreDiagnostic::from_store(self)
    }

    /// Commit a new payload to the inactive slot.
    ///
    /// The returned store models the post-write physical state. If a real
    /// substrate traps before the inactive slot is fully written, the prior
    /// valid slot remains authoritative under `authoritative`.
    pub fn commit_payload(
        &mut self,
        payload: Vec<u8>,
    ) -> Result<&CommittedGenerationBytes, CommitRecoveryError> {
        let next_generation =
            match self.authoritative() {
                Ok(record) => record.generation.checked_add(1).ok_or(
                    CommitRecoveryError::GenerationOverflow {
                        generation: record.generation,
                    },
                )?,
                Err(CommitRecoveryError::NoValidGeneration) if self.is_uninitialized() => 0,
                Err(err) => return Err(err),
            };

        self.commit_payload_at_generation(next_generation, payload)
    }

    /// Commit `payload` as an explicitly numbered physical generation.
    ///
    /// This is the low-level physical-slot primitive used by
    /// [`crate::LedgerCommitStore`]. Normal ledger commits should use
    /// [`crate::LedgerCommitStore::commit`] or [`crate::AllocationBootstrap`] so
    /// payloads are decoded, current-format checked, and integrity-validated
    /// before they can become authoritative.
    ///
    /// The physical slot generation is checked against the recovered physical
    /// predecessor. This method does not inspect `payload`.
    pub fn commit_payload_at_generation(
        &mut self,
        generation: u64,
        payload: Vec<u8>,
    ) -> Result<&CommittedGenerationBytes, CommitRecoveryError> {
        match self.authoritative() {
            Ok(record) => {
                let expected = record.generation.checked_add(1).ok_or(
                    CommitRecoveryError::GenerationOverflow {
                        generation: record.generation,
                    },
                )?;
                if generation != expected {
                    return Err(CommitRecoveryError::UnexpectedGeneration {
                        expected,
                        actual: generation,
                    });
                }
            }
            Err(CommitRecoveryError::NoValidGeneration) if self.is_uninitialized() => {}
            Err(err) => return Err(err),
        }

        let next = CommittedGenerationBytes::new(generation, payload);

        if self.inactive_slot_index() == CommitSlotIndex::Slot0 {
            self.slot0 = Some(next);
        } else {
            self.slot1 = Some(next);
        }

        self.authoritative()
    }

    /// Simulate a torn write into the inactive slot.
    ///
    /// This helper is intentionally part of the model because recovery behavior
    /// is an ABI requirement, not an implementation detail.
    #[cfg(test)]
    pub fn write_corrupt_inactive_slot(&mut self, generation: u64, payload: Vec<u8>) {
        let mut corrupt = CommittedGenerationBytes::new(generation, payload);
        corrupt.checksum = corrupt.checksum.wrapping_add(1);

        if self.inactive_slot_index() == CommitSlotIndex::Slot0 {
            self.slot0 = Some(corrupt);
        } else {
            self.slot1 = Some(corrupt);
        }
    }
}

impl DualProtectedCommitStore for DualCommitStore {
    type Slot = CommittedGenerationBytes;

    fn slot0(&self) -> Option<&Self::Slot> {
        self.slot0.as_ref()
    }

    fn slot1(&self) -> Option<&Self::Slot> {
        self.slot1.as_ref()
    }
}

///
/// CommitStoreDiagnostic
///
/// Read-only diagnostic summary of protected commit recovery state.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CommitStoreDiagnostic {
    /// First physical commit slot diagnostic.
    pub slot0: CommitSlotDiagnostic,
    /// Second physical commit slot diagnostic.
    pub slot1: CommitSlotDiagnostic,
    /// Highest valid generation selected by recovery.
    pub authoritative_generation: Option<u64>,
    /// Recovery error when no authoritative generation can be selected.
    pub recovery_error: Option<CommitRecoveryError>,
}

impl CommitStoreDiagnostic {
    /// Build a read-only recovery diagnostic from a dual protected commit store.
    #[must_use]
    pub fn from_store<S: DualProtectedCommitStore>(store: &S) -> Self {
        let (authoritative_generation, recovery_error) = match store.authoritative_slot() {
            Ok(slot) => (Some(slot.record.generation()), None),
            Err(err) => (None, Some(err)),
        };
        Self {
            slot0: CommitSlotDiagnostic::from_slot(store.slot0()),
            slot1: CommitSlotDiagnostic::from_slot(store.slot1()),
            authoritative_generation,
            recovery_error,
        }
    }
}

///
/// CommitSlotDiagnostic
///
/// Read-only diagnostic summary for one protected commit slot.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CommitSlotDiagnostic {
    /// Whether a physical slot record is present.
    pub present: bool,
    /// Generation encoded by the slot, if present.
    pub generation: Option<u64>,
    /// Whether marker and checksum validation succeeded.
    pub valid: bool,
}

impl CommitSlotDiagnostic {
    fn from_slot<T: ProtectedGenerationSlot>(slot: Option<&T>) -> Self {
        match slot {
            Some(record) => Self {
                present: true,
                generation: Some(record.generation()),
                valid: record.validates(),
            },
            None => Self {
                present: false,
                generation: None,
                valid: false,
            },
        }
    }
}

///
/// CommitRecoveryError
///
/// Protected commit recovery failure.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Deserialize, Eq, thiserror::Error, PartialEq, Serialize)]
pub enum CommitRecoveryError {
    /// No committed slot passed marker and checksum validation.
    #[error("no valid committed ledger generation")]
    NoValidGeneration,
    /// Both physical slots validated at the same generation but contained different bytes.
    #[error("ambiguous committed ledger generation {generation}")]
    AmbiguousGeneration {
        /// Ambiguous physical generation.
        generation: u64,
    },
    /// Physical generation advancement would overflow.
    #[error("committed ledger generation {generation} cannot be advanced without overflow")]
    GenerationOverflow {
        /// Last valid physical generation.
        generation: u64,
    },
    /// Caller attempted to commit a physical generation other than the next generation.
    #[error("expected committed ledger generation {expected}, got {actual}")]
    UnexpectedGeneration {
        /// Expected next physical generation.
        expected: u64,
        /// Actual requested physical generation.
        actual: u64,
    },
}

fn generation_checksum(generation: &CommittedGenerationBytes) -> u64 {
    let mut hash = FNV_OFFSET;
    hash = hash_u64(hash, generation.generation);
    hash = hash_u64(hash, generation.commit_marker);
    hash = hash_usize(hash, generation.payload.len());
    for byte in &generation.payload {
        hash = hash_byte(hash, *byte);
    }
    hash
}

fn hash_usize(hash: u64, value: usize) -> u64 {
    hash_u64(hash, value as u64)
}

fn hash_u64(mut hash: u64, value: u64) -> u64 {
    for byte in value.to_le_bytes() {
        hash = hash_byte(hash, byte);
    }
    hash
}

const fn hash_byte(hash: u64, byte: u8) -> u64 {
    (hash ^ byte as u64).wrapping_mul(FNV_PRIME)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload(value: u8) -> Vec<u8> {
        vec![value; 4]
    }

    #[test]
    fn committed_generation_validates_marker_and_checksum() {
        let mut generation = CommittedGenerationBytes::new(7, payload(1));
        assert!(generation.validates());

        generation.checksum = generation.checksum.wrapping_add(1);
        assert!(!generation.validates());
    }

    #[test]
    fn physical_commit_accessors_expose_read_only_state() {
        let mut store = DualCommitStore::default();
        store.commit_payload(payload(1)).expect("first commit");

        let slot = store.slot0().expect("first slot");

        assert_eq!(slot.generation(), 0);
        assert_eq!(slot.payload(), payload(1).as_slice());
        assert_eq!(slot.commit_marker(), COMMIT_MARKER);
        assert_eq!(slot.checksum(), generation_checksum(slot));
        assert!(store.slot1().is_none());
    }

    #[test]
    fn authoritative_selects_highest_valid_generation() {
        let mut store = DualCommitStore::default();
        store.commit_payload(payload(1)).expect("first commit");
        store.commit_payload(payload(2)).expect("second commit");

        let authoritative = store.authoritative().expect("authoritative");
        let authoritative_slot =
            select_authoritative_slot(store.slot0.as_ref(), store.slot1.as_ref())
                .expect("authoritative slot");

        assert_eq!(authoritative.generation, 1);
        assert_eq!(authoritative.payload, payload(2));
        assert_eq!(authoritative_slot.index, CommitSlotIndex::Slot1);
        assert_eq!(authoritative_slot.record.payload, payload(2));
    }

    #[test]
    fn corrupt_newer_slot_leaves_prior_generation_authoritative() {
        let mut store = DualCommitStore::default();
        store.commit_payload(payload(1)).expect("first commit");
        store.write_corrupt_inactive_slot(1, payload(2));

        let authoritative = store.authoritative().expect("authoritative");

        assert_eq!(authoritative.generation, 0);
        assert_eq!(authoritative.payload, payload(1));
    }

    #[test]
    fn no_valid_generation_fails_closed() {
        let mut store = DualCommitStore::default();
        store.write_corrupt_inactive_slot(0, payload(1));
        store.write_corrupt_inactive_slot(1, payload(2));

        let err = store.authoritative().expect_err("no valid slot");

        assert_eq!(err, CommitRecoveryError::NoValidGeneration);
    }

    #[test]
    fn same_generation_identical_slots_recover_deterministically() {
        let committed = CommittedGenerationBytes::new(7, payload(1));
        let store = DualCommitStore {
            slot0: Some(committed.clone()),
            slot1: Some(committed),
        };

        let authoritative = store.authoritative_slot().expect("authoritative");

        assert_eq!(authoritative.index, CommitSlotIndex::Slot0);
        assert_eq!(authoritative.record.generation, 7);
    }

    #[test]
    fn same_generation_divergent_slots_fail_closed() {
        let store = DualCommitStore {
            slot0: Some(CommittedGenerationBytes::new(7, payload(1))),
            slot1: Some(CommittedGenerationBytes::new(7, payload(2))),
        };

        let err = store.authoritative().expect_err("ambiguous generation");

        assert_eq!(
            err,
            CommitRecoveryError::AmbiguousGeneration { generation: 7 }
        );
    }

    #[test]
    fn physical_generation_overflow_fails_closed() {
        let mut store = DualCommitStore {
            slot0: Some(CommittedGenerationBytes::new(u64::MAX, payload(1))),
            slot1: None,
        };

        let err = store
            .commit_payload(payload(2))
            .expect_err("overflow must fail");

        assert_eq!(
            err,
            CommitRecoveryError::GenerationOverflow {
                generation: u64::MAX
            }
        );
    }

    #[test]
    fn diagnostic_reports_authoritative_generation_and_corrupt_slots() {
        let mut store = DualCommitStore::default();
        store.commit_payload(payload(1)).expect("first commit");
        store.write_corrupt_inactive_slot(1, payload(2));

        let diagnostic = store.diagnostic();

        assert_eq!(diagnostic.authoritative_generation, Some(0));
        assert_eq!(diagnostic.recovery_error, None);
        assert_eq!(diagnostic.slot0.generation, Some(0));
        assert!(diagnostic.slot0.valid);
        assert_eq!(diagnostic.slot1.generation, Some(1));
        assert!(!diagnostic.slot1.valid);
    }

    #[test]
    fn diagnostic_reports_no_valid_generation_for_empty_store() {
        let diagnostic = DualCommitStore::default().diagnostic();

        assert_eq!(diagnostic.authoritative_generation, None);
        assert_eq!(
            diagnostic.recovery_error,
            Some(CommitRecoveryError::NoValidGeneration)
        );
        assert!(!diagnostic.slot0.present);
        assert!(!diagnostic.slot1.present);
    }

    #[test]
    fn diagnostic_builds_from_any_dual_protected_store() {
        #[derive(Eq, PartialEq)]
        struct TestSlot {
            generation: u64,
            valid: bool,
        }

        impl ProtectedGenerationSlot for TestSlot {
            fn generation(&self) -> u64 {
                self.generation
            }

            fn validates(&self) -> bool {
                self.valid
            }
        }

        struct TestStore {
            slot0: Option<TestSlot>,
            slot1: Option<TestSlot>,
        }

        impl DualProtectedCommitStore for TestStore {
            type Slot = TestSlot;

            fn slot0(&self) -> Option<&Self::Slot> {
                self.slot0.as_ref()
            }

            fn slot1(&self) -> Option<&Self::Slot> {
                self.slot1.as_ref()
            }
        }

        let diagnostic = CommitStoreDiagnostic::from_store(&TestStore {
            slot0: Some(TestSlot {
                generation: 8,
                valid: true,
            }),
            slot1: Some(TestSlot {
                generation: 9,
                valid: false,
            }),
        });

        assert_eq!(diagnostic.authoritative_generation, Some(8));
        assert!(diagnostic.slot0.valid);
        assert_eq!(diagnostic.slot1.generation, Some(9));
        assert!(!diagnostic.slot1.valid);
    }

    #[test]
    fn uninitialized_distinguishes_empty_from_corrupt() {
        let mut store = DualCommitStore::default();
        assert!(store.is_uninitialized());

        store.write_corrupt_inactive_slot(0, payload(1));

        assert!(!store.is_uninitialized());
    }

    #[test]
    fn commit_after_corrupt_slot_advances_from_prior_valid_generation() {
        let mut store = DualCommitStore::default();
        store.commit_payload(payload(1)).expect("first commit");
        store.write_corrupt_inactive_slot(1, payload(2));
        store.commit_payload(payload(3)).expect("third commit");

        let authoritative = store.authoritative().expect("authoritative");

        assert_eq!(authoritative.generation, 1);
        assert_eq!(authoritative.payload, payload(3));
    }
}
