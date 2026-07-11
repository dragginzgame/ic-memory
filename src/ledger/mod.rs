mod claim;
mod error;
mod integrity;
mod payload;
mod record;
mod stage;

use crate::physical::{CommitRecoveryError, DualCommitStore};
use serde::{Deserialize, Serialize};

pub use claim::{
    ClaimConflict, ClaimOutcome, claim_conflict_record, validate_declaration_claim,
    validate_reservation_claim,
};
pub use error::{
    AllocationReservationError, AllocationRetirementError, AllocationStageError, LedgerCommitError,
    LedgerIntegrityError,
};
pub use payload::{LedgerPayloadEnvelope, LedgerPayloadEnvelopeError};
pub use record::{
    AllocationHistory, AllocationLedger, AllocationRecord, AllocationRetirement, AllocationState,
    GenerationRecord, RecoveredLedger, SchemaMetadataRecord,
};
pub use stage::validate_reservation_declaration;

mod private {
    pub trait Sealed {}
}

trait LedgerCodec: private::Sealed {
    /// Encoding or decoding error type.
    type Error;

    /// Encode a logical allocation ledger into durable bytes.
    fn encode(&self, ledger: &AllocationLedger) -> Result<Vec<u8>, Self::Error>;

    /// Decode durable bytes into a logical allocation ledger.
    fn decode(&self, bytes: &[u8]) -> Result<AllocationLedger, Self::Error>;
}

///
/// CborLedgerCodec
///
/// Crate-owned CBOR ledger codec for persisted [`AllocationLedger`] payloads.
///
/// This is the only logical ledger codec in the current IC stack:
/// `MemoryManager` ID 0 stores an `ic-stable-structures::Cell` containing a
/// [`crate::StableCellLedgerRecord`], whose [`LedgerCommitStore`] contains
/// redundant checksummed CBOR-encoded ledger generations.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct CborLedgerCodec;

impl private::Sealed for CborLedgerCodec {}

impl LedgerCodec for CborLedgerCodec {
    type Error = String;

    fn encode(&self, ledger: &AllocationLedger) -> Result<Vec<u8>, Self::Error> {
        let mut bytes = Vec::new();
        ciborium::into_writer(ledger, &mut bytes).map_err(|err| err.to_string())?;
        Ok(bytes)
    }

    fn decode(&self, bytes: &[u8]) -> Result<AllocationLedger, Self::Error> {
        crate::cbor::from_slice_exact(bytes).map_err(|err| err.to_string())
    }
}

///
/// LedgerCommitStore
///
/// Generation-scoped allocation ledger commit store.
///
/// This type owns the logical commit lifecycle and the crate-owned CBOR ledger
/// encoding. It deliberately does not own stable-memory IO; that remains the
/// substrate or framework owner's responsibility.
///
/// This store commits allocation ledger generations. It does not open
/// stable-memory handles and does not allocate application slots.
///
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LedgerCommitStore {
    /// Protected physical commit slots.
    physical: DualCommitStore,
}

impl LedgerCommitStore {
    /// Borrow the protected physical commit store for diagnostics.
    #[must_use]
    pub const fn physical(&self) -> &DualCommitStore {
        &self.physical
    }

    /// Recover the authoritative allocation ledger using the native CBOR ledger codec.
    pub fn recover(&self) -> Result<RecoveredLedger, LedgerCommitError> {
        let committed = self
            .physical
            .authoritative()
            .map_err(LedgerCommitError::Recovery)?;
        let envelope = LedgerPayloadEnvelope::decode(committed.payload())
            .map_err(LedgerCommitError::PayloadEnvelope)?;
        let codec = CborLedgerCodec;
        let ledger = codec
            .decode(envelope.payload())
            .map_err(LedgerCommitError::Codec)?;
        if committed.generation() != ledger.current_generation {
            return Err(LedgerCommitError::PhysicalLogicalGenerationMismatch {
                physical_generation: committed.generation(),
                logical_generation: ledger.current_generation,
            });
        }
        ledger
            .validate_committed_integrity()
            .map_err(LedgerCommitError::Integrity)?;
        Ok(RecoveredLedger::from_trusted_parts(
            ledger,
            committed.generation(),
        ))
    }

    /// Recover the authoritative ledger, or explicitly initialize an empty store.
    ///
    /// Initialization is allowed only when no physical commit slot has ever
    /// been written. Corrupt or partially written stores fail closed even when
    /// a genesis ledger is supplied.
    ///
    /// Supplying a non-empty `genesis` is a privileged import/migration action.
    /// Normal runtime bootstraps should seed an empty current-format ledger.
    pub fn recover_or_initialize(
        &mut self,
        genesis: &AllocationLedger,
    ) -> Result<RecoveredLedger, LedgerCommitError> {
        match self.recover() {
            Ok(ledger) => Ok(ledger),
            Err(LedgerCommitError::Recovery(CommitRecoveryError::NoValidGeneration))
                if self.physical.is_uninitialized() =>
            {
                self.commit(genesis)
            }
            Err(err) => Err(err),
        }
    }

    /// Commit one logical allocation ledger generation through the native CBOR ledger codec.
    pub fn commit(
        &mut self,
        ledger: &AllocationLedger,
    ) -> Result<RecoveredLedger, LedgerCommitError> {
        ledger
            .validate_committed_integrity()
            .map_err(LedgerCommitError::Integrity)?;
        let codec = CborLedgerCodec;
        let payload =
            LedgerPayloadEnvelope::current(codec.encode(ledger).map_err(LedgerCommitError::Codec)?)
                .try_encode()
                .map_err(LedgerCommitError::PayloadEnvelope)?;
        self.physical
            .commit_payload_at_generation(ledger.current_generation, payload)
            .map_err(LedgerCommitError::Recovery)?;
        self.recover()
    }

    /// Simulate corruption of a logical ledger payload in the inactive slot.
    #[cfg(test)]
    pub fn write_corrupt_inactive_ledger(
        &mut self,
        ledger: &AllocationLedger,
    ) -> Result<(), LedgerCommitError> {
        let codec = CborLedgerCodec;
        let payload =
            LedgerPayloadEnvelope::current(codec.encode(ledger).map_err(LedgerCommitError::Codec)?)
                .try_encode()
                .map_err(LedgerCommitError::PayloadEnvelope)?;
        self.physical
            .write_corrupt_inactive_slot(ledger.current_generation, payload);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Validate,
        declaration::{AllocationDeclaration, DeclarationSnapshot, DeclarationSnapshotError},
        key::StableKey,
        physical::CommittedGenerationBytes,
        schema::{SchemaMetadata, SchemaMetadataError},
        slot::{AllocationSlotDescriptor, MEMORY_MANAGER_INVALID_ID, MemoryManagerSlotError},
    };
    fn declaration(key: &str, id: u8, schema_version: Option<u32>) -> AllocationDeclaration {
        AllocationDeclaration::new(
            key,
            AllocationSlotDescriptor::memory_manager(id).expect("usable slot"),
            None,
            SchemaMetadata { schema_version },
        )
        .expect("declaration")
    }

    fn invalid_schema_metadata() -> SchemaMetadata {
        SchemaMetadata {
            schema_version: Some(0),
        }
    }

    fn declaration_with_invalid_schema(key: &str, id: u8) -> AllocationDeclaration {
        let mut declaration = declaration(key, id, Some(1));
        declaration.schema = invalid_schema_metadata();
        declaration
    }

    fn ledger() -> AllocationLedger {
        AllocationLedger {
            current_generation: 3,
            allocation_history: AllocationHistory::default(),
        }
    }

    fn committed_ledger(current_generation: u64) -> AllocationLedger {
        AllocationLedger {
            current_generation,
            allocation_history: AllocationHistory::from_parts(
                Vec::new(),
                (1..=current_generation)
                    .map(|generation| GenerationRecord {
                        generation,
                        parent_generation: if generation == 1 { 0 } else { generation - 1 },
                        runtime_fingerprint: None,
                        declaration_count: 0,
                        committed_at: None,
                    })
                    .collect(),
            ),
        }
    }

