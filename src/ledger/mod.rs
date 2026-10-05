mod claim;
mod error;
mod integrity;
mod payload;
mod record;
mod stage;

use crate::physical::{
    CommitRecoveryError, CommitStoreDiagnostic, CommittedGenerationBytes, DualCommitStore,
};
use serde::{Deserialize, Serialize};

pub use claim::{
    ClaimConflict, ClaimOutcome, ReservationClaimConflict, validate_declaration_claim,
    validate_reservation_claim,
};
pub use error::{
    AllocationReservationError, AllocationRetirementError, AllocationStageError, LedgerCommitError,
    LedgerIntegrityError,
};
pub use payload::{
    LEDGER_PAYLOAD_FORMAT_VERSION, LedgerPayloadEnvelope, LedgerPayloadEnvelopeError,
};
pub use record::{
    AllocationHistory, AllocationLedger, AllocationRecord, AllocationRetirement, AllocationState,
    GenerationRecord, RecoveredLedger, SchemaMetadataRecord,
};
pub use stage::checked_reservation_count;
pub use stage::{
    stage_reservation_generation, stage_retirement_generation, stage_validated_generation,
};

fn decode_ledger(bytes: &[u8]) -> Result<AllocationLedger, String> {
    crate::cbor::from_slice_exact(bytes).map_err(|err| err.to_string())
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
        Self::recover_committed(committed)
    }

    pub(crate) fn recover_with_diagnostic(
        &self,
    ) -> (
        Result<RecoveredLedger, LedgerCommitError>,
        CommitStoreDiagnostic,
    ) {
        let (committed, diagnostic) = self.physical.authoritative_with_diagnostic();
        let recovered = committed
            .map_err(LedgerCommitError::Recovery)
            .and_then(Self::recover_committed);
        (recovered, diagnostic)
    }

    fn recover_committed(
        committed: &CommittedGenerationBytes,
    ) -> Result<RecoveredLedger, LedgerCommitError> {
        let payload = LedgerPayloadEnvelope::decode_payload(committed.payload())
            .map_err(LedgerCommitError::PayloadEnvelope)?;
        let ledger = decode_ledger(payload).map_err(LedgerCommitError::Codec)?;
        if committed.generation() != ledger.current_generation {
            return Err(LedgerCommitError::PhysicalLogicalGenerationMismatch {
                physical_generation: committed.generation(),
                logical_generation: ledger.current_generation,
            });
        }
        ledger
            .validate_committed_integrity()
            .map_err(LedgerCommitError::Integrity)?;
        Ok(RecoveredLedger::from_trusted_ledger(ledger))
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
            // Physical selection returns this error only when both slots are absent.
            Err(LedgerCommitError::Recovery(CommitRecoveryError::NoValidGeneration)) => {
                self.commit(genesis)
            }
            Err(err) => Err(err),
        }
    }

    /// Commit one logical allocation ledger generation through the native CBOR ledger codec.
    ///
    /// # Panics
    ///
    /// Panics only if the private concrete ledger-encoding invariant is broken.
    pub fn commit(
        &mut self,
        ledger: &AllocationLedger,
    ) -> Result<RecoveredLedger, LedgerCommitError> {
        self.commit_generation(ledger)?;
        // The current writer encoded this integrity-checked ledger, and the
        // physical commit checked its predecessor and established this generation.
        // Existing persisted bytes still cross the full recovery boundary.
        Ok(RecoveredLedger::from_trusted_ledger(ledger.clone()))
    }

    // Bootstrap already owns its staged ledger. Share the checked mutation
    // without cloning that ledger into a proof only to unwrap it immediately.
    pub(crate) fn commit_generation(
        &mut self,
        ledger: &AllocationLedger,
    ) -> Result<(), LedgerCommitError> {
        ledger
            .validate_committed_integrity()
            .map_err(LedgerCommitError::Integrity)?;
        let payload =
            LedgerPayloadEnvelope::encode_ledger(ledger).map_err(LedgerCommitError::Integrity)?;
        self.physical
            .commit_payload_at_generation(ledger.current_generation, payload)
            .map(|_| ())
            .map_err(LedgerCommitError::Recovery)
    }

    /// Simulate corruption of a logical ledger payload in the inactive slot.
    #[cfg(test)]
    pub fn write_corrupt_inactive_ledger(
        &mut self,
        ledger: &AllocationLedger,
    ) -> Result<(), LedgerCommitError> {
        let payload =
            LedgerPayloadEnvelope::encode_ledger(ledger).map_err(LedgerCommitError::Integrity)?;
        self.physical
            .write_corrupt_inactive_slot(ledger.current_generation, payload);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_cbor::hex_fixture;
    use crate::{
        declaration::{AllocationDeclaration, DeclarationSnapshot, DeclarationSnapshotError},
        key::StableKey,
        physical::CommittedGenerationBytes,
        schema::SchemaMetadata,
        slot::MemoryManagerSlot,
    };
    fn declaration(key: &str, id: u8, schema_version: Option<u32>) -> AllocationDeclaration {
        AllocationDeclaration::new(
            key,
            MemoryManagerSlot::new(id).expect("usable slot"),
            None,
            SchemaMetadata::new(schema_version).expect("schema metadata"),
        )
        .expect("declaration")
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
        AllocationRecord::active(1, &declaration(key, id, None))
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
        LedgerPayloadEnvelope::encode_ledger(ledger).expect("fixture envelope")
    }

    fn ledger_from_payload_fixture(contents: &str) -> AllocationLedger {
        let bytes = hex_fixture(contents);
        let envelope = LedgerPayloadEnvelope::decode(&bytes).expect("fixture envelope");

        let ledger = decode_ledger(envelope.payload()).expect("fixture ledger");
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

    #[test]
    fn recovery_rejects_unknown_fields_inside_retirement_state() {
        use crate::{AllocationRetirement, StableCellLedgerRecord};

        let active = active_committed_ledger();
        let retirement =
            AllocationRetirement::new("app.users.v1", MemoryManagerSlot::new(100).unwrap())
                .unwrap();
        let retired = active
            .stage_retirement_generation(&retirement, None)
            .unwrap();
        let mut store = LedgerCommitStore::default();
        let committed = store.commit(&retired).unwrap();
        assert_eq!(committed, store.recover().unwrap());

        let mut value = crate::test_cbor::to_value(&retired).unwrap();
        let history = map_field_mut(value_map_mut(&mut value), "allocation_history");
        let records = value_array_mut(map_field_mut(value_map_mut(history), "records"));
        let state = map_field_mut(value_map_mut(&mut records[0]), "state");
        let retired_state = map_field_mut(value_map_mut(state), "Retired");
        value_map_mut(retired_state).push(("unexpected".into(), true.into()));
        let bytes = crate::test_cbor::to_vec(&value).unwrap();
        let payload = LedgerPayloadEnvelope::current(bytes).try_encode().unwrap();
        let mut physical = DualCommitStore::default();
        physical.commit_payload_at_generation(2, payload).unwrap();
        let record = StableCellLedgerRecord::new(LedgerCommitStore { physical });
        let encoded = crate::test_cbor::to_vec(&record).unwrap();
        let decoded = crate::decode_stable_cell_ledger_record(&encoded).unwrap();
        let before = decoded.clone();
        let error = decoded.store().recover().unwrap_err();
        assert!(matches!(error, LedgerCommitError::Codec(_)));
        assert!(error.to_string().contains("unexpected"));
        assert_eq!(decoded, before);
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

    fn remove_map_field(
        map: &mut Vec<(crate::test_cbor::Value, crate::test_cbor::Value)>,
        field: &str,
    ) {
        let field = crate::test_cbor::Value::Text(field.to_string());
        let index = map
            .iter()
            .position(|(key, _)| key == &field)
            .expect("CBOR map field");
        map.remove(index);
    }

    fn value_array_mut(value: &mut crate::test_cbor::Value) -> &mut Vec<crate::test_cbor::Value> {
        let crate::test_cbor::Value::Array(values) = value else {
            panic!("expected CBOR array");
        };
        values
    }

    fn decode_mutated_ledger(value: crate::test_cbor::Value) -> String {
        let bytes = crate::test_cbor::to_vec(&value).expect("mutated ledger bytes");
        decode_ledger(&bytes).expect_err("mutated ledger must fail closed")
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
    fn generation_record_constructor_rejects_empty_runtime_fingerprint() {
        let generation_err = GenerationRecord::new(1, 0, Some(String::new()), 0, None)
            .expect_err("empty fingerprint must fail");
        assert_eq!(
            generation_err,
            DeclarationSnapshotError::EmptyRuntimeFingerprint
        );
    }

    #[test]
    fn ledger_cbor_round_trips_allocation_ledger() {
        let ledger = committed_ledger(2);

        let encoded = crate::test_cbor::to_vec(&ledger).expect("encode ledger");
        let decoded = decode_ledger(&encoded).expect("decode ledger");

        assert_eq!(decoded, ledger);
    }

    #[test]
    fn ledger_cbor_rejects_trailing_bytes() {
        let mut encoded = crate::test_cbor::to_vec(&committed_ledger(2)).expect("encode ledger");
        encoded.push(0);

        let err = decode_ledger(&encoded).expect_err("trailing bytes must fail closed");

        assert!(err.contains("trailing bytes"));
    }

    #[test]
    fn cbor_ledger_codec_rejects_unknown_top_level_fields() {
        use crate::test_cbor::Value;

        let map = vec![
            (
                Value::Text("current_generation".to_string()),
                Value::Integer(0.into()),
            ),
            (
                Value::Text("allocation_history".to_string()),
                crate::test_cbor::to_value(AllocationHistory::default()).expect("history value"),
            ),
            (Value::Text("future_field".to_string()), Value::Bool(true)),
        ];
        let bytes = crate::test_cbor::to_vec(&Value::Map(map)).expect("unknown-field ledger");

        let err = decode_ledger(&bytes).expect_err("unknown ledger field must fail closed");

        assert!(err.contains("future_field"));
    }

    #[test]
    fn cbor_ledger_codec_rejects_unknown_nested_history_fields() {
        let mut value = active_ledger_value();
        let history = map_field_mut(value_map_mut(&mut value), "allocation_history");
        value_map_mut(history).push((
            crate::test_cbor::Value::Text("future_history_field".to_string()),
            crate::test_cbor::Value::Bool(true),
        ));

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
        value_map_mut(record).push((
            crate::test_cbor::Value::Text("future_record_field".to_string()),
            crate::test_cbor::Value::Bool(true),
        ));

        let err = decode_mutated_ledger(value);

        assert!(err.contains("future_record_field"));
    }

    #[test]
    fn cbor_ledger_codec_rejects_unknown_nested_slot_fields() {
        let mut value = active_ledger_value();
        let history = map_field_mut(value_map_mut(&mut value), "allocation_history");
        let records = map_field_mut(value_map_mut(history), "records");
        let record = value_array_mut(records)
            .first_mut()
            .expect("allocation record");
        let slot = map_field_mut(value_map_mut(record), "slot");
        value_map_mut(slot).push((
            crate::test_cbor::Value::Text("future_slot_field".to_string()),
            crate::test_cbor::Value::Bool(true),
        ));

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
        value_map_mut(generation).push((
            crate::test_cbor::Value::Text("future_generation_field".to_string()),
            crate::test_cbor::Value::Bool(true),
        ));

        let err = decode_mutated_ledger(value);

        assert!(err.contains("future_generation_field"));
    }

    #[test]
    fn cbor_ledger_codec_requires_generation_optional_fields() {
        for field in ["runtime_fingerprint", "committed_at"] {
            let mut value = active_ledger_value();
            let history = map_field_mut(value_map_mut(&mut value), "allocation_history");
            let generations = map_field_mut(value_map_mut(history), "generations");
            let generation = value_array_mut(generations)
                .first_mut()
                .expect("generation record");
            remove_map_field(value_map_mut(generation), field);

            let err = decode_mutated_ledger(value);

            assert!(err.contains(field));
        }
    }

    #[test]
    fn cbor_ledger_codec_requires_schema_version_field() {
        let mut value = active_ledger_value();
        let history = map_field_mut(value_map_mut(&mut value), "allocation_history");
        let records = map_field_mut(value_map_mut(history), "records");
        let record = value_array_mut(records)
            .first_mut()
            .expect("allocation record");
        let schema_history = map_field_mut(value_map_mut(record), "schema_history");
        let schema_record = value_array_mut(schema_history)
            .first_mut()
            .expect("schema record");
        let schema = map_field_mut(value_map_mut(schema_record), "schema");
        remove_map_field(value_map_mut(schema), "schema_version");

        let err = decode_mutated_ledger(value);

        assert!(err.contains("schema_version"));
    }

    #[test]
    fn ledger_commit_store_rejects_unknown_top_level_fields() {
        use crate::test_cbor::Value;

        let map = vec![
            (
                Value::Text("physical".to_string()),
                crate::test_cbor::to_value(DualCommitStore::default()).expect("physical value"),
            ),
            (Value::Text("future_field".to_string()), Value::Bool(true)),
        ];
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
        assert_eq!(ledger.allocation_history.records(), []);
        assert_eq!(ledger.allocation_history.generations(), []);
    }

    #[test]
    fn current_single_active_payload_fixture_recovers() {
        let ledger = ledger_from_payload_fixture(include_str!(
            "../../fixtures/current/single_active_allocation_payload_envelope.hex"
        ));
        let record = record(&ledger, "app.users.v1");

        assert_eq!(ledger.current_generation, 1);
        assert_eq!(record.state(), AllocationState::Active);
        assert_eq!(record.slot().id(), 100);
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
        assert_eq!(record.slot().id(), 101);
    }

    #[test]
    fn current_retired_payload_fixture_recovers() {
        let ledger = ledger_from_payload_fixture(include_str!(
            "../../fixtures/current/retired_allocation_payload_envelope.hex"
        ));
        let record = record(&ledger, "app.users.v1");

        assert_eq!(ledger.current_generation, 2);
        assert_eq!(record.state(), AllocationState::Retired { generation: 2 });
    }

    #[test]
    fn current_memory_manager_slot_fixture_decodes() {
        let bytes = hex_fixture(include_str!(
            "../../fixtures/current/memory_manager_slot.cbor.hex"
        ));
        let slot: MemoryManagerSlot = crate::test_cbor::from_slice(&bytes).expect("slot fixture");

        assert_eq!(slot.id(), 100);
        assert_eq!(
            bytes,
            crate::test_cbor::to_vec(&slot).expect("re-encoded slot")
        );
    }

    #[test]
    fn current_ledger_commit_store_single_active_fixture_recovers() {
        let store = store_from_fixture(include_str!(
            "../../fixtures/current/ledger_commit_store_single_active.cbor.hex"
        ));

        let recovered = store.recover().expect("fixture store recovers");
        assert_eq!(recovered.current_generation(), 1);
        assert_eq!(record(recovered.ledger(), "app.users.v1").slot().id(), 100);
    }

    #[test]
    fn current_dual_slot_valid_newer_fixture_recovers_newer() {
        let store = store_from_fixture(include_str!(
            "../../fixtures/current/dual_slot_store_valid_newer.cbor.hex"
        ));

        assert_eq!(store.physical().diagnostic().recovery, Ok(2));
        let recovered = store.recover().expect("fixture store recovers");
        assert_eq!(recovered.current_generation(), 2);
    }

    #[test]
    fn current_dual_slot_corrupt_newer_fixture_fails_closed() {
        let store = store_from_fixture(include_str!(
            "../../fixtures/current/dual_slot_store_corrupt_newer.cbor.hex"
        ));

        let diagnostic = store.physical().diagnostic();
        assert_eq!(
            diagnostic.recovery,
            Err(CommitRecoveryError::InvalidCommitSlots {
                slot0_invalid: false,
                slot1_invalid: true,
            })
        );
        assert_eq!(
            diagnostic.slot1,
            crate::CommitSlotDiagnostic::Invalid { generation: 2 }
        );
        let err = store
            .recover()
            .expect_err("corrupt fixture must fail closed");
        assert!(matches!(
            err,
            LedgerCommitError::Recovery(CommitRecoveryError::InvalidCommitSlots {
                slot0_invalid: false,
                slot1_invalid: true,
            })
        ));
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
        assert_eq!(staged.allocation_history.records(), []);
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
    fn stage_validated_generation_rejects_overflowing_receiver_as_stale() {
        let mut store = LedgerCommitStore::default();
        let recovered = store.commit(&committed_ledger(0)).unwrap();
        let validated = crate::validate_allocations(
            &recovered,
            DeclarationSnapshot::new(vec![declaration("app.users.v1", 100, Some(1))]).unwrap(),
            &crate::GenericRangePolicy,
        )
        .unwrap();
        let ledger = AllocationLedger::new(u64::MAX, AllocationHistory::default()).unwrap();
        let before = ledger.clone();

        let err = ledger
            .stage_validated_generation(&validated, None)
            .expect_err("receiver cannot match a bounded recovered generation");

        assert_eq!(
            err,
            AllocationStageError::StaleValidatedAllocations {
                validated_generation: 0,
                ledger_generation: u64::MAX,
            }
        );
        assert_eq!(ledger, before);
    }

    #[test]
    fn stage_validated_generation_rejects_same_key_different_slot() {
        let mut ledger = committed_ledger(3);
        ledger.allocation_history.records = vec![active_record("app.users.v1", 100)];
        ledger.validate_committed_integrity().unwrap();
        // A proof minted against another valid history at the same generation
        // cannot override this receiver's allocation facts.
        let mut other_store = LedgerCommitStore::default();
        let other_recovered = other_store.commit(&committed_ledger(3)).unwrap();
        let validated = crate::validate_allocations(
            &other_recovered,
            DeclarationSnapshot::new(vec![declaration("app.users.v1", 101, None)]).unwrap(),
            &crate::GenericRangePolicy,
        )
        .unwrap();
        let before = ledger.clone();

        let err = ledger
            .stage_validated_generation(&validated, None)
            .expect_err("stable key cannot move slots");

        assert!(matches!(
            err,
            AllocationStageError::StableKeySlotConflict { .. }
        ));
        assert_eq!(ledger, before);
    }

    #[test]
    fn stage_validated_generation_rejects_same_slot_different_key() {
        let mut ledger = ledger();
        ledger.allocation_history.records = vec![active_record("app.users.v1", 100)];
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
        record.state = AllocationState::Retired { generation: 3 };
        ledger.allocation_history.records = vec![record];
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
        assert_eq!(staged.allocation_history.records(), []);
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
    fn stage_reservation_generation_rejects_same_key_different_slot() {
        let mut ledger = ledger();
        ledger.allocation_history.records = vec![AllocationRecord::reserved(
            3,
            &declaration("app.future_store.v1", 100, None),
        )];
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
        ledger.allocation_history.records = vec![AllocationRecord::reserved(
            3,
            &declaration("app.future_store.v1", 100, None),
        )];
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
        record.state = AllocationState::Retired { generation: 3 };
        ledger.allocation_history.records = vec![record];
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
            MemoryManagerSlot::new(100).expect("usable slot"),
        )
        .expect("retirement");

        let staged = staged
            .stage_retirement_generation(&retirement, Some(42))
            .expect("retired generation");
        let record = &staged.allocation_history.records()[0];

        assert_eq!(staged.current_generation, 5);
        assert_eq!(record.state, AllocationState::Retired { generation: 5 });
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
            MemoryManagerSlot::new(100).expect("usable slot"),
        )
        .expect("retirement");

        let staged = staged
            .stage_retirement_generation(&retirement, Some(42))
            .expect("reserved retirement generation");
        let record = &staged.allocation_history.records()[0];

        assert_eq!(staged.current_generation, 5);
        assert_eq!(record.state, AllocationState::Retired { generation: 5 });
        assert_eq!(record.first_generation, 4);
        assert_eq!(
            staged.allocation_history.generations()[1].declaration_count(),
            0
        );
    }

    #[test]
    fn stage_retirement_generation_rejects_generation_overflow() {
        let mut ledger = ledger();
        ledger.current_generation = u64::MAX;
        ledger.allocation_history.records = vec![active_record("app.users.v1", 100)];
        let retirement = AllocationRetirement::new(
            "app.users.v1",
            MemoryManagerSlot::new(100).expect("usable slot"),
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
            MemoryManagerSlot::new(101).expect("usable slot"),
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
    fn retirement_rejections_preserve_source_and_slot_error_precedence() {
        let active = validated(3, vec![declaration("app.users.v1", 100, None)]);
        let source = ledger().stage_validated_generation(&active, None).unwrap();
        let retirement =
            AllocationRetirement::new("app.users.v1", MemoryManagerSlot::new(100).unwrap())
                .unwrap();
        let source = source
            .stage_retirement_generation(&retirement, None)
            .unwrap();
        let before = source.clone();

        assert!(matches!(
            source.stage_retirement_generation(&retirement, None),
            Err(AllocationRetirementError::AlreadyRetired { .. })
        ));
        let wrong_slot =
            AllocationRetirement::new("app.users.v1", MemoryManagerSlot::new(101).unwrap())
                .unwrap();
        assert!(matches!(
            source.stage_retirement_generation(&wrong_slot, None),
            Err(AllocationRetirementError::SlotMismatch { .. })
        ));
        let unknown =
            AllocationRetirement::new("app.unknown.v1", MemoryManagerSlot::new(100).unwrap())
                .unwrap();
        assert!(matches!(
            source.stage_retirement_generation(&unknown, None),
            Err(AllocationRetirementError::UnknownStableKey(_))
        ));
        assert_eq!(source, before);
    }

    #[test]
    fn allocation_retirement_constructor_rejects_invalid_key() {
        let err = AllocationRetirement::new("App.users.v1", MemoryManagerSlot::new(100).unwrap())
            .expect_err("invalid key must fail at construction");
        assert!(matches!(err, AllocationRetirementError::Key(_)));
    }

    #[test]
    fn retirement_decode_rejects_unusable_memory_manager_slot() {
        let retirement =
            AllocationRetirement::new("app.users.v1", MemoryManagerSlot::new(100).unwrap())
                .unwrap();
        let mut value = serde_json::to_value(retirement).unwrap();
        value["slot"]["slot"]["MemoryManagerId"] = serde_json::json!(255);
        assert!(serde_json::from_value::<AllocationRetirement>(value).is_err());
    }

    #[test]
    fn retirement_decode_rejects_invalid_stable_key() {
        let retirement = AllocationRetirement::new(
            "app.users.v1",
            MemoryManagerSlot::new(100).expect("usable slot"),
        )
        .expect("retirement");
        let mut value = crate::test_cbor::to_value(retirement).expect("retirement value");
        *map_field_mut(value_map_mut(&mut value), "stable_key") =
            crate::test_cbor::Value::Text("App.users.v1".to_string());
        let bytes = crate::test_cbor::to_vec(&value).expect("retirement bytes");
        assert!(crate::test_cbor::from_slice::<AllocationRetirement>(&bytes).is_err());
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
            MemoryManagerSlot::new(100).expect("usable slot"),
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
            AllocationState::Retired { generation: 6 }
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
                        MemoryManagerSlot::new(100).expect("usable slot"),
                    )
                    .expect("retirement"),
                    Some(committed_at),
                )
                .map_err(|err| err.to_string()),
            Transition::RetireAudit => ledger
                .stage_retirement_generation(
                    &AllocationRetirement::new(
                        "app.audit.v1",
                        MemoryManagerSlot::new(102).expect("usable slot"),
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
    fn new_committed_preserves_structural_rejection_before_history_checks() {
        let history = AllocationHistory::from_parts(
            vec![
                active_record("app.users.v1", 100),
                active_record("app.users.v1", 100),
            ],
            Vec::new(),
        );

        // Duplicate keys take precedence over duplicate slots and the missing
        // current-generation record, even with only one integrity pass.
        assert_eq!(
            AllocationLedger::new_committed(3, history).expect_err("duplicate records"),
            LedgerIntegrityError::DuplicateStableKey {
                stable_key: StableKey::parse("app.users.v1").unwrap(),
            }
        );

        let valid = active_committed_ledger();
        assert_eq!(
            AllocationLedger::new_committed(
                valid.current_generation(),
                valid.allocation_history().clone(),
            )
            .expect("valid committed history"),
            valid
        );
    }

    #[test]
    fn validate_integrity_rejects_duplicate_stable_keys() {
        let mut ledger = ledger();
        for second_id in [101, 100] {
            ledger.allocation_history.records = vec![
                active_record("app.users.v1", 100),
                active_record("app.users.v1", second_id),
            ];

            let err = ledger
                .validate_integrity()
                .expect_err("duplicate key precedes duplicate slot");

            assert_eq!(
                err,
                LedgerIntegrityError::DuplicateStableKey {
                    stable_key: StableKey::parse("app.users.v1").unwrap(),
                }
            );
        }
    }

    #[test]
    fn validate_integrity_rejects_duplicate_slots() {
        let mut ledger = ledger();
        ledger.allocation_history.records = vec![
            active_record("app.users.v1", 100),
            active_record("app.orders.v1", 100),
        ];

        let err = ledger.validate_integrity().expect_err("duplicate slot");

        assert_eq!(
            err,
            LedgerIntegrityError::DuplicateSlot {
                slot: MemoryManagerSlot::new(100).unwrap(),
            }
        );
    }

    #[test]
    fn recovery_rejects_invalid_stable_key_during_ledger_decode() {
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
        assert!(crate::test_cbor::from_slice::<AllocationLedger>(&bytes).is_err());
        let payload = LedgerPayloadEnvelope::current(bytes).encode();
        let mut physical = DualCommitStore::default();
        physical.commit_payload_at_generation(1, payload).unwrap();
        let store = LedgerCommitStore { physical };
        let before = store.clone();
        assert!(matches!(store.recover(), Err(LedgerCommitError::Codec(_))));
        assert_eq!(store, before, "malformed-key recovery must preserve slots");
    }

    #[test]
    fn ledger_decode_rejects_unusable_memory_manager_slot() {
        let mut ledger = committed_ledger(1);
        ledger
            .allocation_history
            .records
            .push(active_record("app.users.v1", 100));
        let mut value = serde_json::to_value(ledger).unwrap();
        value["allocation_history"]["records"][0]["slot"]["slot"]["MemoryManagerId"] =
            serde_json::json!(255);
        let bytes = crate::test_cbor::to_vec(&value).unwrap();
        assert!(decode_ledger(&bytes).is_err());
    }

    #[test]
    fn validate_integrity_rejects_retirement_before_last_observation() {
        let mut ledger = committed_ledger(5);
        let mut record = active_record("app.users.v1", 100);
        record.last_seen_generation = 4;
        record.state = AllocationState::Retired { generation: 3 };
        ledger.allocation_history.records = vec![record];

        let err = ledger
            .validate_committed_integrity()
            .expect_err("retirement must follow the last observation");

        assert!(matches!(
            err,
            LedgerIntegrityError::RetirementNotAfterLastSeen { .. }
        ));
    }

    #[test]
    fn validate_integrity_rejects_retirement_before_allocation() {
        let mut ledger = committed_ledger(2);
        let mut record = active_record("app.users.v1", 100);
        record.state = AllocationState::Retired { generation: 0 };
        ledger.allocation_history.records = vec![record];

        let err = ledger
            .validate_committed_integrity()
            .expect_err("retirement cannot predate allocation");

        assert!(matches!(
            err,
            LedgerIntegrityError::RetiredBeforeFirstGeneration { .. }
        ));
    }

    #[test]
    fn validate_integrity_requires_schema_history_at_first_generation() {
        let mut ledger = committed_ledger(2);
        let mut record = active_record("app.users.v1", 100);
        record.last_seen_generation = 2;
        record.schema_history[0].generation = 2;
        ledger.allocation_history.records = vec![record];

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
        for generation in [3, 4] {
            let mut ledger = committed_ledger(3);
            let mut record = active_record("app.users.v1", 100);
            record.last_seen_generation = 2;
            record.schema_history.push(SchemaMetadataRecord::new(
                generation,
                SchemaMetadata::new(Some(2)).expect("schema"),
            ));
            ledger.allocation_history.records = vec![record];

            let err = ledger
                .validate_committed_integrity()
                .expect_err("schema metadata cannot postdate the last observation");

            assert!(matches!(
                (generation, err),
                (
                    3,
                    LedgerIntegrityError::SchemaHistoryAfterLastSeen {
                        generation: 3,
                        last_seen_generation: 2,
                        ..
                    }
                ) | (
                    4,
                    LedgerIntegrityError::SchemaHistoryOutOfBounds { generation: 4, .. }
                )
            ));
        }
    }

    #[test]
    fn validate_integrity_rejects_non_increasing_schema_history() {
        for generation in [0, 1] {
            let mut ledger = ledger();
            let mut record = active_record("app.users.v1", 100);
            record.schema_history.push(SchemaMetadataRecord {
                generation,
                schema: SchemaMetadata::default(),
            });
            ledger.allocation_history.records = vec![record];

            let err = ledger
                .validate_integrity()
                .expect_err("non-increasing schema history");

            assert!(matches!(
                err,
                LedgerIntegrityError::NonIncreasingSchemaHistory { .. }
            ));
        }
    }

    #[test]
    fn recovery_rejects_zero_schema_version_during_ledger_decode() {
        let mut ledger = committed_ledger(1);
        ledger
            .allocation_history
            .records
            .push(AllocationRecord::active(
                1,
                &declaration("app.users.v1", 100, Some(1)),
            ));
        let mut value = serde_json::to_value(&ledger).unwrap();
        value["allocation_history"]["records"][0]["schema_history"][0]["schema"]["schema_version"] =
            0.into();
        let bytes = crate::test_cbor::to_vec(&value).unwrap();
        assert!(crate::test_cbor::from_slice::<AllocationLedger>(&bytes).is_err());
        let payload = LedgerPayloadEnvelope::current(bytes).encode();
        let mut physical = DualCommitStore::default();
        physical.commit_payload_at_generation(1, payload).unwrap();
        let store = LedgerCommitStore { physical };
        let before = store.clone();
        assert!(matches!(store.recover(), Err(LedgerCommitError::Codec(_))));
        assert_eq!(
            store, before,
            "malformed-schema recovery must preserve slots"
        );
    }

    #[test]
    fn malformed_generation_fingerprints_reject_as_codec_errors_and_preserve_slots() {
        for fingerprint in [
            String::new(),
            "x".repeat(257),
            "é".to_string(),
            "build\n".to_string(),
        ] {
            let mut value = serde_json::to_value(committed_ledger(1)).unwrap();
            value["allocation_history"]["generations"][0]["runtime_fingerprint"] =
                fingerprint.into();
            let bytes = crate::test_cbor::to_vec(&value).unwrap();
            let payload = LedgerPayloadEnvelope::current(bytes).encode();
            let invalid = CommittedGenerationBytes::new(1, payload);
            let genesis = committed_ledger(0);
            let valid = CommittedGenerationBytes::new(0, enveloped_payload(&genesis));
            for (slot0, slot1) in [
                (valid.clone(), invalid.clone()),
                (invalid.clone(), valid.clone()),
            ] {
                let mut store = LedgerCommitStore {
                    physical: DualCommitStore {
                        slot0: Some(slot0),
                        slot1: Some(slot1),
                    },
                };
                let before = store.clone();
                let error = store.recover().unwrap_err();
                assert!(matches!(error, LedgerCommitError::Codec(_)));
                let (recovered, diagnostic) = store.recover_with_diagnostic();
                assert_eq!(recovered, Err(error.clone()));
                assert!(diagnostic.recovery.is_ok(), "physical slots remain valid");
                assert_eq!(store.recover_or_initialize(&genesis), Err(error));
                assert_eq!(
                    store, before,
                    "all rejected paths preserve both protected slots"
                );
            }
        }
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
    fn committed_integrity_rejects_allocation_references_to_genesis() {
        for generation in [0, 1, 2] {
            let mut ledger = committed_ledger(generation);
            let mut record = AllocationRecord::active(0, &declaration("app.genesis.v1", 100, None));
            if generation == 2 {
                record.last_seen_generation = 1;
                record
                    .schema_history
                    .push(SchemaMetadataRecord::new(1, SchemaMetadata::default()));
                record.state = AllocationState::Retired { generation: 2 };
            }
            ledger.allocation_history.records.push(record);
            ledger.validate_integrity().expect("structurally valid DTO");
            assert_eq!(
                ledger.validate_committed_integrity(),
                Err(LedgerIntegrityError::UnknownRecordGeneration {
                    stable_key: StableKey::parse("app.genesis.v1").unwrap(),
                    generation: 0,
                })
            );
        }
    }

    #[test]
    fn validate_committed_integrity_rejects_generation_history_gaps() {
        let mut ledger = committed_ledger(3);
        ledger.allocation_history.generations.remove(1);
        ledger
            .allocation_history
            .records
            .push(AllocationRecord::active(
                0,
                &declaration("app.genesis.v1", 100, None),
            ));

        let err = ledger
            .validate_committed_integrity()
            .expect_err("generation history gap takes precedence over genesis reference");

        assert!(matches!(
            err,
            LedgerIntegrityError::NonIncreasingGenerationRecords { .. }
        ));
    }

    #[test]
    fn committed_integrity_rejects_a_skipped_parent_in_contiguous_history() {
        let mut ledger = committed_ledger(3);
        ledger.allocation_history.generations[2].parent_generation = 1;
        ledger
            .validate_integrity()
            .expect("structurally valid history");

        assert_eq!(
            ledger.validate_committed_integrity(),
            Err(LedgerIntegrityError::BrokenGenerationChain {
                generation: 3,
                expected_parent: 2,
                actual_parent: 1,
            })
        );
    }

    #[test]
    fn ledger_commit_store_rejects_invalid_ledger_before_write() {
        let mut store = LedgerCommitStore::default();
        let mut invalid = ledger();
        invalid.allocation_history.records = vec![
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
        assert_eq!(recovered.physical_generation(), 2);
    }

    #[test]
    fn ledger_commit_store_wraps_logical_payload_in_envelope() {
        let mut store = LedgerCommitStore::default();
        let ledger = committed_ledger(1);

        store.commit(&ledger).expect("commit");

        let committed = store.physical().authoritative().expect("authoritative");
        assert_eq!(
            committed.payload(),
            LedgerPayloadEnvelope::current(crate::test_cbor::to_vec(&ledger).unwrap())
                .try_encode()
                .unwrap(),
            "direct commit encoding preserves the complete public envelope bytes"
        );
        let envelope = LedgerPayloadEnvelope::decode(committed.payload()).expect("envelope");
        assert_eq!(
            envelope.payload(),
            crate::test_cbor::to_vec(&ledger).expect("encode ledger")
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
    fn ledger_commit_store_classifies_unknown_format_version_as_unsupported() {
        let mut payload = enveloped_payload(&committed_ledger(1));
        payload[12..16].copy_from_slice(&(LEDGER_PAYLOAD_FORMAT_VERSION + 1).to_le_bytes());
        let store = LedgerCommitStore {
            physical: DualCommitStore {
                slot0: Some(CommittedGenerationBytes::new(1, payload)),
                slot1: None,
            },
        };

        let err = store
            .recover()
            .expect_err("unknown ledger format version must fail");

        assert_eq!(
            err,
            LedgerCommitError::PayloadEnvelope(LedgerPayloadEnvelopeError::UnsupportedFormat {
                marker: *b"ICMF",
                version: Some(LEDGER_PAYLOAD_FORMAT_VERSION + 1),
            })
        );
    }

    #[test]
    fn recovery_and_initialization_preserve_physical_diagnostics_and_logical_failures() {
        let genesis = enveloped_payload(&committed_ledger(0));
        let committed = CommittedGenerationBytes::new(0, genesis.clone());
        let mut corrupt = committed.clone();
        corrupt.checksum ^= 1;
        let mut unsupported = genesis.clone();
        unsupported[12..16].copy_from_slice(&(LEDGER_PAYLOAD_FORMAT_VERSION + 1).to_le_bytes());
        let cases = [
            ("empty", None, None),
            ("one slot", Some(committed.clone()), None),
            (
                "identical slots",
                Some(committed.clone()),
                Some(committed.clone()),
            ),
            (
                "corrupt inactive",
                Some(committed.clone()),
                Some(corrupt.clone()),
            ),
            ("corrupt active", Some(corrupt), Some(committed.clone())),
            (
                "ambiguous",
                Some(committed),
                Some(CommittedGenerationBytes::new(0, vec![1])),
            ),
            (
                "unsupported format",
                Some(CommittedGenerationBytes::new(0, unsupported)),
                None,
            ),
            (
                "invalid CBOR",
                Some(CommittedGenerationBytes::new(
                    0,
                    LedgerPayloadEnvelope::current(vec![255])
                        .try_encode()
                        .unwrap(),
                )),
                None,
            ),
            (
                "generation mismatch",
                Some(CommittedGenerationBytes::new(1, genesis)),
                None,
            ),
            (
                "invalid committed integrity",
                Some(CommittedGenerationBytes::new(
                    3,
                    enveloped_payload(&ledger()),
                )),
                None,
            ),
        ];

        for (label, slot0, slot1) in cases {
            let empty = slot0.is_none() && slot1.is_none();
            let store = LedgerCommitStore {
                physical: DualCommitStore { slot0, slot1 },
            };
            let before = store.clone();
            let (recovered, diagnostic) = store.recover_with_diagnostic();
            assert_eq!(recovered, store.recover(), "{label}");
            assert_eq!(diagnostic, store.physical().diagnostic(), "{label}");
            assert_eq!(store, before, "{label}");

            let mut initializing = store.clone();
            let initialized = initializing.recover_or_initialize(&committed_ledger(0));
            if empty {
                let initialized = initialized.expect("empty store accepts genesis");
                assert_eq!(initialized.current_generation(), 0);
                assert_eq!(initialized.physical_generation(), 0);
                assert_eq!(initializing.recover().unwrap(), initialized);
            } else {
                assert_eq!(initialized, recovered, "{label}");
                assert_eq!(initializing, before, "{label}");
            }

            if let Err(LedgerCommitError::Recovery(error)) = recovered {
                assert_eq!(diagnostic.recovery, Err(error), "{label}");
            } else {
                assert!(diagnostic.recovery.is_ok(), "{label}");
            }
        }
    }

    #[test]
    fn ledger_commit_store_recovers_current_format_genesis_and_first_real_commit() {
        let mut store = LedgerCommitStore::default();
        let genesis = committed_ledger(0);

        let recovered = store
            .recover_or_initialize(&genesis)
            .expect("current-format genesis ledger");
        assert_eq!(recovered.current_generation(), 0);
        assert_eq!(recovered.physical_generation(), 0);
        assert_eq!(recovered.ledger().allocation_history().generations(), []);
        assert_eq!(recovered, store.recover().unwrap());

        let first = recovered
            .ledger()
            .stage_validated_generation(
                &validated(0, vec![declaration("app.users.v1", 100, Some(1))]),
                None,
            )
            .expect("first real generation");
        let recovered = store.commit(&first).expect("first commit");

        assert_eq!(recovered, store.recover().unwrap());
        assert_eq!(recovered.current_generation(), 1);
        assert_eq!(recovered.physical_generation(), 1);
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
    fn ledger_commit_store_rejects_corrupt_latest_slot_without_rollback() {
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

        store.commit(&second).expect("second commit");
        store.physical.slot0.as_mut().expect("latest slot").checksum ^= 1;

        let err = store
            .recover()
            .expect_err("corrupt latest must not roll back");

        assert_eq!(
            err,
            LedgerCommitError::Recovery(CommitRecoveryError::InvalidCommitSlots {
                slot0_invalid: true,
                slot1_invalid: false,
            })
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
    fn ledger_commit_store_rejects_corrupt_inactive_ledger() {
        let mut store = LedgerCommitStore::default();
        let first = committed_ledger(1);
        let second = committed_ledger(2);

        store.commit(&first).expect("first commit");
        store
            .write_corrupt_inactive_ledger(&second)
            .expect("corrupt write");
        let err = store.recover().expect_err("corrupt slot");

        assert_eq!(
            err,
            LedgerCommitError::Recovery(CommitRecoveryError::InvalidCommitSlots {
                slot0_invalid: false,
                slot1_invalid: true,
            })
        );
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
        assert_eq!(recovered.physical_generation(), 3);
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
            LedgerCommitError::Recovery(CommitRecoveryError::InvalidCommitSlots {
                slot0_invalid: true,
                slot1_invalid: false,
            })
        ));
    }
    #[test]
    fn history_boundary_round_trips_and_rejects_next_generation_without_mutation() {
        let mut source = committed_ledger(crate::constants::MAX_LEDGER_GENERATIONS as u64 - 1);
        source
            .allocation_history
            .records
            .push(active_record("app.users.v1", 100));
        let retirement =
            AllocationRetirement::new("app.users.v1", MemoryManagerSlot::new(100).unwrap())
                .unwrap();
        let retired = source
            .stage_retirement_generation(&retirement, None)
            .unwrap();
        retired.validate_committed_integrity().unwrap();
        assert_eq!(
            retired.allocation_history.generations().len(),
            crate::constants::MAX_LEDGER_GENERATIONS
        );
        assert_eq!(
            retired.allocation_history.records()[0].schema_history,
            source.allocation_history.records()[0].schema_history
        );
        assert_eq!(
            source.allocation_history.records()[0].state,
            AllocationState::Active
        );
        assert!(matches!(
            retired.stage_retirement_generation(&retirement, None),
            Err(AllocationRetirementError::Integrity(
                LedgerIntegrityError::LimitExceeded {
                    resource: "generation history",
                    ..
                }
            ))
        ));

        let ledger = committed_ledger(crate::constants::MAX_LEDGER_GENERATIONS as u64);
        let mut store = LedgerCommitStore::default();
        let recovered = store.commit(&ledger).expect("boundary is admissible");
        let record = crate::StableCellLedgerRecord::new(store.clone());
        let bytes = crate::test_cbor::to_vec(&record).unwrap();
        assert_eq!(record.encoded_size(), bytes.len());
        let recovered_record = crate::decode_stable_cell_ledger_record(&bytes).unwrap();
        assert_eq!(
            recovered_record.store().recover().unwrap().ledger(),
            &ledger
        );
        let before = store.clone();
        let err = crate::AllocationBootstrap::new(&mut store)
            .validate_and_commit(
                DeclarationSnapshot::new(Vec::new()).unwrap(),
                &crate::GenericRangePolicy,
                None,
            )
            .unwrap_err();
        assert!(matches!(
            err,
            crate::BootstrapError::Staging(AllocationStageError::Integrity(
                LedgerIntegrityError::LimitExceeded { .. }
            ))
        ));
        assert_eq!(store, before);
        assert_eq!(
            recovered.current_generation(),
            crate::constants::MAX_LEDGER_GENERATIONS as u64
        );
        let oversize = committed_ledger(crate::constants::MAX_LEDGER_GENERATIONS as u64 + 1);
        assert!(matches!(
            store.commit(&oversize),
            Err(LedgerCommitError::Integrity(
                LedgerIntegrityError::LimitExceeded { .. }
            ))
        ));
        assert_eq!(store, before);
    }

    #[test]
    fn record_count_boundary_and_malicious_history_reject_on_recovery() {
        let mut ledger = committed_ledger(1);
        ledger.allocation_history.records = (0..255_u8)
            .map(|id| active_record(&format!("app.slot{id}.v1"), id))
            .collect();
        let mut store = LedgerCommitStore::default();
        store.commit(&ledger).unwrap();
        assert_eq!(
            store
                .recover()
                .unwrap()
                .ledger()
                .allocation_history()
                .records()
                .len(),
            255
        );
        ledger
            .allocation_history
            .records
            .push(active_record("app.extra.v1", 254));
        assert!(matches!(
            store.commit(&ledger),
            Err(LedgerCommitError::Integrity(
                LedgerIntegrityError::LimitExceeded { .. }
            ))
        ));
        let payload = enveloped_payload(&ledger);
        let mut invalid = LedgerCommitStore {
            physical: DualCommitStore {
                slot0: Some(CommittedGenerationBytes::new(1, payload)),
                slot1: None,
            },
        };
        assert!(invalid.recover().is_err());
        assert!(invalid.recover_or_initialize(&committed_ledger(0)).is_err());
        for bytes in [
            vec![0x9b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
            vec![0x81; 34],
            vec![0xa1],
        ] {
            let payload = LedgerPayloadEnvelope::current(bytes).try_encode().unwrap();
            let invalid = LedgerCommitStore {
                physical: DualCommitStore {
                    slot0: Some(CommittedGenerationBytes::new(1, payload)),
                    slot1: None,
                },
            };
            assert!(matches!(
                invalid.recover(),
                Err(LedgerCommitError::Codec(_))
            ));
        }
    }

    #[test]
    fn ledger_collection_limits_preserve_refusal_order_and_committed_slots() {
        let limit = crate::constants::MAX_LEDGER_GENERATIONS;
        let mut oversized = committed_ledger(1);
        let mut record = active_record("app.users.v1", 100);
        record.schema_history =
            vec![SchemaMetadataRecord::new(1, crate::SchemaMetadata::default()); limit + 1];
        oversized.allocation_history.records =
            vec![active_record("app.users.v1", 100); crate::constants::MAX_ALLOCATIONS + 1];
        oversized.allocation_history.records[0] = record;
        let generation = oversized.allocation_history.generations[0].clone();
        oversized
            .allocation_history
            .generations
            .resize(limit + 1, generation);

        let mut store = LedgerCommitStore::default();
        store.commit(&committed_ledger(0)).unwrap();
        store.commit(&committed_ledger(1)).unwrap();
        let before = store.clone();
        // Later collection bounds and malformed duplicate histories cannot
        // displace the first outer collection refusal.
        for (resource, maximum) in [
            ("allocation records", crate::constants::MAX_ALLOCATIONS),
            ("generation history", limit),
            ("schema history", limit),
        ] {
            let expected = LedgerIntegrityError::LimitExceeded {
                resource,
                limit: maximum,
            };
            assert_eq!(oversized.validate_integrity(), Err(expected.clone()));
            assert_eq!(
                oversized.validate_committed_integrity(),
                Err(expected.clone())
            );
            assert_eq!(
                store.commit(&oversized),
                Err(LedgerCommitError::Integrity(expected.clone()))
            );
            assert_eq!(
                oversized.stage_validated_generation(&validated(1, Vec::new()), None),
                Err(AllocationStageError::Integrity(expected))
            );
            assert_eq!(store, before, "{resource} refusal preserves both slots");
            match resource {
                "allocation records" => oversized.allocation_history.records.truncate(1),
                "generation history" => oversized.allocation_history.generations.truncate(1),
                _ => (),
            }
        }
        assert_eq!(store.recover(), before.recover());
    }

    #[test]
    fn schema_collection_limit_accepts_exact_history_and_rejects_the_next_record() {
        let limit = crate::constants::MAX_LEDGER_GENERATIONS;
        let mut ledger = committed_ledger(limit as u64);
        let mut record = active_record("app.users.v1", 100);
        record.last_seen_generation = limit as u64;
        record.schema_history = (1..=limit as u64)
            .map(|generation| {
                SchemaMetadataRecord::new(generation, crate::SchemaMetadata::default())
            })
            .collect();
        ledger.allocation_history.records.push(record);
        assert_eq!(ledger.validate_committed_integrity(), Ok(()));

        // Overflow is aggregate: every individual history remains bounded.
        ledger
            .allocation_history
            .records
            .push(active_record("app.orders.v1", 101));
        assert_eq!(
            ledger.validate_committed_integrity(),
            Err(LedgerIntegrityError::LimitExceeded {
                resource: "schema history",
                limit,
            })
        );
    }

    #[test]
    fn encoded_byte_limit_rejects_before_commit_mutation() {
        let mut store = LedgerCommitStore::default();
        store.commit(&committed_ledger(0)).unwrap();
        let before = store.clone();
        let mut ledger = committed_ledger(crate::constants::MAX_LEDGER_GENERATIONS as u64);
        for generation in &mut ledger.allocation_history.generations {
            generation.runtime_fingerprint =
                Some("x".repeat(crate::constants::DIAGNOSTIC_STRING_MAX_BYTES));
        }
        assert!(matches!(
            store.commit(&ledger),
            Err(LedgerCommitError::Integrity(
                LedgerIntegrityError::LimitExceeded {
                    resource: "ledger bytes",
                    ..
                }
            ))
        ));
        assert_eq!(store, before);
        let mut envelope = LedgerPayloadEnvelope::current(Vec::new())
            .try_encode()
            .unwrap();
        envelope[16..24]
            .copy_from_slice(&((crate::constants::MAX_LEDGER_BYTES + 1) as u64).to_le_bytes());
        assert!(matches!(
            LedgerPayloadEnvelope::decode(&envelope),
            Err(LedgerPayloadEnvelopeError::PayloadTooLarge { .. })
        ));
    }
}
