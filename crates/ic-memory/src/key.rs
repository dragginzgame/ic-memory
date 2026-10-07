use serde::{Deserialize, Deserializer, Serialize, de::Error as _};
use std::{fmt, str::FromStr};

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
    if stable_key.len() > 128 {
        return invalid(stable_key, "must be at most 128 bytes");
    }
    if !stable_key.is_ascii() {
        return invalid(stable_key, "must be ASCII");
    }
    if stable_key.bytes().any(|byte| byte.is_ascii_uppercase()) {
        return invalid(stable_key, "must be lowercase");
    }
    if stable_key.contains(char::is_whitespace) {
        return invalid(stable_key, "must not contain whitespace");
    }
    if stable_key.contains('/') || stable_key.contains('-') {
        return invalid(stable_key, "must not contain slashes or hyphens");
    }
    if stable_key.starts_with('.') || stable_key.ends_with('.') {
        return invalid(stable_key, "must not start or end with a dot");
    }

    let Some(version_index) = stable_key.rfind(".v") else {
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

    for segment in prefix.split('.') {
        validate_segment(stable_key, segment)?;
    }

    Ok(())
}

fn validate_segment(stable_key: &str, segment: &str) -> Result<(), StableKeyError> {
    let mut bytes = segment.bytes();
    let Some(first) = bytes.next() else {
        return invalid(stable_key, "must not contain empty segments");
    };
    if !first.is_ascii_lowercase() {
        return invalid(stable_key, "segments must start with a lowercase letter");
    }
    if !bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_') {
        return invalid(
            stable_key,
            "segments may contain only lowercase letters, digits, and underscores",
        );
    }
    Ok(())
}

fn invalid<T>(stable_key: &str, reason: &'static str) -> Result<T, StableKeyError> {
    Err(StableKeyError {
        stable_key: stable_key.to_string(),
        reason,
    })
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
    fn accepts_canonical_keys() {
        assert_eq!(
            StableKey::parse("app.users.primary.v1")
                .expect("valid key")
                .as_str(),
            "app.users.primary.v1"
        );
        assert!(StableKey::parse("framework.core.auth_state.v12").is_ok());
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
