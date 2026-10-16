use crate::{StableKey, text::validate_diagnostic_text};
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
        if !crate::key::is_key_prefix(&key_prefix) {
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
    #[serde(rename = "excluded_ids", serialize_with = "serialize_excluded_ids")]
    excluded: [u64; 4],
}

impl MemoryAllocationPool {
    /// Canonicalize host grants and physical exclusions before bootstrap.

    pub fn new(
        mut authorities: Vec<MemoryAuthority>,
        exclusions: Vec<u8>,
    ) -> Result<Self, MemoryAllocationPoolError> {
        if authorities.len() > 254 || exclusions.len() > 255 {
            return Err(MemoryAllocationPoolError::TooManyEntries);
        }
        crate::sort::sort_unstable_by(&mut authorities, |a, b| a.key_prefix.cmp(&b.key_prefix));
        for pair in authorities.windows(2) {
            if pair[1].key_prefix.starts_with(&pair[0].key_prefix) {
                return Err(MemoryAllocationPoolError::OverlappingNamespaces {
                    existing_prefix: pair[0].key_prefix.clone(),
                    candidate_prefix: pair[1].key_prefix.clone(),
                });
            }
        }
        let mut excluded = [0; 4];
        for id in exclusions
            .into_iter()
            .chain(0..=crate::MEMORY_MANAGER_GOVERNANCE_MAX_ID)
        {
            crate::validate_memory_manager_id(id)?;
            excluded[usize::from(id / 64)] |= 1 << (id % 64);
        }
        Ok(Self {
            authorities,
            excluded,
        })
    }

    /// Current owner grants, in canonical namespace order.
    #[must_use]
    pub fn authorities(&self) -> &[MemoryAuthority] {
        &self.authorities
    }

    /// Excluded usable IDs, including governance, in ascending order.
    pub fn excluded_ids(&self) -> impl Iterator<Item = u8> + '_ {
        (0..crate::MEMORY_MANAGER_INVALID_ID).filter(|id| is_excluded(&self.excluded, *id))
    }

    /// Whether an ID belongs to the shared application pool.
    #[must_use]
    pub fn contains(&self, id: u8) -> bool {
        id != crate::MEMORY_MANAGER_INVALID_ID && !is_excluded(&self.excluded, id)
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
            #[serde(deserialize_with = "crate::cbor::deserialize_records")]
            excluded_ids: Vec<u8>,
        }
        let pool = Pool::deserialize(deserializer)?;
        Self::new(pool.authorities, pool.excluded_ids).map_err(D::Error::custom)
    }
}

fn is_excluded(excluded: &[u64; 4], id: u8) -> bool {
    excluded[usize::from(id / 64)] & (1 << (id % 64)) != 0
}

