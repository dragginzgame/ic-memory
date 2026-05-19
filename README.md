

# ic-memory

<p align="center">
  <strong style="font-size: 3em;">DO NOT USE</strong>
</p>

<p align="center">
  <img src="images/under-construction.gif" alt="Animated warning banner" width="400">
</p>

---

`ic-memory` helps Internet Computer canisters avoid opening the wrong stable
memory after an upgrade.

It remembers this mapping forever:

```text
logical store -> physical stable-memory slot
```

If a future version tries to move that store to a different slot, or reuse that
slot for a different store, `ic-memory` rejects the layout before stable-memory
handles are opened.

<p align="center">
  <img src="images/balloon-meme.jpg" alt="Meme showing ic-memory keeping ic-stable-structures stable memory allocations from drifting" width="375">
</p>

## Why Use It?

Use `ic-memory` when a canister has more than one stable store and the layout
can change over time.

It is most useful for:

- IC frameworks
- generated canisters
- multi-store canisters
- plugin or module systems
- canister families that evolve across releases
- any project where stable-memory ID reuse would be a serious bug

You probably do not need it for a tiny canister with one hand-written stable
structure and a fixed layout.

## What It Protects Against

The dangerous bug is slot drift.

Version 1 ships with:

```text
app.users.v1  -> MemoryManager ID 100
app.orders.v1 -> MemoryManager ID 101
```

A later upgrade accidentally ships with:

```text
app.users.v1  -> MemoryManager ID 101
app.orders.v1 -> MemoryManager ID 100
```

That can still compile. It can even install.

But now the canister may open orders data as users data, and users data as
orders data.

`ic-memory` catches that mismatch first.

<p align="center">
  <img src="images/dont-overwrite.png" alt="Retro computer warning: don't overwrite your memory" width="500">
</p>

## How It Fits

`ic-stable-structures` stores the data.

`ic-memory` checks that each logical store is still opening the same physical
slot it owned before.

A typical framework flow is:

1. Recover the saved allocation ledger.
2. Declare the stores this binary expects.
3. Validate those declarations against history and policy.
4. Commit the new generation.
5. Open stable-memory handles only after validation passes.

The important rule: validate layout before touching stable data.

## Basic Declaration

Declare every stable store with a stable name and a physical slot:

```rust
use ic_memory::DeclarationCollector;

let snapshot = DeclarationCollector::new()
    .with_memory_manager("app.orders.v1", 100, "orders")
    .expect("valid allocation declaration")
    .seal()
    .expect("valid declaration snapshot");
assert_eq!(snapshot.len(), 1);
```

That snapshot is what you validate against the recovered ledger before opening
the store.

## Stable Keys

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

Examples:

```rust
use ic_memory::StableKey;

StableKey::parse("app.orders.v1").expect("app key");
StableKey::parse("canic.core.app_state.v1").expect("Canic framework key");
StableKey::parse("canic.core.intent_records.v1").expect("Canic store key");
StableKey::parse("icydb.test_db.users.data.v1").expect("IcyDB table data key");
StableKey::parse("icydb.demo_rpg.commit.control.v1").expect("IcyDB commit key");
```

Suggested namespace conventions:

- `ic_memory.*` is reserved for `ic-memory` governance records.
- `canic.core.*` is appropriate for Canic framework-owned stores.
- `icydb.<memory_namespace>.<store_name>.<role>.vN` works for generated IcyDB
  stores, such as `icydb.test_db.users.data.v1`.
- Application-owned stores can use an application namespace, such as
  `app.orders.v1` or `myapp.audit_log.v1`.

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
    .reserve_ids(10, 99, "canic.framework")
    .expect("framework range");

authority
    .validate_id_authority_mode(42, "canic.framework", MemoryManagerRangeMode::Reserved)
    .expect("Canic-owned framework ID");
```

An open stack composes records from multiple packages and rejects overlaps:

```rust
use ic_memory::MemoryManagerRangeAuthority;

let canic_records = MemoryManagerRangeAuthority::new()
    .reserve_ids(10, 99, "canic.framework")
    .expect("Canic range")
    .to_records();

let database_records = MemoryManagerRangeAuthority::new()
    .reserve_ids(120, 149, "database.framework")
    .expect("database range")
    .to_records();

let authority = MemoryManagerRangeAuthority::from_records(
    canic_records
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
    .reserve_ids(10, 99, "canic.framework")
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

## What It Does Not Do

`ic-memory` does not replace `ic-stable-structures`.

It also does not handle:

- schema migrations
- controller authorization
- application data validation
- endpoint routing
- IC management-canister calls

It only protects stable-memory allocation ownership.

## Status

`ic-memory` is early infrastructure extracted from Canic. The public API is
intended to stabilize around persistent allocation ownership, but framework
authors should still treat this line as young infrastructure while the
standalone boundary settles.

The non-negotiable invariants are recorded in [SAFETY.md](SAFETY.md).

The protected physical checksum detects torn writes and accidental corruption.
It is not a cryptographic integrity mechanism and must not be treated as
adversarial tamper resistance.
