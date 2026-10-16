use crate::hash::{FNV_OFFSET, fnv64};
use serde::{Deserialize, Serialize};

// ICMDIRCT identifies the direct-CBOR single-snapshot physical contract.
const COMMIT_MARKER: u64 = 0x4943_4D44_4952_4354;

///
/// CommittedGenerationBytes
///
/// Committed ledger generation payload protected by a checksum.
///
/// This private DTO is the current physical snapshot representation. Its
/// recovered bytes are untrusted until marker/checksum validation and ledger
/// decoding/integrity validation have both succeeded.
/// The payload encodes as a bounded CBOR byte string. Diagnostic JSON output
/// is permitted, but JSON arrays are not a snapshot input format. Direct serde
/// decoding is a DTO operation; maintained durable readers additionally
/// preflight CBOR before allocation.
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

// The persisted codec admits opaque byte strings; fixtures use this same shape.
mod opaque_payload {
    use crate::constants::MAX_COMMITTED_PAYLOAD_BYTES;
    use serde::{Deserializer, Serializer, de::Visitor};

    pub fn serialize<S: Serializer>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
        if bytes.len() > MAX_COMMITTED_PAYLOAD_BYTES {
            return Err(serde::ser::Error::custom(
                "ledger byte string exceeds payload bound",
            ));
        }
        serializer.serialize_bytes(bytes)
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
    #[cfg(test)]
    pub const fn commit_marker(&self) -> u64 {
        self.commit_marker
    }

    /// Return the checksum over the generation, marker, and payload bytes.
    ///
    /// The checksum is non-cryptographic and detects accidental corruption
    /// only.
    #[must_use]
    #[cfg(test)]
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
/// SnapshotCommitStore
///
/// One checksummed current ledger snapshot. The enclosing stable Cell is
/// published synchronously before allocation authority. IC message execution
/// owns atomic stable-memory commit and rollback; native backends must supply
/// their own transaction boundary. Corrupt current bytes never initialize or
/// accept another generation.
///

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotCommitStore {
    #[serde(deserialize_with = "crate::cbor::deserialize_present_option")]
    pub(crate) current: Option<CommittedGenerationBytes>,
}

impl SnapshotCommitStore {
    /// Borrow untrusted bytes only for encoding and capacity planning.
    pub const fn current(&self) -> Option<&CommittedGenerationBytes> {
        self.current.as_ref()
    }

    /// Recover the current snapshot only after marker and checksum validation.
    pub fn authoritative(&self) -> Result<&CommittedGenerationBytes, CommitRecoveryError> {
        let current = self
            .current()
            .ok_or(CommitRecoveryError::NoValidGeneration)?;
        if current.commit_marker != COMMIT_MARKER {
            return Err(CommitRecoveryError::UnsupportedFormat {
                marker: current.commit_marker,
            });
        }
        if !current.validates() {
            return Err(CommitRecoveryError::InvalidSnapshot {
                generation: current.generation(),
            });
        }
        Ok(current)
    }

    pub(crate) fn authoritative_with_diagnostic(
        &self,
    ) -> (
        Result<&CommittedGenerationBytes, CommitRecoveryError>,
        CommitSnapshotDiagnostic,
    ) {
        let recovery = self.authoritative();
        let current = match self.current() {
            None => CommitSnapshotDiagnostic::Empty,
            Some(record) if recovery.is_ok() => CommitSnapshotDiagnostic::Valid {
                generation: record.generation(),
            },
            Some(record) => CommitSnapshotDiagnostic::Invalid {
                generation: record.generation(),
            },
        };
        (recovery, current)
    }

    /// Report the same checked current record without decoding logical ownership.
    pub fn diagnostic(&self) -> CommitSnapshotDiagnostic {
        self.authoritative_with_diagnostic().1
    }

