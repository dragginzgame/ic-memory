# Installed runtime qualification

This standalone, unpublished host package keeps PocketIC dependencies out of
the library's normal graph. It installs the supplied
[`wasm-io-qualification`](../../examples/wasm_io_qualification.rs) fixture in a
caller-owned PocketIC 16.0.0 server. It never starts or downloads a server.

Use the [qualification record](../../docs/runtime-io-qualification.md) for
inputs, commands, observed results and remaining limitations. Both the supplied
Wasm path and server URL are required. `--measure-valid-io` skips the rejection
and upgrade checks; its output alone is not qualification evidence.
