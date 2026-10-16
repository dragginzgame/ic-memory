use crate::{
    declaration::AllocationDeclaration,
    slot::{
        IC_MEMORY_AUTHORITY_OWNER, IC_MEMORY_LEDGER_STABLE_KEY, MEMORY_MANAGER_LEDGER_ID,
        is_ic_memory_stable_key,
    },
    text::validate_diagnostic_text,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

///
/// MemoryRequest
///
/// Key-only request resolved after ledger recovery. All admitted owners share the
/// host's free application pool; known keys retain their durable slot.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MemoryRequest {
    authority: String,
    stable_key: crate::StableKey,
}

impl<'de> Deserialize<'de> for MemoryRequest {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error as _;
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Request {
            authority: String,
            stable_key: crate::StableKey,
        }
        let request = Request::deserialize(deserializer)?;
        validate_external_authority(&request.authority).map_err(D::Error::custom)?;
        validate_external_key(&request.stable_key).map_err(D::Error::custom)?;
        Ok(Self {
            authority: request.authority,
            stable_key: request.stable_key,
        })
    }
}

impl MemoryRequest {
    /// Build a checked logical request before sealing.
    pub fn new(
        authority: impl Into<String>,
        stable_key: &str,
    ) -> Result<Self, MemoryDeclarationError> {
        let authority = authority.into();
        let stable_key = checked_request_key(&authority, stable_key)?;
        Ok(Self {
            authority,
            stable_key,
        })
    }

    /// Borrow the requested durable key.
    #[must_use]
    pub const fn stable_key(&self) -> &crate::StableKey {
        &self.stable_key
    }

    /// Borrow the declaring authority.
    #[must_use]
    pub fn authority(&self) -> &str {
        &self.authority
    }
}

///
/// MemoryDeclarationError
///
/// Failure to construct a complete allocation declaration snapshot.
///

#[non_exhaustive]
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum MemoryDeclarationError {
    #[error("at most 254 external requests are supported")]
    TooManyDeclarations,
    #[error("duplicate requested stable key {stable_key}")]
    DuplicateRequest { stable_key: crate::StableKey },
    /// Declaration validation failed.
    #[error(transparent)]
    Declaration(#[from] crate::DeclarationSnapshotError),
    /// A request supplied an invalid authority identifier.
    #[error("authority {reason}")]
    InvalidAuthority {
        /// Validation failure.
        reason: &'static str,
    },
    /// A request attempted to impersonate the internal authority.
    #[error("authority '{authority}' is reserved for ic-memory runtime internals")]
    ReservedAuthority {
        /// Reserved authority identifier.
        authority: String,
    },
    /// A request attempted to claim the internal stable-key namespace.
    #[error("stable key '{stable_key}' is reserved for ic-memory runtime internals")]
    ReservedStableKey {
        /// Reserved stable key.
        stable_key: String,
    },
}

///
/// SealedDeclarationSnapshot
///
/// Immutable, canonical host-composed allocation requests supplied to each concrete [`crate::MemoryRuntime`].
///
/// Construction takes ownership of the complete request list and validates its
/// canonical meaning. Clones share the same immutable snapshot. This value contains
/// declaration authority only; it contains no memory handles, recovery state,
/// bootstrap lifecycle, or committed allocation capability.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SealedDeclarationSnapshot {
    inner: Arc<Vec<MemoryRequest>>,
}

///
/// DeclarationSnapshot
///
/// Internal canonical output of resolution, without open authority.
/// Sealing, recovered-ledger admission and the resolver establish its bounds
/// and unique keys/slots. Only this module constructs production snapshots.
///

#[derive(Debug, Eq, PartialEq)]
pub struct DeclarationSnapshot {
    declarations: Vec<AllocationDeclaration>,
}

impl DeclarationSnapshot {
    pub(crate) fn declarations(&self) -> &[AllocationDeclaration] {
        &self.declarations
    }

    pub(crate) fn into_declarations(self) -> Vec<AllocationDeclaration> {
        self.declarations
    }

    // Explicit synthetic staging inputs remain confined to unit fixtures.
    #[cfg(test)]
    pub(crate) fn for_test(mut declarations: Vec<AllocationDeclaration>) -> Self {
        crate::sort::sort_unstable_by(&mut declarations, |a, b| a.stable_key.cmp(&b.stable_key));
        Self { declarations }
    }
}

