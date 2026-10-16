use super::{
    AllocationLedger, AllocationRecord, AllocationStageError, AllocationState, ClaimOutcome,
    RecoveredLedger,
};
use crate::{
    RuntimeBootstrapError, RuntimeBootstrapPolicy,
    declaration::{AllocationDeclaration, DeclarationSnapshot},
};

// Own the exact recovered ledger throughout validation and staging. A claim's
// checked index is consumed immediately; no proof can be rebound to another
// same-generation history and no second ownership scan is needed.
pub fn stage_generation<P: RuntimeBootstrapPolicy>(
    recovered: RecoveredLedger,
    snapshot: DeclarationSnapshot,
    policy: &P,
) -> Result<(AllocationLedger, Vec<AllocationDeclaration>), RuntimeBootstrapError<P::Error>> {
    let mut next = recovered.into_ledger();
    for declaration in snapshot.declarations() {
        match crate::validation::check_declaration(&next, declaration, policy)? {
            ClaimOutcome::Existing { record_index } => {
                let record = &mut next.records[record_index];
                record.state = AllocationState::Active;
                record.schema.clone_from(&declaration.schema);
            }
            ClaimOutcome::New => next.records.push(AllocationRecord::active(declaration)),
        }
    }
    // Keep successor failures after all policy/history callbacks, as before.
    // Staged mutation remains local and is discarded on every failure.
    next.validate_bounds().map_err(AllocationStageError::from)?;
    next.current_generation = checked_next_generation(next.current_generation)
        .map_err(|generation| AllocationStageError::GenerationOverflow { generation })?;
    Ok((next, snapshot.into_declarations()))
}

pub const fn checked_next_generation(current_generation: u64) -> Result<u64, u64> {
    match current_generation.checked_add(1) {
        Some(next_generation) => Ok(next_generation),
        None => Err(current_generation),
    }
}
