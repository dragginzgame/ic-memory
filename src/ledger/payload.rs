const LEDGER_PAYLOAD_MAGIC: &[u8; 8] = b"ICMEMLED";
const LEDGER_PAYLOAD_HEADER_LEN: usize = 8 + 8;

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
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(LEDGER_PAYLOAD_HEADER_LEN + self.payload.len());
        bytes.extend_from_slice(LEDGER_PAYLOAD_MAGIC);
        bytes.extend_from_slice(
            &u64::try_from(self.payload.len())
                .expect("payload length does not fit in u64")
                .to_le_bytes(),
        );
        bytes.extend_from_slice(&self.payload);
        bytes
    }

    /// Manually decode the logical payload envelope.
    pub fn decode(bytes: &[u8]) -> Result<Self, LedgerPayloadEnvelopeError> {
        if bytes.len() < LEDGER_PAYLOAD_HEADER_LEN {
            return Err(LedgerPayloadEnvelopeError::Truncated {
                actual: bytes.len(),
                minimum: LEDGER_PAYLOAD_HEADER_LEN,
            });
        }

        let magic = <[u8; 8]>::try_from(&bytes[0..8]).expect("magic slice length");
        if &magic != LEDGER_PAYLOAD_MAGIC {
            return Err(LedgerPayloadEnvelopeError::BadMagic { found: magic });
        }

        let payload_len = u64::from_le_bytes(bytes[8..16].try_into().expect("u64 slice"));
        let payload_len = usize::try_from(payload_len)
            .map_err(|_| LedgerPayloadEnvelopeError::PayloadTooLarge { len: payload_len })?;
        let expected_len = LEDGER_PAYLOAD_HEADER_LEN
            .checked_add(payload_len)
            .ok_or(LedgerPayloadEnvelopeError::PayloadLengthOverflow { len: payload_len })?;
        if bytes.len() != expected_len {
            return Err(LedgerPayloadEnvelopeError::LengthMismatch {
                declared: payload_len,
                actual: bytes.len().saturating_sub(LEDGER_PAYLOAD_HEADER_LEN),
            });
        }

        Ok(Self {
            payload: bytes[LEDGER_PAYLOAD_HEADER_LEN..].to_vec(),
        })
    }

    /// Borrow the logical ledger payload bytes.
    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

///
/// LedgerPayloadEnvelopeError
///
/// Logical payload envelope could not be classified before ledger decode.
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
    /// Declared payload length does not fit in this platform's address space.
    #[error("ledger payload envelope length {len} is too large")]
    PayloadTooLarge {
        /// Declared payload length.
        len: u64,
    },
    /// Declared payload length overflowed the total envelope length.
    #[error("ledger payload envelope length {len} overflows total length")]
    PayloadLengthOverflow {
        /// Declared payload length.
        len: usize,
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
