use super::{
    AllocationLedger, AllocationRecord, AllocationStageError, AllocationState, ClaimConflict,
    ClaimOutcome, validate_declaration_claim,
};
use crate::{capability::ValidatedAllocations, declaration::AllocationDeclaration};

// Bootstrap transfers its already-owned ledger without copying its records.
// Mutation remains local until protected commit. Empty validated commits still
// advance the generation without retaining an event.
pub fn stage_validated_generation(
    ledger: AllocationLedger,
    validated: &ValidatedAllocations,
) -> Result<AllocationLedger, AllocationStageError> {
    if validated.base_generation() != ledger.current_generation {
        return Err(AllocationStageError::StaleValidatedAllocations {
            validated_generation: validated.base_generation(),
            ledger_generation: ledger.current_generation,
        });
    }
    ledger.validate_bounds()?;
    let parent_generation = ledger.current_generation;
    let next_generation = checked_next_generation(parent_generation)
        .map_err(|generation| AllocationStageError::GenerationOverflow { generation })?;
    let staged_declarations = validated.declarations();
    let mut next = ledger;
    next.current_generation = next_generation;

    for declaration in staged_declarations {
        record_declaration(&mut next, declaration)?;
    }

    next.validate_bounds()?;
    Ok(next)
}

fn record_declaration(
    ledger: &mut AllocationLedger,
    declaration: &AllocationDeclaration,
) -> Result<(), AllocationStageError> {
    match validate_declaration_claim(ledger, declaration) {
        Ok(ClaimOutcome::Existing { record_index }) => {
            // Claim validation rejected retired identities. Both an existing
            // active claim and a matching reservation become active here.
            let record = &mut ledger.records[record_index];
            record.state = AllocationState::Active;
            record.schema.clone_from(&declaration.schema);
            Ok(())
        }
        Ok(ClaimOutcome::New) => {
            let record = AllocationRecord::active(declaration);
            ledger.records.push(record);
            Ok(())
        }
        Err(conflict) => Err(map_declaration_stage_conflict(declaration, conflict)),
    }
}

pub const fn checked_next_generation(current_generation: u64) -> Result<u64, u64> {
    match current_generation.checked_add(1) {
        Some(next_generation) => Ok(next_generation),
        None => Err(current_generation),
    }
}

fn map_declaration_stage_conflict(
    declaration: &AllocationDeclaration,
    conflict: ClaimConflict<'_>,
) -> AllocationStageError {
    match conflict {
        ClaimConflict::StableKeyMoved { record } => AllocationStageError::StableKeySlotConflict {
            stable_key: declaration.stable_key.clone(),
            historical_slot: record.slot.clone(),
            declared_slot: declaration.slot.clone(),
        },
        ClaimConflict::SlotReused { record } => AllocationStageError::SlotStableKeyConflict {
            slot: declaration.slot.clone(),
            historical_key: record.stable_key.clone(),
            declared_key: declaration.stable_key.clone(),
        },
        ClaimConflict::Tombstoned { record } => AllocationStageError::RetiredAllocation {
            stable_key: declaration.stable_key.clone(),
            slot: record.slot.clone(),
        },
    }
}
