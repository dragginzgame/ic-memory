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
  -> dual protected committed AllocationLedger payloads
```

`CborLedgerCodec` is the built-in codec for those committed ledger payloads.

A typical framework flow is:

1. Recover the saved allocation ledger.
2. Declare the stores this binary expects.
3. Validate those declarations against history and policy.
4. Commit the new generation.
5. Open stable-memory handles only after validation passes.

The important rule: validate layout before touching stable data.

## Runtime Ownership

Exactly one owner should bootstrap the default `ic-memory` runtime in a
canister. Canic can be that owner, IcyDB can be that owner, or the application
can be that owner.

All crates using the default runtime compose into one bootstrap authority.

If multiple layers need separate allocation domains, they should use distinct
ledger stores and custom integration code.

## Declaration-Only Hooks

Use `eager_init!` when a crate needs to register declarations before bootstrap
without opening a TLS stable structure:

```rust,ignore
ic_memory::eager_init!({
    ic_memory::register_static_memory_manager_declaration(
        121,
        env!("CARGO_PKG_NAME"),
        "OrdersDataStore",
        "icydb.test_db.orders.data.v1",
    )
    .expect("valid ic-memory declaration");
});
```

Hooks registered with `eager_init!` run before the declaration snapshot is
sealed. Stable structures opened with `ic_memory_key!` require validated
allocations to be published first.

Frameworks or libraries that need custom policy metadata can inspect
`static_memory_declarations()` and `static_memory_range_declarations()`, then
bootstrap with `runtime::bootstrap_default_memory_manager_with_policy(...)`.

## Manual Bootstrap

The macro runtime is built on the lower-level ledger API. Frameworks that need
to own a custom ledger substrate can still drive that API directly.

The safe order is fixed:

```text
recover persisted allocation ledger
declare this binary's expected stable stores
validate declarations against ledger/history/policy
commit the new generation
only then open stable-memory handles
```

Manual sketch:

```rust,ignore
let declarations = DeclarationCollector::new()
    .with_memory_manager("app.orders.v1", 100, "orders")?
    .seal()?;

let commit = AllocationBootstrap::new(record.store_mut()).initialize_validate_and_commit(
    &CborLedgerCodec,
    &genesis_ledger,
    declarations,
    &policy,
    runtime_fingerprint,
)?;

let orders =
    AllocationSession::new(storage, commit.validated).open(&StableKey::parse("app.orders.v1")?)?;
```

The helper names for `record`, `genesis_ledger`, `policy`,
`runtime_fingerprint`, and `storage` are placeholders. Frameworks and libraries
wire those to their own stable-memory persistence and collection construction.
The ordering is the contract.

`AllocationLedger::new(...)` builds a structurally valid ledger DTO. Use
`AllocationLedger::new_committed(...)` only when you are manually constructing
committed ledger state and want the stricter committed-generation checks.
Normal integrations should usually recover through the commit/recovery flow
instead of hand-assembling committed state.

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
does not write to the allocation ledger.

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

The crate also exposes range-authority helpers for frameworks that want to split
ID ranges between infrastructure and application stores.

Canic currently uses `10..=99` as a framework-reserved example range. That is
Canic policy, not an `ic-memory` rule.

## What It Does Not Do

`ic-memory` does not replace `ic-stable-structures`.

It owns allocation governance. It does not re-export or wrap every
`ic-stable-structures` collection type.

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

Earlier drafts exposed some durable DTO fields directly. Current versions use
checked constructors and accessors so invalid allocation state is harder to
construct accidentally.

The protected physical checksum detects torn writes and accidental corruption.
It is not a cryptographic integrity mechanism and must not be treated as
adversarial tamper resistance.