    /// Replace the current snapshot after checking its successor and byte bound.
    /// The owner must persist the enclosing record before publishing authority.
    pub fn commit_payload_at_generation(
        &mut self,
        generation: u64,
        payload: Vec<u8>,
    ) -> Result<&CommittedGenerationBytes, CommitRecoveryError> {
        if payload.len() > crate::constants::MAX_COMMITTED_PAYLOAD_BYTES {
            return Err(CommitRecoveryError::PayloadTooLarge {
                len: payload.len(),
                limit: crate::constants::MAX_COMMITTED_PAYLOAD_BYTES,
            });
        }
        match self.authoritative() {
            Ok(prior) => {
                let prior_generation = prior.generation();
                let expected = prior_generation.checked_add(1).ok_or(
                    CommitRecoveryError::GenerationOverflow {
                        generation: prior_generation,
                    },
                )?;
                if generation != expected {
                    return Err(CommitRecoveryError::UnexpectedGeneration {
                        expected,
                        actual: generation,
                    });
                }
            }
            // The current field was explicitly present and empty, never corrupt.
            Err(CommitRecoveryError::NoValidGeneration) => (),
            Err(error) => return Err(error),
        }
        Ok(self
            .current
            .insert(CommittedGenerationBytes::new(generation, payload)))
    }

    #[cfg(test)]
    pub fn write_corrupt_snapshot(&mut self, generation: u64, payload: Vec<u8>) {
        let mut corrupt = CommittedGenerationBytes::new(generation, payload);
        corrupt.checksum ^= 1;
        self.current = Some(corrupt);
    }
}

///
/// CommitSnapshotDiagnostic
///
/// Read-only marker/checksum status of the one current snapshot.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum CommitSnapshotDiagnostic {
    /// No current snapshot has been committed.
    Empty,
    /// The current snapshot passed marker/checksum validation.
    Valid {
        /// Current generation.
        generation: u64,
    },
    /// The current snapshot failed marker/checksum validation.
    Invalid {
        /// Untrusted generation observed in the rejected snapshot.
        generation: u64,
    },
}

