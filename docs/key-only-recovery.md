<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-memory/ic-memory-readme-header.svg" alt="IC Memory — Stops upgrades from mixing up stored data" width="100%">
</p>

# Key-only allocation and bounded recovery

The runtime accepts permanent key requests and one host-owned allocation pool.
Placement adds no fields to the durable ledger and creates no second allocation
map. See the [0.34 pool qualification](allocation-pool-qualification.md).

## Placement and host adoption

`MemoryRequest::new(authority, key, schema)` checks key grammar, owner metadata
and schema metadata. Register it before sealing, use `ic_memory_declaration!`,
or pass requests to `SealedDeclarationSnapshot::new(&requests)`.

The host supplies `MemoryAllocationPool::new(namespace_grants, exclusions)`.
Each `MemoryAuthority::new(owner, prefix)` admits a disjoint key namespace.
Governance IDs 0..=9 are excluded permanently; all owners share every other
eligible ID. Physical exclusions retain unmanaged custody. A populated eligible
ID without a ledger record rejects bootstrap with `UnmanagedAllocation` before
initial ledger-cell writes. Diagnostics perform the same read-only check.

The runtime recovers the ledger at ID 0, runs host admission, resolves requests,
validates the completed snapshot, stages and persists one generation, then
publishes committed authority. The default runtime delegates to this flow.

New requests are sorted by key and take the lowest unoccupied pool ID. Known
keys preserve their committed IDs. All historical states occupy slots, including
omitted, reserved and retired records. Custom policy rejection fails the whole
attempt; placement does not search alternate IDs to evade it. Matching
reservations activate through normal validation. Retirement is permanent.

Namespace grants authorize current owner labels. Previous owner labels are not
stored in the ledger or enforced. The host can replace a label while preserving
the key and its ID. Labels are not caller authentication or linked-code isolation.

A library adopts a bootstrapped runtime through `verify_authority`, then
`memory_id` or `open_memory(key)`. Default equivalents are
`verify_default_memory_manager_authority`, `default_memory_manager_memory_id`
and `open_default_memory_manager_memory(key)`. These do not bootstrap, replace
policy, inspect persisted ownership or construct an absent default runtime.
Verification checks required keys, current owners and schema metadata without
certifying application schema semantics or replaying admission.

Exhaustion, invalid grants, excluded historical IDs, retirement and corruption
fail closed. Growth refusal returns typed errors; failed persistence publishes
no capability. A fresh admitted attempt can leave an empty initialized ledger
cell; previous committed mappings remain unchanged. IC message traps roll back
stable writes; arbitrary native backing is not a crash-atomic storage protocol.

See the runnable [key-only example](../crates/ic-memory/examples/key_only.rs)
and [composed host](../crates/ic-memory/examples/composed_host.rs).

## Omitted-store inspection

Omitted keys retain their IDs but cannot open under the current capability.
Include independently known prior keys in the source request snapshot, or select
known historical keys during `prepare_bootstrap` through
`BootstrapAdmission::include_historical`. Both routes finish before resolution
and the single persistence boundary. Historical selection rejects unknown,
retired, duplicate, foreign-namespace and physically excluded claims. Ignoring
an admission error does not make the attempt succeed.

```rust,no_run
ic_memory::ic_memory_declaration!(authority = "db", key = "db.current.v1");
ic_memory::ic_memory_declaration!(authority = "db", key = "db.removed_journal.v1");
fn reconcile() -> Result<(), Box<dyn std::error::Error>> {
    let pool = ic_memory::MemoryAllocationPool::new(
        vec![ic_memory::MemoryAuthority::new("db", "db.")?], vec![],
    )?;
    ic_memory::bootstrap_default_memory_manager(&pool)?;
    let journal = ic_memory::open_default_memory_manager_memory("db.removed_journal.v1")?;
    // Consumer validates journal debt and pending effects before retirement.
    drop(journal);
    Ok(())
}
```

Allocation metadata contains no database incarnation, journal debt or schema
support proof. Consumers own control-store admission, reconciliation and
lifecycle decisions. Selecting a key is not permission to revive an
application-retired database. See the [admission contract](recovered-admission.md).

Warm bootstrap requires the same snapshot, exact canonical pool and custom
policy identity. It does not replay preparation. Changed configuration is
admitted only by a new runtime; existing handles are not dynamically revoked.

## Recovery and admission limits

The current ledger is bounded by the usable memory-ID domain, not the number
of upgrades. It contains one retained ownership record per identity, its current
lifecycle state, and latest schema metadata. There is no generation trail or
schema trail. A checked `u64` commit counter binds proofs to commits.

