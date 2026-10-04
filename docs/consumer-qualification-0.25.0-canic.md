# Published 0.25.0 Canic memory qualification — 2026-10-04

Canic Core's checked-slot adoption passes 32 focused native tests, strict
library/test Clippy and default-feature Wasm compilation against published
ic-memory **0.25.0**, without an ic-memory source override.

The producer package identifies release commit
`2c76eb60dfc1d05eea513e6937e207369b2813a9`; its Rust sources match this release
byte for byte. The registry checksum is
`843a6128daaf0b3b81edeb71fb3386e0a333a4ceb5388d140c5d5ddfcd350fa2`.

## Consumer source and graph

The isolated consumer is a snapshot of Canic's working tree, based on commit
`3978e02d28fea9022c7a6e84a7ec6d6e4a0d4af3`. Its manifest and lockfile already
selected published ic-memory 0.25.0. The adoption changes policy signatures to
`MemoryManagerSlot`, removes the slot-error adapter, and uses `.id()` in
diagnostic projection. The optional public diagnostic ID field remains intact.
The same source adoption is applied to the live Canic checkout; its unrelated
worktree changes are preserved.

This is a working-source qualification, not qualification of a released Canic
package. The snapshot includes its existing Canic-local workspace patches and
the dependency updates already selected by the maintainer. No ic-memory path
patch or fabricated registry checksum is used. The ABI guard verifies that the
main deployed runtime graph contains exactly one ic-memory package identity.

The manifest SHA-256 is
`b7fa9c9c691d1c88f8a0e02dff63e934c8cf2d38b38c8d98ca990ef8e77eb4b1`;
the lockfile SHA-256 is
`fcd32582a0c11b07a10ba8a130a6b09defe2bae87870af08bd413b215abf0b31`.

## Focused checks

All commands use Rust 1.99.0, `--locked --offline` and
`CARGO_TARGET_DIR=/tmp/canic-memory-consolidation-target`, sequentially, from
`/tmp/ic-memory-0.25-published-adoption/canic`.

| Cargo arguments after `--locked --offline` | Result |
| --- | --- |
| `test -p canic-core --lib memory::policy::tests` | 6 passed |
| `test -p canic-core --lib ops::runtime::memory::tests` | 14 passed |
| `test -p canic-core --lib ops::runtime::public_metrics::memory::tests` | 8 passed |
| `test -p canic-core --test stable_memory_abi_guard` | 4 passed |
| `clippy -p canic-core --lib --tests -- -D warnings` | Passed |
| `check -p canic-core --target wasm32-unknown-unknown` | Passed |

These exercise namespace/range ownership, reservation refusal, configured
bootstrap, diagnostic projection, bounded allocation accounting, growth
refusal/retry, public metrics and the managed-memory ABI. Changed live Rust
files also pass scoped formatting and whitespace checks.

Raw logs, captured manifests, the source adoption patch, consumer worktree
status and snapshot input hashes are retained in the ignored
`target/qualification/0.25.0/canic/` directory.

## Published blob alignment and fixture follow-up

After the Core checks, ic-blob-storage 0.14.6 became available from the registry.
It requires ic-memory 0.25 and identifies release commit
`edc8a9b4df672569b03e1454d620c70dbbf4fa52`. Its registry checksum is
`43e690430fa6ae71200439a505ad7b8e3d36a9fd82a7fb9bf78048beb4fe6860`.
Canic's independent adapter now selects this published service and ic-memory
0.25; both adapter and consumer lockfiles select ic-memory 0.25.0. The consumer's
normal Wasm dependency graph contains exactly one ic-memory package identity.
No upstream source override is used.

The adapter manifest SHA-256 is
`4b18fb14a4379cb17a215ef76861899d9e584a510cd642fa28b8da8ee0a56704`;
the adapter lockfile SHA-256 is
`83d4317f13ea1c6eac2d2e98bf0674193e8fb4911d000cdf40aab6cb4dd73625`;
the consumer lockfile SHA-256 is
`09e556e57e994ff97f6b69d9bd8c32808b39b2720e75e20ff668e9aef7c767cf`.

Both independent libraries pass Rust 1.99.0 Clippy with `--locked --offline
--lib -- -D warnings`, one build job and their own target directories. The
consumer also passes its governed `canic build consumer-app blob --workspace .
--config canic.toml --icp-root . --profile fast --json`, with
`CARGO_BUILD_JOBS=1 CARGO_NET_OFFLINE=true`. Declaration extraction, role admission,
runtime compilation and finalization complete without bypassing the managed
build guard. The selected existing CLI reports 0.110.52 and has SHA-256
`e4619ef3a55674acb3fb41ebc3c448a2640573b99d61238cbb64d7d70d93c98a`;
this does not qualify a freshly rebuilt CLI. The resulting consumer Wasm SHA-256
is `d9be333370c94a07bf60fd4f3fab5dc68968c1ad4dc1341dcb3daf1e02104666`.

The embedded allocation peer initially fails read-only verification as stale.
The maintained `refresh_embedded_root` helper regenerates the Wasm and source
proof, then `verify_embedded_root` passes. Both helpers run through
`cargo +1.99.0 run --locked --offline -p canic-testing-internal --example NAME`,
with one build job and native-only `canic-host` dev-profile overrides
`opt-level=0`, `codegen-units=64`, `debug=0`; the Fast Wasm profile is unchanged.
The refreshed fixture SHA-256 is
`24cd00206abf8c5249e9ea9aa905fe8d34883d7c6d3ff53743281a25bf1f28bf`.
The captured source proof records its producer inputs, toolchain and lock hashes.
Raw library/build/fixture logs, the build result, graph, dependency patch and
manifests/lockfiles accompany the original Core evidence in
`target/qualification/0.25.0/canic/`.

These follow-up checks qualify library compilation, a composed Wasm build and
fixture byte consistency. Earlier PocketIC and Toko-copy results remain scoped
to their original 0.24 graph; installed behavior on this graph is not qualified.

## Remaining adoption and qualification

IcyDB's inspected manifest and slot policy callers still use 0.24. Its prepared
source adoption patch has been apply-checked, but was neither applied nor
compiled during this Canic qualification.

No complete Canic suite, installed lifecycle proof, live deployment, consumer
version change, commit, tag or push was performed. Consumer MSRV, other feature
combinations and execution cost were not measured. The build reports its code
size, but no comparable size measurement or performance improvement is claimed.
