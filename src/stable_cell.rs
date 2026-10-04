use crate::{LedgerCommitStore, constants::WASM_PAGE_SIZE_BYTES};
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
#[serde(deny_unknown_fields)]
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

    /// Measure the current encoded record without allocating a payload buffer.
    pub(crate) fn encoded_size(&self) -> usize {
        let mut writer = CountingWriter(0);
        encode_record(self, &mut writer);
        writer.0
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
///

#[non_exhaustive]
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum StableCellPayloadError {
    /// Declared bytes exceed the current recovery ceiling before allocation.
    #[error("stable-cell ledger payload length {value_len} exceeds recovery limit")]
    TooLarge { value_len: u64 },
    /// Memory contents do not start with the stable-cell marker.
    #[error("memory is not an ic-stable-structures Cell")]
    NotStableCell,
    /// Stable-cell layout version does not match the adapter's current shape.
    #[error("unexpected stable-cell layout version {version}")]
    UnexpectedLayoutVersion {
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
}

///
/// StableCellLedgerError
///
/// Stable-cell ledger record validation failure.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum StableCellLedgerError {
    /// Stable-cell envelope is corrupt or unexpected.
    #[error(transparent)]
    Payload(#[from] StableCellPayloadError),
    /// Stable-cell value bytes are not a valid ledger record.
    #[error("stable-cell ledger record decode failed: {0}")]
    Record(#[source] ciborium::de::Error<std::io::Error>),
}

/// Decode the raw value payload from an `ic-stable-structures::Cell` memory.
///
/// This helper is intentionally narrow: it recognizes the physical stable-cell
/// envelope and returns the value bytes. It does not deserialize those bytes or
/// decide whether they represent a valid allocation ledger.
pub fn decode_stable_cell_payload<M: Memory>(
    memory: &M,
) -> Result<Vec<u8>, StableCellPayloadError> {
    if memory.size() == 0 {
        return Err(StableCellPayloadError::NotStableCell);
    }

    let mut header = [0; STABLE_CELL_HEADER_SIZE];
    memory.read(0, &mut header);
    if &header[0..3] != STABLE_CELL_MAGIC {
        return Err(StableCellPayloadError::NotStableCell);
    }
    if header[3] != STABLE_CELL_LAYOUT_VERSION {
        return Err(StableCellPayloadError::UnexpectedLayoutVersion { version: header[3] });
    }

    let value_len = u64::from(u32::from_le_bytes([
        header[4], header[5], header[6], header[7],
    ]));
    let available_bytes = memory.size().saturating_mul(WASM_PAGE_SIZE_BYTES);
    let payload_capacity = available_bytes.saturating_sub(STABLE_CELL_VALUE_OFFSET);
    if value_len > payload_capacity {
        return Err(StableCellPayloadError::InvalidLength {
            value_len,
            available_bytes: payload_capacity,
        });
    }
    if value_len > crate::constants::MAX_LEDGER_RECORD_BYTES as u64 {
        return Err(StableCellPayloadError::TooLarge { value_len });
    }
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the admitted recovery ceiling is representable as usize"
    )]
    let value_len = value_len as usize;

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
) -> Result<StableCellLedgerRecord, ciborium::de::Error<std::io::Error>> {
    crate::cbor::from_slice_exact(bytes)
}

pub fn decode_stable_cell_ledger_record_from_memory<M: Memory>(
    memory: &M,
) -> Result<StableCellLedgerRecord, StableCellLedgerError> {
    if memory.size() == 0 {
        return Ok(StableCellLedgerRecord::default());
    }

    let payload = decode_stable_cell_payload(memory)?;
    decode_stable_cell_ledger_record(&payload).map_err(StableCellLedgerError::Record)
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
    decode_stable_cell_ledger_record_from_memory(memory)?;
    Ok(())
}

fn serialize_record(record: &StableCellLedgerRecord) -> Vec<u8> {
    let mut bytes = Vec::new();
    encode_record(record, &mut bytes);
    bytes
}

fn encode_record(record: &StableCellLedgerRecord, writer: impl std::io::Write) {
    ciborium::into_writer(record, writer).unwrap_or_else(|err| {
        panic!("StableCellLedgerRecord serialize failed: {err}");
    });
}

struct CountingWriter(usize);

impl std::io::Write for CountingWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 = self
            .0
            .checked_add(bytes.len())
            .ok_or_else(|| std::io::Error::other("encoded record length overflow"))?;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_cbor::hex_fixture;
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
    fn current_stable_cell_record_fixture_recovers() {
        let bytes = hex_fixture(include_str!(
            "../fixtures/current/stable_cell_record.cbor.hex"
        ));
        let record = decode_stable_cell_ledger_record(&bytes).expect("stable-cell fixture");

        assert_eq!(record.encoded_size(), bytes.len());
        assert_eq!(
            bytes,
            crate::test_cbor::to_vec(&record).expect("re-encoded stable-cell fixture")
        );
        assert_eq!(
            record
                .store()
                .recover()
                .expect("fixture store recovers")
                .current_generation(),
            1
        );
    }

    #[test]
    fn malformed_payload_bytes_fail_closed_without_mutating_memory() {
        let original = hex_fixture(include_str!(
            "../fixtures/current/stable_cell_record.cbor.hex"
        ));
        let start = original
            .windows(8)
            .position(|bytes| bytes == b"\x67payload")
            .unwrap()
            + 8;
        let oversized = u32::try_from(crate::constants::MAX_COMMITTED_PAYLOAD_BYTES + 1).unwrap();
        let mut cases = Vec::new();
        for replacement in [
            vec![0x5a, 0xff, 0xff, 0xff, 0xff],
            vec![0x5f, 0xff],
            vec![0x41],
            vec![0xff],
        ] {
            let mut bytes = original[..start].to_vec();
            bytes.extend(replacement);
            cases.push(bytes);
        }
        let mut bytes = original[..start].to_vec();
        bytes.push(0x5a);
        bytes.extend_from_slice(&oversized.to_be_bytes());
        bytes.resize(bytes.len() + oversized as usize, 0);
        cases.push(bytes);
        for bytes in cases {
            let memory = VectorMemory::default();
            let pages = (bytes.len() + STABLE_CELL_HEADER_SIZE).div_ceil(65_536);
            memory.grow(pages as u64);
            memory.write(0, STABLE_CELL_MAGIC);
            memory.write(3, &[STABLE_CELL_LAYOUT_VERSION]);
            memory.write(4, &u32::try_from(bytes.len()).unwrap().to_le_bytes());
            memory.write(STABLE_CELL_VALUE_OFFSET, &bytes);
            let before = memory.borrow().clone();
            for _ in 0..2 {
                assert!(matches!(
                    decode_stable_cell_ledger_record_from_memory(&memory),
                    Err(StableCellLedgerError::Record(_))
                ));
                assert!(matches!(
                    validate_stable_cell_ledger_memory(&memory),
                    Err(StableCellLedgerError::Record(_))
                ));
            }
            assert_eq!(*memory.borrow(), before);
        }
    }

    #[test]
    fn stable_cell_ledger_record_rejects_trailing_bytes() {
        let mut bytes = serialize_record(&StableCellLedgerRecord::default());
        bytes.push(0);

        let err =
            decode_stable_cell_ledger_record(&bytes).expect_err("trailing bytes must fail closed");

        assert!(err.to_string().contains("trailing bytes"));
    }

    #[test]
    fn stable_cell_ledger_record_rejects_unknown_top_level_fields() {
        use crate::test_cbor::Value;

        let map = vec![
            (
                Value::Text("store".to_string()),
                crate::test_cbor::to_value(LedgerCommitStore::default()).expect("store value"),
            ),
            (Value::Text("future_field".to_string()), Value::Bool(true)),
        ];
        let bytes = crate::test_cbor::to_vec(&Value::Map(map)).expect("unknown-field stable cell");

        let err = decode_stable_cell_ledger_record(&bytes)
            .expect_err("unknown stable-cell record field must fail closed");

        assert!(err.to_string().contains("future_field"));
    }

    #[test]
    fn stable_cell_ledger_record_requires_both_commit_slot_fields() {
        use crate::test_cbor::Value;

        for (missing, present) in [("slot0", "slot1"), ("slot1", "slot0")] {
            let physical = vec![(Value::Text(present.to_string()), Value::Null)];
            let store = vec![(Value::Text("physical".to_string()), Value::Map(physical))];
            let record = vec![(Value::Text("store".to_string()), Value::Map(store))];
            let bytes = crate::test_cbor::to_vec(&Value::Map(record)).expect("record bytes");

            let err = decode_stable_cell_ledger_record(&bytes)
                .expect_err("missing commit slot must fail closed");

            assert!(err.to_string().contains(missing));
        }
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
    fn stable_cell_payload_rejects_empty_memory_without_panic() {
        let memory = VectorMemory::default();

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
    #[test]
    fn oversized_cell_length_is_rejected_before_payload_read() {
        struct HeaderOnly {
            value_len: u32,
            pages: u64,
        }
        impl Memory for HeaderOnly {
            fn size(&self) -> u64 {
                self.pages
            }
            fn grow(&self, _: u64) -> i64 {
                panic!("must not grow")
            }
            fn write(&self, _: u64, _: &[u8]) {
                panic!("must not write")
            }
            fn read(&self, offset: u64, bytes: &mut [u8]) {
                assert_eq!(offset, 0);
                assert_eq!(bytes.len(), STABLE_CELL_HEADER_SIZE);
                bytes[..4].copy_from_slice(b"SCL\x01");
                bytes[4..8].copy_from_slice(&self.value_len.to_le_bytes());
            }
        }
        for value_len in [
            u32::try_from(crate::constants::MAX_LEDGER_RECORD_BYTES + 1).unwrap(),
            u32::MAX,
        ] {
            // The advertised value fits the backing, but exceeds the recovery ceiling.
            let memory = HeaderOnly {
                value_len,
                pages: 65_537,
            };
            assert!(matches!(
                decode_stable_cell_ledger_record_from_memory(&memory),
                Err(StableCellLedgerError::Payload(StableCellPayloadError::TooLarge {
                    value_len: rejected,
                })) if rejected == u64::from(value_len)
            ));
            // Physical-capacity rejection keeps precedence when both checks fail.
            let memory = HeaderOnly {
                value_len,
                pages: 1,
            };
            assert_eq!(
                decode_stable_cell_payload(&memory),
                Err(StableCellPayloadError::InvalidLength {
                    value_len: u64::from(value_len),
                    available_bytes: WASM_PAGE_SIZE_BYTES - STABLE_CELL_VALUE_OFFSET,
                })
            );
        }
    }

    #[test]
    fn hostile_cbor_is_rejected_on_production_record_decode() {
        let huge_array = [0x9b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        let huge_string = [0x7b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        let mut nested = vec![0x81; crate::constants::MAX_LEDGER_NESTING + 1];
        nested.push(0);
        for bytes in [&huge_array[..], &huge_string[..], &nested, &[0xa1], &[0xff]] {
            assert!(decode_stable_cell_ledger_record(bytes).is_err());
        }
    }
}
