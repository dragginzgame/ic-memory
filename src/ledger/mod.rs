pub(crate) mod claim;
mod error;
mod integrity;
mod record;
mod stage;

use crate::physical::{CommitRecoveryError, DualCommitStore};
use serde::{Deserialize, Serialize};

pub use error::{
    AllocationReservationError, AllocationRetirementError, AllocationStageError, LedgerCommitError,
    LedgerCompatibilityError, LedgerIntegrityError,
};
pub use record::{
    AllocationHistory, AllocationLedger, AllocationRecord, AllocationRetirement, AllocationState,
    CURRENT_LEDGER_SCHEMA_VERSION, CURRENT_PHYSICAL_FORMAT_ID, GenerationRecord,
    LedgerCompatibility, SchemaMetadataRecord,
};

///
/// LedgerCodec
///
/// Integration-supplied encoding for persisted allocation ledgers.
///
/// Decoding returns an untrusted durable DTO. Callers should recover ledgers
/// through [`LedgerCommitStore`], which checks physical/logical generation,
/// compatibility, and committed ledger integrity before returning authoritative
/// state.
///

pub trait LedgerCodec {
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
/// Native CBOR ledger codec for persisted [`AllocationLedger`] payloads.
///
/// This is the default codec for the native IC stack:
/// `MemoryManager` ID 0 stores an `ic-stable-structures::Cell` containing a
/// [`crate::StableCellLedgerRecord`], whose [`LedgerCommitStore`] contains
/// dual protected CBOR-encoded ledger generations.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CborLedgerCodec;

impl LedgerCodec for CborLedgerCodec {
    type Error = serde_cbor::Error;

    fn encode(&self, ledger: &AllocationLedger) -> Result<Vec<u8>, Self::Error> {
        serde_cbor::to_vec(ledger)
    }

    fn decode(&self, bytes: &[u8]) -> Result<AllocationLedger, Self::Error> {
        serde_cbor::from_slice(bytes)
    }
}

///
/// LedgerCommitStore
///
/// Generation-scoped allocation ledger commit store.
///
/// This type owns the generic commit lifecycle. It deliberately does not own
/// serialization or stable-memory IO; those remain substrate/integration
/// responsibilities.
///
/// This store commits allocation ledger generations. It does not open
/// stable-memory handles and does not allocate application slots.
///
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
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

    #[cfg(test)]
    pub(crate) const fn physical_mut(&mut self) -> &mut DualCommitStore {
        &mut self.physical
    }

    /// Recover the authoritative allocation ledger using `codec`.
    pub fn recover<C: LedgerCodec>(
        &self,
        codec: &C,
    ) -> Result<AllocationLedger, LedgerCommitError<C::Error>> {
        self.recover_with_compatibility(codec, LedgerCompatibility::current())
    }

    /// Recover the authoritative allocation ledger using explicit compatibility rules.
    ///
    /// This is an advanced migration/adaptor hook. Normal readers should use
    /// [`LedgerCommitStore::recover`], which applies this crate version's current
    /// compatibility rules. Do not broaden `compatibility` unless the supplied
    /// codec and this reader's integrity checks fully understand every accepted
    /// ledger schema version.
    pub fn recover_with_compatibility<C: LedgerCodec>(
        &self,
        codec: &C,
        compatibility: LedgerCompatibility,
    ) -> Result<AllocationLedger, LedgerCommitError<C::Error>> {
        let committed = self
            .physical
            .authoritative()
            .map_err(LedgerCommitError::Recovery)?;
        let ledger = codec
            .decode(committed.payload())
            .map_err(LedgerCommitError::Codec)?;
        if committed.generation() != ledger.current_generation {
            return Err(LedgerCommitError::PhysicalLogicalGenerationMismatch {
                physical_generation: committed.generation(),
                logical_generation: ledger.current_generation,
            });
        }
        compatibility
            .validate(&ledger)
            .map_err(LedgerCommitError::Compatibility)?;
        ledger
            .validate_committed_integrity()
            .map_err(LedgerCommitError::Integrity)?;
        Ok(ledger)
    }

    /// Recover the authoritative ledger, or explicitly initialize an empty store.
    ///
    /// Initialization is allowed only when no physical commit slot has ever
    /// been written. Corrupt or partially written stores fail closed even when
    /// a genesis ledger is supplied.
    pub fn recover_or_initialize<C: LedgerCodec>(
        &mut self,
        codec: &C,
        genesis: &AllocationLedger,
    ) -> Result<AllocationLedger, LedgerCommitError<C::Error>> {
        self.recover_or_initialize_with_compatibility(
            codec,
            genesis,
            LedgerCompatibility::current(),
        )
    }

