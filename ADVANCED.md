<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-memory/ic-memory-readme-header.svg" alt="IC Memory — Stops upgrades from mixing up stored data" width="100%">
</p>

# Advanced ic-memory

This document covers explicit composition and the owned memory runtime. Most
applications should start with the README.

## Contents

- [How it fits](#how-it-fits)
- [Runtime ownership](#runtime-ownership)
- [Policy authority](#policy-authority)
- [Explicit component declarations](#explicit-component-declarations)
- [Runtime diagnostics](#runtime-diagnostics)
- [Explicit `MemoryRuntime<M>`](#explicit-memoryruntimem)
- [Recovery and persistence](#recovery-and-persistence)
- [Stable key rules](#stable-key-rules)
- [Host allocation pool](#host-allocation-pool)
- [Current MemoryManager rules](#current-memorymanager-rules)
- [What it does not do](#what-it-does-not-do)
- [Status](#status)

## How It Fits

`ic-stable-structures` stores the data.

`ic-memory` checks that each logical store is still opening the same physical
slot it owned before.

The native IC ledger anchor is:

```text
MemoryManager ID 0
  -> ic-stable-structures::Cell value bytes
  -> LedgerCommitStore (fallibly decoded DTO)
  -> one checksummed current snapshot
  -> current-format allocation ledger
  -> RecoveredLedger
  -> one owned validation/staging attempt
  -> persisted stable Cell
  -> CommittedAllocations
```

The logical payload uses the built-in `ic-memory` CBOR ledger format. Callers do
not provide a custom codec. `LedgerCommitStore` does not implement
`Storable`: the runtime owns bounded publication bytes and allocation persistence,
while the fallible decode helpers support read-only inspection.

The runtime keeps this internal ledger allocation in its ownership records,
but it removes `ic_memory.*` governance keys from the committed allocations it
publishes for application opens. `MemoryRuntime::open_memory` rejects
those reserved keys.

A framework supplies sealed key requests and one host allocation pool
to one runtime. A cold bootstrap then:

1. Recovers the saved allocation ledger into `RecoveredLedger`.
2. Runs the host's `prepare_bootstrap` hook to admit consumer identity and select
   authorized historical keys from bounded allocation metadata.
3. Resolves logical requests while preserving existing assignments.
4. Validates the completed declarations against retained ownership and current policy.
5. Stages and persists one generation before publishing `CommittedAllocations`.
6. Opens application stable-memory handles through that runtime.

The important rule: validate layout before touching stable data.

The runtime fallibly decodes the ledger stable-cell once per cold bootstrap
attempt and reuses that record for recovery and staging. Corrupt cell envelopes
or ledger-record bytes are reported as bootstrap errors before admission.
`ic-stable-structures::Cell` performs capacity-checked writes. Fresh bootstrap
initializes a readable cell with empty protected slots before admission;
the staged generation is persisted only after validation succeeds.

## Runtime Ownership

`MemoryRuntime<M>` is the canonical owner for one backing memory instance. It
owns the `MemoryManager<Rc<M>>`, allocation-ledger persistence, bootstrap lifecycle,
committed allocation capability, memory opens, recovery diagnostics, and live
memory-size inspection. The runtime and its opened handles share the backing
memory and growth accounting; the manager owns bucket metadata.

The decoded ledger record belongs to a single bootstrap attempt. Every retry
decodes persisted memory again before recovery and admission; no record remains
cached after success or failure. Diagnostics read persisted memory directly.

The host composes component requests into one immutable
`SealedDeclarationSnapshot` and supplies it explicitly to each runtime. No global
registry collects declarations. Each runtime performs its own recovery, policy
evaluation, persistence and capability publication on its backing memory.

Exactly one host owns and bootstraps a `MemoryRuntime` for a given backing
memory. That host retains the instance and passes runtime references or opened
`RuntimeMemory` handles to components. Canic and application code own their
canister/thread storage; the Memory library has no singleton, implicit
construction, cached construction failure or TLS lifecycle.

The public API is exported from `ic_memory::...` at the crate root. Use
`MemoryRuntime::bootstrap`, `MemoryRuntime::doctor_report` and
`MemoryRuntime::open_memory`. Implementation modules remain private.

Separate allocation domains require distinct backing memories and runtime
objects, with an explicit bootstrap owner for each domain.

Bootstrap is once per runtime object, not once per process. A second call on the
same successfully bootstrapped runtime is idempotent and does not advance the
ledger generation only when the sealed snapshot, host allocation pool and
`RuntimeBootstrapPolicy::runtime_bootstrap_identity()` match the established
bootstrap binding. Independently sealed snapshots with equal canonical contents
also match. A changed snapshot, pool or policy identity returns a typed error
without evaluating policy or touching the ledger. A different runtime always
inspects its own ledger memory. No public reset API is provided; constructing a
new runtime is the correct way to own a new backing memory.

## Policy Authority

The host owns one `MemoryAllocationPool`. Governance IDs 0..=9 are always
excluded. Each `MemoryAuthority` admits a named owner under a permanent key
prefix ending in a dot. Namespace grants must be disjoint; several disjoint
namespaces can share an owner. Additional physical exclusions retain custody
for unmanaged users. Namespace grants do not reserve numeric subranges.

The runtime checks current namespace ownership and physical eligibility before
applying custom `AllocationPolicy`. New keys receive the lowest unused pool ID
in canonical key order. Known keys retain their IDs; changed eligibility causes
rejection, never relocation. Every retained ledger state occupies its ID.
A populated pool ID without a ledger record causes `UnmanagedAllocation`;
explicitly exclude unmanaged custody instead of assigning it inferred ownership.

Owner labels are current host policy. The ledger retains key-to-ID identities,
not previous owner labels. The host can change a namespace's owner at a cold
bootstrap while preserving its keys and bytes. This is neither authentication
of callers nor isolation of linked Rust code.

Custom runtime policies implement `RuntimeBootstrapPolicy`. `PolicyIdentity`
names their family and semantic version, optionally with a caller-computed
32-byte configuration digest. Change this identity when effective custom rules
change. The runtime binds successful bootstrap to that identity, the immutable
request snapshot and the exact canonical pool. This binding is transient;
it is not durable upgrade history.

Consumers contribute preparation through the host's single `prepare_bootstrap`
hook. Warm consumers verify their requirements with `verify_authority` and then
open committed keys. Verification checks keys and current owner labels without rerunning admission or choosing geometry. See the
[composed-host example](crates/ic-memory/examples/composed_host.rs) and
[admission contract](docs/recovered-admission.md).

## Explicit Component Declarations

Components provide complete `Vec<MemoryRequest>` factories using the same stable
keys as their stores. The host combines the selected lists before bootstrap:

```rust,no_run
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let requests = vec![ic_memory::MemoryRequest::new(
    "icydb.test_db", "icydb.test_db.orders.data.v1")?];
let snapshot = ic_memory::SealedDeclarationSnapshot::new(requests)?;
# Ok(())
# }
```

Pass that snapshot and the checked pool to the chosen runtime. Store opening
never registers a declaration, invokes a constructor hook or selects a numeric
ID. All selected component requirements must be present before opening storage.

`snapshot.requests()` exposes canonical source requests. Resolution feeds the
checked declaration snapshot to commitment; current placement projections
borrow the resulting capability. There is no second resolved-row map or
alternative component input. Governance is validated internally before
application capability publication.

## Runtime Diagnostics

`MemoryRuntime::doctor_report(&snapshot, &pool, &policy)` reports stable-cell
status, protected recovery, recovered ledger, live sizes, source requests and
the tested pool. Read-only validation checks custody, namespace ownership,
placement and custom allocation policy. It never executes preparation, predicts
its completed historical selections or certifies consumer admission.

For prebootstrap inspection, construct an owned runtime with explicit backing
and configuration. Reports compare exact canonical requests, pool and policy
identity with the successful bootstrap binding. They do not use a fingerprint.
Typed diagnostic outcomes include
stable codes and messages. Invalid persisted records fail recovery and are not
exported as valid ledgers. Physical allocation reports do not decode ownership;
`pool_eligible` describes policy membership, not free space or historical custody.

## Explicit `MemoryRuntime<M>`

```rust,no_run
use ic_memory::{MemoryAllocationPool, MemoryAuthority, MemoryRequest,
    MemoryRuntime, SealedDeclarationSnapshot, GenericAllocationPolicy,
    ic_stable_structures::VectorMemory};
let source = SealedDeclarationSnapshot::new(vec![
    MemoryRequest::new("app", "app.users.v1")?,
])?;
let pool = MemoryAllocationPool::new(vec![MemoryAuthority::new("app", "app.")?], vec![])?;
let mut runtime = MemoryRuntime::new(VectorMemory::default())?;
runtime.bootstrap(&source, &pool, &GenericAllocationPolicy)?;
let users = runtime.open_memory("app.users.v1")?;
let doctor = runtime.doctor_report(&source, &pool, &GenericAllocationPolicy);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Construction accepts empty memory or the supported manager layout. Foreign or
unsupported nonempty backing is rejected without writes. Fresh construction
reserves the metadata page before writing a header; growth refusal returns a
typed error and leaves backing retryable. Geometry mismatches reject before
admission or persistence. The host handles construction errors explicitly.

The runtime owns its manager and committed capability. Opens consult only that
runtime's capability; they cannot accept another runtime's authority. Capability
publication follows successful persistence. One bootstrap owner is required for
each concrete backing memory.

## Recovery and Persistence

Frameworks own backing memory and application lifecycle; `MemoryRuntime` owns
ledger recovery, validation and persistence on that memory. Its internal commit
order is fixed:

```text
recover persisted allocation ledger
admit host policy and resolve the explicit requests
validate declarations against ledger ownership and policy
stage and persist the new generation
publish committed authority, then open stable-memory handles
```

There is no public manual commit writer or persistence-confirmation API. Use an
owned runtime with explicit backing, configuration, complete request snapshot
and checked pool.

Maintained recovery paths enforce byte, collection and nesting limits before the
relevant allocations and decoding. See the
[current recovery limits](docs/key-only-recovery.md#recovery-and-admission-limits).
Every cold bootstrap attempt decodes its stable-cell record once and uses that
record for recovery and staging. Empty memory is initialized before admission;
corrupt cell envelopes or record bytes produce typed failures. Capacity admission
counts encoded bytes without allocating another serialization buffer.

The internal commit store checks present physical slots, checksums, current
payload format, physical/logical generation binding and ledger integrity before
returning `RecoveredLedger`. Decoded DTOs alone never grant open authority.
Opaque generation payloads are bounded CBOR byte strings; oversized commits and
unknown current-format fields are rejected rather than silently accepted.

`AllocationLedger::new(counter, records)` checks count, unique ownership and
empty-genesis rules. It returns a passive DTO, not a persistence owner. Inspect
`AllocationLedger::records()` and each record's key, slot and state for current ownership.
See the [current ledger contract](docs/current-ledger.md) for retained-data rules.

Internal pre-commit validation state cannot open storage. `CommittedAllocations`
is the non-serializable open capability published by the runtime only after its
stable-cell write succeeds. Diagnostic exports and durable records are not open
capabilities.

## Stable Key Rules

Stable keys are permanent logical store names. They should describe ownership
and purpose, not the current memory ID.

Format:

```text
namespace.component.store_or_role.vN
```

Rules:

- ASCII only.
- Lowercase only.
- Dot-separated segments.
- Each segment starts with a lowercase letter.
- Segments may contain lowercase letters, digits, and underscores.
- No whitespace, slashes, or hyphens.
- Must end with a nonzero version suffix such as `.v1` or `.v12`.
- Maximum length is 128 bytes.

Suggested namespace conventions:

- `ic_memory.*` is reserved for `ic-memory` governance records.
- Application-owned stores can use an application namespace, such as
  `app.orders.v1` or `myapp.audit_log.v1`.
- Frameworks and generated stores should use namespaces they own, such as
  `framework.cache.index.v1` or `database.users.data.v1`.

Canic and IcyDB examples:

- `canic.core.*` is appropriate for Canic framework-owned stores.
- `icydb.<memory_namespace>.<store_name>.<role>.vN` works for generated IcyDB
  stores, such as `icydb.test_db.users.data.v1`.

Changing a key creates a new logical allocation identity. If the durable store
is the same, keep the stable key and apply schema changes through its storage owner.

## Host Allocation Pool

Only the host declares physical exclusions. This reserves IDs 200..=210 for an
unmanaged client while both components share every other eligible ID:

```rust
use ic_memory::{MemoryAllocationPool, MemoryAuthority};
let pool = MemoryAllocationPool::new(
    vec![MemoryAuthority::new("framework", "framework.").unwrap(),
         MemoryAuthority::new("db", "db.").unwrap()],
    (200..=210).collect(),
).unwrap();
assert!(pool.contains(10));
assert!(!pool.contains(0));
assert!(!pool.contains(205));
```

Pool membership does not imply an ID is unused. Durable records independently
retain all assigned IDs. Pool construction and decoding canonicalize exclusions
and reject invalid or overlapping namespace grants. They grant no open capability.

## Current MemoryManager Rules

For the checked `MemoryManagerSlot` allocation identity:

- IDs `0..=254` are usable stable-memory slots.
- ID `255` is rejected because it is the unallocated sentinel.
- `MemoryManagerSlot::new(id)` and deserialization check this bound;
  `.id()` returns a usable ID without another validation step.
- IDs `0..=9` are reserved for `ic-memory` governance.
- ID `0` is assigned to the allocation ledger.
- Stable keys under `ic_memory.*` are reserved for `ic-memory` governance and
  cannot be opened through the public runtime.

## What It Does Not Do

`ic-memory` does not replace `ic-stable-structures`.

It owns allocation governance and re-exports its exact substrate dependency as
`ic_memory::ic_stable_structures`. Downstream code can import collections such
as `StableBTreeMap`, backing memories, `Memory`, and `Storable` through that
namespace without a separate dependency. These are the upstream types, used
with `RuntimeMemory<M>` handles from the owned runtime. Re-exporting the
collections does not wrap their behavior or change manager ownership.

It also does not handle:

- schema migrations
- schema compatibility or data semantics
- controller authorization
- application data validation
- endpoint routing
- IC management-canister calls
- malicious-controller protection
- disaster recovery

It only protects stable-memory allocation ownership.

## Status

`ic-memory` is early infrastructure extracted from Canic. The public API is
intended to stabilize around persistent allocation ownership, but framework
authors should still treat this line as young infrastructure while the
standalone boundary settles.

The two commit slots are serialized together inside the stable cell and rely on
ICP message execution for atomic commit and rollback. If the enclosing record
remains decodable, their non-cryptographic checksums detect accidental slot
corruption. Any present invalid slot fails closed instead of falling back to an
older generation. The checksums do not provide adversarial tamper resistance.
