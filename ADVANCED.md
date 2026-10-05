<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-memory/ic-memory-readme-header.svg" alt="IC Memory — Internet Computer helper library" width="100%">
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
- [Range authority](#range-authority)
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

The default runtime keeps this internal ledger allocation in durable history,
but it removes `ic_memory.*` governance keys from the committed allocations it
publishes for application opens. Public default-runtime open helpers reject
those reserved keys.

A framework supplies sealed fixed declarations, logical requests and host grants
to one runtime. A cold bootstrap then:

1. Recovers the saved allocation ledger into `RecoveredLedger`.
2. Runs the host's `prepare_bootstrap` hook to admit consumer identity and select
   authorized historical keys from bounded allocation metadata.
3. Resolves logical requests while preserving existing assignments.
4. Validates the completed declarations against history and current policy.
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
`default_memory_manager_doctor_report()`, and
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
ledger generation only when the sealed snapshot and
`RuntimeBootstrapPolicy::runtime_bootstrap_identity()` match the established
bootstrap binding. Independently sealed snapshots with equal canonical contents
also match. A changed snapshot or policy identity returns a typed error
without evaluating policy or touching the ledger. A different runtime always
inspects its own ledger memory. No public reset API is provided; constructing a
new runtime is the correct way to own a new backing memory.

## Policy Authority

There is one authority order in the default runtime:

1. `ic-memory` always owns its governance range.
2. Registered `ic_memory_range!` claims are authoritative generic range policy.
3. The caller-supplied `AllocationPolicy` is applied after generic range checks.

Policies passed to runtime bootstrap also implement `RuntimeBootstrapPolicy`.
Its `PolicyIdentity` names the policy family and semantic version, not the
policy object's address or Rust type. Configuration-dependent policies should
attach a caller-computed 32-byte digest of their effective configuration.
Frameworks must change the version or digest when their effective rules change.
Names are validated printable ASCII and bounded to 256 bytes.

The policy identity is an in-memory repeat-call and doctor-report binding only.
It is not written to the allocation ledger and therefore is not upgrade audit
history. An integration that needs durable policy history must make a separate
explicit persisted-format decision rather than infer it from this runtime
binding.

```rust,ignore
impl RuntimeBootstrapPolicy for FrameworkPolicy {
    fn runtime_bootstrap_identity(
        &self,
    ) -> Result<PolicyIdentity, PolicyIdentityError> {
        PolicyIdentity::new("framework.memory-bootstrap-policy", 2).map(|identity| {
            identity.with_configuration_digest(self.effective_configuration_digest())
        })
    }
}
```

That means a framework adapter must choose deliberately which layer owns range
decisions.

If a package registers a user range, `ic-memory` enforces that the package's
declarations stay inside that range. If any user range is registered, all user
`MemoryManager` declarations are checked against registered range ownership.
This is the standalone multi-crate composition mode.

A framework can omit all registered user ranges and enforce fixed application
claims through `bootstrap_default_memory_manager_with_policy(...)` and its
`AllocationPolicy`. Logical placement and historical selection always
require explicit registered host grants; a custom policy alone cannot supply
their eligible pool. `Allowed` ranges supply fresh automatic placements;
`Reserved` ranges permit matching existing or fixed claims without supplying
new automatic slots.

Omitting `mode` from `ic_memory_range!` selects `Reserved`. Hosts admitting new
logical requests must pass `mode = Allowed`. If no free ID exists in a matching
`Allowed` range, resolution returns `MemoryResolutionError::Exhausted`; free IDs
in reserved ranges do not satisfy that request.

Canic-specific namespace and framework range rules are Canic policy. They are
not hard-coded `ic-memory` rules. Canic should adapt to `ic-memory` by either:

- registering the framework and package ranges it wants `ic-memory` to enforce;
- or leaving all user ranges unclaimed for fixed-only allocations and
  enforcing those rules in Canic's policy adapter.

Consumers contribute their preparation to the host's single
`RuntimeBootstrapPolicy::prepare_bootstrap` hook. Warm consumers call
`verify_authority(&requirements, authority)` or
`verify_default_memory_manager_authority(...)`, then open committed keys. They
do not bootstrap again with their own policy. Verification checks fixed IDs,
logical keys, current authority and diagnostic metadata without replaying
admission or changing the host's geometry. See the
[composed-host example](examples/composed_host.rs) and
[recovered-admission contract](docs/recovered-admission.md).

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
sealed. Configured bootstrap checks its bucket geometry before sealing and
releases the runtime borrow while hooks run. Hooks may observe readiness and
physical totals on that unbootstrapped runtime.
Stable structures opened with `ic_memory_key!` require committed
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

`MemoryRuntime::doctor_report(&snapshot, &policy)` builds a serializable report
for that runtime before or after bootstrap and runs validation through the
supplied policy. The default
`default_memory_manager_doctor_report()` entry point observes an existing runtime,
seals the linked snapshot, and evaluates the built-in policy. If no runtime exists,
it returns `RuntimeDiagnosticError::NotBootstrapped` before sealing declarations
or initializing memory. Default export and commit-recovery diagnostics also
leave an absent runtime untouched. Only bootstrap constructs the default runtime
and selects its bucket configuration. For prebootstrap diagnostics with explicit
configuration, construct an owned `MemoryRuntime::new_with_config(memory, config)`
and call its recovery or doctor methods.
Custom-policy default runtimes should use
`default_memory_manager_doctor_report_with_policy(&policy)`. Reports include
stable-cell status, protected commit recovery, recovered ledger export,
registered declarations, registered and effective range authority, validation
under the tested policy, and live memory sizes for recovered ledger records.
Effective range authority is a validated table from the sealed snapshot.
Registration and sealing failures return typed errors before a report is built.
Doctor validation covers the supplied declaration set and allocation policy;
it does not run `prepare_bootstrap`, predict historical completion or certify
consumer admission. Use the bounded allocation summary/report when metrics
need physical accounting without decoding ledger history.

The report identifies the tested policy and declaration-snapshot fingerprint,
shows the binding established by successful bootstrap, and reports whether the
two match. The snapshot fingerprint is deterministic, versioned,
non-cryptographic diagnostic metadata; it is neither durable allocation
authority nor an adversarial integrity proof.

Mutually exclusive diagnostic outcomes use enums or `Result` values instead of
nullable field pairs, so machine-readable reports cannot express contradictory
success and failure states. Failures include a stable `DiagnosticCode` beside
the human-readable message, so automation does not need to parse prose.

The first snapshot request runs deferred generated registration and
`eager_init!` hooks exactly once before sealing, so doctor and bootstrap always
use the same immutable declaration set. Each measured allocation carries its
live `DiagnosticMemorySize` directly. A report built without size measurements
omits that field. Invalid persisted slots fail ledger recovery before measurement;
doctor reports the recovery failure and does not export the invalid ledger.
Doctor borrows the resolved declarations for validation without constructing an
allocation capability.

## Explicit `MemoryRuntime<M>`

The explicit runtime requires only
`M: ic_memory::ic_stable_structures::Memory`:

```rust,ignore
let declarations = ic_memory::sealed_declaration_snapshot()?;
let mut runtime = ic_memory::MemoryRuntime::new(backing_memory)?;
runtime.bootstrap(&declarations, &policy)?;

let users = runtime.open_memory("app.users.v1", 120)?;
let export = runtime.diagnostic_export()?;
let recovery = runtime.commit_recovery_diagnostic()?;
let doctor = runtime.doctor_report(&declarations, &policy);
```

Runtime construction accepts empty backing memory or the current
`ic-stable-structures` `MemoryManager` layout. It returns
`RuntimeConstructionError` before initialization when nonempty memory has
foreign magic or an unsupported manager version, leaving rejected bytes
unchanged.

Fresh construction reserves the manager's metadata page before writing its
header. Ordinary backing growth refusal returns
`RuntimeConstructionError::Growth(RuntimeGrowError::BackingRefused { .. })`
without writes; the same unchanged backing can be retried. Configured default
bootstrap propagates this through `RuntimeStateError::Construction`.

`runtime.committed_allocations()` borrows the capability stored under the
runtime. Opening memory never accepts a capability from another runtime; it
consults the capability and `MemoryManager` owned by the same object. Capability
publication happens only after the stable-cell record write succeeds.

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

This lower-level path accepts resolved fixed declarations. Logical requests and
recovered-metadata admission belong to `MemoryRuntime::bootstrap`, which adds
preparation and resolution before the same validation/persistence boundary.
Use the runtime when composing those features; a diagnostic export cannot
resolve requests or authorize historical opens.

Maintained recovery paths enforce byte, collection, nesting and history limits
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
    IC_MEMORY_AUTHORITY_OWNER, MemoryManagerAuthorityRecord, MemoryManagerIdRange,
    MemoryManagerRangeAuthority, MemoryManagerRangeMode, memory_manager_governance_range,
};

