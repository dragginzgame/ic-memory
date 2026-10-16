mod claim;
mod codec;
mod error;
mod integrity;
mod record;
mod stage;

use crate::physical::{CommitSnapshotDiagnostic, CommittedGenerationBytes};
use serde::{Deserialize, Serialize};

pub use claim::{ClaimConflict, ClaimOutcome, validate_declaration_claim};
pub use error::{AllocationStageError, LedgerCommitError, LedgerIntegrityError};
pub use record::{AllocationLedger, AllocationRecord, AllocationState, RecoveredLedger};
pub use stage::stage_generation;

use codec::decode_ledger;

///
/// LedgerCommitStore
///
/// Generation-scoped allocation ledger commit store.
///
/// This type owns the logical commit lifecycle and the crate-owned CBOR ledger
/// encoding. It deliberately does not own stable-memory IO; that remains the
/// substrate or framework owner's responsibility.
///
/// The runtime commits allocation ledger generations through this passive DTO.
/// Decoding it grants no memory access or persistence authority. It does not
/// open stable-memory handles or allocate application slots.
///

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LedgerCommitStore {
    /// Protected current physical snapshot.
    #[serde(deserialize_with = "crate::cbor::deserialize_present_option")]
    pub(crate) current: Option<CommittedGenerationBytes>,
}

impl LedgerCommitStore {
    /// Recover the authoritative allocation ledger using the native CBOR ledger codec.
    pub(crate) fn recover(&self) -> Result<RecoveredLedger, LedgerCommitError> {
        let committed = self.authoritative().map_err(LedgerCommitError::Recovery)?;
        Self::recover_committed(committed)
    }

    pub(crate) fn recover_with_diagnostic(
        &self,
    ) -> (
        Result<RecoveredLedger, LedgerCommitError>,
        CommitSnapshotDiagnostic,
    ) {
        let (committed, diagnostic) = self.authoritative_with_diagnostic();
        let recovered = committed
            .map_err(LedgerCommitError::Recovery)
            .and_then(Self::recover_committed);
        (recovered, diagnostic)
    }

    fn recover_committed(
        committed: &CommittedGenerationBytes,
    ) -> Result<RecoveredLedger, LedgerCommitError> {
        let ledger = decode_ledger(committed.payload()).map_err(LedgerCommitError::Codec)?;
        if committed.generation() != ledger.current_generation {
            return Err(LedgerCommitError::PhysicalLogicalGenerationMismatch {
                physical_generation: committed.generation(),
                logical_generation: ledger.current_generation,
            });
        }
        ledger
            .validate_integrity()
            .map_err(LedgerCommitError::Integrity)?;
        Ok(RecoveredLedger::from_trusted_ledger(ledger))
    }

    /// Recover the authoritative ledger, or use logical genesis for an empty store.
    ///
    /// Only explicit snapshot absence admits the empty generation-zero base.
    /// No payload is encoded or committed here. Corrupt or partially written
    /// stores fail closed; the caller must persist its snapshot successor
    /// before publishing allocation authority.
    pub(crate) fn recover_or_genesis(&self) -> Result<RecoveredLedger, LedgerCommitError> {
        self.recover()
            .or_else(|error| RecoveredLedger::genesis_for_absent_snapshot(&error).ok_or(error))
    }

