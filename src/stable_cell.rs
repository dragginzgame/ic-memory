use crate::LedgerCommitStore;
use ic_stable_structures::{Memory, Storable, storable::Bound};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use thiserror::Error;

/// Stable-cell magic prefix written by `ic-stable-structures::Cell`.
pub const STABLE_CELL_MAGIC: &[u8; 3] = b"SCL";
/// Stable-cell layout version supported by this adapter.
pub const STABLE_CELL_LAYOUT_VERSION: u8 = 1;
/// Stable-cell header byte length.
pub const STABLE_CELL_HEADER_SIZE: usize = 8;
/// Byte offset where the stable-cell value payload starts.
pub const STABLE_CELL_VALUE_OFFSET: u64 = 8;
const WASM_PAGE_SIZE: u64 = 65_536;

///
/// StableCellLedgerRecord
///
/// `ic-stable-structures::Cell` record containing an `ic-memory` allocation
/// ledger commit store.
///
/// This is a substrate adapter DTO. It owns no framework policy and does not
/// open application allocations.
///

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct StableCellLedgerRecord {
    store: LedgerCommitStore,
}

impl StableCellLedgerRecord {
    /// Construct a record from a commit store.
    #[must_use]
    pub const fn new(store: LedgerCommitStore) -> Self {
        Self { store }
    }

    /// Borrow the embedded commit store.
    #[must_use]
    pub const fn store(&self) -> &LedgerCommitStore {
        &self.store
    }

    /// Mutably borrow the embedded commit store.
    pub const fn store_mut(&mut self) -> &mut LedgerCommitStore {
        &mut self.store
    }

    /// Consume this record and return the embedded commit store.
    #[must_use]
    pub fn into_store(self) -> LedgerCommitStore {
        self.store
    }
}

impl Storable for StableCellLedgerRecord {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(serialize_record(self))
    }

    fn into_bytes(self) -> Vec<u8> {
        serialize_record(&self)
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        decode_stable_cell_ledger_record(&bytes).unwrap_or_else(|err| {
            panic!("StableCellLedgerRecord deserialize failed: {err}");
        })
    }
}

///
/// StableCellPayloadError
///
/// Stable-cell payload decode failure.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum StableCellPayloadError {
    /// Memory contents do not start with the stable-cell marker.
    #[error("memory is not an ic-stable-structures Cell")]
    NotStableCell,
    /// Stable-cell format version is not supported.
    #[error("unsupported stable-cell layout version {version}")]
    UnsupportedVersion {
        /// Observed stable-cell version.
        version: u8,
    },
    /// Stable-cell header length does not fit inside the memory.
    #[error("stable-cell payload length {value_len} exceeds available bytes {available_bytes}")]
    InvalidLength {
        /// Encoded value length.
        value_len: u64,
        /// Available payload bytes in memory.
        available_bytes: u64,
    },
    /// Stable-cell length cannot be represented on the current host.
    #[error("stable-cell payload length {value_len} cannot fit in usize")]
    LengthOverflow {
        /// Encoded value length.
        value_len: u64,
    },
}

