# Host-wide allocation pool qualification

Prepared for pending **0.34.0** on released 0.33.4,
`9137192d4b3425a229fa21df5f343883f56d80e8`. This is local working-tree evidence,
not committed, published or deployed acceptance. The maintainer reports no
current users/deployed installations and requests a complete source hard cut.
[Memory #44](https://github.com/dragginzgame/ic-memory/issues/44) owns coordinated
acceptance.

## Contract and retained bytes

Components contribute permanent keys, named owners and schema metadata. The
host admits disjoint namespaces and physical exclusions through one checked
`MemoryAllocationPool`. All owners share available IDs 10..=254; governance
0..=9 stays excluded. Populated eligible IDs absent from the recovered ledger
reject before initial ledger-cell writes, requiring explicit unmanaged custody.
The doctor uses the same read-only custody check.

The existing `ICMS` stable-cell marker/version 1 and `ICMEMLED` ledger family,
CBOR shapes, ID 0 ledger root and persisted manager geometry remain unchanged.
There is no format transition, reset, old-format reader or additional allocator.
The ledger retains key-to-ID bindings, states and latest schema; it does not
persist owner labels. Current host namespace grants define owner policy.
Changing a label at cold bootstrap preserves IDs and payloads when current
admission allows it. Labels are not authentication or linked-code isolation.

Known keys retain their IDs. All retained states occupy space; omitted and
retired keys never release it. New keys resolve deterministically in key order.
Recovery precedes admission, resolution and validation. Only successful durable
commit publishes application capability. Warm bootstrap binds the exact source,
canonical pool and custom policy identity without replaying preparation.

The old range-owned API, fixed component registrations, numeric runtime-open
argument and alternate key-open alias are removed together. Generated macros
accept key/owner requests only. Resolution produces the existing checked declaration snapshot, then transfers
it to the committed capability. No extra resolved-row map is retained. Current
owners are projected from the bound host pool; low-level ledger persistence
still accepts resolved IDs.

[Exact symbol retirement inventory](allocation-pool-removals.md) distinguishes
deletions from retained moves and narrowed renames.

## Focused local evidence

Logs are retained under `target/qualification/0.34.0-pool/`. Failed intermediate
runs remain separately named rather than overwritten as passing evidence.

| Check | Evidence | Result |
| --- | --- | --- |
| Library recovery, allocation, custody, registry, codec and commit tests | `lib-tests-final.log` | 240 pass |
| Public integration callers and nine compile-fail boundaries | `public-tests.log` | Pass |
| Host admission, two cold reopens and native worker ownership | `composed-host-tests.log` | Two pass |
| Runnable key-only and recovered-admission examples | `key-only-run.log`, `recovered-admission-run.log` | Pass |
| Package README doctests | `doctests-final.log` | Four pass |
| Strict library/tests/examples Clippy | `clippy-final.log` | Pass |
| Rust 1.88 library floor | `msrv-core.log` | Pass |
| Rust 1.88 Wasm library and six maintained probes/IO fixture | `msrv-wasm.log` | Pass |

The composed native regressions verify canonical order, added/omitted owners,
shared-space allocation and exhaustion, unchanged historical IDs and markers,
current-label replacement, physical exclusions, foreign/retired claims and
read-only diagnostics. Existing admission tests qualify corruption ordering,
known-only selections, ignored-error poisoning, reserved activation and actual
persistence-growth refusal/retry without partial capability. The same current
codec, integrity and commit tests remain; no obsolete-name prohibition was added.

No feature matrix is invented: the library declares no optional Cargo features.
Both selected lockfiles and package manifests match their captured inputs,
preserving the incoming root syn lockfile edit. Git configuration also matches;
indexed entries remain equal to HEAD with no staged changes. The raw index byte
checksum differs from the initial capture, so byte-for-byte index preservation
is not claimed. No staging command or fixture effect targeted the real index. Full validation, Wasm size gates, release,
publication and installed PocketIC execution were not invoked for this batch.

## Downstream qualification boundary

Read-only review identified the owning updates below. Inspected dirty sibling
files are not attributed to their committed HEADs. No sibling source was edited.

| Consumer | Reviewed HEAD | Owning issue |
| --- | --- | --- |
| Canic | `ac55e50334dd6479ec36f404e89e60bcfe9184d6` | [Canic #510](https://github.com/dragginzgame/canic/issues/510) |
| IcyDB | `76dc93ead6b66c8db9cb5c402355867dbdf188e3` | [IcyDB #338](https://github.com/dragginzgame/icydb/issues/338) |
| Blob | `9a808cdf3cee2dd7153e50dd6fa10592f72f6137` | [Blob #48](https://github.com/dragginzgame/ic-blob-storage/issues/48) |
| Jobs | `7939c8a53add5ab019cc26e97bd93274b5ed3a43` | [Jobs #11](https://github.com/dragginzgame/ic-jobs/issues/11) |

They currently select Memory 0.33. Source/generator adoption and native macOS
Intel/Apple Silicon evidence require an immutable delivered minor and exact
consumer graphs. Final acceptance also requires a real Canic + IcyDB + Jobs
consumer through Testkit-owned PocketIC, proving IC init/upgrade, retained
journals/readiness, timer reconstruction, bounded overdue work, message rollback
and unresolved-effect blocking. Native Memory tests and Wasm checks do not
establish those consumer behaviors. Audit a single Memory/stable-structures and
Timers identity in that final graph. #44 must remain open until these obligations
are qualified; upstream API completion alone is insufficient.


## Follow-up flow audit

The [2026-10-10 flow audit](audits/recurring/allocation-pool-flow-2026-10-10.md)
removes mixed source/resolved snapshot state, duplicated owner/placement maps,
repeated pool checks and the redundant custom-error wrapper. Follow-up evidence
is retained separately under `target/qualification/0.34.0-audit/`; the earlier
logs above are not relabelled as qualification of the later code.