    /// Commit one logical allocation ledger generation through the native CBOR ledger codec.
    ///
    /// # Panics
    ///
    /// Panics only if the private concrete ledger-encoding invariant is broken.
    #[cfg(test)]
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
            .validate_integrity()
            .map_err(LedgerCommitError::Integrity)?;
        let payload = codec::encode_ledger(ledger).map_err(LedgerCommitError::Integrity)?;
        self.commit_payload_at_generation(ledger.current_generation, payload)
            .map(|_| ())
            .map_err(LedgerCommitError::Recovery)
    }

    /// Simulate corruption of the current logical ledger snapshot.
    #[cfg(test)]
    pub fn write_corrupt_ledger(
        &mut self,
        ledger: &AllocationLedger,
    ) -> Result<(), LedgerCommitError> {
        let payload = codec::encode_ledger(ledger).map_err(LedgerCommitError::Integrity)?;
        self.write_corrupt_snapshot(ledger.current_generation, payload);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CommitRecoveryError, test_cbor::hex_fixture};

    #[test]
    fn repeated_commits_keep_metadata_bounded() {
        let mut store = LedgerCommitStore::default();
        store
            .commit(&AllocationLedger::new(65_530, Vec::new()).unwrap())
            .unwrap();
        let mut largest_payload = 0;
        for version in 1_u32..=1024 {
            let snapshot = DeclarationSnapshot::for_test(vec![declaration("app.users.v1", 100)]);
            let recovered = store.recover().unwrap();
            let (next, _) =
                stage_generation(recovered, snapshot, &crate::GenericAllocationPolicy).unwrap();
            store.commit_generation(&next).unwrap();
            assert_eq!(next.current_generation(), 65_530 + u64::from(version));
            let recovered = store.recover().unwrap();
            assert_eq!(recovered.ledger().records().len(), 1);
            largest_payload = largest_payload.max(store.authoritative().unwrap().payload().len());
        }
        // CBOR integer widths may grow; the number of records and metadata do not.
        assert!(largest_payload < 200);
        assert!(
            crate::stable_cell::EncodedLedgerRecord::new(&store)
                .unwrap()
                .len()
                < 600
        );
    }

    #[test]
    fn largest_current_record_set_fits_writer_and_reader_bounds() {
        let records = (0..255_u8)
            .map(|id| {
                let key = format!("app.store{id:03}.{}.v1", "x".repeat(112));
                assert_eq!(key.len(), 128);
                let declaration = declaration(&key, id);
                let mut record = AllocationRecord::active(&declaration);
                // Reserved is the longest encoded lifecycle variant.
                record.state = AllocationState::Reserved;
                record
            })
            .collect();
        let ledger = AllocationLedger::new(u64::MAX - 1, records).unwrap();
        let payload = encoded_payload(&ledger);
        assert!(payload.len() <= crate::constants::MAX_COMMITTED_PAYLOAD_BYTES);
        let mut store = LedgerCommitStore::default();
        store.commit(&ledger).unwrap();
        let next = AllocationLedger::new(u64::MAX, ledger.records().to_vec()).unwrap();
        store.commit(&next).unwrap();
        let record = store;
        let bytes = crate::stable_cell::EncodedLedgerRecord::new(&record)
            .unwrap()
            .into_bytes();
        assert!(bytes.len() <= crate::constants::MAX_LEDGER_RECORD_BYTES);
        let decoded = crate::decode_stable_cell_commit_store(&bytes).unwrap();
        assert_eq!(decoded.recover().unwrap().ledger(), &next);
    }

    #[test]
    fn validated_counter_overflow_rejects_without_commit_mutation() {
        let mut store = LedgerCommitStore::default();
        store
            .commit(&AllocationLedger::new(u64::MAX, Vec::new()).unwrap())
            .unwrap();
        let before = store.clone();
        let recovered = store.recover().unwrap();
        let result = stage_generation(
            recovered,
            DeclarationSnapshot::for_test(Vec::new()),
            &crate::GenericAllocationPolicy,
        );
        assert!(matches!(
            result,
            Err(crate::RuntimeBootstrapError::Staging(
                AllocationStageError::GenerationOverflow {
                    generation: u64::MAX
                }
            ))
        ));
        assert_eq!(store, before);
    }

    #[test]
    fn unsupported_discriminator_precedes_logical_decode_and_preserves_storage() {
        let mut committed = CommittedGenerationBytes::new(1, vec![0xff]);
        committed.commit_marker = u64::MAX;
        let store = LedgerCommitStore {
            current: Some(committed),
        };
        let before = store.clone();
        let expected = LedgerCommitError::Recovery(CommitRecoveryError::UnsupportedFormat {
            marker: u64::MAX,
        });
        assert_eq!(store.recover(), Err(expected.clone()));
        assert_eq!(store.recover_or_genesis(), Err(expected));
        assert_eq!(store, before);
    }
    use crate::{
        declaration::AllocationDeclaration, key::StableKey, physical::CommittedGenerationBytes,
        requests::DeclarationSnapshot, slot::MemoryManagerSlot,
    };
    use ic_stable_structures::Storable;
    fn declaration(key: &str, id: u8) -> AllocationDeclaration {
        AllocationDeclaration::new(key, MemoryManagerSlot::new(id).expect("usable slot"))
            .expect("declaration")
    }

    fn ledger() -> AllocationLedger {
        AllocationLedger {
            current_generation: 3,
            records: Vec::new(),
        }
    }

    fn committed_ledger(current_generation: u64) -> AllocationLedger {
        AllocationLedger {
            current_generation,
            records: Vec::new(),
        }
    }

    fn active_record(key: &str, id: u8) -> AllocationRecord {
        AllocationRecord::active(&declaration(key, id))
    }

    fn snapshot_from(declarations: Vec<AllocationDeclaration>) -> DeclarationSnapshot {
        DeclarationSnapshot::for_test(declarations)
    }

    fn stage(
        ledger: AllocationLedger,
        snapshot: &DeclarationSnapshot,
    ) -> Result<AllocationLedger, crate::RuntimeBootstrapError<std::convert::Infallible>> {
        stage_generation(
            RecoveredLedger::from_trusted_ledger(ledger),
            snapshot_from(snapshot.declarations().to_vec()),
            &crate::GenericAllocationPolicy,
        )
        .map(|(ledger, _)| ledger)
    }

    fn record<'ledger>(ledger: &'ledger AllocationLedger, key: &str) -> &'ledger AllocationRecord {
        ledger
            .records()
            .iter()
            .find(|record| record.stable_key.as_str() == key)
            .expect("allocation record")
    }

    fn encoded_payload(ledger: &AllocationLedger) -> Vec<u8> {
        codec::encode_ledger(ledger).expect("fixture ledger")
    }

    fn ledger_from_payload_fixture(contents: &str) -> AllocationLedger {
        let bytes = hex_fixture(contents);
        let ledger = decode_ledger(&bytes).expect("fixture ledger");
        ledger
            .validate_integrity()
            .expect("fixture ledger integrity");
        assert_eq!(bytes, encoded_payload(&ledger));
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
            records: vec![active_record("app.users.v1", 100)],
        }
    }

    fn active_ledger_value() -> crate::test_cbor::Value {
        crate::test_cbor::to_value(active_committed_ledger()).expect("ledger value")
    }

    #[test]
    fn recovery_rejects_unknown_fields_inside_retirement_state() {
        use crate::LedgerCommitStore;

        let retired = AllocationLedger::new(
            2,
            vec![crate::testing::allocation_record(
                &declaration("app.users.v1", 100),
                AllocationState::Retired,
            )],
        )
        .unwrap();
        let mut store = LedgerCommitStore::default();
        let committed = store.commit(&retired).unwrap();
        assert_eq!(committed, store.recover().unwrap());

        let mut value = crate::test_cbor::to_value(&retired).unwrap();
        let records = value_array_mut(map_field_mut(value_map_mut(&mut value), "records"));
        let state = map_field_mut(value_map_mut(&mut records[0]), "state");
        *state = serde_json::from_value::<crate::test_cbor::Value>(
            serde_json::json!({"Retired": {"unexpected": true}}),
        )
        .unwrap();
        let bytes = crate::test_cbor::to_vec(&value).unwrap();
        let payload = bytes;
        let mut physical = LedgerCommitStore::default();
        physical.commit_payload_at_generation(2, payload).unwrap();
        let record = physical;
        let encoded = crate::test_cbor::to_vec(&record).unwrap();
        let decoded = crate::decode_stable_cell_commit_store(&encoded).unwrap();
        let before = decoded.clone();
        let error = decoded.recover().unwrap_err();
        assert!(matches!(error, LedgerCommitError::Codec(_)));

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
    fn allocation_record_accessors_expose_read_only_views() {
        let ledger = AllocationLedger::new(1, vec![active_record("app.users.v1", 100)]).unwrap();
        assert_eq!(ledger.records().len(), 1);
        let record = &ledger.records()[0];
        assert_eq!(record.stable_key().as_str(), "app.users.v1");
        assert_eq!(record.slot().id(), 100);
        assert_eq!(record.state(), AllocationState::Active);
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
            (Value::Text("records".to_string()), Value::Array(vec![])),
            (Value::Text("future_field".to_string()), Value::Bool(true)),
        ];
        let bytes = crate::test_cbor::to_vec(&Value::Map(map)).expect("unknown-field ledger");

        let err = decode_ledger(&bytes).expect_err("unknown ledger field must fail closed");

        assert!(err.contains("future_field"));
    }

    #[test]
    fn cbor_ledger_codec_rejects_unknown_nested_record_fields() {
        let mut value = active_ledger_value();
        let records = map_field_mut(value_map_mut(&mut value), "records");
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
    fn ledger_commit_store_rejects_unknown_top_level_fields() {
        use crate::test_cbor::Value;

        let map = vec![
            (Value::Text("current".to_string()), Value::Null),
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
            "../../fixtures/current/empty_genesis_ledger.cbor.hex"
        ));

        assert_eq!(ledger.current_generation, 0);
        assert_eq!(ledger.records(), []);
    }

    #[test]
    fn current_single_active_payload_fixture_recovers() {
        let ledger = ledger_from_payload_fixture(include_str!(
            "../../fixtures/current/single_active_allocation_ledger.cbor.hex"
        ));
        let record = record(&ledger, "app.users.v1");

        assert_eq!(ledger.current_generation, 1);
        assert_eq!(record.state(), AllocationState::Active);
        assert_eq!(record.slot().id(), 100);
    }

    #[test]
    fn current_reserved_payload_fixture_recovers() {
        let ledger = ledger_from_payload_fixture(include_str!(
            "../../fixtures/current/reserved_allocation_ledger.cbor.hex"
        ));
        let record = record(&ledger, "app.future_store.v1");

        assert_eq!(ledger.current_generation, 1);
        assert_eq!(record.state(), AllocationState::Reserved);
        assert_eq!(record.slot().id(), 101);
    }

    #[test]
    fn current_retired_payload_fixture_recovers() {
        let ledger = ledger_from_payload_fixture(include_str!(
            "../../fixtures/current/retired_allocation_ledger.cbor.hex"
        ));
        let record = record(&ledger, "app.users.v1");

        assert_eq!(ledger.current_generation, 2);
        assert_eq!(record.state(), AllocationState::Retired);
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
    fn current_snapshot_fixture_recovers() {
        let store = store_from_fixture(include_str!(
            "../../fixtures/current/snapshot_store_valid.cbor.hex"
        ));

        assert_eq!(
            store.diagnostic(),
            crate::CommitSnapshotDiagnostic::Valid { generation: 2 }
        );
        let recovered = store.recover().expect("fixture store recovers");
        assert_eq!(recovered.current_generation(), 2);
    }

    #[test]
    fn current_corrupt_snapshot_fixture_fails_closed() {
        let store = store_from_fixture(include_str!(
            "../../fixtures/current/snapshot_store_corrupt.cbor.hex"
        ));

        let diagnostic = store.diagnostic();
        assert_eq!(
            diagnostic,
            crate::CommitSnapshotDiagnostic::Invalid { generation: 2 }
        );
        let err = store
            .recover()
            .expect_err("corrupt fixture must fail closed");
        assert!(matches!(
            err,
            LedgerCommitError::Recovery(CommitRecoveryError::InvalidSnapshot { generation: 2 })
        ));
    }

    #[test]
    fn stage_generation_records_new_allocations() {
        let declarations = vec![declaration("app.users.v1", 100)];
        let snapshot = snapshot_from(declarations);

        let staged = stage(ledger(), &snapshot).expect("staged generation");

        assert_eq!(staged.current_generation, 4);
        assert_eq!(staged.records().len(), 1);
    }

    #[test]
    fn stage_generation_allows_empty_generation_boundary() {
        let snapshot = snapshot_from(Vec::new());

        let staged = stage(ledger(), &snapshot).expect("empty snapshot generation");

        assert_eq!(staged.current_generation, 4);
        assert_eq!(staged.records(), []);
    }

    #[test]
    fn stage_generation_rejects_same_key_different_slot() {
        let mut ledger = committed_ledger(3);
        ledger.records = vec![active_record("app.users.v1", 100)];
        ledger.validate_integrity().unwrap();
        let snapshot = snapshot_from(vec![declaration("app.users.v1", 101)]);

        let err = stage(ledger, &snapshot).expect_err("stable key cannot move slots");

        assert!(matches!(
            err,
            crate::RuntimeBootstrapError::Validation(
                crate::AllocationValidationError::StableKeySlotConflict { .. }
            )
        ));
    }

    #[test]
    fn stage_generation_rejects_same_slot_different_key() {
        let mut ledger = ledger();
        ledger.records = vec![active_record("app.users.v1", 100)];
        let snapshot = snapshot_from(vec![declaration("app.orders.v1", 100)]);

        let err = stage(ledger, &snapshot).expect_err("slot cannot be reused by another key");

        assert!(matches!(
            err,
            crate::RuntimeBootstrapError::Validation(
                crate::AllocationValidationError::SlotStableKeyConflict { .. }
            )
        ));
    }

    #[test]
    fn stage_generation_rejects_retired_redeclaration() {
        let mut ledger = ledger();
        let mut record = active_record("app.users.v1", 100);
        record.state = AllocationState::Retired;
        ledger.records = vec![record];
        let snapshot = snapshot_from(vec![declaration("app.users.v1", 100)]);

        let err = stage(ledger, &snapshot).expect_err("retired allocation cannot be redeclared");

        assert!(matches!(
            err,
            crate::RuntimeBootstrapError::Validation(
                crate::AllocationValidationError::RetiredAllocation { .. }
            )
        ));
    }

    #[test]
    fn stage_generation_preserves_omitted_records() {
        let first = snapshot_from(vec![
            declaration("app.users.v1", 100),
            declaration("app.orders.v1", 101),
        ]);
        let second = snapshot_from(vec![declaration("app.users.v1", 100)]);

        let staged = stage(ledger(), &first).expect("first generation");
        let staged = stage(staged, &second).expect("second generation");

        assert_eq!(staged.current_generation, 5);
        assert_eq!(staged.records().len(), 2);
        let omitted = staged
            .records()
            .iter()
            .find(|record| record.stable_key.as_str() == "app.orders.v1")
            .expect("omitted record");
        assert_eq!(omitted.state, AllocationState::Active);
    }

    #[test]
    fn stage_generation_activates_reserved_record() {
        let staged = AllocationLedger::new(
            4,
            vec![crate::testing::allocation_record(
                &declaration("app.future_store.v1", 100),
                AllocationState::Reserved,
            )],
        )
        .unwrap();
        let active = snapshot_from(vec![declaration("app.future_store.v1", 100)]);

        let staged = stage(staged, &active).expect("active generation");
        let record = &staged.records()[0];

        assert_eq!(record.state, AllocationState::Active);
    }

    #[test]
    fn transition_matrix_preserves_committed_invariants() {
        fn visit(ledger: AllocationLedger, depth: usize) {
            ledger.validate_integrity().unwrap();
            if depth == 0 {
                return;
            }
            for declarations in [
                vec![],
                vec![declaration("app.users.v1", 100)],
                vec![declaration("app.users.v1", 100)],
                vec![declaration("app.orders.v1", 101)],
                vec![declaration("app.other.v1", 100)],
            ] {
                let next = stage(ledger.clone(), &snapshot_from(declarations))
                    .unwrap_or_else(|_| ledger.clone());
                visit(next, depth - 1);
            }
        }
        visit(AllocationLedger::empty_genesis(), 4);
        for state in [
            AllocationState::Reserved,
            AllocationState::Active,
            AllocationState::Retired,
        ] {
            let record =
                crate::testing::allocation_record(&declaration("app.users.v1", 100), state);
            visit(AllocationLedger::new(1, vec![record]).unwrap(), 4);
        }
    }

    #[test]
    fn validate_integrity_rejects_duplicate_stable_keys() {
        let mut ledger = ledger();
        for second_id in [101, 100] {
            ledger.records = vec![
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
        ledger.records = vec![
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
        ledger.records.push(active_record("app.users.v1", 100));
        let mut bytes = crate::test_cbor::to_vec(&ledger).expect("encode ledger");
        let key_start = bytes
            .windows(b"app.users.v1".len())
            .position(|window| window == b"app.users.v1")
            .expect("encoded stable key");
        bytes[key_start] = b'A';
        assert!(crate::test_cbor::from_slice::<AllocationLedger>(&bytes).is_err());
        let payload = bytes;
        let mut physical = LedgerCommitStore::default();
        physical.commit_payload_at_generation(1, payload).unwrap();
        let store = physical;
        let before = store.clone();
        assert!(matches!(store.recover(), Err(LedgerCommitError::Codec(_))));
        assert_eq!(store, before, "malformed-key recovery must preserve slots");
    }

    #[test]
    fn ledger_decode_rejects_unusable_memory_manager_slot() {
        let mut ledger = committed_ledger(1);
        ledger.records.push(active_record("app.users.v1", 100));
        let mut value = serde_json::to_value(ledger).unwrap();
        value["records"][0]["slot"] = serde_json::json!(255);
        let bytes = crate::test_cbor::to_vec(&value).unwrap();
        assert!(decode_ledger(&bytes).is_err());
    }

    #[test]
    fn integrity_rejects_nonempty_genesis_before_commit() {
        let invalid = AllocationLedger {
            current_generation: 0,
            records: vec![active_record("app.users.v1", 100)],
        };
        assert_eq!(
            invalid.validate_integrity(),
            Err(LedgerIntegrityError::NonemptyGenesis)
        );
        let mut store = LedgerCommitStore::default();
        let before = store.clone();
        assert_eq!(
            store.commit(&invalid),
            Err(LedgerCommitError::Integrity(
                LedgerIntegrityError::NonemptyGenesis
            ))
        );
        assert_eq!(store, before);
    }

    #[test]
    fn ledger_commit_store_rejects_invalid_ledger_before_write() {
        let mut store = LedgerCommitStore::default();
        let mut invalid = ledger();
        invalid.records = vec![
            active_record("app.users.v1", 100),
            active_record("app.orders.v1", 100),
        ];

        let err = store.commit(&invalid).expect_err("invalid ledger");

        assert!(matches!(
            err,
            LedgerCommitError::Integrity(LedgerIntegrityError::DuplicateSlot { .. })
        ));
        assert!(store.current().is_none());
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
    fn ledger_commit_store_encodes_direct_logical_cbor() {
        let mut store = LedgerCommitStore::default();
        let ledger = committed_ledger(1);

        store.commit(&ledger).expect("commit");

        let committed = store.authoritative().expect("authoritative");
        assert_eq!(
            committed.payload(),
            crate::test_cbor::to_vec(&ledger).unwrap(),
            "direct commit encoding matches logical CBOR"
        );
    }

    #[test]
    fn ledger_commit_store_rejects_malformed_logical_cbor() {
        let committed = CommittedGenerationBytes::new(1, vec![b'X'; 28]);
        let store = LedgerCommitStore {
            current: Some(committed),
        };

        let err = store.recover().expect_err("bad logical CBOR");

        assert!(matches!(err, LedgerCommitError::Codec(_)));
    }

    #[test]
    fn recovery_and_initialization_preserve_physical_diagnostics_and_logical_failures() {
        let genesis = encoded_payload(&committed_ledger(0));
        let committed = CommittedGenerationBytes::new(0, genesis.clone());
        let mut corrupt = committed.clone();
        corrupt.checksum ^= 1;
        let mut unsupported = committed.clone();
        unsupported.commit_marker = u64::MAX;
        let cases = [
            ("empty", None),
            ("current", Some(committed)),
            ("corrupt", Some(corrupt)),
            ("unsupported format", Some(unsupported)),
            (
                "invalid CBOR",
                Some(CommittedGenerationBytes::new(0, vec![255])),
            ),
            (
                "generation mismatch",
                Some(CommittedGenerationBytes::new(1, genesis)),
            ),
            (
                "invalid committed integrity",
                Some(CommittedGenerationBytes::new(3, encoded_payload(&ledger()))),
            ),
        ];

        for (label, current) in cases {
            let empty = current.is_none();
            let store = LedgerCommitStore { current };
            let before = store.clone();
            let (recovered, diagnostic) = store.recover_with_diagnostic();
            assert_eq!(recovered, store.recover(), "{label}");
            assert_eq!(diagnostic, store.diagnostic(), "{label}");
            assert_eq!(store, before, "{label}");

            let initializing = store.clone();
            let initialized = initializing.recover_or_genesis();
            if empty {
                let initialized = initialized.expect("empty store accepts genesis");
                assert_eq!(initialized.current_generation(), 0);
                assert_eq!(initialized.ledger().records(), []);
            } else {
                assert_eq!(initialized, recovered, "{label}");
            }
            assert_eq!(initializing, before, "{label}");

            if let Err(LedgerCommitError::Recovery(error)) = recovered {
                let expected = match error {
                    CommitRecoveryError::NoValidGeneration => {
                        crate::CommitSnapshotDiagnostic::Empty
                    }
                    CommitRecoveryError::InvalidSnapshot { generation } => {
                        crate::CommitSnapshotDiagnostic::Invalid { generation }
                    }
                    CommitRecoveryError::UnsupportedFormat { .. } => {
                        crate::CommitSnapshotDiagnostic::Invalid {
                            generation: before.current().unwrap().generation(),
                        }
                    }
                    _ => panic!("unexpected physical recovery failure: {error}"),
                };
                assert_eq!(diagnostic, expected, "{label}");
            } else {
                assert!(
                    matches!(diagnostic, crate::CommitSnapshotDiagnostic::Valid { .. }),
                    "{label}"
                );
            }
        }
    }

    #[test]
    fn ledger_commit_store_recovers_current_format_genesis_and_first_real_commit() {
        let mut store = LedgerCommitStore::default();
        let recovered = store
            .recover_or_genesis()
            .expect("current-format genesis ledger");
        assert_eq!(recovered.current_generation(), 0);

        assert!(store.current().is_none());
        assert!(matches!(
            store.recover(),
            Err(LedgerCommitError::Recovery(
                CommitRecoveryError::NoValidGeneration
            ))
        ));

        let first = stage(
            recovered.ledger().clone(),
            &snapshot_from(vec![declaration("app.users.v1", 100)]),
        )
        .expect("first real generation");
        let recovered = store.commit(&first).expect("first commit");

        assert_eq!(recovered, store.recover().unwrap());
        assert_eq!(recovered.current_generation(), 1);
    }

    #[test]
    fn ledger_commit_store_rejects_corrupt_current_snapshot() {
        let mut store = LedgerCommitStore::default();
        let genesis = committed_ledger(0);
        store.commit(&genesis).expect("genesis commit");
        let first = stage(
            genesis,
            &snapshot_from(vec![declaration("app.users.v1", 100)]),
        )
        .expect("first generation");
        let first = store.commit(&first).expect("first commit");
        let second = stage(
            first.ledger().clone(),
            &snapshot_from(vec![declaration("app.users.v1", 100)]),
        )
        .expect("second generation");

        store.commit(&second).expect("second commit");
        store.current.as_mut().expect("latest slot").checksum ^= 1;

        let err = store
            .recover()
            .expect_err("corrupt latest must not roll back");

        assert_eq!(
            err,
            LedgerCommitError::Recovery(CommitRecoveryError::InvalidSnapshot { generation: 2 })
        );
    }

    #[test]
    fn ledger_commit_store_rejects_corrupt_current_ledger() {
        let mut store = LedgerCommitStore::default();
        let first = committed_ledger(1);
        let second = committed_ledger(2);

        store.commit(&first).expect("first commit");
        store.write_corrupt_ledger(&second).expect("corrupt write");
        let err = store.recover().expect_err("corrupt slot");

        assert_eq!(
            err,
            LedgerCommitError::Recovery(CommitRecoveryError::InvalidSnapshot { generation: 2 })
        );
    }

    #[test]
    fn ledger_commit_store_rejects_physical_logical_generation_mismatch() {
        let store = LedgerCommitStore {
            current: Some(CommittedGenerationBytes::new(
                7,
                encoded_payload(&committed_ledger(6)),
            )),
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
    fn ledger_commit_store_rejects_corrupt_store_even_with_genesis() {
        let mut store = LedgerCommitStore::default();
        store
            .write_corrupt_ledger(&ledger())
            .expect("corrupt write");

        let err = store.recover_or_genesis().expect_err("corrupt state");

        assert!(matches!(
            err,
            LedgerCommitError::Recovery(CommitRecoveryError::InvalidSnapshot { generation: 3 })
        ));
    }

    #[test]
    fn record_count_boundary_and_malicious_count_reject_on_recovery() {
        let mut ledger = committed_ledger(1);
        ledger.records = (0..255_u8)
            .map(|id| active_record(&format!("app.slot{id}.v1"), id))
            .collect();
        let mut store = LedgerCommitStore::default();
        store.commit(&ledger).unwrap();
        assert_eq!(store.recover().unwrap().ledger().records().len(), 255);
        ledger.records.push(active_record("app.extra.v1", 254));
        assert!(matches!(
            store.commit(&ledger),
            Err(LedgerCommitError::Integrity(
                LedgerIntegrityError::LimitExceeded { .. }
            ))
        ));
        let payload = encoded_payload(&ledger);
        let invalid = LedgerCommitStore {
            current: Some(CommittedGenerationBytes::new(1, payload)),
        };
        assert!(invalid.recover().is_err());
        assert!(invalid.recover_or_genesis().is_err());
        for bytes in [
            vec![0x9b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
            vec![0x81; 34],
            vec![0xa1],
        ] {
            let payload = bytes;
            let invalid = LedgerCommitStore {
                current: Some(CommittedGenerationBytes::new(1, payload)),
            };
            assert!(matches!(
                invalid.recover(),
                Err(LedgerCommitError::Codec(_))
            ));
        }
    }

    #[test]
    fn oversized_logical_text_remains_opaque_to_record_decode_and_fails_closed() {
        for len in [257, 8192] {
            let mut malformed = serde_json::to_value(active_committed_ledger()).unwrap();
            malformed["records"][0]["stable_key"] = "x".repeat(len).into();
            let payload = crate::test_cbor::to_vec(&malformed).unwrap();
            let mut store = LedgerCommitStore::default();
            store.commit_payload_at_generation(1, payload).unwrap();
            let record = store;
            let bytes = crate::stable_cell::EncodedLedgerRecord::new(&record)
                .unwrap()
                .into_bytes();
            let decoded = crate::decode_stable_cell_commit_store(&bytes).unwrap();
            assert_eq!(decoded, record);
            let before = decoded.clone();
            assert!(matches!(
                decoded.recover(),
                Err(LedgerCommitError::Codec(_))
            ));
            assert!(matches!(
                decoded.recover_or_genesis(),
                Err(LedgerCommitError::Codec(_))
            ));
            assert_eq!(decoded, before);
        }
    }
}
