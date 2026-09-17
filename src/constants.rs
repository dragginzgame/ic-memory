/// Maximum byte length for printable diagnostic metadata fields.
pub const DIAGNOSTIC_STRING_MAX_BYTES: usize = 256;

/// WebAssembly page size used by `ic-stable-structures` memory implementations.
pub const WASM_PAGE_SIZE_BYTES: u64 = 65_536;

/// Maximum encoded logical ledger size (16 MiB).
pub const MAX_LEDGER_BYTES: usize = 16 * 1024 * 1024;
/// Family magic, format marker, version and encoded logical length.
pub const LEDGER_PAYLOAD_HEADER_LEN: usize = 8 + 4 + 4 + 8;
/// Maximum opaque generation payload, including its logical envelope header.
pub const MAX_COMMITTED_PAYLOAD_BYTES: usize = MAX_LEDGER_BYTES + LEDGER_PAYLOAD_HEADER_LEN;
/// Two bounded CBOR byte strings plus record metadata (32 MiB + 4 KiB).
pub const MAX_LEDGER_RECORD_BYTES: usize = 2 * MAX_LEDGER_BYTES + 4096;
/// Maximum retained generations; daily upgrades have over 179 years of headroom.
pub const MAX_LEDGER_GENERATIONS: usize = 65_536;
/// Maximum CBOR container nesting on maintained decode paths.
pub const MAX_LEDGER_NESTING: usize = 32;
