// Logical ownership is direct CBOR inside the protected physical byte string.
// The physical snapshot is the sole format/framing owner.
pub(super) fn encode_ledger(
    ledger: &super::AllocationLedger,
) -> Result<Vec<u8>, super::LedgerIntegrityError> {
    let mut writer =
        crate::bounded_writer::BoundedWriter::new(Vec::new(), crate::constants::MAX_LEDGER_BYTES);
    match ciborium::into_writer(ledger.records(), &mut writer) {
        Ok(()) => Ok(writer.bytes),
        Err(ciborium::ser::Error::Io(err)) if err.kind() == std::io::ErrorKind::WriteZero => {
            Err(super::LedgerIntegrityError::LimitExceeded {
                resource: "ledger bytes",
                limit: crate::constants::MAX_LEDGER_BYTES,
            })
        }
        Err(ciborium::ser::Error::Io(_) | ciborium::ser::Error::Value(_)) => {
            panic!("allocation ledger serialization failed")
        }
    }
}

pub(super) fn decode_ledger(
    bytes: &[u8],
    generation: u64,
) -> Result<super::AllocationLedger, String> {
    // The protected header owns generation; the payload is only a bounded array.
    #[derive(serde::Deserialize)]
    #[serde(transparent)]
    struct Records(
        #[serde(deserialize_with = "crate::cbor::deserialize_records")]
        Vec<super::AllocationRecord>,
    );

    // Ciborium can present byte strings as sequences; only arrays are this wire shape.
    if bytes.first().is_none_or(|byte| byte >> 5 != 4) {
        return Err("ledger payload must be a CBOR record array".to_string());
    }
    let Records(records) = crate::cbor::from_slice_exact(bytes).map_err(|err| err.to_string())?;
    Ok(super::AllocationLedger {
        current_generation: generation,
        records,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ledger_writer_admits_exact_ceiling_and_refuses_without_partial_append() {
        use std::io::Write;

        let mut writer = crate::bounded_writer::BoundedWriter::new(
            Vec::new(),
            crate::constants::MAX_COMMITTED_PAYLOAD_BYTES,
        );
        let chunk = [42; 4096];
        for _ in 0..crate::constants::MAX_LEDGER_BYTES / chunk.len() {
            writer.write_all(&chunk).unwrap();
        }
        assert_eq!(
            writer.bytes.len(),
            crate::constants::MAX_COMMITTED_PAYLOAD_BYTES
        );
        writer.write_all(&[]).unwrap();
        assert_eq!(
            writer.write(&[42]).unwrap_err().kind(),
            std::io::ErrorKind::WriteZero
        );
        assert_eq!(
            writer.bytes.len(),
            crate::constants::MAX_COMMITTED_PAYLOAD_BYTES
        );
        assert!(writer.bytes.iter().all(|&byte| byte == 42));

        let mut writer = crate::bounded_writer::BoundedWriter::new(
            Vec::new(),
            crate::constants::MAX_COMMITTED_PAYLOAD_BYTES,
        );
        let before = writer.bytes.clone();
        assert!(
            writer
                .write(&vec![42; crate::constants::MAX_LEDGER_BYTES + 1])
                .is_err()
        );
        assert_eq!(writer.bytes, before);
    }

    #[test]
    fn bounded_ledger_encoding_matches_canonical_cbor() {
        for generations in [0, 1, 24, 256, 1024] {
            let ledger = super::super::AllocationLedger {
                current_generation: generations,
                records: Vec::new(),
            };
            let mut payload = Vec::new();
            ciborium::into_writer(ledger.records(), &mut payload).unwrap();
            let expected = payload;
            assert_eq!(encode_ledger(&ledger).unwrap(), expected);
        }
    }
}
