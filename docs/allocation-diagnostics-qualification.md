# Allocation diagnostics buffer qualification

Historical compatible 0.26.3 prototype, folded into the pending 0.27.0 hard cut,
after released
`62e15131c6145fa66a77528d96b00dcbf64a892c` (0.26.2). This records local Linux
measurements and Wasm compiler output, not installed IC execution or publication.
The recorded source predates the ownership-ledger cut; these numbers qualify
only that buffer prototype. The manifest remains 0.26.2 until maintainer release
preparation. Current-cut evidence belongs to [ledger qualification](current-ledger-qualification.md).

## Scope and tradeoff

[`layout::read`](../src/runtime/layout.rs) validates the current pinned
ic-stable-structures 0.7.2 manager layout for reopen and allocation diagnostics.
It previously allocated a 32 KiB `Vec` for the bucket table on each call. A fixed
local array now replaces that temporary allocation. Iteration borrows the array,
avoiding an owned array iterator that could copy the complete table.

The validator still reads the 2,080-byte header and complete 32,768-byte table in
two backing reads. It checks the unused tail as well as the allocated prefix.
There is no cache, chunked IO, new dependency, unsafe initialization or persistent
buffer. APIs, formats, typed failures, corruption checks and growth behavior are
unchanged. The scoped Clippy expectation documents why this bounded array exceeds
the general 16 KiB stack-array lint threshold.

The table moves from heap to stack; this is not a reduction in total live bytes.
Detailed reports retain their peak heap because their output rows are larger
than the temporary table. No IC instruction/cycle saving, general Wasm-size
saving, stable-memory saving or reliable timing improvement is established.
The focused raw Wasm comparison below qualifies one existing probe only.

## Matched native measurements

Linux x86-64, Rust 1.99.0 (`b940084d7eb6a299eb4bfeb8e34901bc051e7ac4`, LLVM
23.1.1), unchanged root lockfile, standard Cargo release profile, one test thread.
Both variants used the same then-current
[`allocation_diagnostics_allocations` harness](../tests/allocation_measurements.rs).
Baseline production source is released 0.26.2; candidate differs in the validator
above. Fixtures are prepared outside counting: one-page buckets, an empty manager
or two two-page allocations at IDs 100 and 254, then the same manager after an
empty-declaration bootstrap to generation 1. The ledger is initialized only by
runtime bootstrap. Generation 0 in the CSV denotes an unbootstrapped runtime.

Each operation runs 100 times, consumes its result through `black_box`, and drops
owned output before the next iteration. This thread's `System` allocation
requests are counted, excluding existing input memory and fixture preparation;
the counter verifies that every measured allocation is released. These are
requested heap bytes, not RSS, allocator internals or whole-application memory.

| Operation, both manager fixtures | Baseline allocations/call | Candidate allocations/call | Peak requested heap, baseline → candidate |
| --- | ---: | ---: | ---: |
| Numeric summary before/after bootstrap | 1 | 0 | 32,768 → 0 bytes |
| Detailed report before bootstrap | 4 | 3 | 36,748 → 36,748 bytes |
| Detailed report after bootstrap | 14 | 13 | 36,838 → 36,838 bytes |

No measured operation reallocates. Full matched rows, including timings, are in
[`allocation-diagnostics-native-allocations.csv`](measurements/allocation-diagnostics-native-allocations.csv).
Custom backing implementations can perform their own allocations; these results
use `VectorMemory`, not an arbitrary implementation of `Memory`.

## Wasm stack inspection

Rust 1.99.0 standard release assembly for the library's `Ic0StableMemory`
instantiation shows the `layout::read` frame changing from 4,656 to 37,408 bytes.
The calling `measure_allocations` frame remains 7,776 bytes. The existing runtime
integration probe separately shows the inlined constructor frame changing from
4,176 to 35,376 bytes. There is one local table, rather than a second full-table
iterator copy. These are individual compiler-generated frames, not whole-call
stack high-water measurements or stack-overflow qualification. Consumer linker
stack budgets must include callers and backing-memory implementations.

## Focused raw Wasm comparison

The existing `wasm-runtime-integration-size-probe`, compiled with Rust 1.99.0,
the selected root lockfile and standard Cargo release settings, is 451,284 bytes
with released 0.26.2 production source and 450,697 bytes with the candidate:
587 bytes smaller. The final candidate rebuild is byte-identical to the earlier
candidate artifact. Its zeroed table uses a Wasm `memory.fill` instruction at
runtime, rather than embedding another 32 KiB of zeros in the binary.

