use super::{AllocationLedger, AllocationRecord, AllocationState, LedgerIntegrityError};
use std::collections::BTreeSet;

impl AllocationLedger {
    pub(crate) fn validate_bounds(&self) -> Result<(), LedgerIntegrityError> {
        for (resource, count, limit) in [
            (
                "allocation records",
                self.allocation_history.records().len(),
                crate::constants::MAX_ALLOCATIONS,
            ),
            (
                "generation history",
                self.allocation_history.generations().len(),
                crate::constants::MAX_LEDGER_GENERATIONS,
            ),
            (
                "schema history",
                self.allocation_history
                    .records()
                    .iter()
                    .map(|r| r.schema_history.len())
                    .sum(),
                crate::constants::MAX_LEDGER_GENERATIONS,
            ),
        ] {
            if count > limit {
                return Err(LedgerIntegrityError::LimitExceeded { resource, limit });
            }
        }
        Ok(())
    }

    pub(crate) fn validate_staging_bounds(&self) -> Result<(), LedgerIntegrityError> {
        self.validate_bounds()?;
        if self.allocation_history.generations().len() >= crate::constants::MAX_LEDGER_GENERATIONS {
            return Err(LedgerIntegrityError::LimitExceeded {
                resource: "generation history",
                limit: crate::constants::MAX_LEDGER_GENERATIONS,
            });
        }
        Ok(())
    }

    /// Validate structural ledger invariants before recovery or commit.
    pub fn validate_integrity(&self) -> Result<(), LedgerIntegrityError> {
        self.validate_bounds()?;
        let mut stable_keys = BTreeSet::new();
        // Slot construction and decoding have already excluded the sentinel.
        let mut slots = [false; crate::constants::MAX_ALLOCATIONS];

        for record in self.allocation_history.records() {
            if !stable_keys.insert(&record.stable_key) {
                return Err(LedgerIntegrityError::DuplicateStableKey {
                    stable_key: record.stable_key.clone(),
                });
            }
            let occupied = &mut slots[usize::from(record.slot.id())];
            if *occupied {
                return Err(LedgerIntegrityError::DuplicateSlot {
                    slot: record.slot.clone(),
                });
            }
            *occupied = true;
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
            if generation.parent_generation >= generation.generation {
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

        let mut expected_parent = 0;
        for generation in self.allocation_history.generations() {
            if generation.generation != expected_parent + 1 {
                return Err(LedgerIntegrityError::NonIncreasingGenerationRecords {
                    generation: generation.generation,
                });
            }

            if generation.parent_generation != expected_parent {
                return Err(LedgerIntegrityError::BrokenGenerationChain {
                    generation: generation.generation,
                    expected_parent,
                    actual_parent: generation.parent_generation,
                });
            }

            expected_parent = generation.generation;
        }

        // The checked chain contains exactly generations 1..=current_generation.
        // Structural validation bounds every record reference by its first
        // generation and current_generation, so only genesis exclusion remains.
        for record in self.allocation_history.records() {
            if record.first_generation == 0 {
                return Err(LedgerIntegrityError::UnknownRecordGeneration {
                    stable_key: record.stable_key.clone(),
                    generation: 0,
                });
            }
        }

        Ok(())
    }
}

fn validate_record_integrity(
    current_generation: u64,
    record: &AllocationRecord,
) -> Result<(), LedgerIntegrityError> {
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

    match record.state {
        AllocationState::Retired {
            generation: retired_generation,
        } => {
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
            if retired_generation <= record.last_seen_generation {
                return Err(LedgerIntegrityError::RetirementNotAfterLastSeen {
                    stable_key: record.stable_key.clone(),
                    last_seen_generation: record.last_seen_generation,
                    retired_generation,
                });
            }
        }
        AllocationState::Reserved | AllocationState::Active => {}
    }

    validate_schema_history_integrity(current_generation, record)
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

    let first_schema_generation = record.schema_history[0].generation;
    if first_schema_generation != record.first_generation {
        return Err(LedgerIntegrityError::SchemaHistoryStartMismatch {
            stable_key: record.stable_key.clone(),
            first_generation: record.first_generation,
            schema_generation: first_schema_generation,
        });
    }

    let mut previous = None;
    for schema in &record.schema_history {
        if previous.is_some_and(|generation| schema.generation <= generation) {
            return Err(LedgerIntegrityError::NonIncreasingSchemaHistory {
                stable_key: record.stable_key.clone(),
            });
        }
        // The matching first entry and strict ordering establish the lower bound.
        if schema.generation > current_generation {
            return Err(LedgerIntegrityError::SchemaHistoryOutOfBounds {
                stable_key: record.stable_key.clone(),
                generation: schema.generation,
            });
        }
        if schema.generation > record.last_seen_generation {
            return Err(LedgerIntegrityError::SchemaHistoryAfterLastSeen {
                stable_key: record.stable_key.clone(),
                generation: schema.generation,
                last_seen_generation: record.last_seen_generation,
            });
        }
        previous = Some(schema.generation);
    }

    Ok(())
}
