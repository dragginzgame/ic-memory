use crate::{MemoryManagerIdRange, StableKey, text::validate_diagnostic_text};
use serde::{Deserialize, Deserializer, Serialize, de::Error as _};

///
/// MemoryAuthority
///
/// Host-owned permission for one named owner to request keys in a namespace.
/// The prefix ends in a dot, so `app.` never grants `application.`. This policy
/// metadata grants neither caller authentication nor a sandbox for linked code.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MemoryAuthority {
    authority: String,
    key_prefix: String,
}

impl MemoryAuthority {
    /// Validate a host grant. Multiple disjoint prefixes may share an owner.
    pub fn new(
        authority: impl Into<String>,
        key_prefix: impl Into<String>,
    ) -> Result<Self, MemoryAllocationPoolError> {
        let authority = authority.into();
        validate_diagnostic_text(&authority).map_err(|error| {
            MemoryAllocationPoolError::InvalidAuthority {
                reason: error.reason(),
            }
        })?;
        if authority == crate::IC_MEMORY_AUTHORITY_OWNER {
            return Err(MemoryAllocationPoolError::ReservedAuthority);
        }
        let key_prefix = key_prefix.into();
        if !key_prefix.ends_with('.') || StableKey::parse(format!("{key_prefix}v1")).is_err() {
            return Err(MemoryAllocationPoolError::InvalidKeyPrefix { key_prefix });
        }
        if key_prefix.starts_with(crate::IC_MEMORY_STABLE_KEY_PREFIX) {
            return Err(MemoryAllocationPoolError::InvalidKeyPrefix { key_prefix });
        }
        Ok(Self {
            authority,
            key_prefix,
        })
    }

    /// The admitted linked-code owner label.
    #[must_use]
    pub fn authority(&self) -> &str {
        &self.authority
    }

    /// Permanent key namespace admitted by the host.
    #[must_use]
    pub fn key_prefix(&self) -> &str {
        &self.key_prefix
    }
}

impl<'de> Deserialize<'de> for MemoryAuthority {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Grant {
            authority: String,
            key_prefix: String,
        }
        let grant = Grant::deserialize(deserializer)?;
        Self::new(grant.authority, grant.key_prefix).map_err(D::Error::custom)
    }
}

///
/// MemoryAllocationPool
///
/// One host-wide pool over usable application IDs 10..=254. Governance IDs
/// 0..=9 remain excluded. Explicit additional exclusions retain physical
/// custody for unmanaged users; namespace grants never partition this pool.
///
/// The pool is immutable bootstrap policy, not durable allocation state.
/// Current, omitted, reserved and retired ledger records independently retain
/// their IDs. It is not inferred from application bytes or slot contents.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MemoryAllocationPool {
    authorities: Vec<MemoryAuthority>,
    excluded_ranges: Vec<MemoryManagerIdRange>,
}

impl MemoryAllocationPool {
    /// Canonicalize host grants and physical exclusions before bootstrap.
    ///
    /// # Panics
    ///
    /// Panics only if an internal bounded-ID canonicalization invariant is broken.
    pub fn new(
        mut authorities: Vec<MemoryAuthority>,
        exclusions: Vec<MemoryManagerIdRange>,
    ) -> Result<Self, MemoryAllocationPoolError> {
        if authorities.len() > 254 || exclusions.len() > 255 {
            return Err(MemoryAllocationPoolError::TooManyEntries);
        }
        authorities.sort_unstable_by(|a, b| a.key_prefix.cmp(&b.key_prefix));
        for pair in authorities.windows(2) {
            if pair[1].key_prefix.starts_with(&pair[0].key_prefix) {
                return Err(MemoryAllocationPoolError::OverlappingNamespaces {
                    existing_prefix: pair[0].key_prefix.clone(),
                    candidate_prefix: pair[1].key_prefix.clone(),
                });
            }
        }
        let mut excluded = [false; 255];
        for range in exclusions
            .into_iter()
            .chain(std::iter::once(crate::memory_manager_governance_range()))
        {
            for id in range.start()..=range.end() {
                excluded[usize::from(id)] = true;
            }
        }
        let mut excluded_ranges = Vec::new();
        let mut id = 0;
        while id < 255 {
            if !excluded[id] {
                id += 1;
                continue;
            }
            let start = id;
            while id + 1 < 255 && excluded[id + 1] {
                id += 1;
            }
            excluded_ranges.push(
                MemoryManagerIdRange::new(
                    u8::try_from(start).expect("usable start"),
                    u8::try_from(id).expect("usable end"),
                )
                .expect("ordered usable range"),
            );
            id += 1;
        }
        Ok(Self {
            authorities,
            excluded_ranges,
        })
    }

    /// Current owner grants, in canonical namespace order.
    #[must_use]
    pub fn authorities(&self) -> &[MemoryAuthority] {
        &self.authorities
    }

    /// Physical exclusions including the permanent governance reservation.
    #[must_use]
    pub fn excluded_ranges(&self) -> &[MemoryManagerIdRange] {
        &self.excluded_ranges
    }

    /// Whether an ID belongs to the shared application pool.
    #[must_use]
    pub fn contains(&self, id: u8) -> bool {
        id != crate::MEMORY_MANAGER_INVALID_ID
            && !self.excluded_ranges.iter().any(|range| range.contains(id))
    }