///
/// StableCellLedgerError
///
/// Stable-cell ledger record validation failure.
#[derive(Debug, Error)]
pub enum StableCellLedgerError {
    /// Stable-cell envelope is corrupt or unsupported.
    #[error(transparent)]
    Payload(#[from] StableCellPayloadError),
    /// Stable-cell value bytes are not a valid ledger record.
    #[error("stable-cell ledger record decode failed")]
    Record(#[source] serde_cbor::Error),
}

/// Decode the raw value payload from an `ic-stable-structures::Cell` memory.
///
/// This helper is intentionally narrow: it recognizes the physical stable-cell
/// envelope and returns the value bytes. It does not deserialize those bytes or
/// decide whether they represent a valid allocation ledger.
pub fn decode_stable_cell_payload<M: Memory>(
    memory: &M,
) -> Result<Vec<u8>, StableCellPayloadError> {
    let mut header = [0; STABLE_CELL_HEADER_SIZE];
    memory.read(0, &mut header);
    if &header[0..3] != STABLE_CELL_MAGIC {
        return Err(StableCellPayloadError::NotStableCell);
    }
    if header[3] != STABLE_CELL_LAYOUT_VERSION {
        return Err(StableCellPayloadError::UnsupportedVersion { version: header[3] });
    }

    let value_len = u64::from(u32::from_le_bytes([
        header[4], header[5], header[6], header[7],
    ]));
    let available_bytes = memory.size().saturating_mul(WASM_PAGE_SIZE);
    let payload_capacity = available_bytes.saturating_sub(STABLE_CELL_VALUE_OFFSET);
    if value_len > payload_capacity {
        return Err(StableCellPayloadError::InvalidLength {
            value_len,
            available_bytes: payload_capacity,
        });
    }
    let value_len = usize::try_from(value_len)
        .map_err(|_| StableCellPayloadError::LengthOverflow { value_len })?;

    let mut bytes = vec![0; value_len];
    memory.read(STABLE_CELL_VALUE_OFFSET, &mut bytes);
    Ok(bytes)
}

/// Decode a `StableCellLedgerRecord` from stable-cell value bytes.
///
/// This decodes only the cell value payload, not the enclosing stable-cell
/// header. Use [`decode_stable_cell_payload`] first when inspecting raw stable
/// memory.
///
/// The returned record is decoded DTO state, not authority. Recover through the
/// embedded [`LedgerCommitStore`] before trusting any ledger payload.
pub fn decode_stable_cell_ledger_record(
    bytes: &[u8],
) -> Result<StableCellLedgerRecord, serde_cbor::Error> {
    serde_cbor::from_slice(bytes)
}

/// Validate an existing stable-cell ledger record before opening it with
/// `ic-stable-structures::Cell`.
///
/// `Cell::init` decodes the existing value through [`Storable::from_bytes`].
/// That trait is panic-based, so the runtime preflights the raw memory with
/// this fallible helper first. Empty memory is treated as uninitialized and is
/// safe for `Cell::init` to create.
pub fn validate_stable_cell_ledger_memory<M: Memory>(
    memory: &M,
) -> Result<(), StableCellLedgerError> {
    if memory.size() == 0 {
        return Ok(());
    }

    let payload = decode_stable_cell_payload(memory)?;
    decode_stable_cell_ledger_record(&payload).map_err(StableCellLedgerError::Record)?;
    Ok(())
}

fn serialize_record(record: &StableCellLedgerRecord) -> Vec<u8> {
    serde_cbor::to_vec(record).unwrap_or_else(|err| {
        panic!("StableCellLedgerRecord serialize failed: {err}");
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ic_stable_structures::{Cell, VectorMemory};

    #[test]
    fn stable_cell_ledger_record_round_trips_through_cell() {
        let memory = VectorMemory::default();
        let record = StableCellLedgerRecord::default();
        let cell = Cell::init(memory.clone(), record.clone());

        assert_eq!(cell.get(), &record);
        let payload = decode_stable_cell_payload(&memory).expect("decode stable cell payload");
        let decoded = StableCellLedgerRecord::from_bytes(Cow::Owned(payload));
        assert_eq!(decoded, record);
    }

    #[test]
    fn stable_cell_payload_rejects_non_cell_memory() {
        let memory = VectorMemory::default();
        memory.grow(1);
        memory.write(0, b"BAD");

        assert_eq!(
            decode_stable_cell_payload(&memory),
            Err(StableCellPayloadError::NotStableCell)
        );
    }

    #[test]
    fn stable_cell_ledger_preflight_classifies_bad_record_without_panic() {
        let memory = VectorMemory::default();
        memory.grow(1);
        memory.write(0, STABLE_CELL_MAGIC);
        memory.write(3, &[STABLE_CELL_LAYOUT_VERSION]);
        memory.write(4, &1_u32.to_le_bytes());
        memory.write(STABLE_CELL_VALUE_OFFSET, &[0xff]);

        let err =
            validate_stable_cell_ledger_memory(&memory).expect_err("bad record must be classified");

        assert!(matches!(err, StableCellLedgerError::Record(_)));
    }
}
