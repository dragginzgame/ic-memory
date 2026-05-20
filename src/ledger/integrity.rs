use super::{AllocationLedger, AllocationRecord, AllocationState, LedgerIntegrityError};
use crate::{declaration::validate_runtime_fingerprint, key::StableKey, validation::Validate};
use std::collections::BTreeSet;

impl AllocationLedger {
    /// Validate structural ledger invariants before recovery or commit.
    pub fn validate_integrity(&self) -> Result<(), LedgerIntegrityError> {
        let mut stable_keys = BTreeSet::new();
        let mut slots = BTreeSet::new();

        for record in self.allocation_history.records() {
            if !stable_keys.insert(record.stable_key.clone()) {
                return Err(LedgerIntegrityError::DuplicateStableKey {
                    stable_key: record.stable_key.clone(),
                });
            }
            if !slots.insert(record.slot.clone()) {
                return Err(LedgerIntegrityError::DuplicateSlot {
                    slot: Box::new(record.slot.clone()),
                });
            }
            validate_record_integrity(self.current_generation, record)?;
        }

        let mut generations = BTreeSet::new();
        for generation in self.allocation_history.generations() {
            if !generations.insert(generation.generation) {
                return Err(LedgerIntegrityError::DuplicateGeneration {
                    generation: generation.generation,
                });
            }
            if generation.generation > self.current_generation {
                return Err(LedgerIntegrityError::FutureGeneration {
                    generation: generation.generation,
                    current_generation: self.current_generation,
                });
            }
            if generation
                .parent_generation
                .is_some_and(|parent| parent >= generation.generation)
            {
                return Err(LedgerIntegrityError::InvalidParentGeneration {
                    generation: generation.generation,
                    parent_generation: generation.parent_generation,
                });
            }
        }

        Ok(())
    }

    /// Validate strict committed-ledger invariants before recovery or commit.
    ///
    /// Public durable structs are DTOs: decoded or manually constructed values
    /// are untrusted until this method succeeds.
    pub fn validate_committed_integrity(&self) -> Result<(), LedgerIntegrityError> {
        self.validate_integrity()?;

        if self.current_generation != 0
            && !self
                .allocation_history
                .generations()
                .iter()
                .any(|record| record.generation == self.current_generation)
        {
            return Err(LedgerIntegrityError::MissingCurrentGenerationRecord {
                current_generation: self.current_generation,
            });
        }

        let mut previous = None;
        let mut known_generations = BTreeSet::new();
        for generation in self.allocation_history.generations() {
            validate_runtime_fingerprint(generation.runtime_fingerprint.as_deref())
                .map_err(LedgerIntegrityError::DiagnosticMetadata)?;

            let expected_generation = previous.map_or(1, |previous| previous + 1);
            if generation.generation != expected_generation {
                return Err(LedgerIntegrityError::NonIncreasingGenerationRecords {
                    generation: generation.generation,
                });
            }

            let expected_parent =
                previous.or_else(|| (generation.parent_generation == Some(0)).then_some(0));
            if generation.parent_generation != expected_parent {
                return Err(LedgerIntegrityError::BrokenGenerationChain {
                    generation: generation.generation,
                    expected_parent,
                    actual_parent: generation.parent_generation,
                });
            }

            known_generations.insert(generation.generation);
            previous = Some(generation.generation);
        }

        for record in self.allocation_history.records() {
            validate_known_record_generation(
                &known_generations,
                &record.stable_key,
                record.first_generation,
            )?;
            validate_known_record_generation(
                &known_generations,
                &record.stable_key,
                record.last_seen_generation,
            )?;
            if let Some(retired_generation) = record.retired_generation {
                validate_known_record_generation(
                    &known_generations,
                    &record.stable_key,
                    retired_generation,
                )?;
            }
            for schema in &record.schema_history {
                validate_known_record_generation(
                    &known_generations,
                    &record.stable_key,
                    schema.generation,
                )?;
            }
        }

        Ok(())
    }
}

fn validate_record_integrity(
    current_generation: u64,
    record: &AllocationRecord,
) -> Result<(), LedgerIntegrityError> {
    record
        .stable_key
        .validate()
        .map_err(LedgerIntegrityError::InvalidStableKey)?;
    record
        .slot
        .validate()
        .map_err(LedgerIntegrityError::InvalidSlotDescriptor)?;

    if record.first_generation > record.last_seen_generation {
        return Err(LedgerIntegrityError::InvalidRecordGenerationOrder {
            stable_key: record.stable_key.clone(),
            first_generation: record.first_generation,
            last_seen_generation: record.last_seen_generation,
        });
    }
    if record.last_seen_generation > current_generation {
        return Err(LedgerIntegrityError::FutureRecordGeneration {
            stable_key: record.stable_key.clone(),
            generation: record.last_seen_generation,
            current_generation,
        });
    }

    match (record.state, record.retired_generation) {
        (AllocationState::Retired, Some(retired_generation)) => {
            if retired_generation < record.first_generation {
                return Err(LedgerIntegrityError::RetiredBeforeFirstGeneration {
                    stable_key: record.stable_key.clone(),
                    first_generation: record.first_generation,
                    retired_generation,
                });
            }
            if retired_generation > current_generation {
                return Err(LedgerIntegrityError::FutureRecordGeneration {
                    stable_key: record.stable_key.clone(),
                    generation: retired_generation,
                    current_generation,
                });
            }
        }
        (AllocationState::Retired, None) => {
            return Err(LedgerIntegrityError::MissingRetiredGeneration {
                stable_key: record.stable_key.clone(),
            });
        }
        (AllocationState::Reserved | AllocationState::Active, Some(_)) => {
            return Err(LedgerIntegrityError::UnexpectedRetiredGeneration {
                stable_key: record.stable_key.clone(),
            });
        }
        (AllocationState::Reserved | AllocationState::Active, None) => {}
    }

    validate_schema_history_integrity(current_generation, record)
}

fn validate_known_record_generation(
    known_generations: &BTreeSet<u64>,
    stable_key: &StableKey,
    generation: u64,
) -> Result<(), LedgerIntegrityError> {
    if known_generations.contains(&generation) {
        return Ok(());
    }
    Err(LedgerIntegrityError::UnknownRecordGeneration {
        stable_key: stable_key.clone(),
        generation,
    })
}

fn validate_schema_history_integrity(
    current_generation: u64,
    record: &AllocationRecord,
) -> Result<(), LedgerIntegrityError> {
    if record.schema_history.is_empty() {
        return Err(LedgerIntegrityError::EmptySchemaHistory {
            stable_key: record.stable_key.clone(),
        });
    }

    let mut previous = None;
    for schema in &record.schema_history {
        schema
            .schema
            .validate()
            .map_err(|error| LedgerIntegrityError::InvalidSchemaMetadata {
                stable_key: record.stable_key.clone(),
                generation: schema.generation,
                error,
            })?;
        if previous.is_some_and(|generation| schema.generation <= generation) {
            return Err(LedgerIntegrityError::NonIncreasingSchemaHistory {
                stable_key: record.stable_key.clone(),
            });
        }
        if schema.generation < record.first_generation || schema.generation > current_generation {
            return Err(LedgerIntegrityError::SchemaHistoryOutOfBounds {
                stable_key: record.stable_key.clone(),
                generation: schema.generation,
            });
        }
        previous = Some(schema.generation);
    }

    Ok(())
}