    /// Validate current key ownership independently of numeric placement.
    pub fn validate_authority(
        &self,
        key: &StableKey,
        authority: &str,
    ) -> Result<(), MemoryAllocationPoolError> {
        let admitted_authority =
            self.authority_for_key(key)
                .ok_or_else(|| MemoryAllocationPoolError::UnclaimedKey {
                    stable_key: key.clone(),
                })?;
        if admitted_authority != authority {
            return Err(MemoryAllocationPoolError::AuthorityMismatch {
                stable_key: key.clone(),
                requested_authority: authority.to_string(),
                admitted_authority: admitted_authority.to_string(),
            });
        }
        Ok(())
    }

    // Committed keys were admitted by resolution. Diagnostic and adoption
    // projections borrow the same host owner instead of retaining another map.
    pub(crate) fn authority_for_key(&self, key: &StableKey) -> Option<&str> {
        self.authorities
            .iter()
            .find(|grant| key.as_str().starts_with(&grant.key_prefix))
            .map(MemoryAuthority::authority)
    }

    /// Validate physical eligibility without granting ownership of a key.
    pub fn validate_id(&self, id: u8) -> Result<(), MemoryAllocationPoolError> {
        crate::validate_memory_manager_id(id)?;
        if !self.contains(id) {
            return Err(MemoryAllocationPoolError::ExcludedSlot { id });
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for MemoryAllocationPool {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Pool {
            authorities: Vec<MemoryAuthority>,
            excluded_ranges: Vec<MemoryManagerIdRange>,
        }
        let pool = Pool::deserialize(deserializer)?;
        Self::new(pool.authorities, pool.excluded_ranges).map_err(D::Error::custom)
    }
}

///
/// MemoryAllocationPoolError
///
/// Invalid host configuration or a request outside admitted ownership/custody.
///

#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum MemoryAllocationPoolError {
    #[error("host allocation policy exceeds the bounded slot domain")]
    TooManyEntries,
    #[error("authority {reason}")]
    InvalidAuthority { reason: &'static str },
    #[error("ic-memory governance authority cannot be granted externally")]
    ReservedAuthority,
    #[error("invalid application key namespace prefix {key_prefix}")]
    InvalidKeyPrefix { key_prefix: String },
    #[error("key namespaces {existing_prefix} and {candidate_prefix} overlap")]
    OverlappingNamespaces {
        existing_prefix: String,
        candidate_prefix: String,
    },
    #[error("stable key {stable_key} has no host namespace grant")]
    UnclaimedKey { stable_key: StableKey },
    #[error(
        "stable key {stable_key} is granted to {admitted_authority}, not {requested_authority}"
    )]
    AuthorityMismatch {
        stable_key: StableKey,
        requested_authority: String,
        admitted_authority: String,
    },
    #[error("MemoryManager ID {id} is excluded from the application pool")]
    ExcludedSlot { id: u8 },
    #[error(transparent)]
    Slot(#[from] crate::MemoryManagerSlotError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_policy_has_one_physical_pool_and_disjoint_namespaces() {
        let grant = |owner, prefix| MemoryAuthority::new(owner, prefix).unwrap();
        let pool = MemoryAllocationPool::new(
            vec![grant("db", "db."), grant("app", "app.")],
            vec![
                MemoryManagerIdRange::new(20, 25).unwrap(),
                MemoryManagerIdRange::new(23, 30).unwrap(),
                MemoryManagerIdRange::new(31, 31).unwrap(),
            ],
        )
        .unwrap();
        assert_eq!(pool.authorities()[0].authority(), "app");
        assert_eq!(
            pool.excluded_ranges(),
            &[
                MemoryManagerIdRange::new(0, 9).unwrap(),
                MemoryManagerIdRange::new(20, 31).unwrap()
            ]
        );
        for id in [0, 9, 20, 31, 255] {
            assert!(!pool.contains(id));
        }
        for id in [10, 19, 32, 254] {
            assert!(pool.contains(id));
        }
        assert!(matches!(
            MemoryAllocationPool::new(vec![grant("app", "app."), grant("db", "app.db.")], vec![]),
            Err(MemoryAllocationPoolError::OverlappingNamespaces { .. })
        ));
        assert!(
            pool.validate_authority(&StableKey::parse("application.rows.v1").unwrap(), "app")
                .is_err()
        );
        assert!(
            pool.validate_authority(&StableKey::parse("app.rows.v1").unwrap(), "db")
                .is_err()
        );
    }

    #[test]
    fn decoding_cannot_bypass_namespace_or_physical_exclusion_admission() {
        let pool: MemoryAllocationPool = serde_json::from_str(
            r#"{
            "authorities":[{"authority":"app","key_prefix":"app."}],
            "excluded_ranges":[{"start":10,"end":10}]
        }"#,
        )
        .unwrap();
        assert!(!pool.contains(0));
        assert!(!pool.contains(10));
        assert!(pool.contains(11));
        for json in [
            r#"{"authorities":[{"authority":"ic-memory","key_prefix":"app."}],"excluded_ranges":[]}"#,
            r#"{"authorities":[{"authority":"app","key_prefix":"ic_memory."}],"excluded_ranges":[]}"#,
            r#"{"authorities":[],"excluded_ranges":[{"start":254,"end":255}]}"#,
            r#"{"authorities":[],"excluded_ranges":[],"unknown":true}"#,
        ] {
            assert!(serde_json::from_str::<MemoryAllocationPool>(json).is_err());
        }
        assert_eq!(
            serde_json::from_str::<MemoryAllocationPool>(&serde_json::to_string(&pool).unwrap())
                .unwrap(),
            pool
        );
    }
}
