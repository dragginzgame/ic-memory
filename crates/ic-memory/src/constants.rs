/// Maximum byte length for printable diagnostic metadata fields.
pub const DIAGNOSTIC_STRING_MAX_BYTES: usize = 256;

/// WebAssembly page size used by `ic-stable-structures` memory implementations.
pub const WASM_PAGE_SIZE_BYTES: u64 = 65_536;

/// Maximum allocation identities in the usable ID domain, including governance.
pub const MAX_ALLOCATIONS: usize = crate::slot::MEMORY_MANAGER_INVALID_ID as usize;

/// Maximum encoded ownership ledger size (64 KiB for at most 255 records).
pub const MAX_LEDGER_BYTES: usize = 64 * 1024;
/// Family magic, format marker, version and encoded logical length.
pub const LEDGER_PAYLOAD_HEADER_LEN: usize = 8 + 4 + 4 + 8;
/// Maximum opaque generation payload, including its logical envelope header.
pub const MAX_COMMITTED_PAYLOAD_BYTES: usize = MAX_LEDGER_BYTES + LEDGER_PAYLOAD_HEADER_LEN;
/// Two bounded CBOR byte strings plus record metadata (128 KiB + 4 KiB).
pub const MAX_LEDGER_RECORD_BYTES: usize = 2 * MAX_LEDGER_BYTES + 4096;
/// Maximum CBOR container nesting on maintained decode paths.
pub const MAX_LEDGER_NESTING: usize = 32;
