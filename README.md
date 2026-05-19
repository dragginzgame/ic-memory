# ic-memory

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
  <img src="images/balloon-meme.jpg" alt="Meme showing ic-memory keeping ic-stable-structures stable memory allocations from drifting" width="500">
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
use ic_memory::{
    AllocationDeclaration, AllocationSlotDescriptor, DeclarationCollector, SchemaMetadata,
};

let mut declarations = DeclarationCollector::default();

declarations.push(
    AllocationDeclaration::new(
        "app.orders.v1",
        AllocationSlotDescriptor::memory_manager(100).expect("usable slot"),
        Some("orders".to_string()),
        SchemaMetadata::default(),
    )
    .expect("valid allocation declaration"),
);

let snapshot = declarations.seal().expect("valid declaration snapshot");
assert_eq!(snapshot.len(), 1);
```

That snapshot is what you validate against the recovered ledger before opening
the store.

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
