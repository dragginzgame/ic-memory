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
    preflight(bytes)?;
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

// Allocation-free syntax walk. Reject advertised lengths before serde can use
// them as allocation hints; bound recursion independently of the codec.
fn preflight(bytes: &[u8]) -> Result<(), ciborium::de::Error<std::io::Error>> {
    fn invalid() -> ciborium::de::Error<std::io::Error> {
        ciborium::de::Error::semantic(0, "CBOR ledger size, nesting or structural bound exceeded")
    }
    fn value(
        bytes: &[u8],
        pos: &mut usize,
        depth: usize,
    ) -> Result<(), ciborium::de::Error<std::io::Error>> {
        if depth > crate::constants::MAX_LEDGER_NESTING {
            return Err(invalid());
        }
        let head = *bytes.get(*pos).ok_or_else(invalid)?;
        *pos += 1;
        let major = head >> 5;
        let info = head & 31;
        let n = match info {
            0..=23 => u64::from(info),
            24..=27 => {
                let width = 1_usize << (info - 24);
                let end = pos.checked_add(width).ok_or_else(invalid)?;
                let data = bytes.get(*pos..end).ok_or_else(invalid)?;
                *pos = end;
                data.iter().fold(0_u64, |n, b| (n << 8) | u64::from(*b))
            }
            // Current writers always emit definite lengths.
            _ => return Err(invalid()),
        };
        match major {
            0 | 1 | 7 => (),
            2 | 3 => {
                let n = usize::try_from(n).map_err(|_| invalid())?;
                *pos = pos
                    .checked_add(n)
                    .filter(|end| *end <= bytes.len())
                    .ok_or_else(invalid)?;
            }
            4 | 5 => {
                let count = if major == 5 {
                    n.checked_mul(2).ok_or_else(invalid)?
                } else {
                    n
                };
                if count > (bytes.len() - *pos) as u64 {
                    return Err(invalid());
                }
                for _ in 0..count {
                    value(bytes, pos, depth + 1)?;
                }
            }
            6 => value(bytes, pos, depth + 1)?,
            _ => return Err(invalid()),
        }
        Ok(())
    }
    if bytes.len() > crate::constants::MAX_LEDGER_RECORD_BYTES {
        return Err(invalid());
    }
    let mut pos = 0;
    value(bytes, &mut pos, 0)?;
    if pos != bytes.len() {
        return Err(ciborium::de::Error::semantic(
            pos,
            "trailing bytes after CBOR value",
        ));
    }
    Ok(())
}

pub fn deserialize_records<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    deserialize_bounded_vec::<D, T, 255>(deserializer)
}

pub fn deserialize_history<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    deserialize_bounded_vec::<D, T, { crate::constants::MAX_LEDGER_GENERATIONS }>(deserializer)
}

fn deserialize_bounded_vec<'de, D, T, const LIMIT: usize>(
    deserializer: D,
) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct Bounded<T, const LIMIT: usize>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const LIMIT: usize> serde::de::Visitor<'de> for Bounded<T, LIMIT> {
        type Value = Vec<T>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(formatter, "at most {LIMIT} ledger entries")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut seq: A,
        ) -> Result<Self::Value, A::Error> {
            if seq.size_hint().is_some_and(|n| n > LIMIT) {
                return Err(serde::de::Error::custom("ledger collection limit exceeded"));
            }
            let mut values = Vec::new();
            while let Some(value) = seq.next_element()? {
                if values.len() == LIMIT {
                    return Err(serde::de::Error::custom("ledger collection limit exceeded"));
                }
                values.push(value);
            }
            Ok(values)
        }
    }
    deserializer.deserialize_seq(Bounded::<T, LIMIT>(std::marker::PhantomData))
}
