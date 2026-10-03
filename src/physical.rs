use crate::hash::{FNV_OFFSET, fnv64};
use serde::{Deserialize, Serialize};

const COMMIT_MARKER: u64 = 0x4943_4D45_4D43_4F4D;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CommitSlotIndex {
    Slot0,
    Slot1,
}

impl CommitSlotIndex {
    const fn opposite(self) -> Self {
        match self {
            Self::Slot0 => Self::Slot1,
            Self::Slot1 => Self::Slot0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct AuthoritativeSlot<'slot> {
    index: CommitSlotIndex,
    record: &'slot CommittedGenerationBytes,
}

fn select_authoritative_slot<'slot>(
    slot0: Option<&'slot CommittedGenerationBytes>,
    slot1: Option<&'slot CommittedGenerationBytes>,
) -> Result<AuthoritativeSlot<'slot>, CommitRecoveryError> {
    let slot0_invalid = slot0.is_some_and(|slot| !slot.validates());
    let slot1_invalid = slot1.is_some_and(|slot| !slot.validates());
    if slot0_invalid || slot1_invalid {
        return Err(CommitRecoveryError::InvalidCommitSlots {
            slot0_invalid,
            slot1_invalid,
        });
    }

    let slot0 = slot0.map(|record| AuthoritativeSlot {
        index: CommitSlotIndex::Slot0,
        record,
    });
    let slot1 = slot1.map(|record| AuthoritativeSlot {
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
/// Committed ledger generation payload protected by a checksum.
///
/// This is an advanced low-level DTO for framework or stable-IO owners. Its
/// recovered bytes are untrusted until marker/checksum validation and ledger
/// decoding/integrity validation have both succeeded.
/// Binary serde formats encode the payload as a bounded byte string; human-readable
/// formats retain an array of bytes. Direct serde decoding is a DTO operation:
/// maintained durable readers additionally preflight CBOR before allocation.
///

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
    #[serde(with = "opaque_payload")]
    pub(crate) payload: Vec<u8>,
}

// The binary representation belongs to the persisted codec. Human-readable DTOs
// continue to round-trip byte arrays without accepting arrays in durable CBOR.
mod opaque_payload {
    use crate::constants::MAX_COMMITTED_PAYLOAD_BYTES;
    use serde::{Deserializer, Serialize, Serializer, de::Visitor};

    pub fn serialize<S: Serializer>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
        if bytes.len() > MAX_COMMITTED_PAYLOAD_BYTES {
            return Err(serde::ser::Error::custom(
                "ledger byte string exceeds payload bound",
            ));
        }
        if serializer.is_human_readable() {
            bytes.serialize(serializer)
        } else {
            serializer.serialize_bytes(bytes)
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        struct Bytes;
        impl Visitor<'_> for Bytes {
            type Value = Vec<u8>;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a bounded opaque ledger byte string")
            }

            fn visit_bytes<E: serde::de::Error>(self, bytes: &[u8]) -> Result<Self::Value, E> {
                if bytes.len() > MAX_COMMITTED_PAYLOAD_BYTES {
                    return Err(E::custom("ledger byte string exceeds payload bound"));
                }
                Ok(bytes.to_vec())
            }

            fn visit_byte_buf<E: serde::de::Error>(self, bytes: Vec<u8>) -> Result<Self::Value, E> {
                if bytes.len() > MAX_COMMITTED_PAYLOAD_BYTES {
                    return Err(E::custom("ledger byte string exceeds payload bound"));
                }
                Ok(bytes)
            }
        }
        if deserializer.is_human_readable() {
            return crate::cbor::deserialize_bounded_vec::<D, u8, MAX_COMMITTED_PAYLOAD_BYTES>(
                deserializer,
            );
        }
        deserializer.deserialize_byte_buf(Bytes)
    }
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
    /// The checksum is non-cryptographic and detects accidental corruption
    /// only.
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

///
/// DualCommitStore
///
/// Redundant commit store for encoded ledger generations.
///
/// This is an advanced low-level API for framework or stable-IO owners. Most
/// applications should recover, validate, and commit through the allocation
/// ledger flow rather than manipulating encoded physical commit slots directly.
///
/// Writers stage a complete generation record into the inactive slot. Readers
/// recover by selecting the highest-generation slot after every present slot
/// passes marker and checksum validation. Any present invalid slot fails
/// closed; recovery never rolls durable allocation history back to an older
/// generation.
///
/// In the default runtime both slots are serialized together inside one
/// `ic-stable-structures::Cell`; they are not independently atomic physical
/// writes. ICP message execution supplies atomic stable-memory commit and
/// rollback. The checksum is for accidental-corruption detection only. It is
/// not a cryptographic hash and does not provide adversarial tamper resistance.
///

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DualCommitStore {
    /// First commit slot.
    #[serde(deserialize_with = "crate::cbor::deserialize_present_option")]
    pub(crate) slot0: Option<CommittedGenerationBytes>,
    /// Second commit slot.
    #[serde(deserialize_with = "crate::cbor::deserialize_present_option")]
    pub(crate) slot1: Option<CommittedGenerationBytes>,
}

impl DualCommitStore {
    /// Return true when no commit slot has ever been written.
    #[must_use]
    pub const fn is_uninitialized(&self) -> bool {
        self.slot0.is_none() && self.slot1.is_none()
    }

