<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-memory/ic-memory-readme-header.svg" alt="IC Memory — Stops upgrades from mixing up stored data" width="100%">
</p>

# Key-only allocation and bounded recovery

This is the current contract for completed issues #2–#4. Fixed declarations
remain useful for host composition. Logical requests add no fields to the
durable ledger and create no second allocation map.

## Placement and host adoption

`MemoryRequest::new(authority, key, schema)` checks the existing key grammar,
authority identifier and schema metadata. Register it with
`register_memory_request` before static sealing, use the key-only
`ic_memory_declaration!` form, or pass it to `SealedDeclarationSnapshot::new`
with explicitly owned fixed declarations and range grants.

The runtime opens its fixed ledger root (ID 0), recovers ownership records, runs host
admission, resolves requests, validates the complete resolved snapshot under
current policy, stages one generation, persists it, then publishes
`CommittedAllocations`. The default runtime delegates to this same
implementation. Resolution does not commit.

Resolution sorts requests by stable key. Every known key retains its durable
slot. Every new key takes the lowest unused ID covered by an explicit `Allowed`
grant to its authority. All historical states occupy slots, including omitted,
reserved and retired records. Fixed current declarations occupy slots before
request resolution. Policy rejection fails the whole attempt; placement does
not probe custom policy for alternate IDs. Known keys are never relocated to
satisfy changed policy. Matching reservations activate through existing claim
validation; a reservation is never free space for another key.

A standalone host can explicitly grant itself IDs 10–254. A composed host grants
libraries smaller pools. `Reserved` ranges can admit explicit fixed claims or
matching historical requests under current policy, but supply no new automatic
placements. Raw unregistered manager clients must be declared or reserved by
the host; no diagnostic can infer their ownership.

`MemoryResolutionError::Exhausted` identifies the bounded key and authority;
range errors identify the rejected slot/authority. Historical conflicts and
retirement retain `AllocationValidationError` variants. Persistence growth refusal
returns `RuntimeBootstrapError::LedgerGrowth` carrying a `RuntimeGrowError`;
an oversized encoded record returns `StableCellLedgerWriteTooLarge`. No failed
bootstrap publishes a capability. `RuntimeMemory::grow` reserves backing capacity
before upstream manager bucket assignment for application and ledger handles.
Direct growth returns `Result<u64, RuntimeGrowError>`; ordinary refusal preserves
virtual extents and manager metadata. Only the upstream `Memory` trait adapter
maps these errors to its required `-1` sentinel. A fresh failed attempt
can leave an initialized empty ledger cell; an existing committed mapping remains
unchanged.

A library adopting a bootstrapped host uses `verify_authority` to check every
fixed declaration and logical request under its authority, then `memory_id`
and `open_memory_by_key`. Default-runtime equivalents are
`verify_default_memory_manager_authority`, `default_memory_manager_memory_id`
and `open_default_memory_manager_memory_by_key`. These calls neither bootstrap nor
replace policy/configuration. They do not read ownership records or construct an absent
default runtime. Verification reports typed missing-key, fixed-ID, current
authority and diagnostic-metadata mismatches; it does not validate application
schema semantics or replay admission. The runnable
[`key_only` example](../examples/key_only.rs) covers standalone ownership and a
composed host with automatic requests, a fixed control slot and a prior journal
reservation. The [composed-host regression](../examples/composed_host.rs) adds
consumer admission, two cold reopens and bootstrap on each native worker.

## Omitted-store inspection

Omission retains ownership but removes the key from current open authority.
Opening an omitted or unknown key returns
`RuntimeOpenError::StableKeyNotCommitted`. Diagnostics reporting an unknown
current binding do not make the slot available.

For lifecycle reconciliation, every journal to inspect must be explicitly
included before commitment. Consumers that already know the keys can include
them in the original sealed manifest, as below. Consumers that discover roles
from allocation metadata can instead contribute
`RuntimeBootstrapPolicy::prepare_bootstrap` to the host policy and use
`BootstrapAdmission::include_historical` before resolution. See the
[admission contract and example](recovered-admission.md). Both routes retain the
single persistence boundary and current grant/policy checks.

```rust
ic_memory::ic_memory_range!(authority = "db", start = 100, end = 119, mode = Allowed);
ic_memory::ic_memory_declaration!(authority = "db", key = "db.current.v1");
// A known prior store's journal, retained explicitly for reconciliation.
ic_memory::ic_memory_declaration!(authority = "db", key = "db.removed_journal.v1");

fn reconcile() {
    ic_memory::bootstrap_default_memory_manager().unwrap();
    let journal = ic_memory::open_default_memory_manager_memory_by_key(
        "db.removed_journal.v1",
    ).unwrap();
    // Consumer decodes journal debt/commit markers and refuses unsafe retirement.
    # let _ = journal;
}
```

The static-manifest route requires keys independently known before sealing.
The admission hook closes the allocation-metadata discovery gap without reading
an unopened control store or extending the global sealed registry: it completes
the runtime-local request set before resolution. Its historical selections reject
unknown or retired keys, whereas ordinary new declarations may allocate fresh
slots. Naming a key in an open call grants nothing.

Allocation metadata does not contain IcyDB incarnation, journal debt or accepted
schema. Consumers must supply their own identity/role interpretation and retain
control-store and lifecycle checks after commitment. IcyDB's maintained generated
upgrade qualification now covers omitted-journal reconciliation, debt and pending
markers; see the [issue reconciliation](issue-reconciliation.md#downstream-acceptance).
Application-specific host composition and lifecycle qualification remain
consumer responsibilities.

A foreign authority without the slot grant or a revoked grant fails before
persistence. Explicit generic retirement produces `RetiredAllocation` even if
the key is requested again. Current custom policy is also rechecked. The host
owns range grants; they are current authorization, not persisted identities.
Policy changes take effect at a new runtime/bootstrap, not by revoking already
issued handles within a live runtime.

The production-runtime tests write a marker to B, recreate with only A plus a
new key, verify B cannot open and its slot cannot be reused, then explicitly
include B in a subsequent manifest and read its marker. They also cover foreign
and revoked grants and explicit retirement. There is no historical-open API,
new retirement endpoint or data clearing. IcyDB still owns schema/incarnation,
journal-debt, pending-commit and database-retirement decisions. Generic
redeclaration does not promise that IcyDB can reintroduce a retired database.

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
| External fixed + logical declarations | 254 | Snapshot sealing before copying/canonicalizing inputs |
| External range declarations | 254 | Snapshot sealing before copying inputs |
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

## 0.14.0 qualification and costs

Raw, uncompressed Wasm is measured with matching Rust 1.97.1, the committed
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
