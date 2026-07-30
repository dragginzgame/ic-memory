# Advanced ic-memory

This document covers the lower-level pieces behind the macro runtime. Most
applications should start with the README.

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

The default runtime keeps this internal ledger allocation in durable history,
but it removes `ic_memory.*` governance keys from the committed allocations it
publishes for application opens. Public default-runtime open helpers reject
those reserved keys.

A typical framework flow is:

1. Recover the saved allocation ledger into `RecoveredLedger`.
2. Declare the stores this binary expects.
3. Validate those declarations against history and policy.
4. Commit the new generation.
5. Open stable-memory handles only after commit persistence succeeds.

The important rule: validate layout before touching stable data.

The default runtime also preflights the ledger stable-cell before opening it
through `ic-stable-structures::Cell`. Corrupt cell envelopes or ledger-record
bytes are reported as bootstrap errors instead of relying on panic behavior
inside `Cell::init`.

## Runtime Ownership

`MemoryRuntime<M>` is the canonical owner for one backing memory instance. It
owns the `MemoryManager<M>`, allocation-ledger cell, bootstrap lifecycle,
committed allocation capability, memory opens, recovery diagnostics, and live
memory-size inspection.

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
`default_memory_manager_doctor_report()`, and
`open_default_memory_manager_memory(...)`.

If multiple layers need separate allocation domains, they should use distinct
backing memories and runtime objects with an explicit bootstrap owner for each
domain.

The default convenience layer is one
`thread_local! RefCell<MemoryRuntime<DefaultMemoryImpl>>`. Native threads get
independent default memory instances and therefore independent runtime
lifecycle and authority. IC Wasm execution is single-threaded, so that TLS
runtime naturally has canister-instance lifetime. Default entry points use
fallible TLS borrowing and return a typed reentrancy error instead of panicking
or consulting another runtime.

Bootstrap is once per runtime object, not once per process. A second call on the
same successfully bootstrapped runtime is idempotent and does not advance the
ledger generation only when the sealed snapshot and
`RuntimeBootstrapPolicy::runtime_bootstrap_identity()` match the established
bootstrap binding. A changed snapshot or policy identity returns a typed error
without evaluating policy or touching the ledger. A different runtime always
inspects its own ledger memory. No public reset API is provided; constructing a
new runtime is the correct way to own a new backing memory.

## Policy Authority

There is one authority order in the default runtime:

1. `ic-memory` always owns its governance range.
2. Registered `ic_memory_range!` claims are authoritative generic range policy.
3. The caller-supplied `AllocationPolicy` is applied after generic range checks.

Policies passed to runtime bootstrap also implement `RuntimeBootstrapPolicy`.
Its identity names the policy configuration and semantics, not the policy
object's address or Rust type. Frameworks must change that identity when their
effective rules change.

That means a framework adapter must choose deliberately which layer owns range
decisions.

If a package registers a user range, `ic-memory` enforces that the package's
declarations stay inside that range. If any user range is registered, all user
`MemoryManager` declarations are checked against registered range ownership.
This is the standalone multi-crate composition mode.

If a framework such as Canic wants its own policy to decide application space,
it should not register `ic_memory_range!` claims for that application space.
It can still use `bootstrap_default_memory_manager_with_policy(...)` and reject
keys or slots in its `AllocationPolicy`.

Canic-specific namespace and framework range rules are Canic policy. They are
not hard-coded `ic-memory` rules. Canic should adapt to `ic-memory` by either:

- registering the framework and package ranges it wants `ic-memory` to enforce;
- or leaving application ranges unclaimed and enforcing those rules in Canic's
  policy adapter.

## Declaration-Only Hooks

Use `eager_init!` when a crate needs to register declarations before bootstrap
without opening a TLS stable structure:

```rust,ignore
ic_memory::eager_init!({
    ic_memory::register_static_memory_manager_declaration(
        121,
        "icydb.test_db",
        "OrdersDataStore",
        "icydb.test_db.orders.data.v1",
    )
    .expect("valid ic-memory declaration");
});
```

