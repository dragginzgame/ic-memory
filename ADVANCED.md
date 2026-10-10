<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-memory/ic-memory-readme-header.svg" alt="IC Memory — Stops upgrades from mixing up stored data" width="100%">
</p>

# Advanced ic-memory

This document covers the lower-level pieces behind the macro runtime. Most
applications should start with the README.

## Contents

- [How it fits](#how-it-fits)
- [Runtime ownership](#runtime-ownership)
- [Policy authority](#policy-authority)
- [Declaration-only hooks](#declaration-only-hooks)
- [Default runtime diagnostics](#default-runtime-diagnostics)
- [Explicit `MemoryRuntime<M>`](#explicit-memoryruntimem)
- [Manual bootstrap](#manual-bootstrap)
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
  -> ic-stable-structures::Cell<StableCellLedgerRecord, _>
  -> LedgerCommitStore
  -> redundant committed generation bytes
  -> LedgerPayloadEnvelope
  -> RecoveredLedger
  -> ValidatedAllocations
  -> PendingBootstrapCommit
  -> CommittedAllocations
```

The logical payload inside the `LedgerPayloadEnvelope` is the built-in
`ic-memory` CBOR ledger format. Callers do not provide a custom codec.

The default runtime keeps this internal ledger allocation in its ownership records,
but it removes `ic_memory.*` governance keys from the committed allocations it
publishes for application opens. Public default-runtime open helpers reject
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

Linked crates compose declarations into one immutable
`SealedDeclarationSnapshot`. That process-global snapshot is declaration
authority, not a process-global bootstrap result. Every concrete runtime
receives the same declaration meaning but performs its own recovery, policy
evaluation, persistence, and capability publication.

Exactly one owner should bootstrap a given ledger store. Canic can own a
`MemoryRuntime`, IcyDB can own one, or the application can use the default TLS
runtime. Framework integrations should store and pass the explicit runtime
object alongside the backing memory they own.

The intended public API is exported from `ic_memory::...` at the crate root.
Implementation modules such as the runtime, ledger, registry, and validation
modules are private. Frameworks should call root exports such as
`bootstrap_default_memory_manager_with_policy(...)`,
`default_memory_manager_doctor_report(&pool)`, and
`open_default_memory_manager_memory(...)`.

If multiple layers need separate allocation domains, they should use distinct
backing memories and runtime objects with an explicit bootstrap owner for each
domain.

The default convenience layer lazily constructs one runtime per thread. Its TLS
storage distinguishes an absent runtime, a successfully constructed runtime,
and a cached construction failure. Bootstrap can construct the runtime;
observations and memory opens never construct an absent runtime or choose its
bucket configuration. A construction failure remains cached for that thread.
Native threads get independent default memory instances and therefore
independent runtime lifecycle and authority. IC Wasm execution is single-threaded,
so that TLS runtime naturally has canister-instance lifetime. Default entry
points use fallible TLS borrowing and return a typed reentrancy error instead of
panicking or consulting another runtime.

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
open committed keys. Verification checks keys, current owner labels and schema
metadata without rerunning admission or choosing geometry. See the
[composed-host example](crates/ic-memory/examples/composed_host.rs) and
[admission contract](docs/recovered-admission.md).

## Declaration-Only Hooks

Use `eager_init!` to register metadata before sealing without opening storage:

```rust,no_run
ic_memory::eager_init!({
    ic_memory::register_memory_request(ic_memory::MemoryRequest::new(
        "icydb.test_db", "icydb.test_db.orders.data.v1",
        ic_memory::SchemaMetadata::default(),
    ).expect("valid request")).expect("registration remains open");
});
```

Configured bootstrap checks geometry before sealing and releases the runtime
borrow while hooks run. Hooks may observe readiness and physical totals before
bootstrap. `ic_memory_key!` returns a typed open result and requires committed
authority. Macros register keys and explicitly named owners, never numeric IDs.

`sealed_declaration_snapshot().requests()` exposes canonical source requests.
A host passes that snapshot and its pool to its runtime. Resolution feeds the existing checked declaration snapshot to commitment;
current placement/schema projections borrow the resulting capability. There is
no second resolved-row map or alternative component input.
Governance is validated internally before application capability publication.

## Default Runtime Diagnostics

`MemoryRuntime::doctor_report(&snapshot, &pool, &policy)` reports stable-cell
status, protected recovery, recovered ledger, live sizes, source requests and
the tested pool. Read-only validation checks custody, namespace ownership,
placement and custom allocation policy. It never executes preparation, predicts
its completed historical selections or certifies consumer admission.

`default_memory_manager_doctor_report(&pool)` observes an existing default
runtime; the custom variant takes `(&pool, &policy)`. An absent runtime returns
`NotBootstrapped` before sealing or initializing storage. Only bootstrap creates
the default runtime and selects its geometry. For prebootstrap inspection,
construct an owned runtime with explicit backing and configuration.

Reports compare the tested policy identity, snapshot fingerprint and pool with
the successful bootstrap binding. The fingerprint is non-cryptographic transient
diagnostic metadata, not durable authority. Typed diagnostic outcomes include
stable codes and messages. Invalid persisted records fail recovery and are not
exported as valid ledgers. Physical allocation reports do not decode ownership;
`pool_eligible` describes policy membership, not free space or historical custody.

## Explicit `MemoryRuntime<M>`

```rust,no_run
use ic_memory::{MemoryAllocationPool, MemoryAuthority, MemoryRequest,
    MemoryRuntime, SchemaMetadata, SealedDeclarationSnapshot, GenericAllocationPolicy,
    ic_stable_structures::VectorMemory};
let source = SealedDeclarationSnapshot::new(&[
    MemoryRequest::new("app", "app.users.v1", SchemaMetadata::default())?,
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
sealing. The default runtime caches construction failures per native thread.

The runtime owns its manager and committed capability. Opens consult only that
runtime's capability; they cannot accept another runtime's authority. Capability
publication follows successful persistence. One bootstrap owner is required for
each concrete backing memory, including each native default-runtime thread.

## Manual Bootstrap

The macro runtime is built on the lower-level ledger API. Frameworks that need
to own stable-memory IO or endpoint lifecycle can still drive that API directly.

The safe order is fixed:

```text
recover persisted allocation ledger
declare this binary's expected stable stores
validate declarations against ledger ownership and policy
commit the new generation
only then open stable-memory handles
```

This lower-level path accepts resolved key-to-ID declarations. Logical requests and
recovered-metadata admission belong to `MemoryRuntime::bootstrap`, which adds
preparation and resolution before the same validation/persistence boundary.
Use the runtime when composing those features; a diagnostic export cannot
resolve requests or authorize historical opens.

Maintained recovery paths enforce byte, collection and nesting limits
before the relevant allocations and decoding. See the
[current recovery limits](docs/key-only-recovery.md#recovery-and-admission-limits).

For a manually persisted stable-cell ledger, call
`decode_stable_cell_ledger_record_from_memory(&ledger_memory)` once and retain
the returned record for recovery and bootstrap. Empty memory returns an
uninitialized record without writing; corrupt envelopes or record bytes return
typed errors. The decoded record still needs protected ledger recovery. This
avoids a separate preflight followed by another panic-based decode through
`Cell::init`. Persistence remains the manual owner's responsibility.

Opaque generation payloads use bounded CBOR byte strings, introduced in 0.14.3;
the current decoder rejects the superseded integer-array representation.

Low-level `DualCommitStore` commits enforce the same opaque-payload byte ceiling
before changing either slot and return `CommitRecoveryError::PayloadTooLarge`
on refusal. Logical ledger decoding and integrity validation remain the
responsibility of `LedgerCommitStore`. Current durable retirement records also
reject unknown fields instead of discarding them.

Successful logical commits return recovery evidence from the checked ledger and
the completed physical commit. Reading existing persisted bytes still performs
the full checksum, format and integrity recovery path. Runtime capacity admission
counts encoded record bytes without allocating a temporary serialization buffer;
the stable cell then serializes the record for persistence.

Decoded ledger and declaration DTOs are not trusted just because serde accepted
them. Recovery first validates every present physical commit slot and selects
the authoritative generation, verifies the logical payload's current format
marker and version, decodes the current-format `ic-memory` CBOR ledger payload,
checks the physical/logical generation binding, and validates committed ledger
integrity. Only the resulting `RecoveredLedger` proof can be passed to
declaration validation to produce pre-commit `ValidatedAllocations`.

Manual sketch:

```rust,ignore
let declarations = DeclarationSnapshot::new(vec![
    AllocationDeclaration::memory_manager("app.orders.v1", 100, "orders")?,
])?;

let commit = AllocationBootstrap::new(record.store_mut()).initialize_validate_and_commit(
    &genesis_ledger,
    declarations,
    &policy,
)?;

persist_record(&record)?;
let committed = commit.confirm_persisted();
let key = StableKey::parse("app.orders.v1")?;
let slot = committed.slot_for(&key).ok_or("allocation was not committed")?;
let orders = open_storage(slot)?;
```

The helper names for `record`, `persist_record`, `genesis_ledger`, `policy`,
and `open_storage` are placeholders. Frameworks and libraries
wire those to their own stable-memory persistence and collection construction.
The ordering is the contract. Calling `confirm_persisted()` before
`persist_record` succeeds violates the protocol.

Supplying `genesis_ledger` is privileged. Normal empty-store bootstraps should
use an empty current-format ledger, like the default runtime does. A non-empty
genesis ledger is an import or migration decision owned by the layer that owns
the ledger store.

`AllocationLedger::new(counter, records)` checks count, unique ownership and
empty-genesis rules. It returns a passive DTO; protected recovery is still needed
before declaration validation. There is one integrity validator for the current
format, and no retained generation or schema trail. Inspect
`AllocationLedger::records()` and `AllocationRecord::schema()` for current state.
See the [current ledger hard cut](docs/current-ledger.md) before upgrading retained
installations.

`ValidatedAllocations` is intentionally opaque and non-serializable pre-commit
state. It can be staged but cannot open storage. `CommittedAllocations` is the
separate non-serializable open capability produced only after persistence is
confirmed. Neither is a durable record or diagnostic export format.

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
is the same, keep the stable key and update schema metadata instead.

## Host Allocation Pool

Only the host declares physical exclusions. This reserves IDs 200..=210 for an
unmanaged client while both components share every other eligible ID:

```rust
use ic_memory::{MemoryAllocationPool, MemoryAuthority, MemoryManagerIdRange};
let pool = MemoryAllocationPool::new(
    vec![MemoryAuthority::new("framework", "framework.").unwrap(),
         MemoryAuthority::new("db", "db.").unwrap()],
    vec![MemoryManagerIdRange::new(200, 210).unwrap()],
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
  cannot be opened through the public default runtime.

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