    /// Borrow the first commit slot.
    ///
    /// Slot records are untrusted recovered state until recovery selects an
    /// authoritative generation.
    #[must_use]
    pub const fn slot0(&self) -> Option<&CommittedGenerationBytes> {
        self.slot0.as_ref()
    }

    /// Borrow the second commit slot.
    ///
    /// Slot records are untrusted recovered state until recovery selects an
    /// authoritative generation.
    #[must_use]
    pub const fn slot1(&self) -> Option<&CommittedGenerationBytes> {
        self.slot1.as_ref()
    }

    fn authoritative_slot(&self) -> Result<AuthoritativeSlot<'_>, CommitRecoveryError> {
        select_authoritative_slot(self.slot0(), self.slot1())
    }

    #[cfg(test)]
    fn inactive_slot_index(&self) -> CommitSlotIndex {
        match self.authoritative_slot() {
            Ok(authoritative) => authoritative.index.opposite(),
            Err(_) if self.slot0.is_none() => CommitSlotIndex::Slot0,
            Err(_) => CommitSlotIndex::Slot1,
        }
    }

    /// Return the authoritative committed record after validating present slots.
    pub fn authoritative(&self) -> Result<&CommittedGenerationBytes, CommitRecoveryError> {
        self.authoritative_slot()
            .map(|authoritative| authoritative.record)
    }

    pub(crate) fn authoritative_with_diagnostic(
        &self,
    ) -> (
        Result<&CommittedGenerationBytes, CommitRecoveryError>,
        CommitStoreDiagnostic,
    ) {
        let recovery = self.authoritative_slot();
        let diagnostic = CommitStoreDiagnostic::from_recovery(self, &recovery);
        (recovery.map(|slot| slot.record), diagnostic)
    }

    /// Build a read-only recovery diagnostic for the protected commit slots.
    #[must_use]
    pub fn diagnostic(&self) -> CommitStoreDiagnostic {
        CommitStoreDiagnostic::from_store(self)
    }

    /// Commit a new payload to the inactive slot.
    ///
    /// The returned record is the new authoritative in-memory slot. The owner
    /// remains responsible for persisting the enclosing store.
    pub fn commit_payload(
        &mut self,
        payload: Vec<u8>,
    ) -> Result<&CommittedGenerationBytes, CommitRecoveryError> {
        self.commit_payload_with_generation(None, payload)
    }

    /// Commit `payload` as an explicitly numbered physical generation.
    ///
    /// This is the low-level physical-slot primitive used by
    /// [`crate::LedgerCommitStore`]. Normal ledger commits should use
    /// [`crate::LedgerCommitStore::commit`] or [`crate::AllocationBootstrap`] so
    /// payloads are decoded, current-format checked, and integrity-validated
    /// before they can become authoritative.
    ///
    /// The commit-slot generation is checked against the recovered
    /// predecessor. This method does not inspect `payload`.
    pub fn commit_payload_at_generation(
        &mut self,
        generation: u64,
        payload: Vec<u8>,
    ) -> Result<&CommittedGenerationBytes, CommitRecoveryError> {
        self.commit_payload_with_generation(Some(generation), payload)
    }

    fn commit_payload_with_generation(
        &mut self,
        requested: Option<u64>,
        payload: Vec<u8>,
    ) -> Result<&CommittedGenerationBytes, CommitRecoveryError> {
        // Validate every predecessor once before mutation, and reuse its slot.
        let (index, generation) = match self.authoritative_slot() {
            Ok(authoritative) => {
                let expected = authoritative.record.generation.checked_add(1).ok_or(
                    CommitRecoveryError::GenerationOverflow {
                        generation: authoritative.record.generation,
                    },
                )?;
                let generation = requested.unwrap_or(expected);
                if generation != expected {
                    return Err(CommitRecoveryError::UnexpectedGeneration {
                        expected,
                        actual: generation,
                    });
                }
                (authoritative.index.opposite(), generation)
            }
            Err(CommitRecoveryError::NoValidGeneration) if self.is_uninitialized() => {
                (CommitSlotIndex::Slot0, requested.unwrap_or(0))
            }
            Err(err) => return Err(err),
        };
        let slot = match index {
            CommitSlotIndex::Slot0 => &mut self.slot0,
            CommitSlotIndex::Slot1 => &mut self.slot1,
        };
        // Construction computes the current marker/checksum. No second scan of
        // the unchanged predecessor or newly constructed payload is needed.
        Ok(slot.insert(CommittedGenerationBytes::new(generation, payload)))
    }