Hooks registered with `eager_init!` run before the declaration snapshot is
sealed. Stable structures opened with `ic_memory_key!` require committed
allocations to be published first, and the macro returns the typed open result
so the integration chooses how to handle failure. Macro range and key
declarations require an explicit stable `authority` string; it is policy
identity and must not be derived implicitly from package metadata.

Frameworks or libraries that need custom policy metadata should call
`sealed_declaration_snapshot()` and inspect its canonical
`registered_declarations()` and `registered_ranges()`. They can pass the same
snapshot to an explicit `MemoryRuntime<M>` or bootstrap the default runtime with
`bootstrap_default_memory_manager_with_policy(...)`. The custom policy receives
external declarations only; ic-memory validates its private ledger declaration
internally.

## Default Runtime Diagnostics

`MemoryRuntime::doctor_report(&snapshot)` builds a serializable report for that
runtime before or after bootstrap. The default
`default_memory_manager_doctor_report()` entry point seals the linked snapshot,
enters the calling thread's runtime fallibly, and returns the same report. It
includes stable-cell status, protected commit recovery, recovered ledger
export, registered declarations, registered and effective range authority,
generic validation preflight, and live memory sizes for recovered ledger
records.

Mutually exclusive diagnostic outcomes use enums or `Result` values instead of
nullable field pairs, so machine-readable reports cannot express contradictory
success and failure states. Failures include a stable `DiagnosticCode` beside
the human-readable message, so automation does not need to parse prose.

The first snapshot request runs deferred generated registration and
`eager_init!` hooks exactly once before sealing, so doctor and bootstrap always
use the same immutable declaration set. The validation field covers generic
range/declaration checks. Frameworks that pass a custom policy should still
diagnose that policy in their own adapter layer.

## Explicit `MemoryRuntime<M>`

The explicit runtime requires only `M: ic_stable_structures::Memory`:

```rust,ignore
let declarations = ic_memory::sealed_declaration_snapshot()?;
let mut runtime = ic_memory::MemoryRuntime::new(backing_memory)?;
runtime.bootstrap(&declarations, &policy)?;

let users = runtime.open_memory("app.users.v1", 120)?;
let export = runtime.diagnostic_export()?;
let recovery = runtime.commit_recovery_diagnostic()?;
let doctor = runtime.doctor_report(&declarations);
```

Runtime construction accepts empty backing memory or the current
`ic-stable-structures` `MemoryManager` layout. It returns
`RuntimeConstructionError` before initialization when nonempty memory has
foreign magic or an unsupported manager version, leaving rejected bytes
unchanged.

`committed` borrows the capability stored under `runtime`. Opening memory never
accepts a capability from another runtime; it consults the capability and
`MemoryManager` owned by the same object. Capability publication happens only
after the stable-cell record write succeeds.

## Manual Bootstrap

The macro runtime is built on the lower-level ledger API. Frameworks that need
to own stable-memory IO or endpoint lifecycle can still drive that API directly.

The safe order is fixed:

```text
recover persisted allocation ledger
declare this binary's expected stable stores
validate declarations against ledger/history/policy
commit the new generation
only then open stable-memory handles
```

Decoded ledger and declaration DTOs are not trusted just because serde accepted
them. Recovery first validates every present physical commit slot and selects
the authoritative generation, verifies the logical payload's current format
marker and version, decodes the current-format `ic-memory` CBOR ledger payload,
checks the physical/logical generation binding, and validates committed ledger
integrity. Only the resulting `RecoveredLedger` proof can be passed to
declaration validation to produce pre-commit `ValidatedAllocations`.

Manual sketch:

```rust,ignore
let declarations = DeclarationCollector::new()
    .with_memory_manager("app.orders.v1", 100, "orders")?
    .seal()?;

let commit = AllocationBootstrap::new(record.store_mut()).initialize_validate_and_commit(
    &genesis_ledger,
    declarations,
    &policy,
    committed_at,
)?;

persist_record(&record)?;
let committed = commit.confirm_persisted();
let key = StableKey::parse("app.orders.v1")?;
let slot = committed.slot_for(&key).ok_or("allocation was not committed")?;
let orders = open_storage(slot)?;
```