fn serialize_excluded_ids<S: serde::Serializer>(
    excluded: &[u64; 4],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    use serde::ser::SerializeSeq as _;
    let count = excluded.iter().map(|word| word.count_ones() as usize).sum();
    let mut seq = serializer.serialize_seq(Some(count))?;
    for id in (0..crate::MEMORY_MANAGER_INVALID_ID).filter(|id| is_excluded(excluded, *id)) {
        seq.serialize_element(&id)?;
    }
    seq.end()
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
    #[test]
    fn namespace_admission_checks_segments_and_complete_key_length() {
        for prefix in ["a.", "app.a_0.", "app.v0.", "app.v1."] {
            let grant = super::MemoryAuthority::new("app", prefix).unwrap();
            assert_eq!(grant.key_prefix(), prefix);
        }
        for prefix in ["", ".", "app", "app..", "app.0.", "App.", "app/a."] {
            assert!(matches!(
                super::MemoryAuthority::new("app", prefix),
                Err(super::MemoryAllocationPoolError::InvalidKeyPrefix { key_prefix })
                    if key_prefix == prefix
            ));
        }
        for byte in 0..=u8::MAX {
            let prefix = format!("app.a{}.", char::from(byte));
            let allowed = byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_';
            assert_eq!(super::MemoryAuthority::new("app", prefix).is_ok(), allowed);
        }
        assert!(super::MemoryAuthority::new("app", format!("{}.", "a".repeat(125))).is_ok());
        assert!(matches!(
            super::MemoryAuthority::new("app", format!("{}.", "a".repeat(126))),
            Err(super::MemoryAllocationPoolError::InvalidKeyPrefix { .. })
        ));
    }

    use super::*;

    #[test]
    fn canonical_policy_has_one_physical_pool_and_disjoint_namespaces() {
        let grant = |owner, prefix| MemoryAuthority::new(owner, prefix).unwrap();
        let pool = MemoryAllocationPool::new(
            vec![grant("db", "db."), grant("app", "app.")],
            (20..=25).chain(23..=30).chain(31..=31).collect(),
        )
        .unwrap();
        assert_eq!(pool.authorities()[0].authority(), "app");
        assert_eq!(
            pool.excluded_ids().collect::<Vec<_>>(),
            (0..=9).chain(20..=31).collect::<Vec<_>>()
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
            "excluded_ids":[10]
        }"#,
        )
        .unwrap();
        assert!(!pool.contains(0));
        assert!(!pool.contains(10));
        assert!(pool.contains(11));
        for json in [
            r#"{"authorities":[{"authority":"ic-memory","key_prefix":"app."}],"excluded_ids":[]}"#,
            r#"{"authorities":[{"authority":"app","key_prefix":"ic_memory."}],"excluded_ids":[]}"#,
            r#"{"authorities":[],"excluded_ids":[255]}"#,
            r#"{"authorities":[],"excluded_ids":[],"unknown":true}"#,
        ] {
            assert!(serde_json::from_str::<MemoryAllocationPool>(json).is_err());
        }
        assert_eq!(
            serde_json::from_str::<MemoryAllocationPool>(&serde_json::to_string(&pool).unwrap())
                .unwrap(),
            pool
        );
    }
    #[test]
    fn exclusion_membership_and_wire_projection_cover_every_usable_id() {
        for ids in [
            vec![0, 9, 10, 63, 64, 127, 128, 191, 192, 254],
            (10..=254).step_by(2).collect(),
            (0..=254).collect(),
        ] {
            let pool = MemoryAllocationPool::new(vec![], ids.clone()).unwrap();
            let reversed =
                MemoryAllocationPool::new(vec![], ids.iter().rev().copied().collect()).unwrap();
            assert_eq!(pool, reversed);
            for id in 0..=u8::MAX {
                assert_eq!(
                    pool.contains(id),
                    id > crate::MEMORY_MANAGER_GOVERNANCE_MAX_ID
                        && id != u8::MAX
                        && !ids.contains(&id)
                );
            }
            let projected: Vec<_> = (0..255).filter(|id| !pool.contains(*id)).collect();
            assert_eq!(pool.excluded_ids().collect::<Vec<_>>(), projected);
            let json = serde_json::to_value(&pool).unwrap();
            assert_eq!(json["excluded_ids"], serde_json::json!(projected));
            assert_eq!(
                serde_json::from_value::<MemoryAllocationPool>(json).unwrap(),
                pool
            );
            let bytes = crate::test_cbor::to_vec(&pool).unwrap();
            assert_eq!(
                crate::cbor::from_slice_exact::<MemoryAllocationPool>(&bytes).unwrap(),
                pool
            );
        }
        assert!(matches!(
            MemoryAllocationPool::new(vec![], vec![255]),
            Err(MemoryAllocationPoolError::Slot(_))
        ));
        assert!(matches!(
            MemoryAllocationPool::new(vec![], vec![10; 256]),
            Err(MemoryAllocationPoolError::TooManyEntries)
        ));
        assert_eq!(
            MemoryAllocationPool::new(vec![], vec![63, 64, 63]).unwrap(),
            MemoryAllocationPool::new(vec![], vec![64, 63]).unwrap()
        );
    }
}
