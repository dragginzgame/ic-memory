use serde::{Deserialize, Deserializer, Serialize, de::Error as _};
use std::{fmt, str::FromStr};

const MAX_KEY_BYTES: usize = 128;

///
/// StableKey
///
/// Canonical durable logical allocation identity.
///
/// A stable key names the logical store, not the current storage backend or
/// `MemoryManager` ID. Once committed, the key is permanently bound to its
/// physical allocation slot; changing the key declares a new logical store.
///
/// Construction and deserialization enforce the same bounded canonical grammar.
/// A checked key establishes identity syntax, not allocation or open authority.
///

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct StableKey(String);

impl<'de> Deserialize<'de> for StableKey {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Retain the serialized newtype shape before checking its owned text.
        #[derive(Deserialize)]
        #[serde(rename = "StableKey")]
        struct Text(String);

        let Text(value) = Text::deserialize(deserializer)?;
        validate(&value).map_err(D::Error::custom)?;
        Ok(Self(value))
    }
}

impl StableKey {
    /// Parse and validate a canonical stable key string.
    ///
    /// Keys are bounded lowercase ASCII dot-separated names ending in a
    /// nonzero `.vN` suffix.
    pub fn parse(value: impl AsRef<str>) -> Result<Self, StableKeyError> {
        let value = value.as_ref();
        validate(value)?;
        Ok(Self(value.to_string()))
    }

    /// Borrow the canonical stable-key string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume the key and return the canonical stable-key string.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl AsRef<str> for StableKey {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for StableKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for StableKey {
    type Err = StableKeyError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

///
/// StableKeyError
///
/// Stable-key grammar validation failure.
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
#[error("stable key '{stable_key}' is invalid: {reason}")]
pub struct StableKeyError {
    /// Rejected stable-key string.
    pub stable_key: String,
    /// Stable-key grammar failure.
    pub reason: &'static str,
}

pub fn validate(stable_key: &str) -> Result<(), StableKeyError> {
    if stable_key.is_empty() {
        return invalid(stable_key, "must not be empty");
    }
    if stable_key.len() > MAX_KEY_BYTES {
        return invalid(stable_key, "must be at most 128 bytes");
    }
    if !stable_key.is_ascii() {
        return invalid(stable_key, "must be ASCII");
    }
    if stable_key.bytes().any(|byte| byte.is_ascii_uppercase()) {
        return invalid(stable_key, "must be lowercase");
    }
    if stable_key
        .bytes()
        .any(|byte| matches!(byte, b'\t'..=b'\r' | b' '))
    {
        return invalid(stable_key, "must not contain whitespace");
    }
    if stable_key.bytes().any(|byte| matches!(byte, b'/' | b'-')) {
        return invalid(stable_key, "must not contain slashes or hyphens");
    }
    if stable_key.starts_with('.') || stable_key.ends_with('.') {
        return invalid(stable_key, "must not start or end with a dot");
    }

    let Some(version_index) = stable_key
        .as_bytes()
        .windows(2)
        .rposition(|pair| pair == b".v")
    else {
        return invalid(stable_key, "must end with .vN");
    };
    let version = &stable_key[version_index + 2..];
    if version.is_empty()
        || version.starts_with('0')
        || !version.bytes().all(|byte| byte.is_ascii_digit())
    {
        return invalid(stable_key, "version suffix must be nonzero .vN");
    }

    let prefix = &stable_key[..version_index];
    if prefix.is_empty() {
        return invalid(
            stable_key,
            "must contain at least one segment before version",
        );
    }

    for segment in prefix.as_bytes().split(|byte| *byte == b'.') {
        if let Err(reason) = validate_segment(segment) {
            return invalid(stable_key, reason);
        }
    }

    Ok(())
}

// A namespace plus its synthetic `v1` suffix must fit the same key grammar.
// Borrow the prefix rather than allocating a temporary key just to discard it.
pub fn is_key_prefix(value: &str) -> bool {
    let Some(prefix) = value.strip_suffix('.') else {
        return false;
    };
    value.len() <= MAX_KEY_BYTES - "v1".len()
        && prefix
            .as_bytes()
            .split(|byte| *byte == b'.')
            .all(|segment| validate_segment(segment).is_ok())
}

fn validate_segment(segment: &[u8]) -> Result<(), &'static str> {
    let mut bytes = segment.iter().copied();
    let Some(first) = bytes.next() else {
        return Err("must not contain empty segments");
    };
    if !first.is_ascii_lowercase() {
        return Err("segments must start with a lowercase letter");
    }
    if !bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_') {
        return Err("segments may contain only lowercase letters, digits, and underscores");
    }
    Ok(())
}

fn invalid<T>(stable_key: &str, reason: &'static str) -> Result<T, StableKeyError> {
    Err(StableKeyError {
        stable_key: stable_key.to_string(),
        reason,
    })
}