///
/// CommitRecoveryError
///
/// Current physical snapshot recovery or successor admission failure.
///

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Deserialize, Eq, thiserror::Error, PartialEq, Serialize)]
pub enum CommitRecoveryError {
    /// Candidate bytes exceed the bounded opaque payload ceiling.
    #[error("committed payload length {len} exceeds byte limit {limit}")]
    PayloadTooLarge {
        /// Candidate byte length.
        len: usize,
        /// Maximum admitted byte length.
        limit: usize,
    },
    /// The current snapshot is explicitly absent.
    #[error("no committed ledger generation is present")]
    NoValidGeneration,
    /// The snapshot marker does not identify the current direct-CBOR format.
    #[error("unsupported physical ledger format marker {marker:#x}")]
    UnsupportedFormat { marker: u64 },
    /// A present snapshot failed marker/checksum validation.
    #[error("current ledger snapshot validation failed at generation {generation}")]
    InvalidSnapshot {
        /// Untrusted generation observed in the rejected snapshot.
        generation: u64,
    },
    /// The current generation cannot advance.
    #[error("committed ledger generation {generation} cannot be advanced without overflow")]
    GenerationOverflow {
        /// Last checked generation.
        generation: u64,
    },
    /// A commit requested a generation other than the checked successor.
    #[error("expected committed ledger generation {expected}, got {actual}")]
    UnexpectedGeneration {
        /// Required successor.
        expected: u64,
        /// Requested generation.
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
    fn rejected_successors_and_oversized_payloads_preserve_current_and_allow_retry() {
        let limit = crate::constants::MAX_COMMITTED_PAYLOAD_BYTES;
        for populated in [false, true] {
            let mut store = SnapshotCommitStore::default();
            if populated {
                store.commit_payload_at_generation(7, vec![1]).unwrap();
            }
            let before = store.clone();
            let generation = if populated { 8 } else { 0 };
            assert_eq!(
                store.commit_payload_at_generation(generation, vec![0; limit + 1]),
                Err(CommitRecoveryError::PayloadTooLarge {
                    len: limit + 1,
                    limit
                })
            );
            assert_eq!(store, before);
            if populated {
                assert_eq!(
                    store.commit_payload_at_generation(9, vec![2]),
                    Err(CommitRecoveryError::UnexpectedGeneration {
                        expected: 8,
                        actual: 9
                    })
                );
                assert_eq!(store, before);
            }
            store
                .commit_payload_at_generation(generation, vec![3])
                .unwrap();
            assert_eq!(store.authoritative().unwrap().generation(), generation);
            let bytes = crate::test_cbor::to_vec(&store).unwrap();
            assert_eq!(
                crate::cbor::from_slice_exact::<SnapshotCommitStore>(&bytes).unwrap(),
                store
            );
        }
    }

    #[test]
    fn commits_replace_the_snapshot_and_diagnostics_share_recovery() {
        let mut store = SnapshotCommitStore::default();
        assert_eq!(store.diagnostic(), CommitSnapshotDiagnostic::Empty);
        for generation in 0..4 {
            let payload = vec![u8::try_from(generation).unwrap(); 4];
            store
                .commit_payload_at_generation(generation, payload.clone())
                .unwrap();
            let record = store.current().unwrap();
            assert_eq!(record.payload(), payload);
            assert_eq!(record.commit_marker(), COMMIT_MARKER);
            assert_eq!(record.checksum(), generation_checksum(record));
            let diagnostic = store.diagnostic();
            assert_eq!(diagnostic, CommitSnapshotDiagnostic::Valid { generation });
            let bytes = crate::test_cbor::to_vec(&diagnostic).unwrap();
            assert_eq!(
                crate::test_cbor::from_slice::<CommitSnapshotDiagnostic>(&bytes).unwrap(),
                diagnostic
            );
        }
    }

    #[test]
    fn corrupt_current_fails_recovery_and_commit_without_mutation() {
        for field in ["checksum", "marker", "payload", "generation"] {
            let mut store = SnapshotCommitStore::default();
            store
                .commit_payload_at_generation(7, vec![1, 2, 3])
                .unwrap();
            let current = store.current.as_mut().unwrap();
            match field {
                "checksum" => current.checksum ^= 1,
                "marker" => {
                    current.commit_marker ^= 1;
                    current.checksum = generation_checksum(current);
                }
                "payload" => current.payload[0] ^= 1,
                _ => current.generation ^= 1,
            }
            let generation = current.generation();
            let expected = if field == "marker" {
                CommitRecoveryError::UnsupportedFormat {
                    marker: current.commit_marker,
                }
            } else {
                CommitRecoveryError::InvalidSnapshot { generation }
            };
            let before = store.clone();
            assert_eq!(store.authoritative(), Err(expected));
            assert_eq!(
                store.diagnostic(),
                CommitSnapshotDiagnostic::Invalid { generation }
            );
            assert_eq!(
                store.commit_payload_at_generation(8, vec![4]),
                Err(expected)
            );
            assert_eq!(store, before);
        }
    }

    #[test]
    fn physical_generation_overflow_preserves_current() {
        let mut store = SnapshotCommitStore::default();
        store
            .commit_payload_at_generation(u64::MAX, vec![1])
            .unwrap();
        let before = store.clone();
        assert_eq!(
            store.commit_payload_at_generation(0, vec![2]),
            Err(CommitRecoveryError::GenerationOverflow {
                generation: u64::MAX
            })
        );
        assert_eq!(store, before);
    }

    #[test]
    fn maximum_payload_writer_reader_agree() {
        let mut store = SnapshotCommitStore::default();
        store
            .commit_payload_at_generation(0, vec![0; crate::constants::MAX_COMMITTED_PAYLOAD_BYTES])
            .unwrap();
        let bytes = crate::test_cbor::to_vec(&store).unwrap();
        assert!(bytes.len() <= crate::constants::MAX_LEDGER_RECORD_BYTES);
        let decoded: SnapshotCommitStore = crate::cbor::from_slice_exact(&bytes).unwrap();
        assert_eq!(decoded, store);
        assert!(decoded.authoritative().is_ok());
        let oversized = CommittedGenerationBytes::new(
            0,
            vec![0; crate::constants::MAX_COMMITTED_PAYLOAD_BYTES + 1],
        );
        assert!(crate::test_cbor::to_vec(&oversized).is_err());
    }

    #[test]
    fn payload_uses_bounded_binary_bytes() {
        let record = CommittedGenerationBytes::new(7, vec![0, 24, 255]);
        let bytes = crate::test_cbor::to_vec(&record).unwrap();
        let value: ciborium::Value = crate::cbor::from_slice_exact(&bytes).unwrap();
        let ciborium::Value::Map(fields) = value else {
            panic!("record map")
        };
        let (_, payload) = fields
            .iter()
            .find(|(key, _)| key.as_text() == Some("payload"))
            .unwrap();
        assert_eq!(*payload, ciborium::Value::Bytes(vec![0, 24, 255]));
        assert_eq!(
            crate::cbor::from_slice_exact::<CommittedGenerationBytes>(&bytes).unwrap(),
            record
        );
    }
}