    /// Simulate corruption in the inactive slot.
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

///
/// CommitStoreDiagnostic
///
/// Read-only diagnostic summary of protected commit recovery state.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CommitStoreDiagnostic {
    /// First physical commit slot diagnostic.
    pub slot0: CommitSlotDiagnostic,
    /// Second physical commit slot diagnostic.
    pub slot1: CommitSlotDiagnostic,
    /// Authoritative generation or the recovery error that prevented selection.
    pub recovery: Result<u64, CommitRecoveryError>,
}

impl CommitStoreDiagnostic {
    /// Build a read-only recovery diagnostic from a dual commit store.
    #[must_use]
    pub fn from_store(store: &DualCommitStore) -> Self {
        let recovery = store.authoritative_slot();
        Self::from_recovery(store, &recovery)
    }

    fn from_recovery(
        store: &DualCommitStore,
        recovery: &Result<AuthoritativeSlot<'_>, CommitRecoveryError>,
    ) -> Self {
        // Selection validates every present slot before considering generations.
        // Reuse that evidence rather than scanning their payloads again.
        let (slot0_invalid, slot1_invalid) = match recovery {
            Err(CommitRecoveryError::InvalidCommitSlots {
                slot0_invalid,
                slot1_invalid,
            }) => (*slot0_invalid, *slot1_invalid),
            _ => (false, false),
        };
        Self {
            slot0: CommitSlotDiagnostic::from_slot(store.slot0(), slot0_invalid),
            slot1: CommitSlotDiagnostic::from_slot(store.slot1(), slot1_invalid),
            recovery: recovery.map(|slot| slot.record.generation()),
        }
    }
}

///
/// CommitSlotDiagnostic
///
/// Read-only diagnostic summary for one protected commit slot.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub enum CommitSlotDiagnostic {
    /// No physical slot record is present.
    Empty,
    /// A present slot passed marker and checksum validation.
    Valid {
        /// Generation encoded by the valid slot.
        generation: u64,
    },
    /// A present slot failed marker or checksum validation.
    Invalid {
        /// Generation encoded by the invalid slot.
        generation: u64,
    },
}

impl CommitSlotDiagnostic {
    const fn from_slot(slot: Option<&CommittedGenerationBytes>, invalid: bool) -> Self {
        match slot {
            Some(record) if !invalid => Self::Valid {
                generation: record.generation(),
            },
            Some(record) => Self::Invalid {
                generation: record.generation(),
            },
            None => Self::Empty,
        }
    }
}

