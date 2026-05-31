# Introduction

Internet Computer canisters often store durable state through
`ic-stable-structures`. Multiple stable stores are commonly multiplexed through
a `MemoryManager`, where each logical store is assigned a physical
stable-memory slot.

Across upgrades, an accidental reassignment can make a canister open one
store's bytes as another store's data. `ic-memory` addresses this class of
failure by maintaining a durable allocation ledger that binds stable logical
keys to physical `MemoryManager` IDs and rejects layout drift before
application stable-memory handles are opened.

This document describes the problem, protocol, core invariants, durable commit
model, and the compact Lean model that accompanies the crate. The Lean model is
executable documentation for the allocation-safety argument; it is not a formal
verification of the Rust implementation.

## Problem Statement

Stable memory is durable across Internet Computer canister upgrades. That
durability is useful only when future binaries interpret the same durable bytes
under the same logical identity.

Consider a canister version that ships with:

```text
app.users.v1  -> MemoryManager ID 100
app.orders.v1 -> MemoryManager ID 101
```

If a later binary accidentally ships with the assignments swapped, the program
may compile and install:

```text
app.users.v1  -> MemoryManager ID 101
app.orders.v1 -> MemoryManager ID 100
```

The failure is not a memory-access violation. It is an allocation-ABI
violation: the binary can open orders data as users data, and users data as
orders data.

`ic-memory` treats this mapping as persistent protocol state:

$$
\text{stable key} \longrightarrow \text{physical allocation slot}
$$

A future binary may reopen the same key on the same slot, or it may introduce a
new key. It may not move an existing key to a new slot, and it may not reuse a
historical slot for another key.