/// Mark later occurrences while preserving each boundary's original traversal.
/// Callers bound the collection and supply its enumerated input positions.
pub fn duplicate_key_positions(
    mut keys: Vec<(&StableKey, usize)>,
) -> [bool; crate::constants::MAX_ALLOCATIONS] {
    crate::sort::sort_unstable_by(&mut keys, Ord::cmp);
    let mut duplicates = [false; crate::constants::MAX_ALLOCATIONS];
    for pair in keys.windows(2) {
        if pair[0].0 == pair[1].0 {
            duplicates[pair[1].1] = true;
        }
    }
    duplicates
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_stores_the_string_it_validates() {
        struct ChangingKey(std::cell::Cell<bool>);

        impl AsRef<str> for ChangingKey {
            fn as_ref(&self) -> &str {
                if self.0.replace(true) {
                    "INVALID"
                } else {
                    "app.rows.v1"
                }
            }
        }

        let key = StableKey::parse(ChangingKey(std::cell::Cell::new(false))).unwrap();
        assert_eq!(key.as_str(), "app.rows.v1");
    }

    #[test]
    fn uniqueness_errors_follow_input_order_and_boundary_precedence() {
        use crate::AllocationDeclaration;
        let duplicate = StableKey::parse("app.b.v1").unwrap();
        for slots in [[10, 11, 12, 13], [10, 11, 10, 13]] {
            let declarations: Vec<_> = ["app.a.v1", "app.b.v1", "app.b.v1", "app.a.v1"]
                .into_iter()
                .zip(slots)
                .map(|(key, slot)| AllocationDeclaration::memory_manager(key, slot).unwrap())
                .collect();
            let ledger = crate::AllocationLedger {
                current_generation: 1,
                records: declarations
                    .iter()
                    .map(crate::AllocationRecord::from_declaration)
                    .collect(),
            };
            assert_eq!(
                ledger.validate_integrity(),
                Err(crate::LedgerIntegrityError::DuplicateStableKey {
                    stable_key: duplicate.clone(),
                })
            );
        }
    }

    #[test]
    fn accepts_canonical_keys() {
        assert_eq!(
            StableKey::parse("app.users.primary.v1")
                .expect("valid key")
                .as_str(),
            "app.users.primary.v1"
        );
        assert!(StableKey::parse("framework.core.auth_state.v12").is_ok());
        assert!(StableKey::parse("app.vault.v1").is_ok());
        assert!(StableKey::parse("app.v1.v2").is_ok());
    }

    #[test]
    fn key_decode_enforces_grammar_and_preserves_current_encoding() {
        let key = StableKey::parse("app.users.v1").unwrap();
        let json = serde_json::to_string(&key).unwrap();
        assert_eq!(json, "\"app.users.v1\"");
        assert_eq!(serde_json::from_str::<StableKey>(&json).unwrap(), key);
        let bytes = crate::test_cbor::to_vec(&key).unwrap();
        assert_eq!(bytes, b"\x6capp.users.v1");
        assert_eq!(
            crate::test_cbor::from_slice::<StableKey>(&bytes).unwrap(),
            key
        );

        for invalid in [
            "App.users.v1",
            "app.users.v0",
            "app.vault",
            "app.users.v1.tail",
            "app..users.v1",
            &"x".repeat(129),
        ] {
            let json = serde_json::to_string(invalid).unwrap();
            assert!(serde_json::from_str::<StableKey>(&json).is_err());
            let bytes = crate::test_cbor::to_vec(&invalid).unwrap();
            assert!(crate::test_cbor::from_slice::<StableKey>(&bytes).is_err());
        }
    }

    #[test]
    fn whitespace_rejection_preserves_input_and_precedence() {
        for whitespace in [' ', '\t', '\n', '\u{b}', '\u{c}', '\r'] {
            let key = format!("app.us{whitespace}ers.v1");
            assert_eq!(
                StableKey::parse(&key),
                Err(StableKeyError {
                    stable_key: key,
                    reason: "must not contain whitespace",
                })
            );
        }
        for (key, reason) in [
            ("app.us\u{a0}ers.v1", "must be ASCII"),
            ("App.us\u{b}ers.v1", "must be lowercase"),
        ] {
            assert_eq!(StableKey::parse(key).unwrap_err().reason, reason);
        }
    }

    #[test]
    fn rejects_noncanonical_keys() {
        for key in [
            "",
            "App.users.v1",
            "app.users",
            "app.users.v0",
            "app..users.v1",
            ".app.users.v1",
            "app.users.v1.",
            "app-users.v1",
            "app/users.v1",
            "app.1users.v1",
        ] {
            assert!(StableKey::parse(key).is_err(), "{key} should fail");
        }
    }
}
