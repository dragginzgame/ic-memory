use serde::de::DeserializeOwned;
use std::io::Cursor;

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
