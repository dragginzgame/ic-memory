use super::{MemoryRuntime, RuntimeLifecycle, RuntimeOpenError};
use crate::{SchemaMetadata, SealedDeclarationSnapshot, StableKey};
use ic_stable_structures::Memory;

///
/// RuntimeAdoptionError
///
/// A consumer's declared requirements do not match the existing committed
/// runtime. Verification never bootstraps, grants authority, opens memory,
/// replays admission, or changes the host's policy or bucket configuration.
///

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum RuntimeAdoptionError {
    /// Runtime readiness, key resolution or requirement verification failed.
    #[error(transparent)]
    Open(#[from] RuntimeOpenError),
    /// The supplied snapshot contains no requests for this authority.
    #[error("no allocation requirements declared by authority '{authority}'")]
    UnknownAuthority { authority: String },
    /// This key was committed under a different current declaration authority.
    #[error(
        "stable key '{stable_key}' is committed under '{committed_authority}', not '{requested_authority}'"
    )]
    AuthorityMismatch {
        stable_key: String,
        committed_authority: String,
        requested_authority: String,
    },
    /// Declared diagnostic schema differs from the commitment.
    #[error("declaration metadata for stable key '{stable_key}' differs from the commitment")]
    DeclarationMetadataMismatch { stable_key: String },
}

impl From<super::RuntimeStateError> for RuntimeAdoptionError {
    fn from(error: super::RuntimeStateError) -> Self {
        Self::Open(RuntimeOpenError::State(error))
    }
}

impl<M: Memory> MemoryRuntime<M> {
    /// Verify every logical request for one authority in
    /// the supplied requirements against this runtime's current commitment.
    ///
    /// Requests must match key, authority and diagnostic schema while
    /// retaining the host's assigned ID. Other authorities and additional
    /// committed keys are ignored. An authority with no requirements rejects.
    /// Grants and application schema semantics are not revalidated. Success
    /// neither grants new access nor proves application lifecycle readiness.
    pub fn verify_authority(
        &self,
        requirements: &SealedDeclarationSnapshot,
        authority: &str,
    ) -> Result<(), RuntimeAdoptionError> {
        let RuntimeLifecycle::Bootstrapped {
            binding,
            committed_allocations,
        } = &self.lifecycle
        else {
            return Err(RuntimeOpenError::NotBootstrapped.into());
        };
        let mut found = false;
        for request in requirements.requests() {
            if request.authority() == authority {
                found = true;
                verify_requirement(
                    committed_allocations,
                    &binding.pool,
                    authority,
                    request.stable_key(),
                    request.schema(),
                )?;
            }
        }
        if !found {
            return Err(RuntimeAdoptionError::UnknownAuthority {
                authority: authority.to_string(),
            });
        }
        Ok(())
    }
}

fn verify_requirement(
    committed: &crate::CommittedAllocations,
    pool: &crate::MemoryAllocationPool,
    authority: &str,
    key: &StableKey,
    schema: &SchemaMetadata,
) -> Result<(), RuntimeAdoptionError> {
    let declaration =
        crate::capability::declaration_for_key(committed.declarations(), key.as_str())
            .ok_or_else(|| RuntimeOpenError::StableKeyNotCommitted(key.to_string()))?;
    let admitted_authority = pool
        .authority_for_key(key)
        .expect("committed key has an admitted owner");
    if admitted_authority != authority {
        return Err(RuntimeAdoptionError::AuthorityMismatch {
            stable_key: key.to_string(),
            committed_authority: admitted_authority.to_string(),
            requested_authority: authority.to_string(),
        });
    }
    if schema != declaration.schema() {
        return Err(RuntimeAdoptionError::DeclarationMetadataMismatch {
            stable_key: key.to_string(),
        });
    }
    Ok(())
}