let authority = MemoryManagerRangeAuthority::from_records(vec![
    MemoryManagerAuthorityRecord::new(
        memory_manager_governance_range(),
        IC_MEMORY_AUTHORITY_OWNER,
        MemoryManagerRangeMode::Reserved,
        None,
    )
    .expect("ic-memory governance record"),
    MemoryManagerAuthorityRecord::new(
        MemoryManagerIdRange::new(10, 99).expect("framework range"),
        "framework.example",
        MemoryManagerRangeMode::Reserved,
        None,
    )
    .expect("framework record"),
])
.expect("non-overlapping ranges");

authority
    .validate_id_authority_mode(42, "framework.example", MemoryManagerRangeMode::Reserved)
    .expect("framework-owned ID");
```

An open stack composes records from multiple packages and rejects overlaps:

```rust
use ic_memory::{
    MemoryManagerAuthorityRecord, MemoryManagerIdRange, MemoryManagerRangeAuthority,
    MemoryManagerRangeMode,
};

let framework_records = vec![
    MemoryManagerAuthorityRecord::new(
        MemoryManagerIdRange::new(10, 99).expect("framework range"),
        "framework.example",
        MemoryManagerRangeMode::Reserved,
        None,
    )
    .expect("framework record"),
];

let database_records = vec![
    MemoryManagerAuthorityRecord::new(
        MemoryManagerIdRange::new(120, 149).expect("database range"),
        "database.framework",
        MemoryManagerRangeMode::Reserved,
        None,
    )
    .expect("database record"),
];

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
    IC_MEMORY_AUTHORITY_OWNER, MEMORY_MANAGER_MAX_ID, MemoryManagerAuthorityRecord,
    MemoryManagerIdRange, MemoryManagerRangeAuthority, MemoryManagerRangeMode,
    memory_manager_governance_range,
};

let authority = MemoryManagerRangeAuthority::from_records(vec![
    MemoryManagerAuthorityRecord::new(
        memory_manager_governance_range(),
        IC_MEMORY_AUTHORITY_OWNER,
        MemoryManagerRangeMode::Reserved,
        None,
    )
    .expect("ic-memory governance record"),
    MemoryManagerAuthorityRecord::new(
        MemoryManagerIdRange::new(10, 99).expect("framework range"),
        "framework.example",
        MemoryManagerRangeMode::Reserved,
        None,
    )
    .expect("framework record"),
    MemoryManagerAuthorityRecord::new(
        MemoryManagerIdRange::new(100, MEMORY_MANAGER_MAX_ID).expect("application range"),
        "applications",
        MemoryManagerRangeMode::Allowed,
        None,
    )
    .expect("application record"),
])
.expect("non-overlapping ranges");

authority
    .validate_complete_coverage(MemoryManagerIdRange::all_usable())
    .expect("closed policy covers every usable ID");
```

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

The crate also exposes range-authority helpers for frameworks that want to split
ID ranges between infrastructure and application stores.

Canic can reserve framework ranges such as `10..=99` through its adapter. That
kind of range is Canic policy, not an `ic-memory` rule.

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
