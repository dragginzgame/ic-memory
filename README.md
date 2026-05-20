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

<p align="center">
  <img src="images/balloon-meme.jpg" alt="Meme showing ic-memory keeping ic-stable-structures stable memory allocations from drifting" width="375">
</p>

## Why Use It?

Use `ic-memory` when a canister has more than one stable store and the layout
can change over time.

It is most useful for frameworks, generated canisters, multi-store apps, plugin
systems, and canister families that evolve across releases.

You probably do not need it for a tiny canister with one hand-written stable
structure and a fixed layout.

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

<p align="center">
  <img src="images/dont-overwrite.png" alt="Retro computer warning: don't overwrite your memory" width="500">
</p>

## Quick Start

Declare the MemoryManager IDs your crate owns:

```rust,ignore
ic_memory::ic_memory_range!(start = 120, end = 129);
```

Open stable structures through `ic_memory_key!`:

```rust,ignore
use std::cell::RefCell;

thread_local! {
    pub static USERS: RefCell<UsersStore> = RefCell::new(UsersStore::init(
        ic_memory::ic_memory_key!(
            "icydb.test_db.users.data.v1",
            UsersStore,
            120,
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

## Multi-Crate Composition

Every crate registers into the same linked `ic-memory` runtime. Crates do not
need to import or name each other:

```rust,ignore
mod package_a {
    ic_memory::ic_memory_range!(start = 100, end = 109);

    thread_local! {
        pub static USERS: RefCell<UsersStore> = RefCell::new(UsersStore::init(
            ic_memory::ic_memory_key!("package_a.users.v1", UsersStore, 100)
        ));
    }
}

mod package_b {
    ic_memory::ic_memory_range!(start = 110, end = 119);

    thread_local! {
        pub static ORDERS: RefCell<OrdersStore> = RefCell::new(OrdersStore::init(
            ic_memory::ic_memory_key!("package_b.orders.v1", OrdersStore, 110)
        ));
    }
}
```

Bootstrap validates the complete layout from every linked crate, commits the
allocation ledger, and publishes validated allocations. TLS-backed stores open
when your code first touches the `thread_local!`.

Duplicate stable keys, duplicate MemoryManager IDs, overlapping ranges, and
out-of-range declarations fail before stable structures open.

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

## More Detail

The short version:

```text
declare ranges
register stable stores
bootstrap once
only then open stable memory
```

Framework authors and policy adapters should read [ADVANCED.md](ADVANCED.md).
The non-negotiable invariants are recorded in [SAFETY.md](SAFETY.md).

`ic-memory` is early infrastructure extracted from Canic. It owns allocation
governance, not schema migration, endpoint routing, authorization, or data
semantics.
