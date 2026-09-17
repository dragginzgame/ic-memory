# Key-only allocation and bounded recovery

This is the current contract for issues #2–#4. Fixed declarations remain useful
for host composition. Logical requests add no fields to the durable ledger and
create no second allocation map.

## Placement and host adoption

`MemoryRequest::new(authority, key, schema)` checks the existing key grammar,
authority identifier and schema metadata. Register it with
`register_memory_request` before static sealing, use the key-only
`ic_memory_declaration!` form, or pass it to `SealedDeclarationSnapshot::new`
with explicitly owned fixed declarations and range grants.

The runtime opens its fixed ledger root (ID 0), recovers history, resolves
requests, validates the complete resolved snapshot under current policy, stages
one generation, persists it, then publishes `CommittedAllocations`. The default
runtime delegates to this same implementation. Resolution does not commit.

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
retirement retain `AllocationValidationError` variants. Persistence refusal
returns `RuntimeBootstrapError::StableCellLedgerWriteTooLarge`. No failed
bootstrap publishes a capability. Ledger growth reserves backing capacity
before upstream manager bucket assignment, avoiding its panic-on-growth-refusal
path. A fresh failed attempt can leave an initialized empty ledger cell; an
existing committed mapping remains unchanged.

A library adopting a bootstrapped host calls `committed_allocations().slot_for`
for its requested keys and `open_memory_by_key`, or the default-runtime
`open_default_memory_manager_memory_by_key`. These calls neither bootstrap nor
replace policy/configuration. They do not read history. The runnable
[`key_only` example](../examples/key_only.rs) covers standalone ownership and a
composed host with automatic requests, a fixed control slot and a prior journal
reservation.

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
control-store and lifecycle checks after commitment. Actual generated IcyDB
integration and its end-to-end qualification remain downstream work.

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

Previously, the stable-cell header was checked against physical capacity and
`usize`, then allocated. CBOR decoding checked trailing bytes after decoding;
metadata and ownership invariants were validated after DTO construction.
Generation staging cloned history and appended a record even for unchanged
inputs. These did not establish a small history or pre-allocation bound.

| Resource | Current ceiling | Earliest enforcement |
| --- | ---: | --- |
| Stable-cell ledger value | 33,558,528 bytes (32 MiB + 4 KiB) | Header check before payload allocation/read; direct record decode before CBOR |
| Logical ledger CBOR | 16,777,216 bytes (16 MiB) | Envelope decode before payload copy/CBOR; writer before envelope creation or commit mutation |
| CBOR container depth | 32 nested edges | Allocation-free syntax walk before serde |
| Advertised CBOR text length | Remaining input bytes | Syntax walk before serde allocation |
| Opaque generation byte string | 16,777,240 bytes (16 MiB + 24-byte envelope) and remaining input bytes | Allocation-free syntax walk before serde; writer checks the same limit |
| Advertised array/map entries | At least one remaining byte per element (two per map pair) | Syntax walk before serde allocation hints |
| External fixed + logical declarations | 254 | Snapshot sealing before copying/canonicalizing inputs |
| External range declarations | 254 | Snapshot sealing before copying inputs |
| Resolved/generic declarations and allocation records | 255, including governance | Declaration validation; record visitor before vector growth; ledger integrity before staging/commit |
| Generation records | 65,536 | Bounded serde visitor before vector growth; staging before history clone |
| Schema records per allocation | 65,536 | Bounded serde visitor before vector growth |
| Total schema records | 65,536 | Integrity/staging checks; encoded-byte ceiling bounds construction before this aggregate check |
| Diagnostic strings | 256 bytes | Existing constructor/integrity checks, after bounded decode |

The syntax walk accepts the definite-length current writer shape, rejects
indefinite containers, excessive nesting, truncation and trailing bytes, and
allocates no temporary tree. It is shared only by maintained ledger/record
production decode owners (plus test helpers), not application data. Direct
caller-selected serde decoders are outside the recovery contract; decoded DTOs
are not capabilities. Typed outer payload/envelope errors, codec errors and
`LedgerIntegrityError::LimitExceeded` fail closed. Oversized existing storage
cannot be replaced with genesis.

The outer ceiling permits two maximum byte-string payloads, including their
24-byte envelopes, plus record metadata. The current persisted shape is a
pre-1.0 hard cut: recreate earlier integer-array records; the format version
remains 1. See [codec qualification](opaque-ledger-payloads.md). Writers
validate structural bounds before encoding and byte bounds before mutating the
commit store; runtimes also check outer bytes before memory growth/write.
Staging rejects excessive prior history before cloning and checks the resulting
record/schema counts. Encoding memory is bounded by admitted structural counts
and diagnostic lengths; the encoded-byte check follows that bounded encoding.
Reader/writer tests round-trip the 65,536-generation boundary and 255-record
boundary and verify rejection leaves the store unchanged.

Repeated bootstrap on the same runtime and same sealed snapshot/policy remains
idempotent. Recreating a runtime for an upgrade adds one generation even for an
unchanged declaration set. There is no compaction: 65,536 generations allow over
179 years of daily upgrades or about 7.5 years of hourly upgrades. Reservations
and retirements also consume generations. Large fingerprints, schema churn and
the byte ceiling can exhaust capacity sooner. Exhaustion is explicit and typed;
operators must monitor history and plan a future format/import decision before
reaching it. No ownership or tombstone is discarded to make room.

Assumptions: backing `Memory` obeys its read/grow/write contract, the runtime is
the sole manager owner, and IC messages roll back stable-memory writes on traps.
The Cell is not a crash-atomic file protocol on arbitrary native backing memory;
arbitrary partially persisted writes fail closed. Existing dual-slot corruption
and interrupted-commit tests remain in the suite.

## Qualification and costs

Raw, uncompressed Wasm is measured with matching Rust 1.97.1, the committed
`wasm-size` profile, `wasm32-unknown-unknown`, and the same maintained core and
diagnostics probe sources at baseline and after the change. No native timing is
used. Baseline is release commit `4a5cd22` (0.13.3).

| Probe | Baseline bytes | Current bytes | Delta |
| --- | ---: | ---: | ---: |
| Core, unchanged fixed-declaration probe | 240,300 | 255,112 | +14,812 |
| Diagnostics, unchanged probe | 289,090 | 307,589 | +18,499 |
| Key-only equivalent of core bootstrap/open | 240,300 | 254,762 | +14,462 |

The key-only probe preserves the core export names and one-slot workload while
replacing the fixed declaration/open with an explicit `Allowed` grant and
key-only request/open. It is 350 bytes smaller than the current fixed probe.
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
capacity preflight adds the existing fixed 34,848-byte manager-layout read only
when the ledger virtual memory needs to grow. No accounting framework or
allocator-strategy configuration was introduced.

The patch touches 18 existing `src` files (+733/−60 lines, including embedded
unit tests) and adds a 373-line runtime qualification module. The recovery
preflight and bounded visitors occupy 121 added lines in `cbor.rs`; structural
history checks add 41 lines in `ledger/integrity.rs`. Supporting examples,
documentation and CI budgets are separate. The semantic bootstrap flow gains
one resolution step between recovery and validation; publication stays after
the same persistence boundary.
