use crate::{
    declaration::{AllocationDeclaration, DeclarationSnapshot},
    schema::SchemaMetadata,
    slot::{
        IC_MEMORY_AUTHORITY_OWNER, IC_MEMORY_LEDGER_LABEL, IC_MEMORY_LEDGER_STABLE_KEY,
        MEMORY_MANAGER_LEDGER_ID, is_ic_memory_stable_key,
    },
    text::validate_diagnostic_text,
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, sync::Arc};

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
    schema: SchemaMetadata,
}

impl<'de> Deserialize<'de> for MemoryRequest {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error as _;
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Request {
            authority: String,
            stable_key: crate::StableKey,
            schema: SchemaMetadata,
        }
        let request = Request::deserialize(deserializer)?;
        Self::new(
            request.authority,
            request.stable_key.as_str(),
            request.schema,
        )
        .map_err(D::Error::custom)
    }
}

impl MemoryRequest {
    /// Build a checked logical request before sealing.
    pub fn new(
        authority: impl Into<String>,
        stable_key: &str,
        schema: SchemaMetadata,
    ) -> Result<Self, MemoryDeclarationError> {
        let authority = authority.into();
        validate_external_authority(&authority)?;
        let stable_key =
            crate::StableKey::parse(stable_key).map_err(crate::DeclarationSnapshotError::Key)?;
        if is_ic_memory_stable_key(stable_key.as_str()) {
            return Err(MemoryDeclarationError::ReservedStableKey {
                stable_key: stable_key.as_str().to_string(),
            });
        }
        Ok(Self {
            authority,
            stable_key,
            schema,
        })
    }

    /// Attach schema metadata from the immutable, integrity-checked recovered ledger.
    pub(crate) const fn with_schema(mut self, schema: SchemaMetadata) -> Self {
        self.schema = schema;
        self
    }

    /// Borrow the requested durable key.
    #[must_use]
    pub const fn stable_key(&self) -> &crate::StableKey {
        &self.stable_key
    }

    /// Borrow the requested diagnostic schema metadata.
    #[must_use]
    pub const fn schema(&self) -> &SchemaMetadata {
        &self.schema
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
    inner: Arc<SealedDeclarationSnapshotInner>,
}

///
/// SealedDeclarationFingerprint
///
/// Deterministic non-cryptographic fingerprint of one canonical sealed
/// declaration snapshot.
///
/// The fingerprint covers canonical source keys, owner labels and schema metadata. It is diagnostic
/// metadata for comparing in-memory bootstrap bindings, not persisted
/// allocation authority or an adversarial integrity proof.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SealedDeclarationFingerprint {
    algorithm_version: u8,
    value: u64,
}

impl SealedDeclarationFingerprint {
    /// Return the diagnostic fingerprint algorithm version.
    #[must_use]
    pub const fn algorithm_version(&self) -> u8 {
        self.algorithm_version
    }

    /// Return the non-cryptographic fingerprint value.
    #[must_use]
    pub const fn value(&self) -> u64 {
        self.value
    }
}

#[derive(Debug, Eq, PartialEq)]
struct SealedDeclarationSnapshotInner {
    requests: Vec<MemoryRequest>,
    fingerprint: SealedDeclarationFingerprint,
}