    /// Recover the authoritative ledger, or initialize an empty store with explicit compatibility.
    ///
    /// This is an advanced migration/adaptor hook. Normal bootstrap code should
    /// use [`LedgerCommitStore::recover_or_initialize`]. Do not broaden
    /// `compatibility` unless the supplied codec and this reader's integrity
    /// checks fully understand every accepted ledger schema version.
    pub fn recover_or_initialize_with_compatibility<C: LedgerCodec>(
        &mut self,
        codec: &C,
        genesis: &AllocationLedger,
        compatibility: LedgerCompatibility,
    ) -> Result<AllocationLedger, LedgerCommitError<C::Error>> {
        match self.recover_with_compatibility(codec, compatibility) {
            Ok(ledger) => Ok(ledger),
            Err(LedgerCommitError::Recovery(CommitRecoveryError::NoValidGeneration))
                if self.physical.is_uninitialized() =>
            {
                self.commit_with_compatibility(genesis, codec, compatibility)
            }
            Err(err) => Err(err),
        }
    }

    /// Commit one logical allocation ledger generation through `codec`.
    pub fn commit<C: LedgerCodec>(
        &mut self,
        ledger: &AllocationLedger,
        codec: &C,
    ) -> Result<AllocationLedger, LedgerCommitError<C::Error>> {
        self.commit_with_compatibility(ledger, codec, LedgerCompatibility::current())
    }

    /// Commit one logical allocation ledger generation through explicit compatibility.
    ///
    /// This is an advanced migration/adaptor hook. Normal writers should use
    /// [`LedgerCommitStore::commit`]. Do not broaden `compatibility` unless the
    /// supplied codec and this reader's integrity checks fully understand every
    /// accepted ledger schema version.
    pub fn commit_with_compatibility<C: LedgerCodec>(
        &mut self,
        ledger: &AllocationLedger,
        codec: &C,
        compatibility: LedgerCompatibility,
    ) -> Result<AllocationLedger, LedgerCommitError<C::Error>> {
        compatibility
            .validate(ledger)
            .map_err(LedgerCommitError::Compatibility)?;
        ledger
            .validate_committed_integrity()
            .map_err(LedgerCommitError::Integrity)?;
        let payload = codec.encode(ledger).map_err(LedgerCommitError::Codec)?;
        self.physical
            .commit_payload_at_generation(ledger.current_generation, payload)
            .map_err(LedgerCommitError::Recovery)?;
        self.recover_with_compatibility(codec, compatibility)
    }

    /// Simulate a torn write of a logical ledger payload into the inactive slot.
    #[cfg(test)]
    pub fn write_corrupt_inactive_ledger<C: LedgerCodec>(
        &mut self,
        ledger: &AllocationLedger,
        codec: &C,
    ) -> Result<(), LedgerCommitError<C::Error>> {
        let payload = codec.encode(ledger).map_err(LedgerCommitError::Codec)?;
        self.physical
            .write_corrupt_inactive_slot(ledger.current_generation, payload);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        declaration::{AllocationDeclaration, DeclarationSnapshot, DeclarationSnapshotError},
        key::StableKey,
        physical::CommittedGenerationBytes,
        schema::{SchemaMetadata, SchemaMetadataError},
        slot::{AllocationSlotDescriptor, MEMORY_MANAGER_INVALID_ID, MemoryManagerSlotError},
    };
    use std::cell::RefCell;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct TestCodec;

    impl LedgerCodec for TestCodec {
        type Error = &'static str;

        fn encode(&self, ledger: &AllocationLedger) -> Result<Vec<u8>, Self::Error> {
            let mut bytes = Vec::with_capacity(16);
            bytes.extend_from_slice(&ledger.ledger_schema_version.to_le_bytes());
            bytes.extend_from_slice(&ledger.physical_format_id.to_le_bytes());
            bytes.extend_from_slice(&ledger.current_generation.to_le_bytes());
            Ok(bytes)
        }

        fn decode(&self, bytes: &[u8]) -> Result<AllocationLedger, Self::Error> {
            let bytes = <[u8; 16]>::try_from(bytes).map_err(|_| "invalid ledger")?;
            let ledger_schema_version =
                u32::from_le_bytes(bytes[0..4].try_into().map_err(|_| "invalid schema")?);
            let physical_format_id =
                u32::from_le_bytes(bytes[4..8].try_into().map_err(|_| "invalid format")?);
            let current_generation =
                u64::from_le_bytes(bytes[8..16].try_into().map_err(|_| "invalid generation")?);
            let mut ledger = committed_ledger(current_generation);
            ledger.ledger_schema_version = ledger_schema_version;
            ledger.physical_format_id = physical_format_id;
            Ok(ledger)
        }
    }