SHA-256 of the baseline artifact is
`294aafc2654ac0e489fb46edac122b510dbdeb2ae190be1a229ef522baa47549`;
the candidate is
`56ebc59300d8917cffd2a9b624fefdd378559dc3c012a92250aae7e1a09e0817`.
Both are retained as `/tmp/ic-memory-0.26.3-wasm-stack-{baseline,candidate}.wasm`.
The final rebuild log is
`/tmp/ic-memory-0.26.3-wasm-stack-candidate-current.log`.

```bash
cargo +1.99.0 rustc --locked --offline --release --target wasm32-unknown-unknown \
  --example wasm-runtime-integration-size-probe -- --emit=asm
wc -c target/wasm32-unknown-unknown/release/examples/wasm_runtime_integration_size_probe.wasm
```

This is a raw artifact comparison for one standard-release probe, not the
repository's `wasm-size` profile, all size budgets, optimized consumer binaries
or a reduction in runtime stack use. The full Wasm-size gate was not rerun.

## Inputs and retained evidence

SHA-256 bindings for the final native measurements and library stack inspection:

| Input | SHA-256 |
| --- | --- |
| Root `Cargo.toml` | `246c49df609c2e3e8ade9c0b8f1156bbf4b19194163984ff3468c2889b2ebe73` |
| Selected root `Cargo.lock` | `68c9efd0e2b98c57451bf41b83c973a8bf86eaca4a66fa8d30373225dd86e655` |
| Candidate `src/runtime/layout.rs` | `75bc78b539f8a5d568f88a55d7c7c0e81509ade5b3ace02849c2e60ef58b5eca` |
| Shared measurement harness | `2bcef63064b58f92e6cfae6852f849afcec9844f8ece40f05dc260ca03bcc9e3` |
| Baseline native measurement binary | `307c19f008fda018e7c64f0cf17c80b47acd423c83ea6ffaa40021856c254cc1` |
| Candidate native measurement binary | `016a2f90948ff91460fe1f81873803a05408cbec401722be35a1f6c2e9a26806` |
| Baseline library Wasm assembly | `9db12e92975a0c9992e82faf3b8b0b3a7fa2433b664c3a1b2761f68aa6f8a1ed` |
| Candidate library Wasm assembly | `6ac5254ed0a8e38238badeed974a0d835436c2269c4fb410d64927085ee38db3` |

Matched logs are `/tmp/ic-memory-0.26.3-diagnostics-{baseline,candidate}-current.log`;
assembly is `/tmp/ic-memory-0.26.3-wasm-library-stack-baseline.s` and
`/tmp/ic-memory-0.26.3-wasm-library-stack-candidate-final.s`. Earlier measurements,
the initial stack-array lint refusal, and failed fixture attempts remain under
the same temporary prefix. The fixture attempts omitted a required declaration
snapshot or supplied nonempty blank ledger memory; the maintained bootstrap
correctly refused that storage. The final fixture leaves ledger initialization
to its runtime owner. Temporary logs are local evidence, not packaged artifacts.

## Focused behavior checks

- Rust 1.99.0 and 1.88.0 each pass the 65 runtime tests, including exact read
  counts/bytes, unchanged backing storage, current/unknown/ledger accounting,
  same-release reopen, growth limits and malformed metadata rejection.
- Extend the malformed-layout fixture to the final unused bucket-table entry;
  rejection returns `BucketTable { index: 32767 }` without writes or growth.
- The matched allocation measurement passes with both lifecycle states and
  manager fixtures. Library Wasm compilation and stack assembly emission pass.
- Strict Rust 1.99.0 Clippy passes for the library and measurement target;
  Rust 1.88.0 checks both offline. Shared snapshot verification, both workspace
  format checks and whitespace checks pass.

Reproduce focused checks and measurements offline:

```bash
cargo +1.99.0 test --locked --offline --release --test allocation_measurements \
  allocation_diagnostics_allocations -- --ignored --test-threads=1 --nocapture
cargo +1.99.0 test --locked --offline --lib runtime:: -- --test-threads=1
cargo +1.88.0 test --locked --offline --lib runtime:: -- --test-threads=1
cargo +1.99.0 clippy --locked --offline --lib --test allocation_measurements -- -D warnings
cargo +1.99.0 rustc --locked --offline --release --target wasm32-unknown-unknown \
  --lib -- --emit=asm
make verify-shared-tooling fmt-check
```

Run the baseline measurement against released production source plus the same
harness; repeating candidate measurements is not a baseline comparison. Full
release gates, installed IC/PocketIC behavior and native macOS execution have
not been requalified. See [host support](host-support.md) and
[release procedures](../RELEASING.md) for their separate obligations.