impl SealedDeclarationSnapshot {
    /// Take ownership of the host's complete requests, checking the bound and
    /// rejecting duplicate keys before publishing their canonical fingerprint.
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
        let fingerprint = sealed_declaration_fingerprint(&requests);
        Ok(Self {
            inner: Arc::new(SealedDeclarationSnapshotInner {
                requests,
                fingerprint,
            }),
        })
    }

    /// Borrow canonical unresolved key-only requests.
    #[must_use]
    pub fn requests(&self) -> &[MemoryRequest] {
        &self.inner.requests
    }

    pub(crate) fn resolve(
        &self,
        ledger: &crate::AllocationLedger,
        historical: Vec<MemoryRequest>,
        pool: &crate::MemoryAllocationPool,
    ) -> Result<DeclarationSnapshot, crate::MemoryResolutionError> {
        if self.requests().len() + historical.len() > 254 {
            return Err(MemoryDeclarationError::TooManyDeclarations.into());
        }
        let mut declarations = Vec::with_capacity(self.requests().len() + historical.len() + 1);
        declarations.push(internal_ledger_declaration());
        let mut occupied = [false; 255];
        for record in ledger.records() {
            occupied[usize::from(record.slot().id())] = true;
        }
        // Retained slots are already occupied. The iterator consumes each new
        // ID once, so no request rescans or updates that recovered occupancy.
        let mut available_ids = 0..crate::MEMORY_MANAGER_INVALID_ID;
        // Only the original requests can allocate new slots and they are already
        // canonical. Admission selections are known-only: all their slots are
        // occupied above regardless of selection order. Final declarations are
        // canonicalized and checked together below.
        for request in self
            .requests()
            .iter()
            .map(Cow::Borrowed)
            .chain(historical.into_iter().map(Cow::Owned))
        {
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
            // Request construction checked authority/key/schema, and recovery
            // checked historical schemas. The final declaration needs only
            // key/schema: do not copy the already-checked authority string.
            // Admission selections transfer those owned fields without copying.
            let declaration = match request {
                Cow::Borrowed(request) => AllocationDeclaration {
                    stable_key: request.stable_key.clone(),
                    slot,
                    label: None,
                    schema: request.schema.clone(),
                },
                Cow::Owned(request) => AllocationDeclaration {
                    stable_key: request.stable_key,
                    slot,
                    label: None,
                    schema: request.schema,
                },
            };
            declarations.push(declaration);
        }
        crate::sort::sort_unstable_by(&mut declarations, |a, b| a.stable_key().cmp(b.stable_key()));
        Ok(DeclarationSnapshot::new(declarations).map_err(MemoryDeclarationError::from)?)
    }

    /// Return the deterministic fingerprint of this sealed declaration meaning.
    #[must_use]
    pub fn fingerprint(&self) -> SealedDeclarationFingerprint {
        self.inner.fingerprint
    }

    #[cfg(test)]
    pub(crate) fn shares_storage_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
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

#[derive(Serialize)]
struct SealedDeclarationFingerprintMaterial<'a> {
    format: &'static str,
    requests: &'a [MemoryRequest],
}

// Fingerprints need the canonical encoded bytes only as input to the hash;
// keep no payload buffer after serialization.
struct FingerprintWriter(u64);

impl std::io::Write for FingerprintWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 = crate::hash::fnv64(self.0, bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn sealed_declaration_fingerprint(requests: &[MemoryRequest]) -> SealedDeclarationFingerprint {
    let material = SealedDeclarationFingerprintMaterial {
        format: "ic-memory.sealed-declaration-fingerprint.v1",
        requests,
    };
    let mut writer = FingerprintWriter(crate::hash::FNV_OFFSET);
    // Concrete derived serializers and this hash writer have no recoverable failures.
    ciborium::into_writer(&material, &mut writer)
        .unwrap_or_else(|_| panic!("sealed declaration fingerprint encodes into hash"));

    SealedDeclarationFingerprint {
        algorithm_version: SEALED_DECLARATION_FINGERPRINT_VERSION,
        value: writer.0,
    }
}

const SEALED_DECLARATION_FINGERPRINT_VERSION: u8 = 1;

fn internal_ledger_declaration() -> AllocationDeclaration {
    AllocationDeclaration::memory_manager(
        IC_MEMORY_LEDGER_STABLE_KEY,
        MEMORY_MANAGER_LEDGER_ID,
        IC_MEMORY_LEDGER_LABEL,
    )
    .unwrap_or_else(|_| unreachable!("built-in ledger declaration constants are valid"))
}

#[cfg(test)]
mod tests;