    #[derive(Debug, Default)]
    struct FullLedgerCodec {
        ledgers: RefCell<Vec<AllocationLedger>>,
    }

    impl LedgerCodec for FullLedgerCodec {
        type Error = &'static str;

        fn encode(&self, ledger: &AllocationLedger) -> Result<Vec<u8>, Self::Error> {
            let mut ledgers = self.ledgers.borrow_mut();
            let index = u64::try_from(ledgers.len()).map_err(|_| "too many ledgers")?;
            ledgers.push(ledger.clone());
            Ok(index.to_le_bytes().to_vec())
        }

        fn decode(&self, bytes: &[u8]) -> Result<AllocationLedger, Self::Error> {
            let bytes = <[u8; 8]>::try_from(bytes).map_err(|_| "invalid ledger index")?;
            let index =
                usize::try_from(u64::from_le_bytes(bytes)).map_err(|_| "invalid ledger index")?;
            self.ledgers
                .borrow()
                .get(index)
                .cloned()
                .ok_or("unknown ledger index")
        }
    }

    fn declaration(key: &str, id: u8, schema_version: Option<u32>) -> AllocationDeclaration {
        AllocationDeclaration::new(
            key,
            AllocationSlotDescriptor::memory_manager(id).expect("usable slot"),
            None,
            SchemaMetadata {
                schema_version,
                schema_fingerprint: None,
            },
        )
        .expect("declaration")
    }

    fn invalid_schema_metadata() -> SchemaMetadata {
        SchemaMetadata {
            schema_version: Some(0),
            schema_fingerprint: None,
        }
    }

    fn declaration_with_invalid_schema(key: &str, id: u8) -> AllocationDeclaration {
        let mut declaration = declaration(key, id, Some(1));
        declaration.schema = invalid_schema_metadata();
        declaration
    }

    fn ledger() -> AllocationLedger {
        AllocationLedger {
            ledger_schema_version: CURRENT_LEDGER_SCHEMA_VERSION,
            physical_format_id: CURRENT_PHYSICAL_FORMAT_ID,
            current_generation: 3,
            allocation_history: AllocationHistory::default(),
        }
    }

