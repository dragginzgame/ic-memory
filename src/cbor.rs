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
                if major == 2 && n > crate::constants::MAX_COMMITTED_PAYLOAD_BYTES {
                    return Err(invalid());
                }
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
    deserialize_bounded_vec::<D, T, { crate::constants::MAX_ALLOCATIONS }>(deserializer)
}

pub fn deserialize_history<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    deserialize_bounded_vec::<D, T, { crate::constants::MAX_LEDGER_GENERATIONS }>(deserializer)
}

pub fn deserialize_bounded_vec<'de, D, T, const LIMIT: usize>(
    deserializer: D,
) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct Bounded<T, const LIMIT: usize>(std::marker::PhantomData<T>);
    struct RejectExcess;
    impl<'de> serde::de::DeserializeSeed<'de> for RejectExcess {
        type Value = ();

        fn deserialize<D: Deserializer<'de>>(self, _: D) -> Result<(), D::Error> {
            Err(serde::de::Error::custom("ledger collection limit exceeded"))
        }
    }
    impl<'de, T: Deserialize<'de>, const LIMIT: usize> serde::de::Visitor<'de> for Bounded<T, LIMIT> {
        type Value = Vec<T>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(formatter, "at most {LIMIT} ledger entries")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut seq: A,
        ) -> Result<Self::Value, A::Error> {
            let hint = seq.size_hint();
            if hint.is_some_and(|n| n > LIMIT) {
                return Err(serde::de::Error::custom("ledger collection limit exceeded"));
            }
            // Definite CBOR arrays already carry their length. Reserve only
            // after admission, with one spare entry for next-generation staging
            // when the collection is non-empty and below its ceiling. Unhinted
            // formats retain incremental growth and reject before excess decode.
            let capacity = hint.map_or(0, |n| {
                if n == 0 {
                    0
                } else {
                    n.saturating_add(1).min(LIMIT)
                }
            });
            let mut values = Vec::with_capacity(capacity);
            while values.len() < LIMIT {
                match seq.next_element()? {
                    Some(value) => values.push(value),
                    None => return Ok(values),
                }
            }
            // Without a size hint, reject any extra entry before decoding it.
            // The seed runs only if the sequence has another element.
            seq.next_element_seed(RejectExcess)?;
            Ok(values)
        }
    }
    deserializer.deserialize_seq(Bounded::<T, LIMIT>(std::marker::PhantomData))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn definite_collections_admit_the_boundary_and_reject_excess_before_decode() {
        #[derive(Debug, Deserialize, Eq, PartialEq)]
        struct Values(#[serde(deserialize_with = "deserialize_bounded_vec::<_, u16, 3>")] Vec<u16>);

        #[derive(Debug)]
        struct MustNotDecode;
        impl<'de> Deserialize<'de> for MustNotDecode {
            fn deserialize<D: Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
                panic!("collection bound must reject before decoding an element");
            }
        }

        for count in 0..=4 {
            let values: Vec<u16> = (0..count).collect();
            let mut bytes = Vec::new();
            ciborium::into_writer(&values, &mut bytes).unwrap();
            let result = from_slice_exact::<Values>(&bytes);
            if count <= 3 {
                assert_eq!(result.unwrap(), Values(values));
            } else {
                assert!(result.is_err());
            }
        }

        let input = serde::de::value::SeqDeserializer::<_, serde::de::value::Error>::new(
            [0_u8; 4].into_iter(),
        );
        assert!(deserialize_bounded_vec::<_, MustNotDecode, 3>(input).is_err());
    }

    #[test]
    fn unhinted_collection_rejects_before_decoding_excess_element() {
        #[derive(Debug)]
        struct Entry;
        impl<'de> Deserialize<'de> for Entry {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let value = u8::deserialize(deserializer)?;
                assert!(value <= 2, "excess entry must not be deserialized");
                Ok(Self)
            }
        }

        let mut input = serde_json::Deserializer::from_str("[1,2]");
        assert_eq!(
            deserialize_bounded_vec::<_, Entry, 2>(&mut input)
                .unwrap()
                .len(),
            2
        );
        let mut input = serde_json::Deserializer::from_str("[1,2,3]");
        let error = deserialize_bounded_vec::<_, Entry, 2>(&mut input).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("ledger collection limit exceeded")
        );
    }

    #[test]
    fn oversized_byte_string_rejects_before_deserializer_runs() {
        #[derive(Debug)]
        struct MustNotDecode;
        impl<'de> Deserialize<'de> for MustNotDecode {
            fn deserialize<D: Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
                panic!("preflight must reject before serde can allocate");
            }
        }
        let len = crate::constants::MAX_COMMITTED_PAYLOAD_BYTES + 1;
        let mut bytes = vec![0x5a];
        bytes.extend_from_slice(&u32::try_from(len).unwrap().to_be_bytes());
        // Both a truncated advertisement and a fully backed oversized value reject.
        assert!(from_slice_exact::<MustNotDecode>(&bytes).is_err());
        bytes.resize(5 + len, 0);
        assert!(from_slice_exact::<MustNotDecode>(&bytes).is_err());
    }
}