impl SealedDeclarationSnapshot {
    /// Take ownership of the host's complete requests, checking the bound and
    /// rejecting duplicate keys before sharing their canonical contents.
    pub fn new(mut requests: Vec<MemoryRequest>) -> Result<Self, MemoryDeclarationError> {
        if requests.len() > 254 {
            return Err(MemoryDeclarationError::TooManyDeclarations);
        }
        crate::sort::sort_unstable_by(&mut requests, |a, b| a.stable_key.cmp(&b.stable_key));
        for pair in requests.windows(2) {
            if pair[0].stable_key == pair[1].stable_key {
                return Err(MemoryDeclarationError::DuplicateRequest {
                    stable_key: pair[1].stable_key.clone(),
                });
            }
        }
        Ok(Self {
            inner: Arc::new(requests),
        })
    }

    /// Borrow canonical unresolved key-only requests.
    #[must_use]
    pub fn requests(&self) -> &[MemoryRequest] {
        &self.inner
    }

    pub(crate) fn resolve(
        &self,
        recovered: &crate::RecoveredLedger,
        occupied: &[bool; crate::constants::MAX_ALLOCATIONS],
        historical: Vec<AllocationDeclaration>,
        pool: &crate::MemoryAllocationPool,
    ) -> Result<DeclarationSnapshot, crate::MemoryResolutionError> {
        let ledger = recovered.ledger();
        if self.requests().len() + historical.len() > 254 {
            return Err(MemoryDeclarationError::TooManyDeclarations.into());
        }
        let mut declarations = Vec::with_capacity(self.requests().len() + historical.len() + 1);
        declarations.push(internal_ledger_declaration());
        // Retained slots are already occupied. The iterator consumes each new
        // ID once, so no request rescans or updates that recovered occupancy.
        let mut available_ids = 0..crate::MEMORY_MANAGER_INVALID_ID;
        // Admission already resolved historical selections against this ledger
        // and pool. Only the original requests can allocate new slots.
        for request in self.requests() {
            let historical = ledger
                .records()
                .iter()
                .find(|record| record.stable_key() == &request.stable_key);
            pool.validate_authority(&request.stable_key, &request.authority)?;
            let id = if let Some(record) = historical {
                // Durable identity selects placement; the host separately grants
                // current key ownership and retains explicit physical exclusions.
                pool.validate_id(record.slot().id())?;
                record.slot().id()
            } else {
                available_ids
                    .find(|id| pool.contains(*id) && !occupied[usize::from(*id)])
                    .ok_or_else(|| crate::MemoryResolutionError::Exhausted {
                        stable_key: request.stable_key.clone(),
                        authority: request.authority.clone(),
                    })?
            };
            let slot = crate::MemoryManagerSlot::new(id).expect("usable id");
            // Resolution retains checked allocation identity without copying the owner.
            declarations.push(AllocationDeclaration {
                stable_key: request.stable_key.clone(),
                slot,
            });
        }
        declarations.extend(historical);
        crate::sort::sort_unstable_by(&mut declarations, |a, b| a.stable_key.cmp(&b.stable_key));
        Ok(DeclarationSnapshot { declarations })
    }

    #[cfg(test)]
    pub(crate) fn shares_storage_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

// Admission needs a checked key, without allocating a discarded request owner.
pub fn checked_request_key(
    authority: &str,
    stable_key: &str,
) -> Result<crate::StableKey, MemoryDeclarationError> {
    validate_external_authority(authority)?;
    let stable_key =
        crate::StableKey::parse(stable_key).map_err(crate::DeclarationSnapshotError::Key)?;
    validate_external_key(&stable_key)?;
    Ok(stable_key)
}

fn validate_external_key(stable_key: &crate::StableKey) -> Result<(), MemoryDeclarationError> {
    if is_ic_memory_stable_key(stable_key.as_str()) {
        return Err(MemoryDeclarationError::ReservedStableKey {
            stable_key: stable_key.as_str().to_string(),
        });
    }
    Ok(())
}

fn validate_external_authority(value: &str) -> Result<(), MemoryDeclarationError> {
    reject_internal_authority(value)?;
    validate_diagnostic_text(value).map_err(|error| MemoryDeclarationError::InvalidAuthority {
        reason: error.reason(),
    })
}

fn reject_internal_authority(value: &str) -> Result<(), MemoryDeclarationError> {
    if value == IC_MEMORY_AUTHORITY_OWNER {
        return Err(MemoryDeclarationError::ReservedAuthority {
            authority: value.to_string(),
        });
    }
    Ok(())
}

fn internal_ledger_declaration() -> AllocationDeclaration {
    AllocationDeclaration::memory_manager(IC_MEMORY_LEDGER_STABLE_KEY, MEMORY_MANAGER_LEDGER_ID)
        .unwrap_or_else(|_| unreachable!("built-in ledger declaration constants are valid"))
}

#[cfg(test)]
mod tests;
