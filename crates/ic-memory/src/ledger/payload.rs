const LEDGER_PAYLOAD_MAGIC: &[u8; 8] = b"ICMEMLED";
const LEDGER_PAYLOAD_FORMAT_MARKER: &[u8; 4] = b"ICMS";
/// Current durable ledger payload format version.
pub const LEDGER_PAYLOAD_FORMAT_VERSION: u32 = 1;
use crate::constants::LEDGER_PAYLOAD_HEADER_LEN;

///
/// LedgerPayloadEnvelope
///
/// Logical ledger payload envelope embedded inside one physically committed
/// generation. This layer is decoded after physical dual-slot recovery selects
/// a committed generation and before any allocation-ledger DTO is decoded.
///
/// This is an advanced protocol byte wrapper, not an authority token. Decoding
/// an envelope only classifies the logical payload; authority is established
/// later when [`crate::LedgerCommitStore`] routes the payload, checks
/// the current ledger format, validates committed ledger integrity, and returns
/// [`crate::RecoveredLedger`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LedgerPayloadEnvelope {
    payload: Vec<u8>,
}

impl LedgerPayloadEnvelope {
    /// Wrap a current logical ledger payload.
    #[must_use]
    pub const fn current(payload: Vec<u8>) -> Self {
        Self { payload }
    }

    /// Manually encode the logical payload envelope.
    ///
    /// # Panics
    ///
    /// Panics if the payload exceeds the current ledger byte ceiling. Use
    /// `try_encode` for typed errors.
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        self.try_encode()
            .expect("payload exceeds the ledger byte ceiling")
    }

    /// Try to encode the logical payload envelope.
    pub fn try_encode(&self) -> Result<Vec<u8>, LedgerPayloadEnvelopeError> {
        if self.payload.len() > crate::constants::MAX_LEDGER_BYTES {
            return Err(LedgerPayloadEnvelopeError::PayloadTooLarge {
                len: self.payload.len() as u64,
            });
        }
        // The byte ceiling bounds both the total usize length and the u64 header.
        let total_len = LEDGER_PAYLOAD_HEADER_LEN + self.payload.len();
        let mut bytes = Vec::with_capacity(total_len);
        bytes.extend_from_slice(&encoded_header(self.payload.len()));
        bytes.extend_from_slice(&self.payload);
        Ok(bytes)
    }

    // Serialize directly into the final envelope buffer. The commit boundary
    // retains its ledger-byte integrity error before any physical mutation.
    pub(super) fn encode_ledger(
        ledger: &super::AllocationLedger,
    ) -> Result<Vec<u8>, super::LedgerIntegrityError> {
        let mut writer = LedgerWriter(vec![0; LEDGER_PAYLOAD_HEADER_LEN]);
        match ciborium::into_writer(ledger, &mut writer) {
            Ok(()) => (),
            Err(ciborium::ser::Error::Io(err)) if err.kind() == std::io::ErrorKind::WriteZero => {
                return Err(super::LedgerIntegrityError::LimitExceeded {
                    resource: "ledger bytes",
                    limit: crate::constants::MAX_LEDGER_BYTES,
                });
            }
            // Concrete derived serializers have no other recoverable failures.
            Err(err) => panic!("allocation ledger serialization failed: {err}"),
        }
        let mut bytes = writer.0;
        let payload_len = bytes.len() - LEDGER_PAYLOAD_HEADER_LEN;
        bytes[..LEDGER_PAYLOAD_HEADER_LEN].copy_from_slice(&encoded_header(payload_len));
        Ok(bytes)
    }

    /// Manually decode the logical payload envelope.
    pub fn decode(bytes: &[u8]) -> Result<Self, LedgerPayloadEnvelopeError> {
        Ok(Self {
            payload: Self::decode_payload(bytes)?.to_vec(),
        })
    }

    // Recovery already owns the committed bytes. Validate the same envelope
    // without copying its bounded payload before logical ledger decoding.
    pub(super) fn decode_payload(bytes: &[u8]) -> Result<&[u8], LedgerPayloadEnvelopeError> {
        let Some(magic) = bytes.get(0..8).and_then(|bytes| bytes.try_into().ok()) else {
            return Err(LedgerPayloadEnvelopeError::Truncated {
                actual: bytes.len(),
                minimum: LEDGER_PAYLOAD_HEADER_LEN,
            });
        };
        if &magic != LEDGER_PAYLOAD_MAGIC {
            return Err(LedgerPayloadEnvelopeError::BadMagic { found: magic });
        }

        let Some(format_marker) = bytes.get(8..12).and_then(|bytes| bytes.try_into().ok()) else {
            return Err(LedgerPayloadEnvelopeError::Truncated {
                actual: bytes.len(),
                minimum: LEDGER_PAYLOAD_HEADER_LEN,
            });
        };
        if &format_marker != LEDGER_PAYLOAD_FORMAT_MARKER {
            return Err(LedgerPayloadEnvelopeError::UnsupportedFormat {
                marker: format_marker,
                version: None,
            });
        }

        let Some(format_version) = bytes.get(12..16).and_then(|bytes| bytes.try_into().ok()) else {
            return Err(LedgerPayloadEnvelopeError::Truncated {
                actual: bytes.len(),
                minimum: LEDGER_PAYLOAD_HEADER_LEN,
            });
        };
        let format_version = u32::from_le_bytes(format_version);
        if format_version != LEDGER_PAYLOAD_FORMAT_VERSION {
            return Err(LedgerPayloadEnvelopeError::UnsupportedFormat {
                marker: format_marker,
                version: Some(format_version),
            });
        }

        let Some(payload_len) = bytes.get(16..24).and_then(|bytes| bytes.try_into().ok()) else {
            return Err(LedgerPayloadEnvelopeError::Truncated {
                actual: bytes.len(),
                minimum: LEDGER_PAYLOAD_HEADER_LEN,
            });
        };
        let payload_len = u64::from_le_bytes(payload_len);
        let payload_len = usize::try_from(payload_len)
            .map_err(|_| LedgerPayloadEnvelopeError::PayloadTooLarge { len: payload_len })?;
        if payload_len > crate::constants::MAX_LEDGER_BYTES {
            return Err(LedgerPayloadEnvelopeError::PayloadTooLarge {
                len: payload_len as u64,
            });
        }
        let expected_len = LEDGER_PAYLOAD_HEADER_LEN + payload_len;
        if bytes.len() != expected_len {
            return Err(LedgerPayloadEnvelopeError::LengthMismatch {
                declared: payload_len,
                actual: bytes.len().saturating_sub(LEDGER_PAYLOAD_HEADER_LEN),
            });
        }

        Ok(&bytes[LEDGER_PAYLOAD_HEADER_LEN..])
    }

    /// Borrow the logical ledger payload bytes.
    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