    fn active_record(key: &str, id: u8) -> AllocationRecord {
        AllocationRecord::from_declaration(1, declaration(key, id, None), AllocationState::Active)
            .expect("valid schema metadata")
    }

    fn validated(
        generation: u64,
        declarations: Vec<AllocationDeclaration>,
    ) -> crate::capability::ValidatedAllocations {
        crate::capability::ValidatedAllocations::new(generation, declarations, None)
    }

    fn record<'ledger>(ledger: &'ledger AllocationLedger, key: &str) -> &'ledger AllocationRecord {
        ledger
            .allocation_history
            .records()
            .iter()
            .find(|record| record.stable_key.as_str() == key)
            .expect("allocation record")
    }

    fn enveloped_payload(ledger: &AllocationLedger) -> Vec<u8> {
        LedgerPayloadEnvelope::current(CborLedgerCodec.encode(ledger).expect("CBOR payload"))
            .encode()
    }

    fn old_versioned_enveloped_payload(ledger: &AllocationLedger) -> Vec<u8> {
        let payload = CborLedgerCodec.encode(ledger).expect("CBOR payload");
        let mut bytes = Vec::with_capacity(8 + 2 + 4 + 4 + 8 + payload.len());
        bytes.extend_from_slice(b"ICMEMLED");
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.extend_from_slice(
            &u64::try_from(payload.len())
                .expect("payload length")
                .to_le_bytes(),
        );
        bytes.extend_from_slice(&payload);
        bytes
    }

    fn hex_fixture(contents: &str) -> Vec<u8> {
        let hex = contents
            .chars()
            .filter(|char| !char.is_whitespace())
            .collect::<String>();
        assert_eq!(hex.len() % 2, 0, "fixture hex must have byte pairs");
        hex.as_bytes()
            .chunks_exact(2)
            .map(|pair| {
                let pair = std::str::from_utf8(pair).expect("fixture hex is utf8");
                u8::from_str_radix(pair, 16).expect("fixture hex byte")
            })
            .collect()
    }

    fn ledger_from_payload_fixture(contents: &str) -> AllocationLedger {
        let bytes = hex_fixture(contents);
        let envelope = LedgerPayloadEnvelope::decode(&bytes).expect("fixture envelope");

        let codec = CborLedgerCodec;
        let ledger = codec.decode(envelope.payload()).expect("fixture ledger");
        ledger
            .validate_committed_integrity()
            .expect("fixture ledger integrity");
        assert_eq!(bytes, enveloped_payload(&ledger));
        ledger
    }

    fn store_from_fixture(contents: &str) -> LedgerCommitStore {
        let bytes = hex_fixture(contents);
        let store: LedgerCommitStore = crate::test_cbor::from_slice(&bytes).expect("fixture store");
        assert_eq!(
            bytes,
            crate::test_cbor::to_vec(&store).expect("re-encoded fixture store")
        );
        store
    }

    fn active_committed_ledger() -> AllocationLedger {
        AllocationLedger {
            current_generation: 1,
            allocation_history: AllocationHistory::from_parts(
                vec![active_record("app.users.v1", 100)],
                vec![GenerationRecord {
                    generation: 1,
                    parent_generation: 0,
                    runtime_fingerprint: None,
                    declaration_count: 1,
                    committed_at: None,
                }],
            ),
        }
    }

    fn active_ledger_value() -> crate::test_cbor::Value {
        crate::test_cbor::to_value(active_committed_ledger()).expect("ledger value")
    }

    fn value_map_mut(
        value: &mut crate::test_cbor::Value,
    ) -> &mut Vec<(crate::test_cbor::Value, crate::test_cbor::Value)> {
        let crate::test_cbor::Value::Map(map) = value else {
            panic!("expected CBOR map");
        };
        map
    }

    fn map_field_mut<'map>(
        map: &'map mut [(crate::test_cbor::Value, crate::test_cbor::Value)],
        field: &str,
    ) -> &'map mut crate::test_cbor::Value {
        map.iter_mut()
            .find(|(key, _)| key == &crate::test_cbor::Value::Text(field.to_string()))
            .map(|(_, value)| value)
            .expect("CBOR map field")
    }

    fn value_array_mut(value: &mut crate::test_cbor::Value) -> &mut Vec<crate::test_cbor::Value> {
        let crate::test_cbor::Value::Array(values) = value else {
            panic!("expected CBOR array");
        };
        values
    }

    fn decode_mutated_ledger(value: crate::test_cbor::Value) -> String {
        let bytes = crate::test_cbor::to_vec(&value).expect("mutated ledger bytes");
        CborLedgerCodec
            .decode(&bytes)
            .expect_err("mutated ledger must fail closed")
    }

    #[test]
    fn allocation_history_accessors_expose_read_only_views() {
        let history = AllocationHistory::from_parts(
            vec![active_record("app.users.v1", 100)],
            vec![GenerationRecord::new(1, 0, None, 1, Some(42)).expect("generation record")],
        );

        assert!(!history.is_empty());
        assert_eq!(history.records().len(), 1);
        assert_eq!(history.generations().len(), 1);
        assert_eq!(history.generations()[0].committed_at(), Some(42));
    }

    #[test]
    fn record_constructors_validate_metadata() {
        let schema_err = SchemaMetadataRecord::new(1, invalid_schema_metadata())
            .expect_err("invalid schema must fail");
        assert_eq!(schema_err, SchemaMetadataError::InvalidVersion);

        let generation_err = GenerationRecord::new(1, 0, Some(String::new()), 0, None)
            .expect_err("empty fingerprint must fail");
        assert_eq!(
            generation_err,
            DeclarationSnapshotError::EmptyRuntimeFingerprint
        );
    }

    #[test]
    fn cbor_ledger_codec_round_trips_allocation_ledger() {
        let ledger = committed_ledger(2);
        let codec = CborLedgerCodec;

        let encoded = codec.encode(&ledger).expect("encode ledger");
        let decoded = codec.decode(&encoded).expect("decode ledger");

        assert_eq!(decoded, ledger);
    }

    #[test]
    fn cbor_ledger_codec_rejects_trailing_bytes() {
        let mut encoded = CborLedgerCodec
            .encode(&committed_ledger(2))
            .expect("encode ledger");
        encoded.push(0);

        let err = CborLedgerCodec
            .decode(&encoded)
            .expect_err("trailing bytes must fail closed");

        assert!(err.contains("trailing bytes"));
    }

    #[test]
    fn cbor_ledger_codec_rejects_unknown_top_level_fields() {
        use crate::test_cbor::Value;

        let mut map = Vec::new();
        crate::test_cbor::map_insert(
            &mut map,
            Value::Text("current_generation".to_string()),
            Value::Integer(0.into()),
        );
        crate::test_cbor::map_insert(
            &mut map,
            Value::Text("allocation_history".to_string()),
            crate::test_cbor::to_value(AllocationHistory::default()).expect("history value"),
        );
        crate::test_cbor::map_insert(
            &mut map,
            Value::Text("future_field".to_string()),
            Value::Bool(true),
        );
        let bytes = crate::test_cbor::to_vec(&Value::Map(map)).expect("unknown-field ledger");

        let err = CborLedgerCodec
            .decode(&bytes)
            .expect_err("unknown ledger field must fail closed");

        assert!(err.contains("future_field"));
    }

    #[test]
    fn cbor_ledger_codec_rejects_unknown_nested_history_fields() {
        let mut value = active_ledger_value();
        let history = map_field_mut(value_map_mut(&mut value), "allocation_history");
        crate::test_cbor::map_insert(
            value_map_mut(history),
            crate::test_cbor::Value::Text("future_history_field".to_string()),
            crate::test_cbor::Value::Bool(true),
        );

        let err = decode_mutated_ledger(value);

        assert!(err.contains("future_history_field"));
    }

    #[test]
    fn cbor_ledger_codec_rejects_unknown_nested_record_fields() {
        let mut value = active_ledger_value();
        let history = map_field_mut(value_map_mut(&mut value), "allocation_history");
        let records = map_field_mut(value_map_mut(history), "records");
        let record = value_array_mut(records)
            .first_mut()
            .expect("allocation record");
        crate::test_cbor::map_insert(
            value_map_mut(record),
            crate::test_cbor::Value::Text("future_record_field".to_string()),
            crate::test_cbor::Value::Bool(true),
        );

        let err = decode_mutated_ledger(value);

        assert!(err.contains("future_record_field"));
    }

    #[test]
    fn cbor_ledger_codec_rejects_unknown_nested_slot_descriptor_fields() {
        let mut value = active_ledger_value();
        let history = map_field_mut(value_map_mut(&mut value), "allocation_history");
        let records = map_field_mut(value_map_mut(history), "records");
        let record = value_array_mut(records)
            .first_mut()
            .expect("allocation record");
        let slot = map_field_mut(value_map_mut(record), "slot");
        crate::test_cbor::map_insert(
            value_map_mut(slot),
            crate::test_cbor::Value::Text("future_slot_field".to_string()),
            crate::test_cbor::Value::Bool(true),
        );

        let err = decode_mutated_ledger(value);

        assert!(err.contains("future_slot_field"));
    }

    #[test]
    fn cbor_ledger_codec_rejects_unknown_nested_generation_fields() {
        let mut value = active_ledger_value();
        let history = map_field_mut(value_map_mut(&mut value), "allocation_history");
        let generations = map_field_mut(value_map_mut(history), "generations");
        let generation = value_array_mut(generations)
            .first_mut()
            .expect("generation record");
        crate::test_cbor::map_insert(
            value_map_mut(generation),
            crate::test_cbor::Value::Text("future_generation_field".to_string()),
            crate::test_cbor::Value::Bool(true),
        );

        let err = decode_mutated_ledger(value);

        assert!(err.contains("future_generation_field"));
    }

    #[test]
    fn ledger_commit_store_rejects_unknown_top_level_fields() {
        use crate::test_cbor::Value;

        let mut map = Vec::new();
        crate::test_cbor::map_insert(
            &mut map,
            Value::Text("physical".to_string()),
            crate::test_cbor::to_value(DualCommitStore::default()).expect("physical value"),
        );
        crate::test_cbor::map_insert(
            &mut map,
            Value::Text("future_field".to_string()),
            Value::Bool(true),
        );
        let bytes = crate::test_cbor::to_vec(&Value::Map(map)).expect("unknown-field store");

        let err = crate::test_cbor::from_slice::<LedgerCommitStore>(&bytes)
            .expect_err("unknown store field must fail closed");

        assert!(err.to_string().contains("future_field"));
    }

    #[test]
    fn current_empty_genesis_payload_fixture_recovers() {
        let ledger = ledger_from_payload_fixture(include_str!(
            "../../fixtures/current/empty_genesis_payload_envelope.hex"
        ));

        assert_eq!(ledger.current_generation, 0);
        assert!(ledger.allocation_history.records().is_empty());
        assert!(ledger.allocation_history.generations().is_empty());
    }

    #[test]
    fn current_single_active_payload_fixture_recovers() {
        let ledger = ledger_from_payload_fixture(include_str!(
            "../../fixtures/current/single_active_allocation_payload_envelope.hex"
        ));
        let record = record(&ledger, "app.users.v1");

        assert_eq!(ledger.current_generation, 1);
        assert_eq!(record.state(), AllocationState::Active);
        assert_eq!(record.slot().memory_manager_id().expect("memory id"), 100);
        assert_eq!(record.first_generation(), 1);
        assert_eq!(record.last_seen_generation(), 1);
    }

    #[test]
    fn current_reserved_payload_fixture_recovers() {
        let ledger = ledger_from_payload_fixture(include_str!(
            "../../fixtures/current/reserved_allocation_payload_envelope.hex"
        ));
        let record = record(&ledger, "app.future_store.v1");

        assert_eq!(ledger.current_generation, 1);
        assert_eq!(record.state(), AllocationState::Reserved);
        assert_eq!(record.slot().memory_manager_id().expect("memory id"), 101);
    }

    #[test]
    fn current_retired_payload_fixture_recovers() {
        let ledger = ledger_from_payload_fixture(include_str!(
            "../../fixtures/current/retired_allocation_payload_envelope.hex"
        ));
        let record = record(&ledger, "app.users.v1");

        assert_eq!(ledger.current_generation, 2);
        assert_eq!(record.state(), AllocationState::Retired);
        assert_eq!(record.retired_generation(), Some(2));
    }

    #[test]
    fn current_memory_manager_descriptor_fixture_validates() {
        let bytes = hex_fixture(include_str!(
            "../../fixtures/current/memory_manager_descriptor.cbor.hex"
        ));
        let descriptor: AllocationSlotDescriptor =
            crate::test_cbor::from_slice(&bytes).expect("descriptor fixture");

        descriptor.validate().expect("valid descriptor");
        assert_eq!(descriptor.memory_manager_id().expect("memory id"), 100);
        assert_eq!(
            bytes,
            crate::test_cbor::to_vec(&descriptor).expect("re-encoded descriptor")
        );
    }

    #[test]
    fn current_ledger_commit_store_single_active_fixture_recovers() {
        let store = store_from_fixture(include_str!(
            "../../fixtures/current/ledger_commit_store_single_active.cbor.hex"
        ));

        let recovered = store.recover().expect("fixture store recovers");
        assert_eq!(recovered.current_generation(), 1);
        assert_eq!(
            record(recovered.ledger(), "app.users.v1")
                .slot()
                .memory_manager_id(),
            Ok(100)
        );
    }

    #[test]
    fn current_dual_slot_valid_newer_fixture_recovers_newer() {
        let store = store_from_fixture(include_str!(
            "../../fixtures/current/dual_slot_store_valid_newer.cbor.hex"
        ));

        assert_eq!(
            store.physical().diagnostic().authoritative_generation,
            Some(2)
        );
        let recovered = store.recover().expect("fixture store recovers");
        assert_eq!(recovered.current_generation(), 2);
    }

    #[test]
    fn current_dual_slot_corrupt_newer_fixture_recovers_prior() {
        let store = store_from_fixture(include_str!(
            "../../fixtures/current/dual_slot_store_corrupt_newer.cbor.hex"
        ));

        let diagnostic = store.physical().diagnostic();
        assert_eq!(diagnostic.authoritative_generation, Some(1));
        assert!(diagnostic.slot1.present);
        assert!(!diagnostic.slot1.valid);
        let recovered = store.recover().expect("fixture store recovers");
        assert_eq!(recovered.current_generation(), 1);
    }

    #[test]
    fn stage_validated_generation_records_new_allocations() {
        let declarations = vec![declaration("app.users.v1", 100, Some(1))];
        let validated = validated(3, declarations);

        let staged = ledger()
            .stage_validated_generation(&validated, Some(42))
            .expect("staged generation");

        assert_eq!(staged.current_generation, 4);
        assert_eq!(staged.allocation_history.records().len(), 1);
        assert_eq!(staged.allocation_history.records()[0].first_generation, 4);
        assert_eq!(staged.allocation_history.generations()[0].generation, 4);
        assert_eq!(
            staged.allocation_history.generations()[0].committed_at,
            Some(42)
        );
    }

    #[test]
    fn stage_validated_generation_allows_empty_generation_boundary() {
        let validated = crate::capability::ValidatedAllocations::new(
            3,
            Vec::new(),
            Some("test-runtime".to_string()),
        );

        let staged = ledger()
            .stage_validated_generation(&validated, Some(42))
            .expect("empty validated generation");

        assert_eq!(staged.current_generation, 4);
        assert!(staged.allocation_history.records().is_empty());
        assert_eq!(staged.allocation_history.generations().len(), 1);
        assert_eq!(staged.allocation_history.generations()[0].generation(), 4);
        assert_eq!(
            staged.allocation_history.generations()[0].parent_generation(),
            3
        );
        assert_eq!(
            staged.allocation_history.generations()[0].runtime_fingerprint(),
            Some("test-runtime")
        );
        assert_eq!(
            staged.allocation_history.generations()[0].declaration_count(),
            0
        );
        assert_eq!(
            staged.allocation_history.generations()[0].committed_at(),
            Some(42)
        );
    }

    #[test]
    fn stage_validated_generation_rejects_stale_validated_allocations() {
        let validated = validated(2, vec![declaration("app.users.v1", 100, Some(1))]);

        let err = ledger()
            .stage_validated_generation(&validated, None)
            .expect_err("stale validated allocations");

        assert_eq!(
            err,
            AllocationStageError::StaleValidatedAllocations {
                validated_generation: 2,
                ledger_generation: 3
            }
        );
    }

    #[test]
    fn stage_validated_generation_rejects_invalid_schema_metadata() {
        let validated = crate::capability::ValidatedAllocations::new(
            3,
            vec![declaration_with_invalid_schema("app.users.v1", 100)],
            None,
        );

        let err = ledger()
            .stage_validated_generation(&validated, None)
            .expect_err("invalid schema metadata");

        assert_eq!(
            err,
            AllocationStageError::InvalidSchemaMetadata {
                stable_key: StableKey::parse("app.users.v1").expect("stable key"),
                error: SchemaMetadataError::InvalidVersion,
            }
        );
    }

    #[test]
    fn stage_validated_generation_rejects_generation_overflow() {
        let ledger = AllocationLedger {
            current_generation: u64::MAX,
            ..ledger()
        };
        let validated = validated(u64::MAX, vec![declaration("app.users.v1", 100, Some(1))]);

        let err = ledger
            .stage_validated_generation(&validated, None)
            .expect_err("overflow must fail");

        assert_eq!(
            err,
            AllocationStageError::GenerationOverflow {
                generation: u64::MAX
            }
        );
    }

    #[test]
    fn stage_validated_generation_rejects_same_key_different_slot() {
        let mut ledger = ledger();
        *ledger.allocation_history.records_mut() = vec![active_record("app.users.v1", 100)];
        let validated = validated(3, vec![declaration("app.users.v1", 101, None)]);

        let err = ledger
            .stage_validated_generation(&validated, None)
            .expect_err("stable key cannot move slots");

        assert!(matches!(
            err,
            AllocationStageError::StableKeySlotConflict { .. }
        ));
    }

    #[test]
    fn stage_validated_generation_rejects_same_slot_different_key() {
        let mut ledger = ledger();
        *ledger.allocation_history.records_mut() = vec![active_record("app.users.v1", 100)];
        let validated = validated(3, vec![declaration("app.orders.v1", 100, None)]);

        let err = ledger
            .stage_validated_generation(&validated, None)
            .expect_err("slot cannot be reused by another key");

        assert!(matches!(
            err,
            AllocationStageError::SlotStableKeyConflict { .. }
        ));
    }

    #[test]
    fn stage_validated_generation_rejects_retired_redeclaration() {
        let mut ledger = ledger();
        let mut record = active_record("app.users.v1", 100);
        record.state = AllocationState::Retired;
        record.retired_generation = Some(3);
        *ledger.allocation_history.records_mut() = vec![record];
        let validated = validated(3, vec![declaration("app.users.v1", 100, None)]);

        let err = ledger
            .stage_validated_generation(&validated, None)
            .expect_err("retired allocation cannot be redeclared");

        assert!(matches!(
            err,
            AllocationStageError::RetiredAllocation { .. }
        ));
    }

    #[test]
    fn stage_validated_generation_preserves_omitted_records() {
        let first = validated(
            3,
            vec![
                declaration("app.users.v1", 100, Some(1)),
                declaration("app.orders.v1", 101, Some(1)),
            ],
        );
        let second = validated(4, vec![declaration("app.users.v1", 100, Some(1))]);

        let staged = ledger()
            .stage_validated_generation(&first, None)
            .expect("first generation");
        let staged = staged
            .stage_validated_generation(&second, None)
            .expect("second generation");

        assert_eq!(staged.current_generation, 5);
        assert_eq!(staged.allocation_history.records().len(), 2);
        let omitted = staged
            .allocation_history
            .records()
            .iter()
            .find(|record| record.stable_key.as_str() == "app.orders.v1")
            .expect("omitted record");
        assert_eq!(omitted.state, AllocationState::Active);
        assert_eq!(omitted.last_seen_generation, 4);
    }

    #[test]
    fn stage_validated_generation_records_schema_metadata_history() {
        let first = validated(3, vec![declaration("app.users.v1", 100, Some(1))]);
        let second = validated(4, vec![declaration("app.users.v1", 100, Some(2))]);

        let staged = ledger()
            .stage_validated_generation(&first, None)
            .expect("first generation");
        let staged = staged
            .stage_validated_generation(&second, None)
            .expect("second generation");
        let record = &staged.allocation_history.records()[0];

        assert_eq!(record.schema_history.len(), 2);
        assert_eq!(record.schema_history[0].generation, 4);
        assert_eq!(record.schema_history[1].generation, 5);
    }

    #[test]
    fn stage_reservation_generation_records_reserved_allocations() {
        let reservations = vec![declaration("ic_memory.generation_log.v1", 1, None)];

        let staged = ledger()
            .stage_reservation_generation(&reservations, Some(42))
            .expect("reserved generation");

        assert_eq!(staged.current_generation, 4);
        assert_eq!(staged.allocation_history.records().len(), 1);
        assert_eq!(
            staged.allocation_history.records()[0].state,
            AllocationState::Reserved
        );
        assert_eq!(
            staged.allocation_history.generations()[0].declaration_count,
            1
        );
    }

    #[test]
    fn stage_reservation_generation_allows_empty_generation_boundary() {
        let reservations = Vec::new();

        let staged = ledger()
            .stage_reservation_generation(&reservations, Some(42))
            .expect("empty reservation generation");

        assert_eq!(staged.current_generation, 4);
        assert!(staged.allocation_history.records().is_empty());
        assert_eq!(staged.allocation_history.generations().len(), 1);
        assert_eq!(staged.allocation_history.generations()[0].generation(), 4);
        assert_eq!(
            staged.allocation_history.generations()[0].declaration_count(),
            0
        );
        assert_eq!(
            staged.allocation_history.generations()[0].committed_at(),
            Some(42)
        );
    }

    #[test]
    fn stage_reservation_generation_refreshes_existing_reserved_allocation() {
        let first = vec![declaration("app.future_store.v1", 100, Some(1))];
        let staged = ledger()
            .stage_reservation_generation(&first, Some(42))
            .expect("first reservation generation");

        let second = vec![declaration("app.future_store.v1", 100, Some(2))];
        let staged = staged
            .stage_reservation_generation(&second, Some(43))
            .expect("reservation refresh");
        let record = record(&staged, "app.future_store.v1");

        assert_eq!(record.state(), AllocationState::Reserved);
        assert_eq!(record.first_generation(), 4);
        assert_eq!(record.last_seen_generation(), 5);
        assert_eq!(record.schema_history().len(), 2);
        assert_eq!(record.schema_history()[1].generation(), 5);
        assert_eq!(
            staged.allocation_history.generations()[1].declaration_count(),
            1
        );
    }

    #[test]
    fn stage_reservation_generation_rejects_generation_overflow() {
        let ledger = AllocationLedger {
            current_generation: u64::MAX,
            ..ledger()
        };
        let reservations = vec![declaration("ic_memory.generation_log.v1", 1, None)];

        let err = ledger
            .stage_reservation_generation(&reservations, None)
            .expect_err("overflow must fail");

        assert_eq!(
            err,
            AllocationReservationError::GenerationOverflow {
                generation: u64::MAX
            }
        );
    }

    #[test]
    fn stage_reservation_generation_rejects_invalid_schema_metadata() {
        let reservations = vec![declaration_with_invalid_schema(
            "ic_memory.generation_log.v1",
            1,
        )];

        let err = ledger()
            .stage_reservation_generation(&reservations, None)
            .expect_err("invalid reservation schema metadata");

        assert_eq!(
            err,
            AllocationReservationError::InvalidSchemaMetadata {
                stable_key: StableKey::parse("ic_memory.generation_log.v1").expect("stable key"),
                error: SchemaMetadataError::InvalidVersion,
            }
        );
    }

    #[test]
    fn stage_reservation_generation_rejects_invalid_decoded_slot() {
        let mut reservation = declaration("ic_memory.generation_log.v1", 1, None);
        reservation.slot =
            AllocationSlotDescriptor::memory_manager_unchecked(MEMORY_MANAGER_INVALID_ID);

        let err = ledger()
            .stage_reservation_generation(&[reservation], None)
            .expect_err("invalid decoded reservation slot");

        assert!(matches!(
            err,
            AllocationReservationError::InvalidDeclaration(
                DeclarationSnapshotError::MemoryManagerSlot(
                    MemoryManagerSlotError::InvalidMemoryManagerId { id }
                )
            ) if id == MEMORY_MANAGER_INVALID_ID
        ));
    }

    #[test]
    fn stage_reservation_generation_rejects_same_key_different_slot() {
        let mut ledger = ledger();
        *ledger.allocation_history.records_mut() = vec![
            AllocationRecord::reserved(3, declaration("app.future_store.v1", 100, None))
                .expect("valid schema metadata"),
        ];
        let reservations = vec![declaration("app.future_store.v1", 101, None)];

        let err = ledger
            .stage_reservation_generation(&reservations, None)
            .expect_err("reservation key cannot move slots");

        assert!(matches!(
            err,
            AllocationReservationError::StableKeySlotConflict { .. }
        ));
    }

    #[test]
    fn stage_reservation_generation_rejects_same_slot_different_key() {
        let mut ledger = ledger();
        *ledger.allocation_history.records_mut() = vec![
            AllocationRecord::reserved(3, declaration("app.future_store.v1", 100, None))
                .expect("valid schema metadata"),
        ];
        let reservations = vec![declaration("app.other_future_store.v1", 100, None)];

        let err = ledger
            .stage_reservation_generation(&reservations, None)
            .expect_err("reservation slot cannot be reused by another key");

        assert!(matches!(
            err,
            AllocationReservationError::SlotStableKeyConflict { .. }
        ));
    }

    #[test]
    fn stage_reservation_generation_rejects_active_allocation() {
        let active = validated(3, vec![declaration("app.users.v1", 100, None)]);
        let staged = ledger()
            .stage_validated_generation(&active, None)
            .expect("active generation");
        let reservations = vec![declaration("app.users.v1", 100, None)];

        let err = staged
            .stage_reservation_generation(&reservations, None)
            .expect_err("active cannot become reserved");

        assert!(matches!(
            err,
            AllocationReservationError::ActiveAllocation { .. }
        ));
    }

    #[test]
    fn stage_reservation_generation_rejects_retired_allocation() {
        let mut ledger = ledger();
        let mut record = active_record("app.users.v1", 100);
        record.state = AllocationState::Retired;
        record.retired_generation = Some(3);
        *ledger.allocation_history.records_mut() = vec![record];
        let reservations = vec![declaration("app.users.v1", 100, None)];

        let err = ledger
            .stage_reservation_generation(&reservations, None)
            .expect_err("retired cannot revive");

        assert!(matches!(
            err,
            AllocationReservationError::RetiredAllocation { .. }
        ));
    }

    #[test]
    fn stage_validated_generation_activates_reserved_record() {
        let reservations = vec![declaration("app.future_store.v1", 100, Some(1))];
        let staged = ledger()
            .stage_reservation_generation(&reservations, None)
            .expect("reserved generation");
        let active = validated(4, vec![declaration("app.future_store.v1", 100, Some(2))]);

        let staged = staged
            .stage_validated_generation(&active, None)
            .expect("active generation");
        let record = &staged.allocation_history.records()[0];

        assert_eq!(record.state, AllocationState::Active);
        assert_eq!(record.first_generation, 4);
        assert_eq!(record.last_seen_generation, 5);
        assert_eq!(record.schema_history.len(), 2);
    }

    #[test]
    fn stage_retirement_generation_tombstones_named_allocation() {
        let active = validated(3, vec![declaration("app.users.v1", 100, None)]);
        let staged = ledger()
            .stage_validated_generation(&active, None)
            .expect("active generation");
        let retirement = AllocationRetirement::new(
            "app.users.v1",
            AllocationSlotDescriptor::memory_manager(100).expect("usable slot"),
        )
        .expect("retirement");

        let staged = staged
            .stage_retirement_generation(&retirement, Some(42))
            .expect("retired generation");
        let record = &staged.allocation_history.records()[0];

        assert_eq!(staged.current_generation, 5);
        assert_eq!(record.state, AllocationState::Retired);
        assert_eq!(record.retired_generation, Some(5));
        assert_eq!(
            staged.allocation_history.generations()[1].declaration_count,
            0
        );
    }

    #[test]
    fn stage_retirement_generation_tombstones_reserved_allocation() {
        let reservations = vec![declaration("app.future_store.v1", 100, Some(1))];
        let staged = ledger()
            .stage_reservation_generation(&reservations, None)
            .expect("reserved generation");
        let retirement = AllocationRetirement::new(
            "app.future_store.v1",
            AllocationSlotDescriptor::memory_manager(100).expect("usable slot"),
        )
        .expect("retirement");

        let staged = staged
            .stage_retirement_generation(&retirement, Some(42))
            .expect("reserved retirement generation");
        let record = &staged.allocation_history.records()[0];

        assert_eq!(staged.current_generation, 5);
        assert_eq!(record.state, AllocationState::Retired);
        assert_eq!(record.first_generation, 4);
        assert_eq!(record.retired_generation, Some(5));
        assert_eq!(
            staged.allocation_history.generations()[1].declaration_count(),
            0
        );
    }

    #[test]
    fn stage_retirement_generation_rejects_generation_overflow() {
        let mut ledger = ledger();
        ledger.current_generation = u64::MAX;
        *ledger.allocation_history.records_mut() = vec![active_record("app.users.v1", 100)];
        let retirement = AllocationRetirement::new(
            "app.users.v1",
            AllocationSlotDescriptor::memory_manager(100).expect("usable slot"),
        )
        .expect("retirement");

        let err = ledger
            .stage_retirement_generation(&retirement, None)
            .expect_err("overflow must fail");

        assert_eq!(
            err,
            AllocationRetirementError::GenerationOverflow {
                generation: u64::MAX
            }
        );
    }

    #[test]
    fn stage_retirement_generation_requires_matching_slot() {
        let active = validated(3, vec![declaration("app.users.v1", 100, None)]);
        let staged = ledger()
            .stage_validated_generation(&active, None)
            .expect("active generation");
        let retirement = AllocationRetirement::new(
            "app.users.v1",
            AllocationSlotDescriptor::memory_manager(101).expect("usable slot"),
        )
        .expect("retirement");

        let err = staged
            .stage_retirement_generation(&retirement, None)
            .expect_err("slot mismatch");

        assert!(matches!(
            err,
            AllocationRetirementError::SlotMismatch { .. }
        ));
    }

    #[test]
    fn allocation_retirement_constructor_rejects_invalid_slot() {
        let err = AllocationRetirement::new(
            "app.users.v1",
            AllocationSlotDescriptor::memory_manager_unchecked(MEMORY_MANAGER_INVALID_ID),
        )
        .expect_err("invalid retirement slot must fail at construction");

        assert!(matches!(
            err,
            AllocationRetirementError::MemoryManagerSlot(
                MemoryManagerSlotError::InvalidMemoryManagerId { id }
            ) if id == MEMORY_MANAGER_INVALID_ID
        ));
    }

    #[test]
    fn stage_retirement_generation_rejects_invalid_decoded_slot() {
        let mut ledger = ledger();
        *ledger.allocation_history.records_mut() = vec![active_record("app.users.v1", 100)];
        let mut retirement = AllocationRetirement::new(
            "app.users.v1",
            AllocationSlotDescriptor::memory_manager(100).expect("usable slot"),
        )
        .expect("retirement");
        retirement.slot =
            AllocationSlotDescriptor::memory_manager_unchecked(MEMORY_MANAGER_INVALID_ID);

        let err = ledger
            .stage_retirement_generation(&retirement, None)
            .expect_err("decoded invalid retirement must fail at the boundary");

        assert!(matches!(
            err,
            AllocationRetirementError::MemoryManagerSlot(
                MemoryManagerSlotError::InvalidMemoryManagerId { id }
            ) if id == MEMORY_MANAGER_INVALID_ID
        ));
    }

    #[test]
    fn stage_retirement_generation_rejects_invalid_decoded_stable_key() {
        let mut ledger = ledger();
        *ledger.allocation_history.records_mut() = vec![active_record("app.users.v1", 100)];
        let retirement = AllocationRetirement::new(
            "app.users.v1",
            AllocationSlotDescriptor::memory_manager(100).expect("usable slot"),
        )
        .expect("retirement");
        let mut value = crate::test_cbor::to_value(retirement).expect("retirement value");
        *map_field_mut(value_map_mut(&mut value), "stable_key") =
            crate::test_cbor::Value::Text("App.users.v1".to_string());
        let bytes = crate::test_cbor::to_vec(&value).expect("retirement bytes");
        let retirement: AllocationRetirement =
            crate::test_cbor::from_slice(&bytes).expect("decoded DTO");

        let err = ledger
            .stage_retirement_generation(&retirement, None)
            .expect_err("decoded invalid stable key must fail at the boundary");

        assert!(matches!(err, AllocationRetirementError::Key(_)));
    }

    #[test]
    fn snapshot_can_feed_validated_generation() {
        let snapshot = DeclarationSnapshot::new(vec![declaration("app.users.v1", 100, None)])
            .expect("snapshot");
        let (declarations, runtime_fingerprint) = snapshot.into_parts();
        let validated =
            crate::capability::ValidatedAllocations::new(3, declarations, runtime_fingerprint);

        let staged = ledger()
            .stage_validated_generation(&validated, None)
            .expect("validated generation");

        assert_eq!(staged.allocation_history.records().len(), 1);
    }

    #[test]
    fn stage_validated_generation_records_runtime_fingerprint() {
        let validated = crate::capability::ValidatedAllocations::new(
            3,
            vec![declaration("app.users.v1", 100, None)],
            Some("wasm:abc123".to_string()),
        );

        let staged = ledger()
            .stage_validated_generation(&validated, None)
            .expect("validated generation");

        assert_eq!(
            staged.allocation_history.generations()[0].runtime_fingerprint,
            Some("wasm:abc123".to_string())
        );
    }

    #[test]
    fn strict_committed_integrity_accepts_full_lifecycle() {
        let mut ledger = committed_ledger(0);
        ledger
            .validate_committed_integrity()
            .expect("genesis ledger with no history");

        ledger = ledger
            .stage_validated_generation(
                &validated(0, vec![declaration("app.users.v1", 100, Some(1))]),
                Some(1),
            )
            .expect("first real commit after genesis");
        ledger
            .validate_committed_integrity()
            .expect("first real commit");

        ledger = ledger
            .stage_validated_generation(
                &validated(1, vec![declaration("app.users.v1", 100, Some(1))]),
                Some(2),
            )
            .expect("repeated active declaration");
        ledger
            .validate_committed_integrity()
            .expect("repeated active declaration");
        assert_eq!(record(&ledger, "app.users.v1").schema_history.len(), 1);

        ledger = ledger
            .stage_validated_generation(
                &validated(2, vec![declaration("app.users.v1", 100, Some(2))]),
                Some(3),
            )
            .expect("schema drift");
        ledger
            .validate_committed_integrity()
            .expect("schema metadata drift");
        assert_eq!(record(&ledger, "app.users.v1").schema_history.len(), 2);

        ledger = ledger
            .stage_reservation_generation(
                &[declaration("app.future_store.v1", 101, Some(1))],
                Some(4),
            )
            .expect("reservation-only generation");
        ledger
            .validate_committed_integrity()
            .expect("reservation-only generation");
        assert_eq!(
            record(&ledger, "app.future_store.v1").state,
            AllocationState::Reserved
        );

        ledger = ledger
            .stage_validated_generation(
                &validated(4, vec![declaration("app.future_store.v1", 101, Some(2))]),
                Some(5),
            )
            .expect("reservation activation");
        ledger
            .validate_committed_integrity()
            .expect("reservation activation");
        assert_eq!(
            record(&ledger, "app.future_store.v1").state,
            AllocationState::Active
        );

        let retirement = AllocationRetirement::new(
            "app.users.v1",
            AllocationSlotDescriptor::memory_manager(100).expect("usable slot"),
        )
        .expect("retirement");
        ledger = ledger
            .stage_retirement_generation(&retirement, Some(6))
            .expect("retirement generation");
        ledger
            .validate_committed_integrity()
            .expect("retirement generation");
        assert_eq!(ledger.current_generation, 6);
        assert_eq!(
            record(&ledger, "app.users.v1").state,
            AllocationState::Retired
        );
        assert_eq!(
            record(&ledger, "app.future_store.v1").last_seen_generation,
            5
        );
    }

    #[derive(Clone, Copy, Debug)]
    enum Transition {
        DeclareUsers,
        DeclareOrders,
        ReserveAudit,
        ActivateAudit,
        RetireUsers,
        RetireAudit,
        EmptyValidated,
        EmptyReservation,
    }

    const TRANSITIONS: [Transition; 8] = [
        Transition::DeclareUsers,
        Transition::DeclareOrders,
        Transition::ReserveAudit,
        Transition::ActivateAudit,
        Transition::RetireUsers,
        Transition::RetireAudit,
        Transition::EmptyValidated,
        Transition::EmptyReservation,
    ];

    fn apply_transition(
        ledger: &AllocationLedger,
        transition: Transition,
        committed_at: u64,
    ) -> Result<AllocationLedger, String> {
        match transition {
            Transition::DeclareUsers => ledger
                .stage_validated_generation(
                    &validated(
                        ledger.current_generation,
                        vec![declaration("app.users.v1", 100, Some(1))],
                    ),
                    Some(committed_at),
                )
                .map_err(|err| err.to_string()),
            Transition::DeclareOrders => ledger
                .stage_validated_generation(
                    &validated(
                        ledger.current_generation,
                        vec![declaration("app.orders.v1", 101, Some(1))],
                    ),
                    Some(committed_at),
                )
                .map_err(|err| err.to_string()),
            Transition::ReserveAudit => ledger
                .stage_reservation_generation(
                    &[declaration("app.audit.v1", 102, Some(1))],
                    Some(committed_at),
                )
                .map_err(|err| err.to_string()),
            Transition::ActivateAudit => ledger
                .stage_validated_generation(
                    &validated(
                        ledger.current_generation,
                        vec![declaration("app.audit.v1", 102, Some(1))],
                    ),
                    Some(committed_at),
                )
                .map_err(|err| err.to_string()),
            Transition::RetireUsers => ledger
                .stage_retirement_generation(
                    &AllocationRetirement::new(
                        "app.users.v1",
                        AllocationSlotDescriptor::memory_manager(100).expect("usable slot"),
                    )
                    .expect("retirement"),
                    Some(committed_at),
                )
                .map_err(|err| err.to_string()),
            Transition::RetireAudit => ledger
                .stage_retirement_generation(
                    &AllocationRetirement::new(
                        "app.audit.v1",
                        AllocationSlotDescriptor::memory_manager(102).expect("usable slot"),
                    )
                    .expect("retirement"),
                    Some(committed_at),
                )
                .map_err(|err| err.to_string()),
            Transition::EmptyValidated => ledger
                .stage_validated_generation(
                    &validated(ledger.current_generation, Vec::new()),
                    Some(committed_at),
                )
                .map_err(|err| err.to_string()),
            Transition::EmptyReservation => ledger
                .stage_reservation_generation(&[], Some(committed_at))
                .map_err(|err| err.to_string()),
        }
    }

    fn check_transition_sequences(
        ledger: AllocationLedger,
        depth: usize,
        sequence: &mut Vec<Transition>,
    ) {
        ledger
            .validate_committed_integrity()
            .unwrap_or_else(|err| panic!("sequence {sequence:?} violated integrity: {err}"));
        if depth == 0 {
            return;
        }

        for transition in TRANSITIONS {
            sequence.push(transition);
            let next = apply_transition(&ledger, transition, sequence.len() as u64)
                .unwrap_or_else(|_| ledger.clone());
            check_transition_sequences(next, depth - 1, sequence);
            sequence.pop();
        }
    }

    #[test]
    fn transition_matrix_preserves_committed_invariants() {
        check_transition_sequences(committed_ledger(0), 4, &mut Vec::new());
    }

    #[test]
    fn new_committed_requires_strict_generation_history() {
        let structurally_valid =
            AllocationLedger::new(3, AllocationHistory::default()).expect("structurally valid DTO");

        assert_eq!(structurally_valid.current_generation, 3);

        let err = AllocationLedger::new_committed(3, AllocationHistory::default())
            .expect_err("committed ledger needs generation history");

        assert_eq!(
            err,
            LedgerIntegrityError::MissingCurrentGenerationRecord {
                current_generation: 3
            }
        );
    }

    #[test]
    fn validate_integrity_rejects_duplicate_stable_keys() {
        let mut ledger = ledger();
        *ledger.allocation_history.records_mut() = vec![
            active_record("app.users.v1", 100),
            active_record("app.users.v1", 101),
        ];

        let err = ledger.validate_integrity().expect_err("duplicate key");

        assert!(matches!(
            err,
            LedgerIntegrityError::DuplicateStableKey { .. }
        ));
    }

    #[test]
    fn validate_integrity_rejects_duplicate_slots() {
        let mut ledger = ledger();
        *ledger.allocation_history.records_mut() = vec![
            active_record("app.users.v1", 100),
            active_record("app.orders.v1", 100),
        ];

        let err = ledger.validate_integrity().expect_err("duplicate slot");

        assert!(matches!(err, LedgerIntegrityError::DuplicateSlot { .. }));
    }

    #[test]
    fn validate_committed_integrity_rejects_decoded_invalid_stable_key() {
        let mut ledger = committed_ledger(1);
        ledger
            .allocation_history
            .records
            .push(active_record("app.users.v1", 100));
        let mut bytes = crate::test_cbor::to_vec(&ledger).expect("encode ledger");
        let key_start = bytes
            .windows(b"app.users.v1".len())
            .position(|window| window == b"app.users.v1")
            .expect("encoded stable key");
        bytes[key_start] = b'A';
        let decoded: AllocationLedger =
            crate::test_cbor::from_slice(&bytes).expect("decode ledger");

        let err = decoded
            .validate_committed_integrity()
            .expect_err("invalid decoded key must fail");

        assert!(matches!(err, LedgerIntegrityError::InvalidStableKey(_)));
    }

    #[test]
    fn validate_committed_integrity_rejects_decoded_invalid_memory_manager_slot() {
        let mut ledger = committed_ledger(1);
        let mut record = active_record("app.users.v1", 100);
        record.slot = AllocationSlotDescriptor::memory_manager_unchecked(MEMORY_MANAGER_INVALID_ID);
        ledger.allocation_history.records.push(record);

        let err = ledger
            .validate_committed_integrity()
            .expect_err("invalid decoded slot must fail");

        assert!(matches!(
            err,
            LedgerIntegrityError::InvalidSlotDescriptor(
                MemoryManagerSlotError::InvalidMemoryManagerId { id }
            ) if id == MEMORY_MANAGER_INVALID_ID
        ));
    }

    #[test]
    fn validate_integrity_rejects_retired_record_without_retired_generation() {
        let mut ledger = ledger();
        let mut record = active_record("app.users.v1", 100);
        record.state = AllocationState::Retired;
        *ledger.allocation_history.records_mut() = vec![record];

        let err = ledger
            .validate_integrity()
            .expect_err("missing retired generation");

        assert!(matches!(
            err,
            LedgerIntegrityError::MissingRetiredGeneration { .. }
        ));
    }

    #[test]
    fn validate_integrity_rejects_non_retired_record_with_retired_generation() {
        let mut ledger = ledger();
        let mut record = active_record("app.users.v1", 100);
        record.retired_generation = Some(2);
        *ledger.allocation_history.records_mut() = vec![record];

        let err = ledger
            .validate_integrity()
            .expect_err("unexpected retired generation");

        assert!(matches!(
            err,
            LedgerIntegrityError::UnexpectedRetiredGeneration { .. }
        ));
    }

    #[test]
    fn validate_integrity_rejects_retirement_before_last_observation() {
        let mut ledger = committed_ledger(5);
        let mut record = active_record("app.users.v1", 100);
        record.last_seen_generation = 4;
        record.state = AllocationState::Retired;
        record.retired_generation = Some(3);
        *ledger.allocation_history.records_mut() = vec![record];

        let err = ledger
            .validate_committed_integrity()
            .expect_err("retirement must follow the last observation");

        assert!(matches!(
            err,
            LedgerIntegrityError::RetirementNotAfterLastSeen { .. }
        ));
    }

    #[test]
    fn validate_integrity_requires_schema_history_at_first_generation() {
        let mut ledger = committed_ledger(2);
        let mut record = active_record("app.users.v1", 100);
        record.last_seen_generation = 2;
        record.schema_history[0].generation = 2;
        *ledger.allocation_history.records_mut() = vec![record];

        let err = ledger
            .validate_committed_integrity()
            .expect_err("schema history must begin with the allocation");

        assert!(matches!(
            err,
            LedgerIntegrityError::SchemaHistoryStartMismatch { .. }
        ));
    }

    #[test]
    fn validate_integrity_rejects_schema_history_after_last_observation() {
        let mut ledger = committed_ledger(3);
        let mut record = active_record("app.users.v1", 100);
        record.last_seen_generation = 2;
        record.schema_history.push(
            SchemaMetadataRecord::new(3, SchemaMetadata::new(Some(2)).expect("schema"))
                .expect("schema record"),
        );
        *ledger.allocation_history.records_mut() = vec![record];

        let err = ledger
            .validate_committed_integrity()
            .expect_err("schema metadata cannot postdate the last observation");

        assert!(matches!(
            err,
            LedgerIntegrityError::SchemaHistoryAfterLastSeen { .. }
        ));
    }

    #[test]
    fn validate_integrity_rejects_non_increasing_schema_history() {
        let mut ledger = ledger();
        let mut record = active_record("app.users.v1", 100);
        record.schema_history.push(SchemaMetadataRecord {
            generation: 1,
            schema: SchemaMetadata::default(),
        });
        *ledger.allocation_history.records_mut() = vec![record];

        let err = ledger
            .validate_integrity()
            .expect_err("non-increasing schema history");

        assert!(matches!(
            err,
            LedgerIntegrityError::NonIncreasingSchemaHistory { .. }
        ));
    }

    #[test]
    fn validate_integrity_rejects_invalid_schema_metadata_history() {
        let mut ledger = committed_ledger(1);
        let mut record = active_record("app.users.v1", 100);
        record.schema_history[0].schema = invalid_schema_metadata();
        *ledger.allocation_history.records_mut() = vec![record];

        let err = ledger
            .validate_committed_integrity()
            .expect_err("invalid committed schema metadata");

        assert_eq!(
            err,
            LedgerIntegrityError::InvalidSchemaMetadata {
                stable_key: StableKey::parse("app.users.v1").expect("stable key"),
                generation: 1,
                error: SchemaMetadataError::InvalidVersion,
            }
        );
    }

    #[test]
    fn validate_committed_integrity_requires_current_generation_record() {
        let err = ledger()
            .validate_committed_integrity()
            .expect_err("missing current generation");

        assert_eq!(
            err,
            LedgerIntegrityError::MissingCurrentGenerationRecord {
                current_generation: 3
            }
        );
    }

    #[test]
    fn validate_committed_integrity_rejects_generation_history_gaps() {
        let mut ledger = committed_ledger(3);
        ledger.allocation_history.generations_mut().remove(1);

        let err = ledger
            .validate_committed_integrity()
            .expect_err("generation history gap");

        assert!(matches!(
            err,
            LedgerIntegrityError::NonIncreasingGenerationRecords { .. }
        ));
    }

    #[test]
    fn ledger_commit_store_rejects_invalid_ledger_before_write() {
        let mut store = LedgerCommitStore::default();
        let mut invalid = ledger();
        *invalid.allocation_history.records_mut() = vec![
            active_record("app.users.v1", 100),
            active_record("app.orders.v1", 100),
        ];

        let err = store.commit(&invalid).expect_err("invalid ledger");

        assert!(matches!(
            err,
            LedgerCommitError::Integrity(LedgerIntegrityError::DuplicateSlot { .. })
        ));
        assert!(store.physical().is_uninitialized());
    }

    #[test]
    fn ledger_commit_store_recovers_latest_committed_ledger() {
        let mut store = LedgerCommitStore::default();
        let first = committed_ledger(1);
        let second = committed_ledger(2);

        store.commit(&first).expect("first commit");
        store.commit(&second).expect("second commit");
        let recovered = store.recover().expect("recovered ledger");

        assert_eq!(recovered.current_generation(), 2);
    }

    #[test]
    fn ledger_commit_store_wraps_logical_payload_in_envelope() {
        let mut store = LedgerCommitStore::default();
        let ledger = committed_ledger(1);

        store.commit(&ledger).expect("commit");

        let committed = store.physical().authoritative().expect("authoritative");
        let envelope = LedgerPayloadEnvelope::decode(committed.payload()).expect("envelope");
        assert_eq!(
            envelope.payload(),
            CborLedgerCodec.encode(&ledger).expect("payload")
        );
    }

    #[test]
    fn ledger_commit_store_rejects_bad_payload_envelope_before_ledger_decode() {
        let committed = CommittedGenerationBytes::new(1, vec![b'X'; 28]);
        let store = LedgerCommitStore {
            physical: DualCommitStore {
                slot0: Some(committed),
                slot1: None,
            },
        };

        let err = store.recover().expect_err("bad envelope");

        assert!(matches!(
            err,
            LedgerCommitError::PayloadEnvelope(LedgerPayloadEnvelopeError::BadMagic { .. })
        ));
    }

    #[test]
    fn ledger_commit_store_rejects_old_versioned_payload_envelope_shape() {
        let store = LedgerCommitStore {
            physical: DualCommitStore {
                slot0: Some(CommittedGenerationBytes::new(
                    1,
                    old_versioned_enveloped_payload(&committed_ledger(1)),
                )),
                slot1: None,
            },
        };

        let err = store.recover().expect_err("old envelope shape must fail");

        assert!(matches!(
            err,
            LedgerCommitError::PayloadEnvelope(
                LedgerPayloadEnvelopeError::PayloadLengthOverflow { .. }
                    | LedgerPayloadEnvelopeError::LengthMismatch { .. }
            )
        ));
    }

    #[test]
    fn ledger_commit_store_recovers_current_format_genesis_and_first_real_commit() {
        let mut store = LedgerCommitStore::default();
        let genesis = committed_ledger(0);

        let recovered = store
            .recover_or_initialize(&genesis)
            .expect("current-format genesis ledger");
        assert_eq!(recovered.current_generation(), 0);
        assert!(
            recovered
                .ledger()
                .allocation_history()
                .generations()
                .is_empty()
        );

        let first = recovered
            .ledger()
            .stage_validated_generation(
                &validated(0, vec![declaration("app.users.v1", 100, Some(1))]),
                None,
            )
            .expect("first real generation");
        let recovered = store.commit(&first).expect("first commit");

        assert_eq!(recovered.current_generation(), 1);
        assert_eq!(
            recovered.ledger().allocation_history().generations()[0].generation,
            1
        );
        assert_eq!(
            record(recovered.ledger(), "app.users.v1").first_generation,
            1
        );
    }

    #[test]
    fn ledger_commit_store_recovers_full_payload_after_corrupt_latest_slot() {
        let mut store = LedgerCommitStore::default();
        let genesis = committed_ledger(0);
        store.commit(&genesis).expect("genesis commit");
        let first = genesis
            .stage_validated_generation(
                &validated(0, vec![declaration("app.users.v1", 100, Some(1))]),
                None,
            )
            .expect("first generation");
        let first = store.commit(&first).expect("first commit");
        let second = first
            .ledger()
            .stage_validated_generation(
                &validated(1, vec![declaration("app.users.v1", 100, Some(2))]),
                None,
            )
            .expect("second generation");

        store
            .write_corrupt_inactive_ledger(&second)
            .expect("corrupt latest");
        let recovered = store.recover().expect("recover prior generation");

        assert_eq!(recovered.current_generation(), 1);
        assert_eq!(
            record(recovered.ledger(), "app.users.v1")
                .schema_history
                .len(),
            1
        );
    }

    #[test]
    fn ledger_commit_store_recovers_identical_duplicate_slots() {
        let ledger = committed_ledger(0)
            .stage_validated_generation(
                &validated(0, vec![declaration("app.users.v1", 100, Some(1))]),
                None,
            )
            .expect("first generation");
        let payload = enveloped_payload(&ledger);
        let committed = CommittedGenerationBytes::new(ledger.current_generation, payload);
        let store = LedgerCommitStore {
            physical: DualCommitStore {
                slot0: Some(committed.clone()),
                slot1: Some(committed),
            },
        };

        let recovered = store.recover().expect("recovered");

        assert_eq!(recovered.ledger(), &ledger);
    }

    #[test]
    fn ledger_commit_store_ignores_corrupt_inactive_ledger() {
        let mut store = LedgerCommitStore::default();
        let first = committed_ledger(1);
        let second = committed_ledger(2);

        store.commit(&first).expect("first commit");
        store
            .write_corrupt_inactive_ledger(&second)
            .expect("corrupt write");
        let recovered = store.recover().expect("recovered ledger");

        assert_eq!(recovered.current_generation(), 1);
    }

    #[test]
    fn ledger_commit_store_rejects_physical_logical_generation_mismatch() {
        let store = LedgerCommitStore {
            physical: DualCommitStore {
                slot0: Some(CommittedGenerationBytes::new(
                    7,
                    enveloped_payload(&committed_ledger(6)),
                )),
                slot1: None,
            },
        };
        let err = store.recover().expect_err("mismatch");

        assert_eq!(
            err,
            LedgerCommitError::PhysicalLogicalGenerationMismatch {
                physical_generation: 7,
                logical_generation: 6
            }
        );
    }

    #[test]
    fn ledger_commit_store_rejects_non_next_logical_generation() {
        let mut store = LedgerCommitStore::default();
        store.commit(&committed_ledger(1)).expect("first commit");

        let err = store
            .commit(&committed_ledger(3))
            .expect_err("skipped generation");

        assert_eq!(
            err,
            LedgerCommitError::Recovery(CommitRecoveryError::UnexpectedGeneration {
                expected: 2,
                actual: 3
            })
        );
    }

    #[test]
    fn ledger_commit_store_initializes_empty_store_explicitly() {
        let mut store = LedgerCommitStore::default();
        let genesis = committed_ledger(3);

        let recovered = store
            .recover_or_initialize(&genesis)
            .expect("initialized ledger");

        assert_eq!(recovered.current_generation(), 3);
        assert!(!store.physical().is_uninitialized());
    }

    #[test]
    fn ledger_commit_store_rejects_corrupt_store_even_with_genesis() {
        let mut store = LedgerCommitStore::default();
        store
            .write_corrupt_inactive_ledger(&ledger())
            .expect("corrupt write");

        let err = store
            .recover_or_initialize(&ledger())
            .expect_err("corrupt state");

        assert!(matches!(
            err,
            LedgerCommitError::Recovery(CommitRecoveryError::NoValidGeneration)
        ));
    }
}
