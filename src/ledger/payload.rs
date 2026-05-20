use super::{CURRENT_LEDGER_SCHEMA_VERSION, CURRENT_PHYSICAL_FORMAT_ID};

const LEDGER_PAYLOAD_MAGIC: &[u8; 8] = b"ICMEMLED";
const LEDGER_PAYLOAD_HEADER_LEN: usize = 8 + 2 + 4 + 4 + 8;

/// Current logical ledger payload envelope version.
pub const CURRENT_LEDGER_PAYLOAD_ENVELOPE_VERSION: u16 = 1;

///
/// LedgerPayloadEnvelope
///
/// Logical ledger payload envelope embedded inside one physically committed
/// generation. This layer is decoded after physical dual-slot recovery selects
/// a committed generation and before any allocation-ledger DTO is decoded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LedgerPayloadEnvelope {
    envelope_version: u16,
    ledger_schema_version: u32,
    physical_format_id: u32,
    payload: Vec<u8>,
}

impl LedgerPayloadEnvelope {
    /// Wrap a current logical ledger payload.
    #[must_use]
    pub const fn current(payload: Vec<u8>) -> Self {
        Self {
            envelope_version: CURRENT_LEDGER_PAYLOAD_ENVELOPE_VERSION,
            ledger_schema_version: CURRENT_LEDGER_SCHEMA_VERSION,
            physical_format_id: CURRENT_PHYSICAL_FORMAT_ID,
            payload,
        }
    }

    #[cfg(test)]
    pub(crate) const fn from_parts(
        envelope_version: u16,
        ledger_schema_version: u32,
        physical_format_id: u32,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            envelope_version,
            ledger_schema_version,
            physical_format_id,
            payload,
        }
    }

    /// Manually encode the logical payload envelope.
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(LEDGER_PAYLOAD_HEADER_LEN + self.payload.len());
        bytes.extend_from_slice(LEDGER_PAYLOAD_MAGIC);
        bytes.extend_from_slice(&self.envelope_version.to_le_bytes());
        bytes.extend_from_slice(&self.ledger_schema_version.to_le_bytes());
        bytes.extend_from_slice(&self.physical_format_id.to_le_bytes());
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

        let envelope_version = u16::from_le_bytes(bytes[8..10].try_into().expect("u16 slice"));
        let ledger_schema_version =
            u32::from_le_bytes(bytes[10..14].try_into().expect("u32 slice"));
        let physical_format_id = u32::from_le_bytes(bytes[14..18].try_into().expect("u32 slice"));
        let payload_len = u64::from_le_bytes(bytes[18..26].try_into().expect("u64 slice"));
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
            envelope_version,
            ledger_schema_version,
            physical_format_id,
            payload: bytes[LEDGER_PAYLOAD_HEADER_LEN..].to_vec(),
        })
    }

    /// Return the envelope format version.
    #[must_use]
    pub const fn envelope_version(&self) -> u16 {
        self.envelope_version
    }

    /// Return the logical ledger schema version declared by the envelope.
    #[must_use]
    pub const fn ledger_schema_version(&self) -> u32 {
        self.ledger_schema_version
    }

    /// Return the physical ledger format ID declared by the envelope.
    #[must_use]
    pub const fn physical_format_id(&self) -> u32 {
        self.physical_format_id
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