// Keep a single serialization pass, but stop before an oversized write or a
// geometric capacity reservation can exceed the final envelope's byte ceiling.
// Partial output is local and discarded on refusal, before physical mutation.
struct LedgerWriter(Vec<u8>);

impl std::io::Write for LedgerWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.write_all(bytes)?;
        Ok(bytes.len())
    }

    // Ciborium uses write_all; this buffer accepts an entire slice or refuses
    // it, so it needs no partial-write retry loop from the default adapter.
    #[inline]
    fn write_all(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        let limit = crate::constants::MAX_COMMITTED_PAYLOAD_BYTES;
        if bytes.len() > limit - self.0.len() {
            return Err(std::io::ErrorKind::WriteZero.into());
        }
        let required = self.0.len() + bytes.len();
        if required > self.0.capacity() {
            let capacity = self.0.capacity().saturating_mul(2).max(required).min(limit);
            self.0.reserve_exact(capacity - self.0.len());
        }
        self.0.extend_from_slice(bytes);
        Ok(())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

// Both writers establish the payload bound before constructing this header.
fn encoded_header(payload_len: usize) -> [u8; LEDGER_PAYLOAD_HEADER_LEN] {
    let mut header = [0; LEDGER_PAYLOAD_HEADER_LEN];
    header[..8].copy_from_slice(LEDGER_PAYLOAD_MAGIC);
    header[8..12].copy_from_slice(LEDGER_PAYLOAD_FORMAT_MARKER);
    header[12..16].copy_from_slice(&LEDGER_PAYLOAD_FORMAT_VERSION.to_le_bytes());
    header[16..24].copy_from_slice(&(payload_len as u64).to_le_bytes());
    header
}

///
/// LedgerPayloadEnvelopeError
///
/// Logical payload envelope could not be classified before ledger decode.
///

#[non_exhaustive]
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum LedgerPayloadEnvelopeError {
    /// Not enough bytes for an envelope header.
    #[error("ledger payload envelope is truncated: {actual} bytes, need at least {minimum}")]
    Truncated {
        /// Bytes present.
        actual: usize,
        /// Minimum bytes required.
        minimum: usize,
    },
    /// Magic bytes do not identify an `ic-memory` ledger payload.
    #[error("ledger payload envelope has bad magic {found:?}")]
    BadMagic {
        /// Magic bytes found.
        found: [u8; 8],
    },
    /// The payload belongs to the `ic-memory` ledger family but does not carry
    /// the current format discriminator.
    #[error("unsupported ic-memory ledger payload format (marker={marker:?}, version={version:?})")]
    UnsupportedFormat {
        /// Format marker found after the ledger-family magic.
        marker: [u8; 4],
        /// Format version when the current marker was present.
        version: Option<u32>,
    },
    /// Payload length exceeds the ledger byte ceiling or cannot fit in this
    /// platform's address space.
    #[error("ledger payload envelope length {len} is too large")]
    PayloadTooLarge {
        /// Declared payload length.
        len: u64,
    },
    /// Declared payload length does not match the bytes present.
    #[error("ledger payload envelope declared {declared} payload bytes but contained {actual}")]
    LengthMismatch {
        /// Declared payload length.
        declared: usize,
        /// Actual payload length.
        actual: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ledger_writer_admits_exact_ceiling_and_refuses_without_partial_append() {
        use std::io::Write;

        let mut writer = LedgerWriter(vec![0; LEDGER_PAYLOAD_HEADER_LEN]);
        let chunk = [42; 4096];
        for _ in 0..crate::constants::MAX_LEDGER_BYTES / chunk.len() {
            writer.write_all(&chunk).unwrap();
        }
        assert_eq!(
            writer.0.len(),
            crate::constants::MAX_COMMITTED_PAYLOAD_BYTES
        );
        writer.write_all(&[]).unwrap();
        assert_eq!(
            writer.write(&[42]).unwrap_err().kind(),
            std::io::ErrorKind::WriteZero
        );
        assert_eq!(
            writer.0.len(),
            crate::constants::MAX_COMMITTED_PAYLOAD_BYTES
        );
        assert!(
            writer.0[LEDGER_PAYLOAD_HEADER_LEN..]
                .iter()
                .all(|&byte| byte == 42)
        );

        let mut writer = LedgerWriter(vec![0; LEDGER_PAYLOAD_HEADER_LEN]);
        let before = writer.0.clone();
        assert!(
            writer
                .write(&vec![42; crate::constants::MAX_LEDGER_BYTES + 1])
                .is_err()
        );
        assert_eq!(writer.0, before);
    }

    #[test]
    fn bounded_ledger_encoding_matches_canonical_cbor() {
        for generations in [0, 1, 24, 256, 1024] {
            let ledger = super::super::AllocationLedger {
                current_generation: generations,
                records: Vec::new(),
            };
            let mut payload = Vec::new();
            ciborium::into_writer(&ledger, &mut payload).unwrap();
            let expected = LedgerPayloadEnvelope::current(payload)
                .try_encode()
                .unwrap();
            assert_eq!(
                LedgerPayloadEnvelope::encode_ledger(&ledger).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn oversized_payload_is_rejected_before_encoding() {
        let len = crate::constants::MAX_LEDGER_BYTES + 1;
        assert_eq!(
            LedgerPayloadEnvelope::current(vec![0; len]).try_encode(),
            Err(LedgerPayloadEnvelopeError::PayloadTooLarge { len: len as u64 })
        );
    }

    #[test]
    fn malformed_lengths_are_classified_before_payload_copy() {
        let mut bytes = LedgerPayloadEnvelope::current(Vec::new()).encode();
        for len in 0..LEDGER_PAYLOAD_HEADER_LEN {
            assert_eq!(
                LedgerPayloadEnvelope::decode(&bytes[..len]),
                Err(LedgerPayloadEnvelopeError::Truncated {
                    actual: len,
                    minimum: LEDGER_PAYLOAD_HEADER_LEN,
                })
            );
        }
        for len in [crate::constants::MAX_LEDGER_BYTES as u64 + 1, u64::MAX] {
            bytes[16..24].copy_from_slice(&len.to_le_bytes());
            assert_eq!(
                LedgerPayloadEnvelope::decode(&bytes),
                Err(LedgerPayloadEnvelopeError::PayloadTooLarge { len })
            );
        }
        bytes[16..24].copy_from_slice(&1_u64.to_le_bytes());
        assert_eq!(
            LedgerPayloadEnvelope::decode(&bytes),
            Err(LedgerPayloadEnvelopeError::LengthMismatch {
                declared: 1,
                actual: 0,
            })
        );
        bytes[16..24].copy_from_slice(&0_u64.to_le_bytes());
        bytes.push(42);
        assert_eq!(
            LedgerPayloadEnvelope::decode(&bytes),
            Err(LedgerPayloadEnvelopeError::LengthMismatch {
                declared: 0,
                actual: 1,
            })
        );
    }

    #[test]
    fn borrowed_payload_decode_shares_storage_through_the_byte_ceiling() {
        for len in [0, 3, crate::constants::MAX_LEDGER_BYTES] {
            let bytes = LedgerPayloadEnvelope::current(vec![42; len])
                .try_encode()
                .expect("bounded envelope");
            let payload = LedgerPayloadEnvelope::decode_payload(&bytes).expect("valid envelope");

            assert_eq!(payload.len(), len);
            assert_eq!(
                payload.as_ptr(),
                bytes[LEDGER_PAYLOAD_HEADER_LEN..].as_ptr()
            );
            assert!(payload.iter().all(|byte| *byte == 42));
        }
    }
}
