use super::{MemoryRuntime, RuntimeLifecycle, RuntimeOpenError};
use crate::{AllocationDeclaration, SchemaMetadata, SealedDeclarationSnapshot, StableKey};
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
    /// Runtime readiness, key resolution or fixed-ID verification failed.
    #[error(transparent)]
    Open(#[from] RuntimeOpenError),
    /// The supplied snapshot contains no fixed declarations or requests for this authority.
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
    /// Declared diagnostic schema or fixed-declaration label differs from the commitment.
    #[error("declaration metadata for stable key '{stable_key}' differs from the commitment")]
    DeclarationMetadataMismatch { stable_key: String },
}

impl From<super::RuntimeStateError> for RuntimeAdoptionError {
    fn from(error: super::RuntimeStateError) -> Self {
        Self::Open(RuntimeOpenError::State(error))
    }
}

impl<M: Memory> MemoryRuntime<M> {
    /// Verify every fixed declaration and logical request for one authority in
    /// the supplied requirements against this runtime's current commitment.
    ///
    /// Fixed declarations must match key, ID, label and diagnostic schema;
    /// logical requests must match key, authority and diagnostic schema while
    /// retaining the host's assigned ID. Other authorities and additional
    /// committed keys are ignored. An authority with no requirements rejects.
    /// Grants and application schema semantics are not revalidated. Success
    /// neither grants new access nor proves application lifecycle readiness.
    pub fn verify_authority(
        &self,
        requirements: &SealedDeclarationSnapshot,
        authority: &str,
    ) -> Result<(), RuntimeAdoptionError> {
        self.committed_allocations()?;
        let mut found = false;
        for registration in requirements.registered_declarations() {
            if registration.authority() == authority {
                found = true;
                let expected = registration.declaration();
                self.verify_requirement(
                    authority,
                    expected.stable_key(),
                    expected.schema(),
                    Some(expected),
                )?;
            }
        }
        for request in requirements.requests() {
            if request.authority() == authority {
                found = true;
                self.verify_requirement(authority, request.stable_key(), request.schema(), None)?;
            }
        }
        if !found {
            return Err(RuntimeAdoptionError::UnknownAuthority {
                authority: authority.to_string(),
            });
        }
        Ok(())
    }

    fn verify_requirement(
        &self,
        authority: &str,
        key: &StableKey,
        schema: &SchemaMetadata,
        fixed: Option<&AllocationDeclaration>,
    ) -> Result<(), RuntimeAdoptionError> {
        let RuntimeLifecycle::Bootstrapped { binding, .. } = &self.lifecycle else {
            return Err(RuntimeOpenError::NotBootstrapped.into());
        };
        let registration = binding
            .declarations
            .registered_declarations()
            .iter()
            .find(|registration| registration.declaration().stable_key() == key)
            .ok_or_else(|| RuntimeOpenError::StableKeyNotCommitted(key.to_string()))?;
        if registration.authority() != authority {
            return Err(RuntimeAdoptionError::AuthorityMismatch {
                stable_key: key.to_string(),
                committed_authority: registration.authority().to_string(),
                requested_authority: authority.to_string(),
            });
        }
        let committed = registration.declaration();
        if let Some(expected) = fixed
            && expected.slot() != committed.slot()
        {
            return Err(RuntimeOpenError::MemoryIdMismatch {
                stable_key: key.to_string(),
                committed_id: committed
                    .slot()
                    .memory_manager_id()
                    .map_err(RuntimeOpenError::from)?,
                requested_id: expected
                    .slot()
                    .memory_manager_id()
                    .map_err(RuntimeOpenError::from)?,
            }
            .into());
        }
        if schema != committed.schema()
            || fixed.is_some_and(|expected| expected.label() != committed.label())
        {
            return Err(RuntimeAdoptionError::DeclarationMetadataMismatch {
                stable_key: key.to_string(),
            });
        }
        Ok(())
    }
}
