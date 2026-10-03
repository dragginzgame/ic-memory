//! Shared non-cryptographic FNV-1a hashing for checksums and diagnostics.

pub const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;

pub fn fnv64(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash = (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_vectors_and_chunked_inputs_produce_identical_hashes() {
        assert_eq!(fnv64(FNV_OFFSET, b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv64(FNV_OFFSET, b"hello"), 0xa430_d846_80aa_bd0b);
        let mut streamed = FNV_OFFSET;
        for chunk in [b"he".as_slice(), b"", b"ll", b"o"] {
            streamed = fnv64(streamed, chunk);
        }
        assert_eq!(streamed, 0xa430_d846_80aa_bd0b);
    }
}