| Resource | Current ceiling | Earliest enforcement |
| --- | ---: | --- |
| Stable-cell ledger value | 135,168 bytes (128 KiB + 4 KiB) | Header check before payload allocation/read; direct record decode before CBOR |
| Logical ledger CBOR | 65,536 bytes (64 KiB) | Envelope decode before payload copy/CBOR; bounded writer before append or commit mutation |
| CBOR container depth | 32 nested edges | Allocation-free syntax walk before serde |
| Advertised CBOR text length | 256 bytes and remaining input | Syntax walk before serde allocation; keys also obey their 128-byte grammar |
| Opaque commit payload | 65,560 bytes (64 KiB + 24-byte envelope) | Syntax walk before serde; writer checks the same limit |
| Advertised array/map entries | Remaining input bounds | Syntax walk before serde allocation hints |
| External key requests | 254 | Snapshot sealing before copying/canonicalizing inputs |
| Host namespace grants / physical exclusions | 254 / 255 | Pool construction and checked decoding |
| Resolved/generic declarations and retained records | 255, including governance | Snapshot validation; record visitor before vector growth; integrity before staging/commit |
| Diagnostic strings | 256 bytes | CBOR preflight before allocation; constructors retain printable ASCII and non-empty checks |

The syntax walk rejects indefinite containers, oversized text, excessive nesting,
truncation and trailing bytes without allocating a temporary tree. Direct
caller-selected serde decoders remain outside the maintained recovery contract;
DTOs are not capabilities. Typed envelope, codec and integrity failures reject
without replacing existing storage with genesis.

The outer ceiling permits two maximum payloads and record metadata. The current
`ICMS` format marker and version 1 identify the current ownership layout; earlier
ledger layouts are unsupported. Before deploying this hard cut to retained
installations, follow [the explicit data disposition requirements](current-ledger.md).
No automatic clearing, fallback reader or migration engine is provided.

The bounded writer performs one canonical CBOR pass. Admitted definite collection
lengths reserve their vectors; nonempty collections below 255 retain one spare
entry for staging. The maximum key/schema/record set round-trips through the
current writer and reader. Excess counts, overflow and corruption reject before
commit mutation.

Matching warm bootstrap is idempotent. Recreating a runtime, reserving or retiring
advances the commit counter, including unchanged declarations. The counter can
change CBOR integer width by at most eight bytes; no event is appended. Repeated
schema changes replace the latest metadata. Recovery and serialization work depend
on retained record count, not upgrade count. Counter overflow is explicit and
never wraps. Ownership and tombstones are never dropped to make room.

Assumptions: backing `Memory` obeys its read/grow/write contract, the runtime is
the sole manager owner, and IC messages roll back stable-memory writes on traps.
The Cell is not a crash-atomic file protocol on arbitrary native backing memory;
arbitrary partially persisted writes fail closed. Existing dual-slot corruption
and interrupted-commit tests remain in the suite.

## Historical 0.14–0.15 qualification and costs

These historical figures describe the released 0.14–0.15 implementation, not
qualification of the current pool API. Raw, uncompressed Wasm was measured with matching Rust 1.97.1, the committed
`wasm-size` profile, `wasm32-unknown-unknown`, and the same maintained core and
diagnostics probe sources at baseline and after the change. No native timing is
used. Baseline is release commit `4a5cd22` (0.13.3).

| Probe | 0.13.3 bytes | 0.14.0 bytes | Delta |
| --- | ---: | ---: | ---: |
| Core, unchanged fixed-declaration probe | 240,300 | 255,112 | +14,812 |
| Diagnostics, unchanged probe | 289,090 | 307,589 | +18,499 |
| Key-only equivalent of core bootstrap/open | 240,300 | 254,762 | +14,462 |

The key-only probe preserves the core export names and one-slot workload while
replacing the fixed declaration/open with an explicit `Allowed` grant and
key-only request/open. It is 350 bytes smaller than the 0.14.0 fixed probe.
CI and `make wasm-size` now enforce 260,000 bytes for core/key-only and 315,000
for diagnostics, admitting the measured implementation with limited headroom.
The old 245,000/290,000 budgets were exceeded, not silently left failing.

IC instruction/cycle measurements for fresh bootstrap and repeated opening are
**unavailable**: this workspace has no configured IC execution measurement
harness. Raw Wasm size is not a substitute for those measurements.

Durable per-allocation metadata delta is **zero bytes** for identical resolved
keys, slots, labels and schema: requests resolve into the existing declaration/ledger
shape. Sealed request metadata and the resolved snapshot are transient. New
placement uses a 255-entry occupied table and at most 254 requests over 255 IDs
and bounded range grants. Key-only opens retain the existing bounded linear
capability lookup. Recovery
adds one allocation-free linear CBOR scan per decoded layer and bounded vector
visitors; history validation retains its existing ordered-set work. Persistence
capacity preflight in that release added the fixed 34,848-byte manager-layout
read when ledger virtual memory needed to grow. The 0.15 runtime instead shares
one live assigned-bucket count, seeded from validated construction metadata and
updated after successful growth, across all handles. Growth performs no metadata
reads or table scans. No accounting framework or
allocator-strategy configuration was introduced.

The patch touches 18 existing `src` files (+733/−60 lines, including embedded
unit tests) and adds a 373-line runtime qualification module. The recovery
preflight and bounded visitors occupy 121 added lines in `cbor.rs`; structural
history checks add 41 lines in `ledger/integrity.rs`. Supporting examples,
documentation and CI budgets are separate. The semantic bootstrap flow gains
one resolution step between recovery and validation; publication stays after
the same persistence boundary.
