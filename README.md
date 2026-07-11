# ic-memory

<p align="center">
  <img src="images/under-construction.gif" alt="Animated warning banner" width="400">
</p>

<p align="center">
  <strong>EARLY INFRASTRUCTURE: validate before opening stable memory.</strong>
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

## Why Use It?

Use `ic-memory` when a canister has more than one stable store and the layout
can change over time.

It is most useful for frameworks, generated canisters, multi-store apps, plugin
systems, and canister families that evolve across releases.

You probably do not need it for a tiny canister with one hand-written stable
structure and a fixed layout.

<p align="center">
  <img src="images/balloon-meme.jpg" alt="Meme showing ic-memory keeping ic-stable-structures stable memory allocations from drifting" width="375">
</p>

## The Bug

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
orders data. `ic-memory` catches that mismatch first.

## Quick Start

Declare both direct dependencies:

```toml
[dependencies]
ic-memory = "0.9.0"
ic-stable-structures = "0.7.2"
```

Declare the MemoryManager IDs your crate owns:

```rust,ignore
ic_memory::ic_memory_range!(authority = "icydb.test_db", start = 120, end = 129);
```

The authority string is explicit stable policy metadata. It is not persisted
allocation identity; the stable key and memory ID fill that role. Use the same
authority value for the package's range and key declarations, and do not derive
it from a Cargo package name or module path.

Open stable structures through `ic_memory_key!`:

```rust,ignore
use std::cell::RefCell;

thread_local! {
    pub static USERS: RefCell<UsersStore> = RefCell::new(UsersStore::init(
        ic_memory::ic_memory_key!(
            authority = "icydb.test_db",
            key = "icydb.test_db.users.data.v1",
            ty = UsersStore,
            id = 120,
        )
    ));
}
```

Bootstrap once before touching stable data:

```rust,ignore
#[ic_cdk::init]
fn init() {
    ic_memory::bootstrap_default_memory_manager().expect("valid stable-memory layout");
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    ic_memory::bootstrap_default_memory_manager().expect("valid stable-memory layout");
}
```

That is the normal path.

The default runtime API is exported from the crate root. Use helpers such as
`ic_memory::bootstrap_default_memory_manager()`,
`ic_memory::bootstrap_default_memory_manager_with_policy(...)`,
`ic_memory::committed_allocations()`,
`ic_memory::open_default_memory_manager_memory(...)`, and the macros shown
above; implementation modules are private.

## Multi-Crate Composition

Every crate registers into the same linked `ic-memory` runtime. Crates do not
need to import or name each other:

```rust,ignore
mod package_a {
    ic_memory::ic_memory_range!(authority = "package_a", start = 100, end = 109);

    thread_local! {
        pub static USERS: RefCell<UsersStore> = RefCell::new(UsersStore::init(
            ic_memory::ic_memory_key!(
                authority = "package_a",
                key = "package_a.users.v1",
                ty = UsersStore,
                id = 100,
            )
        ));
    }
}

mod package_b {
    ic_memory::ic_memory_range!(authority = "package_b", start = 110, end = 119);

    thread_local! {
        pub static ORDERS: RefCell<OrdersStore> = RefCell::new(OrdersStore::init(
            ic_memory::ic_memory_key!(
                authority = "package_b",
                key = "package_b.orders.v1",
                ty = OrdersStore,
                id = 110,
            )
        ));
    }
}
```

Bootstrap validates the complete layout from every linked crate, commits the
allocation ledger, and publishes committed allocations. TLS-backed stores open
when your code first touches the `thread_local!`.

Duplicate stable keys, duplicate MemoryManager IDs, overlapping ranges, and
out-of-range declarations fail before stable structures open.

`ic-memory` follows the `ic-stable-structures::MemoryManager` ID domain exactly:
IDs `0..=254` are usable, and ID `255` is always the unallocated sentinel. It is
not an application slot and cannot be declared or reserved.

The default runtime reserves `MemoryManager` IDs `0..=9` and stable keys under
`ic_memory.*` for allocation-governance records. The ledger itself lives at ID
`0`; it remains in the durable ledger for recovery, but public runtime helpers
do not publish or open that internal allocation as application memory.

Range claims are authoritative in the default runtime. If a crate registers
`ic_memory_range!`, its declared memories must stay inside that range. Framework
adapters that want their own range policy, such as Canic, should register only
the ranges they want `ic-memory` to enforce and put the rest in their policy
adapter.

The committed allocation state is an in-memory capability published only after
bootstrap persistence succeeds; it is not a serde payload and should not be
treated as configuration.

## Diagnostics

Use `default_memory_manager_doctor_report()` for operator-facing preflight and
runtime diagnostics. It can be called before or after bootstrap and reports the
stable-cell status, protected commit recovery state, recovered ledger export,
registered declarations, range authority, validation preflight, and live
`MemoryManager` slot sizes when they can be recovered.

Use `default_memory_manager_commit_recovery_diagnostic()` when you only need the
redundant commit-slot status, including empty, corrupt, ambiguous, or
recoverable physical ledger state.

## Stable Keys

Stable keys are permanent logical store names. They should describe ownership
and purpose, not the current memory ID.

```text
namespace.component.store_or_role.vN
```

Examples:

```rust
use ic_memory::StableKey;

StableKey::parse("app.orders.v1").expect("app key");
StableKey::parse("myapp.audit_log.v1").expect("app key");
StableKey::parse("icydb.test_db.users.data.v1").expect("database key");
```

Changing a key creates a new logical allocation identity. If the durable store
is the same, keep the stable key and update schema metadata instead.

Schema metadata is optional diagnostic metadata for the in-place store schema.
Construct it with `SchemaMetadata::new(Some(version))`; version `0` is reserved
for absence and is rejected.

## More Detail

The short version:

```text
declare ranges
register stable stores
bootstrap once
only then open stable memory
```

Framework authors and policy adapters should read [ADVANCED.md](ADVANCED.md).
The non-negotiable invariants are recorded in [SAFETY.md](SAFETY.md). The
protocol whitepaper lives in [whitepaper/src/SUMMARY.md](whitepaper/src/SUMMARY.md)
and builds as an mdBook with `make maintainer-build`.

`ic-memory` is early infrastructure extracted from Canic. It owns allocation
governance, not schema migration, endpoint routing, authorization, or data
semantics.