    fn committed_ledger(current_generation: u64) -> AllocationLedger {
        AllocationLedger {
            ledger_schema_version: CURRENT_LEDGER_SCHEMA_VERSION,
            physical_format_id: CURRENT_PHYSICAL_FORMAT_ID,
            current_generation,
            allocation_history: AllocationHistory::from_parts(
                Vec::new(),
                (1..=current_generation)
                    .map(|generation| GenerationRecord {
                        generation,
                        parent_generation: if generation == 1 {
                            Some(0)
                        } else {
                            Some(generation - 1)
                        },
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
    }

    fn validated(
        generation: u64,
        declarations: Vec<AllocationDeclaration>,
    ) -> crate::session::ValidatedAllocations {
        crate::session::ValidatedAllocations::new(generation, declarations, None)
    }

    fn record<'ledger>(ledger: &'ledger AllocationLedger, key: &str) -> &'ledger AllocationRecord {
        ledger
            .allocation_history
            .records()
            .iter()
            .find(|record| record.stable_key.as_str() == key)
            .expect("allocation record")
    }

    #[test]
    fn allocation_history_accessors_expose_read_only_views() {
        let history = AllocationHistory::from_parts(
            vec![active_record("app.users.v1", 100)],
            vec![GenerationRecord::new(1, Some(0), None, 1, Some(42)).expect("generation record")],
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

        let generation_err = GenerationRecord::new(1, Some(0), Some(String::new()), 0, None)
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
        let validated = crate::session::ValidatedAllocations::new(
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
            Some(3)
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
        let validated = crate::session::ValidatedAllocations::new(
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
    fn stage_reservation_generation_rejects_same_key_different_slot() {
        let mut ledger = ledger();
        *ledger.allocation_history.records_mut() = vec![AllocationRecord::reserved(
            3,
            declaration("app.future_store.v1", 100, None),
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
        *ledger.allocation_history.records_mut() = vec![AllocationRecord::reserved(
            3,
            declaration("app.future_store.v1", 100, None),
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
    fn snapshot_can_feed_validated_generation() {
        let snapshot = DeclarationSnapshot::new(vec![declaration("app.users.v1", 100, None)])
            .expect("snapshot");
        let (declarations, runtime_fingerprint) = snapshot.into_parts();
        let validated =
            crate::session::ValidatedAllocations::new(3, declarations, runtime_fingerprint);

        let staged = ledger()
            .stage_validated_generation(&validated, None)
            .expect("validated generation");

        assert_eq!(staged.allocation_history.records().len(), 1);
    }

    #[test]
    fn stage_validated_generation_records_runtime_fingerprint() {
        let validated = crate::session::ValidatedAllocations::new(
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

    #[test]
    fn new_committed_requires_strict_generation_history() {
        let structurally_valid = AllocationLedger::new(
            CURRENT_LEDGER_SCHEMA_VERSION,
            CURRENT_PHYSICAL_FORMAT_ID,
            3,
            AllocationHistory::default(),
        )
        .expect("structurally valid DTO");

        assert_eq!(structurally_valid.current_generation, 3);

        let err = AllocationLedger::new_committed(
            CURRENT_LEDGER_SCHEMA_VERSION,
            CURRENT_PHYSICAL_FORMAT_ID,
            3,
            AllocationHistory::default(),
        )
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
        let mut bytes = serde_cbor::to_vec(&ledger).expect("encode ledger");
        let key_start = bytes
            .windows(b"app.users.v1".len())
            .position(|window| window == b"app.users.v1")
            .expect("encoded stable key");
        bytes[key_start] = b'A';
        let decoded: AllocationLedger = serde_cbor::from_slice(&bytes).expect("decode ledger");

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
                crate::slot::AllocationSlotDescriptorError::MemoryManager(
                    MemoryManagerSlotError::InvalidMemoryManagerId { id }
                )
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
        let codec = TestCodec;
        let mut invalid = ledger();
        *invalid.allocation_history.records_mut() = vec![
            active_record("app.users.v1", 100),
            active_record("app.orders.v1", 100),
        ];

        let err = store.commit(&invalid, &codec).expect_err("invalid ledger");

        assert!(matches!(
            err,
            LedgerCommitError::Integrity(LedgerIntegrityError::DuplicateSlot { .. })
        ));
        assert!(store.physical().is_uninitialized());
    }

    #[test]
    fn ledger_commit_store_recovers_latest_committed_ledger() {
        let mut store = LedgerCommitStore::default();
        let codec = TestCodec;
        let first = committed_ledger(1);
        let second = committed_ledger(2);

        store.commit(&first, &codec).expect("first commit");
        store.commit(&second, &codec).expect("second commit");
        let recovered = store.recover(&codec).expect("recovered ledger");

        assert_eq!(recovered.current_generation, 2);
    }

    #[test]
    fn ledger_commit_store_recovers_compatible_genesis_and_first_real_commit() {
        let mut store = LedgerCommitStore::default();
        let codec = FullLedgerCodec::default();
        let genesis = committed_ledger(0);

        let recovered = store
            .recover_or_initialize(&codec, &genesis)
            .expect("compatible genesis ledger");
        assert_eq!(recovered.current_generation, 0);
        assert!(recovered.allocation_history.generations().is_empty());

        let first = recovered
            .stage_validated_generation(
                &validated(0, vec![declaration("app.users.v1", 100, Some(1))]),
                None,
            )
            .expect("first real generation");
        let recovered = store.commit(&first, &codec).expect("first commit");

        assert_eq!(recovered.current_generation, 1);
        assert_eq!(recovered.allocation_history.generations()[0].generation, 1);
        assert_eq!(record(&recovered, "app.users.v1").first_generation, 1);
    }

    #[test]
    fn ledger_commit_store_recovers_full_payload_after_corrupt_latest_slot() {
        let mut store = LedgerCommitStore::default();
        let codec = FullLedgerCodec::default();
        let genesis = committed_ledger(0);
        store.commit(&genesis, &codec).expect("genesis commit");
        let first = genesis
            .stage_validated_generation(
                &validated(0, vec![declaration("app.users.v1", 100, Some(1))]),
                None,
            )
            .expect("first generation");
        let first = store.commit(&first, &codec).expect("first commit");
        let second = first
            .stage_validated_generation(
                &validated(1, vec![declaration("app.users.v1", 100, Some(2))]),
                None,
            )
            .expect("second generation");

        store
            .write_corrupt_inactive_ledger(&second, &codec)
            .expect("corrupt latest");
        let recovered = store.recover(&codec).expect("recover prior generation");

        assert_eq!(recovered.current_generation, 1);
        assert_eq!(record(&recovered, "app.users.v1").schema_history.len(), 1);
    }

    #[test]
    fn ledger_commit_store_recovers_identical_duplicate_slots() {
        let codec = FullLedgerCodec::default();
        let ledger = committed_ledger(0)
            .stage_validated_generation(
                &validated(0, vec![declaration("app.users.v1", 100, Some(1))]),
                None,
            )
            .expect("first generation");
        let payload = codec.encode(&ledger).expect("payload");
        let committed = CommittedGenerationBytes::new(ledger.current_generation, payload);
        let store = LedgerCommitStore {
            physical: DualCommitStore {
                slot0: Some(committed.clone()),
                slot1: Some(committed),
            },
        };

        let recovered = store.recover(&codec).expect("recovered");

        assert_eq!(recovered, ledger);
    }

    #[test]
    fn ledger_commit_store_ignores_corrupt_inactive_ledger() {
        let mut store = LedgerCommitStore::default();
        let codec = TestCodec;
        let first = committed_ledger(1);
        let second = committed_ledger(2);

        store.commit(&first, &codec).expect("first commit");
        store
            .write_corrupt_inactive_ledger(&second, &codec)
            .expect("corrupt write");
        let recovered = store.recover(&codec).expect("recovered ledger");

        assert_eq!(recovered.current_generation, 1);
    }

    #[test]
    fn ledger_commit_store_rejects_physical_logical_generation_mismatch() {
        let store = LedgerCommitStore {
            physical: DualCommitStore {
                slot0: Some(CommittedGenerationBytes::new(
                    7,
                    TestCodec.encode(&committed_ledger(6)).expect("payload"),
                )),
                slot1: None,
            },
        };
        let codec = TestCodec;

        let err = store.recover(&codec).expect_err("mismatch");

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
        let codec = TestCodec;
        store
            .commit(&committed_ledger(1), &codec)
            .expect("first commit");

        let err = store
            .commit(&committed_ledger(3), &codec)
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
        let codec = TestCodec;
        let genesis = committed_ledger(3);

        let recovered = store
            .recover_or_initialize(&codec, &genesis)
            .expect("initialized ledger");

        assert_eq!(recovered.current_generation, 3);
        assert!(!store.physical().is_uninitialized());
    }

    #[test]
    fn ledger_commit_store_rejects_corrupt_store_even_with_genesis() {
        let mut store = LedgerCommitStore::default();
        let codec = TestCodec;
        store
            .write_corrupt_inactive_ledger(&ledger(), &codec)
            .expect("corrupt write");

        let err = store
            .recover_or_initialize(&codec, &ledger())
            .expect_err("corrupt state");

        assert!(matches!(
            err,
            LedgerCommitError::Recovery(CommitRecoveryError::NoValidGeneration)
        ));
    }

    #[test]
    fn ledger_commit_store_rejects_incompatible_schema_before_write() {
        let mut store = LedgerCommitStore::default();
        let codec = TestCodec;
        let incompatible = AllocationLedger {
            ledger_schema_version: CURRENT_LEDGER_SCHEMA_VERSION + 1,
            ..committed_ledger(0)
        };

        let err = store
            .commit(&incompatible, &codec)
            .expect_err("incompatible schema");

        assert!(matches!(
            err,
            LedgerCommitError::Compatibility(
                LedgerCompatibilityError::UnsupportedLedgerSchemaVersion { .. }
            )
        ));
        assert!(store.physical().is_uninitialized());
    }

    #[test]
    fn ledger_commit_store_rejects_incompatible_schema_on_recovery() {
        let mut store = LedgerCommitStore::default();
        let codec = TestCodec;
        let incompatible = AllocationLedger {
            ledger_schema_version: CURRENT_LEDGER_SCHEMA_VERSION + 1,
            ..committed_ledger(3)
        };
        let payload = codec.encode(&incompatible).expect("payload");
        store
            .physical_mut()
            .commit_payload_at_generation(incompatible.current_generation, payload)
            .expect("physical commit");

        let err = store.recover(&codec).expect_err("incompatible schema");

        assert!(matches!(
            err,
            LedgerCommitError::Compatibility(
                LedgerCompatibilityError::UnsupportedLedgerSchemaVersion { .. }
            )
        ));
    }

    #[test]
    fn ledger_commit_store_rejects_incompatible_physical_format() {
        let mut store = LedgerCommitStore::default();
        let codec = TestCodec;
        let incompatible = AllocationLedger {
            physical_format_id: CURRENT_PHYSICAL_FORMAT_ID + 1,
            ..committed_ledger(0)
        };

        let err = store
            .recover_or_initialize(&codec, &incompatible)
            .expect_err("incompatible format");

        assert!(matches!(
            err,
            LedgerCommitError::Compatibility(
                LedgerCompatibilityError::UnsupportedPhysicalFormat { .. }
            )
        ));
        assert!(store.physical().is_uninitialized());
    }
}
