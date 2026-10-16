/// Maximum byte length for printable diagnostic metadata fields.
pub const DIAGNOSTIC_STRING_MAX_BYTES: usize = 256;

/// WebAssembly page size used by `ic-stable-structures` memory implementations.
pub const WASM_PAGE_SIZE_BYTES: u64 = 65_536;

/// Maximum allocation identities in the usable ID domain, including governance.
pub const MAX_ALLOCATIONS: usize = crate::slot::MEMORY_MANAGER_INVALID_ID as usize;

/// Maximum encoded ownership ledger size (64 KiB for at most 255 records).
pub const MAX_LEDGER_BYTES: usize = 64 * 1024;
/// Maximum opaque generation payload containing direct logical CBOR.
pub const MAX_COMMITTED_PAYLOAD_BYTES: usize = MAX_LEDGER_BYTES;
/// One bounded CBOR byte string plus record metadata (64 KiB + 4 KiB).
pub const MAX_LEDGER_RECORD_BYTES: usize = MAX_LEDGER_BYTES + 4096;
/// Maximum CBOR container nesting on maintained decode paths.
pub const MAX_LEDGER_NESTING: usize = 32;
