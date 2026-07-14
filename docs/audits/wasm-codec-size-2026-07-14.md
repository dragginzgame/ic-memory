# CBOR Codec Raw Wasm Size Check — 2026-07-14

## Question

Does replacing `serde_cbor` with `ciborium` increase the non-gzipped WebAssembly
artifact size because of its different dependency tree?

## Method

A temporary two-binary probe compiled the same serde DTO and the same exported
encode-plus-decode operation through either `serde_cbor 0.11.2` or
`ciborium 0.2.2`. Both binaries used:

- target `wasm32-unknown-unknown`;
- `rustc 1.95.0`;
- `opt-level = "z"`;
- fat LTO and one codegen unit;
- `panic = "abort"`; and
- stripped symbols.

The DTO contained a `u64`, `String`, `Vec<bool>`, and `Option<u32>`. Its exported
function constructed a value from a runtime argument, encoded it to `Vec<u8>`,
decoded it, and consumed both the bytes and decoded value. This kept equivalent
codec work reachable from the Wasm export while allowing normal dead-code
elimination.

The second row applies Binaryen 108 `wasm-opt -Oz` with bulk-memory and
sign-extension features enabled. All values are raw, uncompressed bytes.

## Result

| Pipeline | `serde_cbor` | `ciborium` | Delta | Delta % |
| --- | ---: | ---: | ---: | ---: |
| Rust release artifact | 134,228 | 78,940 | -55,288 | -41.19% |
| After `wasm-opt -Oz` | 117,648 | 69,549 | -48,099 | -40.88% |

In this controlled encode/decode probe, `ciborium` is materially smaller; the
dependency change does not impose a raw Wasm size increase.

The current normal dependency graph for `wasm32-unknown-unknown` includes
`half 2.7.1` through `ciborium-ll`. Although `crunchy` appears in `Cargo.lock`'s
all-target resolution, it is not in the normal Wasm dependency graph; this
version of `half` uses it only for SPIR-V and development configurations.

## Scope

This isolates codec cost rather than claiming a fixed whole-canister delta.
Final canister size still depends on which application paths remain reachable,
the Rust toolchain, and post-link optimization. A release canister should keep
its own raw artifact budget check, but there is no size evidence here against
the `ciborium` choice.
