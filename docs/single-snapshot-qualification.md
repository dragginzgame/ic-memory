# Single-snapshot qualification for 0.39.0

The implementation replaces the two-slot physical store with one checksummed
current snapshot. The [current ledger contract](current-ledger.md) owns the
format, recovery and retained-installation requirements. Historical 0.27.1
prototype results are separate evidence and are not reused as these results.

## Inputs and scope

The control is the incoming working tree at release source
`4118e41ceebe1a2a7e7ae5c7c9fdaf86c4e4adc2`, including the already-qualified bounded
serialization writer and pending runtime CI cache changes. Its immutable source
archive is `target/qualification/hardcut0390/before.tar`. The candidate applies
the current-snapshot cut, the internal bootstrap-function simplification and
checked request-key reuse to those same inputs. Both selected manifests and
lockfiles, compiler/MSRV, features and tool catalogs remain unchanged. Native
Host helpers remain the incoming 0.18.2 selection. Final source/input hashes and
logs are retained under `target/qualification/hardcut0390/`.

Rust 1.99.0 builds locked/offline. Wasm controls use the same `wasm-size` profile,
fat LTO, one codegen unit, aborting panics and stripped symbols. The selected
Binaryen executable optimizes both controls with the same `-Oz` and bulk-memory
setting. These are probe artifacts, not a composed application-size guarantee.

| Probe | Control raw | Current raw | Raw reduction | Control Oz | Current Oz |
| --- | ---: | ---: | ---: | ---: | ---: |
| Core | 178,190 | 176,888 | 1,302 | 160,654 | 159,485 |
| Diagnostics | 212,221 | 210,266 | 1,955 | 190,786 | 189,021 |
| Admission | 182,999 | 181,736 | 1,263 | 165,135 | 164,023 |
| Runtime integration | 186,722 | 185,450 | 1,272 | 168,128 | 166,966 |
| Installed IO fixture | 195,917 | 194,497 | 1,420 | 175,748 | 174,467 |

## Native allocations and stable IO

The same explicit release-mode workload runs in isolated control/candidate
source trees, with one test thread and the existing thread-local System allocator
observer. Stores have completed two commits before encoding/reopening. Logical
records, generation, keys and maximum schema versions match. Peak bytes are
requested native heap within the measured phase, not RSS or Wasm linear memory.

| Records / keys | Control encoded | Current encoded | Control reopen peak | Current reopen peak |
| --- | ---: | ---: | ---: | ---: |
| 0 | 266 | 144 | 374 | 198 |
| 1 | 466 | 244 | 774 | 398 |
| 64 | 13,150 | 6,586 | 29,534 | 14,778 |
| 255 / short | 51,732 | 25,877 | 117,268 | 58,645 |
| 255 / maximum | 109,872 | 54,947 | 240,944 | 120,483 |

Each reopen uses one fewer allocation. A separate identical JSON/CBOR request
workload reports 3,000 -> 2,000 allocations over 1,000 decoded requests: one
allocation and key copy are removed per request. Native timing is not an IC
instruction or speed claim.

The existing one-page-bucket fixture's 64 bootstrap generations retain 256
writes but reduce write bytes from 535,594 to 268,758. Reads remain 448;
read bytes fall from 4,996,326 to 4,729,492 because fixed manager metadata
observations remain. Physical extent and assigned buckets are unchanged.
Smaller records do not reclaim already grown stable memory or establish a
bucket-count reduction.

## Rejection, persistence and installed execution

Native tests cover maximum bounded payload/record agreement, required current
presence, unknown/invalid framing, checksum/counter/payload corruption, exact
successors, overflow, stale proofs, retained identity and tombstone custody,
policy/admission ordering, and persistence before capability publication.
A refusal before the first native persistence mutation leaves the backing and
runtime unbootstrapped; retry reruns admission and publishes one generation.
This does not claim arbitrary partial native writes roll back.

`make validate` passes, including strict Clippy/Rustdoc, unit/integration and
compile-fail contracts, current fixture round trips, Wasm budgets, working-tree
package verification and MSRV 1.88. Subsequent documentation/fixture formatting
checks pass separately with unchanged runtime behavior.

Installed Testkit 0.25.4 / PocketIC client 16.0.0 and server 16.1.0 qualification
passes for original/O3/Os/Oz in runtime attempt `GfuSea`. Every variant checks 12
invalid-IO traps with rollback, post-persistence upgrade trap rollback, geometry
conflict rollback and clean same-format retry advancing exactly one generation.
A separate raw-Wasm check installs the pre-cut control, rejects the new binary's
upgrade over that image, and verifies the original binary, generation, mapped IDs
and payload markers remain intact. Retrying with the original binary succeeds.
It is rejection/recovery evidence, not a migration or successful cross-format
upgrade. No retained installation has been reset, converted or deployed.

Dirty-source Linux results do not establish hosted macOS acceptance. Consumer
response/generator and retained-image adoption is coordinated by
[Memory #106 — One current ledger snapshot](https://github.com/dragginzgame/ic-memory/issues/106)
and [Canic #567 — Adopt single-snapshot recovery diagnostics](https://github.com/dragginzgame/canic/issues/567).
