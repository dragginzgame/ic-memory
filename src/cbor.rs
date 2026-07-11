use serde::{Deserialize, Deserializer, de::DeserializeOwned};
use std::io::Cursor;

/// Deserialize an explicitly present optional field.
///
/// Serde normally treats an omitted `Option<T>` field as `None`. Durable
/// current-format records use this helper to distinguish an explicit CBOR
/// `null` from a missing field.
pub fn deserialize_present_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

/// Deserialize exactly one CBOR value and reject any trailing bytes.
pub fn from_slice_exact<T: DeserializeOwned>(
    bytes: &[u8],
) -> Result<T, ciborium::de::Error<std::io::Error>> {
    let mut reader = Cursor::new(bytes);
    let value = ciborium::from_reader(&mut reader)?;
    let consumed = usize::try_from(reader.position()).unwrap_or(usize::MAX);
    if consumed != bytes.len() {
        return Err(ciborium::de::Error::semantic(
            consumed,
            "trailing bytes after CBOR value",
        ));
    }
    Ok(value)
}