The helper names for `record`, `persist_record`, `genesis_ledger`, `policy`,
`committed_at`, and `open_storage` are placeholders. Frameworks and libraries
wire those to their own stable-memory persistence and collection construction.
The ordering is the contract. Calling `confirm_persisted()` before
`persist_record` succeeds violates the protocol.

Supplying `genesis_ledger` is privileged. Normal empty-store bootstraps should
use an empty current-format ledger, like the default runtime does. A non-empty
genesis ledger is an import or migration decision owned by the layer that owns
the ledger store.

`AllocationLedger::new(...)` builds a structurally valid ledger DTO. Use
`AllocationLedger::new_committed(...)` only when you are manually constructing
committed ledger state and want the stricter committed-generation checks.
Normal integrations should usually recover through the commit/recovery flow
instead of hand-assembling committed state.

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

## Range Authority

Range authority is policy metadata. It does not allocate stable-memory IDs and
does not write to the allocation ledger. In the default runtime, however,
registered range authority is enforced before the caller-supplied policy, as
described in [Policy Authority](#policy-authority).

Packages should publish only the ranges they own:

```rust
use ic_memory::{
    IC_MEMORY_AUTHORITY_OWNER, MemoryManagerRangeAuthority, MemoryManagerRangeMode,
    memory_manager_governance_range,
};

let authority = MemoryManagerRangeAuthority::new()
    .reserve(memory_manager_governance_range(), IC_MEMORY_AUTHORITY_OWNER)
    .expect("ic-memory governance range")
    .reserve_ids(10, 99, "framework.example")
    .expect("framework range");

authority
    .validate_id_authority_mode(42, "framework.example", MemoryManagerRangeMode::Reserved)
    .expect("framework-owned ID");
```

An open stack composes records from multiple packages and rejects overlaps:

```rust
use ic_memory::MemoryManagerRangeAuthority;

let framework_records = MemoryManagerRangeAuthority::new()
    .reserve_ids(10, 99, "framework.example")
    .expect("framework range")
    .authorities()
    .to_vec();

let database_records = MemoryManagerRangeAuthority::new()
    .reserve_ids(120, 149, "database.framework")
    .expect("database range")
    .authorities()
    .to_vec();

let authority = MemoryManagerRangeAuthority::from_records(
    framework_records
        .into_iter()
        .chain(database_records)
        .collect(),
)
.expect("non-overlapping package ranges");

assert_eq!(authority.authorities().len(), 2);
```

A final closed policy may claim the remaining application space and require full
coverage:

```rust
use ic_memory::{
    IC_MEMORY_AUTHORITY_OWNER, MEMORY_MANAGER_MAX_ID, MemoryManagerIdRange,
    MemoryManagerRangeAuthority, memory_manager_governance_range,
};

let authority = MemoryManagerRangeAuthority::new()
    .reserve(memory_manager_governance_range(), IC_MEMORY_AUTHORITY_OWNER)
    .expect("ic-memory governance range")
    .reserve_ids(10, 99, "framework.example")
    .expect("framework range")
    .allow_ids(100, MEMORY_MANAGER_MAX_ID, "applications")
    .expect("application range");

authority
    .validate_complete_coverage(MemoryManagerIdRange::all_usable())
    .expect("closed policy covers every usable ID");
```

## Current MemoryManager Rules

For the built-in `ic-stable-structures::MemoryManager` slot descriptor:

- IDs `0..=254` are usable stable-memory slots.
- ID `255` is rejected because it is the unallocated sentinel.
- IDs `0..=9` are reserved for `ic-memory` governance.
- ID `0` is assigned to the allocation ledger.
- Stable keys under `ic_memory.*` are reserved for `ic-memory` governance and
  cannot be opened through the public default runtime.

The crate also exposes range-authority helpers for frameworks that want to split
ID ranges between infrastructure and application stores.

Canic can reserve framework ranges such as `10..=99` through its adapter. That
kind of range is Canic policy, not an `ic-memory` rule.

## What It Does Not Do

`ic-memory` does not replace `ic-stable-structures`.

It owns allocation governance. Downstream code imports `ic-stable-structures`
directly; `ic-memory` does not re-export or wrap collection types such as
`StableBTreeMap`.

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
