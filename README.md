# ic-memory

`ic-memory` is a safety system for an Internet Computer application's persistent
data.

When an application is upgraded, its code changes but its stored data remains.
Each database, log, queue, or settings store must reconnect to the same storage
location it used before. If two stores are accidentally swapped, the new code
can interpret one kind of data as another.

`ic-memory` remembers which storage location belongs to each named store. Before
the application opens any of those stores, it checks the new layout against the
saved allocation history. A conflicting upgrade stops with an error instead of
opening the wrong data.

> `ic-memory` protects the connection between a store and its storage location.
> It is not a backup system, a database schema migrator, or a data validator.

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/ic-memory/main/images/ic-memory-upgrade-blocked.svg" alt="Before an upgrade, Users uses storage 100 and Orders uses storage 101. A mistaken upgrade swaps those assignments, so ic-memory blocks the upgrade before either store opens." width="900">
</p>

## Why this matters

Imagine that version 1 of an application stores users and orders separately:

```text
Users  -> storage location 100
Orders -> storage location 101
```

A later version accidentally reverses those locations:

```text
Users  -> storage location 101
Orders -> storage location 100
```

The program can still compile, and the upgrade can still install. Without an
allocation check, it may then read order records as users and user records as
orders. `ic-memory` detects the changed ownership before either location is
opened.

This protection matters because a stable-memory location is a durable part of
an application's storage layout, even though it can look like an ordinary
number in source code.

## Is it useful for my application?

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/ic-memory/main/images/ic-memory-decision-guide.svg" alt="Decision guide: ic-memory is most useful for applications with several persistent stores, libraries or plugins that contribute stores, or storage layouts that change across upgrades. It is not a backup or schema-migration tool." width="800">
</p>

| Situation | Recommendation |
| --- | --- |
| One small, hand-written store whose location never changes | Probably unnecessary |
| Several persistent stores | Useful |
| Libraries, plugins, or generated code contribute stores | Especially useful |
| The storage layout evolves across upgrades | Especially useful |
| You need backups or database-schema migrations | Use a separate mechanism |

`ic-memory` is most useful for frameworks, generated canisters, multi-store
applications, plugin systems, and canister families that evolve over time.

## How it works

At a high level, the application:

1. Gives every persistent store a permanent name.
2. Declares which storage locations its components may use.
3. Bootstraps `ic-memory` before opening stable data.
4. Lets `ic-memory` compare the proposed layout with durable allocation history.
5. Opens stores only after that comparison succeeds.

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/ic-memory/main/images/ic-memory-lifecycle.svg" alt="The ic-memory lifecycle: name each store, remember its storage location, then compare the expected and remembered layouts before opening data. Matching layouts open safely; conflicts stop with an error." width="760">
</p>

The important guarantee is **validation before open**. A diagnostic report or
an uncommitted validation result cannot grant access to a store. The runtime
publishes permission to open application memory only after the allocation
ledger has been recovered, checked, and durably updated.

## What it protects

| `ic-memory` protects | The application or framework still owns |
| --- | --- |
| A named store staying at the same location | Backups and disaster recovery |
| A location not being reused for another store | Database-schema migrations |
| Layout validation before stores open | The meaning and validity of stored data |
| Storage ranges assigned to different components | Controller and endpoint authorization |
| Durable retirement of old allocations | Application lifecycle decisions |

Retiring a store does not make its old location reusable. That tombstone is
intentional: reusing the location could make a rollback open unrelated data.

## Terminology

- **Stable memory** is persistent storage that survives an Internet Computer
  canister upgrade.
- A **stable key** is a permanent, human-readable name for one store, such as
  `app.orders.v1`.
- A **memory ID** or **slot** is the numbered `MemoryManager` storage location
  behind that name.
- An **authority** is an ownership label. It prevents one component from
  claiming storage assigned to another component.
- A **range grant** is a group of memory IDs that an authority may use.
- **Bootstrap** is the check-and-commit step that must succeed before the
  application opens its stores.
- The **allocation ledger** is `ic-memory`'s durable record of which stable key
  owns which slot.

## Quick start for developers

Add the crate:

```toml
[dependencies]
ic-memory = "0.24.11"
```

`ic-memory` re-exports its exact `ic-stable-structures` dependency through
`ic_memory::ic_stable_structures`. Import collections, backing memories, and
traits through that namespace so their types match the runtime:

```rust
use ic_memory::{
    RuntimeMemory,
    ic_stable_structures::{Cell, DefaultMemoryImpl},
};

type CounterStore = Cell<u64, RuntimeMemory<DefaultMemoryImpl>>;
```

A separate `ic-stable-structures` dependency is unnecessary for those imports.

### Declare a fixed storage location

Give the component an authority, grant it a range, and declare the permanent
key and ID of each store:

```rust,no_run
const MEMORY_AUTHORITY: &str = "example_app";

ic_memory::ic_memory_range!(
    authority = MEMORY_AUTHORITY,
    start = 120,
    end = 129,
);

ic_memory::ic_memory_declaration!(
    authority = MEMORY_AUTHORITY,
    key = "example_app.users.v1",
    label = "UsersStore",
    id = 120,
);

fn initialize_stable_storage() -> Result<(), Box<dyn std::error::Error>> {
    ic_memory::bootstrap_default_memory_manager()?;
    let users = ic_memory::open_default_memory_manager_memory(
        "example_app.users.v1",
        120,
    )?;
    drop(users);
    Ok(())
}
```

Call the bootstrap function from both the canister's initialization and
post-upgrade lifecycle before code touches any stable collection.

The default range mode is `Reserved`: it permits declared fixed IDs but does
not provide new automatic allocations.

### Let the host assign a location

Libraries can request a durable key without choosing an ID. The application
that owns the runtime grants an explicit `Allowed` pool:

```rust,no_run
ic_memory::ic_memory_range!(
    authority = "example_app",
    start = 10,
    end = 254,
    mode = Allowed,
);
ic_memory::ic_memory_declaration!(
    authority = "example_app",
    key = "example_app.users.v1",
);

fn initialize() -> Result<(), Box<dyn std::error::Error>> {
    ic_memory::bootstrap_default_memory_manager()?;
    let assigned_id =
        ic_memory::default_memory_manager_memory_id("example_app.users.v1")?;
    let users =
        ic_memory::open_default_memory_manager_memory_by_key("example_app.users.v1")?;
    drop((assigned_id, users));
    Ok(())
}
```

Known keys keep their committed IDs. New requests are sorted by stable key and
receive the lowest unclaimed ID in their authority's `Allowed` ranges. Fixed,
reserved, omitted, and retired allocations remain unavailable. If no eligible
ID remains, bootstrap returns `MemoryResolutionError::Exhausted`.

For examples of both allocation styles, see
[`examples/key_only.rs`](examples/key_only.rs) and
[`examples/composed_host.rs`](examples/composed_host.rs).

## Applications composed from several libraries

Every linked crate contributes declarations to one immutable registry. The
application grants each component only its intended range and bootstraps the
combined layout once.

```text
application bootstrap owner
|-- library A: storage 100-109
|-- library B: storage 110-119
`-- application: storage 120-129

one combined layout check -> one committed allocation view -> stores may open
```

<!--
Diagram insertion point: replace the tree above with a composed-ownership
diagram. Suggested asset name: images/composed-ownership.svg
-->

Libraries adopting an already bootstrapped host can verify that all of their
requirements were included without rerunning bootstrap or replacing the host's
policy:

```rust
fn adopt_host() -> Result<(), Box<dyn std::error::Error>> {
    let requirements = ic_memory::sealed_declaration_snapshot()?;
    ic_memory::verify_default_memory_manager_authority(
        &requirements,
        "library_name",
    )?;
    Ok(())
}
```

Duplicate stable keys, duplicate memory IDs, overlapping ranges, and
out-of-range declarations fail before stable structures open. Hosts must also
declare or reserve allocations used by raw `MemoryManager` clients; diagnostics
cannot infer ownership from bytes alone.

The default runtime reserves IDs `0..=9` and keys under `ic_memory.*` for its
own governance records. ID `0` contains the allocation ledger. Application
code may use IDs through `254`; ID `255` is the upstream unallocated sentinel
and can never be declared.

## Stable keys

A stable key names the durable identity of a store, not its current numeric
location:

```text
namespace.component.store_or_role.vN
```

```rust
use ic_memory::StableKey;

StableKey::parse("app.orders.v1").expect("app key");
StableKey::parse("myapp.audit_log.v1").expect("app key");
StableKey::parse("icydb.test_db.users.data.v1").expect("database key");
```

Changing a key creates a new allocation identity. If the durable store is still
the same store, keep its key and change its optional diagnostic schema metadata
instead.

## Operations and advanced integration

- [Operations and diagnostics](https://github.com/dragginzgame/ic-memory/blob/main/docs/operations.md)
  explains memory attribution, bucket configuration, and runtime reports.
- [Advanced ic-memory](https://github.com/dragginzgame/ic-memory/blob/main/ADVANCED.md)
  covers explicit runtimes, custom policies, recovery, and manual bootstrap.
- [Safety invariants](https://github.com/dragginzgame/ic-memory/blob/main/SAFETY.md)
  records the guarantees future changes must preserve.
- [Documentation index](https://github.com/dragginzgame/ic-memory/blob/main/docs/README.md)
  separates current guidance from historical engineering evidence.
- [Release guide](https://github.com/dragginzgame/ic-memory/blob/main/RELEASING.md)
  is for maintainers preparing and publishing a release.

`ic-memory` is pre-1.0 infrastructure extracted from Canic. Current releases use
the current API and durable format only; earlier pre-1.0 wire formats are not a
compatibility target.
