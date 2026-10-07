# Installed runtime qualification

This independent virtual workspace contains the unpublished host package
`crates/ic-memory-runtime-qualification/` and owns its separate `Cargo.lock`.
The package keeps PocketIC dependencies out of
the library's normal graph. It installs the supplied
[`wasm-io-qualification`](../../crates/ic-memory/examples/wasm_io_qualification.rs) fixture in a
caller-owned PocketIC 16.0.0 server. It never starts or downloads a server.

Use the [qualification record](../../docs/runtime-io-qualification.md) for
inputs, commands, observed results and remaining limitations. Both the supplied
Wasm path and server URL are required. `--measure-valid-io` skips the rejection
and upgrade checks; its output alone is not qualification evidence.
