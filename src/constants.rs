/// Maximum byte length for printable diagnostic metadata fields.
pub const DIAGNOSTIC_STRING_MAX_BYTES: usize = 256;

/// WebAssembly page size used by `ic-stable-structures` memory implementations.
pub const WASM_PAGE_SIZE_BYTES: u64 = 65_536;

/// Maximum encoded logical ledger size (16 MiB).
pub const MAX_LEDGER_BYTES: usize = 16 * 1024 * 1024;
/// Two CBOR byte-vector payloads can each use two bytes per payload byte.
pub const MAX_LEDGER_RECORD_BYTES: usize = 4 * MAX_LEDGER_BYTES + 4096;
/// Maximum retained generations; daily upgrades have over 179 years of headroom.
pub const MAX_LEDGER_GENERATIONS: usize = 65_536;
/// Maximum CBOR container nesting on maintained decode paths.
pub const MAX_LEDGER_NESTING: usize = 32;