///
/// CommitRecoveryError
///
/// Protected commit recovery failure.
///

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Deserialize, Eq, thiserror::Error, PartialEq, Serialize)]
pub enum CommitRecoveryError {
    /// No committed slot is present.
    #[error("no committed ledger generation is present")]
    NoValidGeneration,
    /// At least one present commit slot failed marker/checksum validation.
    #[error(
        "present commit slot validation failed (slot0_invalid={slot0_invalid}, slot1_invalid={slot1_invalid})"
    )]
    InvalidCommitSlots {
        /// Whether the first present slot failed validation.
        slot0_invalid: bool,
        /// Whether the second present slot failed validation.
        slot1_invalid: bool,
    },
    /// Both commit slots validated at the same generation but contained different bytes.
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
    let header = [
        generation.generation.to_le_bytes(),
        generation.commit_marker.to_le_bytes(),
        (generation.payload.len() as u64).to_le_bytes(),
    ];
    let hash = fnv64(FNV_OFFSET, header.as_flattened());
    fnv64(hash, &generation.payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn automatic_and_explicit_commits_preserve_rejected_predecessors() {
        let mut store = DualCommitStore::default();
        // An explicit first physical generation can represent an imported baseline.
        assert_eq!(
            store
                .commit_payload_at_generation(7, payload(1))
                .unwrap()
                .generation(),
            7
        );
        let before = store.clone();
        assert_eq!(
            store.commit_payload_at_generation(9, payload(2)),
            Err(CommitRecoveryError::UnexpectedGeneration {
                expected: 8,
                actual: 9
            })
        );
        assert_eq!(store, before);
        assert_eq!(store.commit_payload(payload(2)).unwrap().generation(), 8);
        assert_eq!(store.slot0().unwrap().generation(), 7);
        assert_eq!(store.slot1().unwrap().generation(), 8);
        assert_eq!(
            store
                .commit_payload_at_generation(9, payload(3))
                .unwrap()
                .generation(),
            9
        );
        assert_eq!(store.slot0().unwrap().generation(), 9);
        assert_eq!(store.slot1().unwrap().generation(), 8);

        store.slot1.as_mut().unwrap().checksum ^= 1;
        let corrupt = store.clone();
        for result in [
            store
                .commit_payload(payload(4))
                .map(CommittedGenerationBytes::generation),
            store
                .commit_payload_at_generation(10, payload(4))
                .map(CommittedGenerationBytes::generation),
        ] {
            assert_eq!(
                result,
                Err(CommitRecoveryError::InvalidCommitSlots {
                    slot0_invalid: false,
                    slot1_invalid: true,
                })
            );
        }
        assert_eq!(store, corrupt);
    }

    #[test]
    fn payload_uses_binary_bytes_and_human_readable_arrays() {
        let record = CommittedGenerationBytes::new(7, vec![0, 24, 255]);
        let bytes = crate::test_cbor::to_vec(&record).unwrap();
        let value: ciborium::Value = crate::cbor::from_slice_exact(&bytes).unwrap();
        let ciborium::Value::Map(mut fields) = value else {
            panic!("record map")
        };
        let (_, payload) = fields
            .iter_mut()
            .find(|(key, _)| key.as_text() == Some("payload"))
            .unwrap();
        assert_eq!(*payload, ciborium::Value::Bytes(vec![0, 24, 255]));
        // Removed durable integer-array representations must reject.
        *payload = ciborium::Value::Array(vec![0.into(), 24.into(), 255.into()]);
        let removed = crate::test_cbor::to_vec(&ciborium::Value::Map(fields)).unwrap();
        assert!(crate::cbor::from_slice_exact::<CommittedGenerationBytes>(&removed).is_err());
        assert_eq!(
            crate::cbor::from_slice_exact::<CommittedGenerationBytes>(&bytes).unwrap(),
            record
        );
        let json = serde_json::to_value(&record).unwrap();
        assert_eq!(json["payload"], serde_json::json!([0, 24, 255]));
        assert_eq!(
            serde_json::from_value::<CommittedGenerationBytes>(json).unwrap(),
            record
        );
    }

    #[test]
    fn maximum_payload_writer_reader_agree() {
        let record = CommittedGenerationBytes::new(
            0,
            vec![0; crate::constants::MAX_COMMITTED_PAYLOAD_BYTES],
        );
        let store = DualCommitStore {
            slot0: Some(record.clone()),
            slot1: Some(record),
        };
        let bytes = crate::test_cbor::to_vec(&store).unwrap();
        assert!(bytes.len() <= crate::constants::MAX_LEDGER_RECORD_BYTES);
        let decoded: DualCommitStore = crate::cbor::from_slice_exact(&bytes).unwrap();
        assert_eq!(decoded, store);
        assert!(decoded.authoritative().is_ok());
        let oversized = CommittedGenerationBytes::new(
            0,
            vec![0; crate::constants::MAX_COMMITTED_PAYLOAD_BYTES + 1],
        );
        assert!(crate::test_cbor::to_vec(&oversized).is_err());
    }

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
    fn corrupt_newer_slot_fails_closed() {
        let mut store = DualCommitStore::default();
        store.commit_payload(payload(1)).expect("first commit");
        store.write_corrupt_inactive_slot(1, payload(2));

        let err = store.authoritative().expect_err("corrupt slot");

        assert_eq!(
            err,
            CommitRecoveryError::InvalidCommitSlots {
                slot0_invalid: false,
                slot1_invalid: true,
            }
        );
    }

    #[test]
    fn two_invalid_commit_slots_fail_closed() {
        let mut store = DualCommitStore::default();
        store.write_corrupt_inactive_slot(0, payload(1));
        store.write_corrupt_inactive_slot(1, payload(2));

        let err = store.authoritative().expect_err("invalid slots");

        assert_eq!(
            err,
            CommitRecoveryError::InvalidCommitSlots {
                slot0_invalid: true,
                slot1_invalid: true,
            }
        );
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
    fn diagnostic_reports_corrupt_slots_without_an_authoritative_generation() {
        let mut store = DualCommitStore::default();
        store.commit_payload(payload(1)).expect("first commit");
        store.write_corrupt_inactive_slot(1, payload(2));

        let diagnostic = store.diagnostic();

        assert_eq!(
            diagnostic.recovery,
            Err(CommitRecoveryError::InvalidCommitSlots {
                slot0_invalid: false,
                slot1_invalid: true,
            })
        );
        assert_eq!(
            diagnostic.slot0,
            CommitSlotDiagnostic::Valid { generation: 0 }
        );
        assert_eq!(
            diagnostic.slot1,
            CommitSlotDiagnostic::Invalid { generation: 1 }
        );
        let bytes = crate::test_cbor::to_vec(&diagnostic).expect("diagnostic bytes");
        let decoded: CommitStoreDiagnostic =
            crate::test_cbor::from_slice(&bytes).expect("diagnostic round trip");
        assert_eq!(decoded, diagnostic);
    }

    #[test]
    fn diagnostic_selection_covers_valid_ties_and_corruption_on_either_slot() {
        let mut store = DualCommitStore {
            slot0: None,
            slot1: Some(CommittedGenerationBytes::new(8, payload(2))),
        };
        assert_eq!(store.diagnostic().recovery, Ok(8));
        assert_eq!(store.diagnostic().slot0, CommitSlotDiagnostic::Empty);
        store.slot0 = Some(CommittedGenerationBytes::new(7, payload(1)));
        assert_eq!(store.diagnostic().recovery, Ok(8));
        store.slot1.clone_from(&store.slot0);
        assert_eq!(store.diagnostic().recovery, Ok(7));

        store.slot1 = Some(CommittedGenerationBytes::new(7, payload(2)));
        let diagnostic = store.diagnostic();
        assert_eq!(
            diagnostic.recovery,
            Err(CommitRecoveryError::AmbiguousGeneration { generation: 7 })
        );
        assert_eq!(
            diagnostic.slot0,
            CommitSlotDiagnostic::Valid { generation: 7 }
        );
        assert_eq!(
            diagnostic.slot1,
            CommitSlotDiagnostic::Valid { generation: 7 }
        );

        store.slot0.as_mut().unwrap().checksum ^= 1;
        let diagnostic = store.diagnostic();
        assert_eq!(
            diagnostic.recovery,
            Err(CommitRecoveryError::InvalidCommitSlots {
                slot0_invalid: true,
                slot1_invalid: false,
            })
        );
        assert_eq!(
            diagnostic.slot0,
            CommitSlotDiagnostic::Invalid { generation: 7 }
        );
        assert_eq!(
            diagnostic.slot1,
            CommitSlotDiagnostic::Valid { generation: 7 }
        );

        store.slot1.as_mut().unwrap().commit_marker = 0;
        let diagnostic = store.diagnostic();
        assert_eq!(
            diagnostic.recovery,
            Err(CommitRecoveryError::InvalidCommitSlots {
                slot0_invalid: true,
                slot1_invalid: true,
            })
        );
        assert_eq!(
            diagnostic.slot0,
            CommitSlotDiagnostic::Invalid { generation: 7 }
        );
        assert_eq!(
            diagnostic.slot1,
            CommitSlotDiagnostic::Invalid { generation: 7 }
        );
    }

    #[test]
    fn diagnostic_reports_no_valid_generation_for_empty_store() {
        let diagnostic = DualCommitStore::default().diagnostic();

        assert_eq!(
            diagnostic.recovery,
            Err(CommitRecoveryError::NoValidGeneration)
        );
        assert_eq!(diagnostic.slot0, CommitSlotDiagnostic::Empty);
        assert_eq!(diagnostic.slot1, CommitSlotDiagnostic::Empty);
    }

    #[test]
    fn uninitialized_distinguishes_empty_from_corrupt() {
        let mut store = DualCommitStore::default();
        assert!(store.is_uninitialized());

        store.write_corrupt_inactive_slot(0, payload(1));

        assert!(!store.is_uninitialized());
    }

    #[test]
    fn commit_after_corrupt_slot_fails_closed() {
        let mut store = DualCommitStore::default();
        store.commit_payload(payload(1)).expect("first commit");
        store.write_corrupt_inactive_slot(1, payload(2));

        let err = store
            .commit_payload(payload(3))
            .expect_err("corrupt history must not be overwritten");

        assert_eq!(
            err,
            CommitRecoveryError::InvalidCommitSlots {
                slot0_invalid: false,
                slot1_invalid: true,
            }
        );
    }
}
