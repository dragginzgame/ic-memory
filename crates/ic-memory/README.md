<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-memory/ic-memory-readme-header.svg" alt="IC Memory — Stops upgrades from mixing up stored data" width="100%">
</p>

<!-- helper-navigation:start -->
<p align="center">
  <a href="https://github.com/dragginzgame/canic"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/canic.svg" width="18" height="18" alt=""> <strong>canic</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/icydb"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/icydb.svg" width="18" height="18" alt=""> <strong>icydb</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-timers"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-timers.svg" width="18" height="18" alt=""> <strong>ic-timers</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-memory"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-memory.svg" width="18" height="18" alt=""> <strong>ic-memory</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-query"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-query.svg" width="18" height="18" alt=""> <strong>ic-query</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-backup"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-backup.svg" width="18" height="18" alt=""> <strong>ic-backup</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-blob-storage"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-blob-storage.svg" width="18" height="18" alt=""> <strong>ic-blob-storage</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-testkit"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-testkit.svg" width="18" height="18" alt=""> <strong>ic-testkit</strong></a>
</p>
<!-- helper-navigation:end -->

**Jump to:** [What it does](#what-it-does) ·
[Why it matters](#why-this-matters) ·
[Is it useful?](#is-it-useful-for-my-application) ·
[How it works](#how-it-works) ·
[Choose an integration style](#choose-an-integration-style) ·
[Quick start](#quick-start-for-developers) ·
[Troubleshooting](https://github.com/dragginzgame/ic-memory/blob/main/docs/troubleshooting.md) ·
[Advanced integration](#operations-and-advanced-integration)

## What it does

`ic-memory` is a safety system for an Internet Computer application's persistent
data.

When an application is upgraded, its code changes but its stored data remains.
Each database, log, queue, or settings store must reconnect to the same storage
location it used before. If two stores are accidentally swapped, the new code
can interpret one kind of data as another.

`ic-memory` remembers which storage location belongs to each named store. Before
the application opens any of those stores, it checks the new layout against the
saved ownership records. A conflicting upgrade stops with an error instead of
opening the wrong data.

> `ic-memory` protects the connection between a store and its storage location.
> It is not a backup system, a database schema migrator, or a data validator.

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

The diagram below shows the same mistake and the point where `ic-memory`
intervenes:

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-memory/ic-memory-upgrade-blocked.svg" alt="Before an upgrade, Users uses storage 100 and Orders uses storage 101. A mistaken upgrade swaps those assignments, so ic-memory blocks the upgrade before either store opens." width="900">
</p>
<p align="center"><em>A changed store-to-location mapping is rejected before application data opens.</em></p>

The program can still compile, and the upgrade can still install. Without an
allocation check, it may then read order records as users and user records as
orders. `ic-memory` detects the changed ownership before either location is
opened.

This protection matters because a stable-memory location is a durable part of
an application's storage layout, even though it can look like an ordinary
number in source code.

## Is it useful for my application?

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-memory/ic-memory-decision-guide.svg" alt="Decision guide: ic-memory is most useful for applications with several persistent stores, libraries or plugins that contribute stores, or storage layouts that change across upgrades. It is not a backup or schema-migration tool." width="800">
</p>
<p align="center"><em>Use ic-memory for evolving multi-store layouts, not as a backup or migration system.</em></p>

`ic-memory` is most useful for frameworks, generated canisters, multi-store
applications, plugin systems, and canister families that evolve over time.

## How it works

The lifecycle has three parts:

1. **Name each store.** The application gives every persistent store a permanent
   identity, such as `app.users.v1`.
2. **Remember its location.** On the first successful installation,
   `ic-memory` records which storage location belongs to each name.
3. **Check before opening.** On every installation or upgrade, the new
   application version declares the layout it expects. `ic-memory` compares that
   layout with the remembered one before any application store opens.

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-memory/ic-memory-lifecycle.svg" alt="The ic-memory lifecycle: name each store, remember its storage location, then compare the expected and remembered layouts before opening data. Matching layouts open safely; conflicts stop with an error." width="760">
</p>
<p align="center"><em>Name stores, retain their locations, and check the complete layout before opening data.</em></p>

If the layouts agree, the application receives permission to open its stores.
If a known name moved, a location changed owner, or another allocation rule is
broken, the check returns an error and `ic-memory` grants no permission to open
application stores.

This check-and-record operation is called **bootstrap**. Its important guarantee
is **validation before open**: a diagnostic report or an uncommitted validation
result cannot grant access to a store. The runtime publishes permission only
after the allocation ledger has been recovered, checked, and durably updated.

The ledger stores one current ownership record per allocated identity. Store
schema versions and schema admission belong to the application. Omitted and retired
identities retain their IDs to prevent accidental reuse. A commit counter and one
checksummed current snapshot support stale-proof checks and corruption detection.

## What happens when a check fails?

Bootstrap returns an error before application stores are opened. `ic-memory`
does not silently repair, move, discard, or reinterpret a conflicting
allocation. A failed attempt publishes no permission to open stores.

Treat the error as an upgrade-safety signal: keep the existing stable memory,
inspect the requested keys, host namespace grants, exclusions, policy and diagnostics, then correct
the new application version. Do not erase the allocation ledger or replace it
with an empty one to make the error disappear. See the
[symptom-based troubleshooting guide](https://github.com/dragginzgame/ic-memory/blob/main/docs/troubleshooting.md) for safe next
steps.

## What it protects

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-memory/ic-memory-scope-boundary.svg" alt="ic-memory protects store-to-location mappings, prevents slot reuse, validates upgrade layouts and component ownership, and checks layouts before opening. Applications still own backups, schema migrations, stored-data semantics, authorization, and disaster recovery." width="800">
</p>
<p align="center"><em>Allocation safety complements backups, schema migration, authorization, and disaster recovery.</em></p>

Retiring a store does not make its old location reusable. That tombstone is
intentional: reusing the location could make a rollback open unrelated data.

## Terminology

- A **stable key** is a permanent store name, such as `app.orders.v1`.
- A **memory ID** is the physical `MemoryManager` location behind that key.
- An **authority** names the linked component requesting a key.
- A **namespace grant** admits that authority's keys under a host-selected prefix.
- The **allocation pool** contains eligible application IDs shared by all owners.
- **Bootstrap** recovers, admits, resolves, validates and commits before opening.
- The **allocation ledger** retains key-to-ID bindings and retirement tombstones.

The application owns one pool and one bootstrap per backing memory. Libraries
contribute key requests, verify that the host included them, and open their
committed keys. Owner labels are current host policy; they are not persisted
ownership history, caller authentication or a sandbox for linked code.

## Quick start for developers

Add the crate:

```toml
[dependencies]
ic-memory = "0.46.0"
```

`ic-memory` exports `Memory`, `DefaultMemoryImpl` and `VectorMemory` for host
backing setup. It does not re-export raw collections or memory managers.
Production collection integrations belong in IC Storage's governed facade,
using the `RuntimeMemory` leases obtained from the host's runtime. Memory's
internal manager, ledger and qualification fixtures retain their substrate use.

### Declare keys; let the host allocate

Components name their owner and permanent keys without choosing IDs:

```rust,no_run
const MEMORY_AUTHORITY: &str = "example_app";
const USERS_KEY: &str = "example_app.users.v1";

fn component_requests() -> Result<Vec<ic_memory::MemoryRequest>, ic_memory::MemoryDeclarationError> {
    Ok(vec![ic_memory::MemoryRequest::new(
        MEMORY_AUTHORITY, USERS_KEY)?])
}

fn initialize_stable_storage() -> Result<(), Box<dyn std::error::Error>> {
    let pool = ic_memory::MemoryAllocationPool::new(
        vec![ic_memory::MemoryAuthority::new(MEMORY_AUTHORITY, "example_app.")?],
        vec![],
    )?;
    let declarations = ic_memory::SealedDeclarationSnapshot::new(component_requests()?)?;
    let mut runtime = ic_memory::MemoryRuntime::new(
        ic_memory::DefaultMemoryImpl::default(),
    )?;
    runtime.bootstrap(&declarations, &pool, &ic_memory::GenericAllocationPolicy)?;
    let users = runtime.open_memory(USERS_KEY)?;
    drop(users);
    Ok(())
}
```

The application retains this runtime across calls, bootstraps it during
initialization and post-upgrade, and injects opened handles into components.
Use `MemoryRuntime::new_with_config` for custom bucket geometry and pass the
host policy to `bootstrap`. The library owns no singleton or thread-local
runtime. Request factories only construct metadata; they never open stores.

Each allocation has at most one live lease per runtime. A second `open_memory`
returns `RuntimeOpenError::AlreadyOpen { id }`; `RuntimeMemory` does not implement
`Clone`. Transfer the handle into its collection and drop that owner before
reopening. A collection's memory-transfer operation keeps the same lease alive;
failed construction and ordinary unwinding release it when the handle is dropped.
For a log, obtain two different allocations for index and data memory.

The host must retain exclusive custody of the backing: do not create another
runtime/manager or perform raw IO against aliases while the runtime or any of
its handles lives. A handle can outlive its runtime, so dropping the runtime
alone does not permit a replacement. Generic backing aliases cannot be detected
by this crate. These are ownership rules for cooperating integrations, not an
isolation boundary against arbitrary linked code. Never leak a handle to force
reopening; a forgotten lease remains occupied.

Known keys retain their committed IDs. New requests are sorted by key and take
the lowest unclaimed ID in the common pool. Current, omitted, reserved and
retired records occupy their IDs permanently. Namespace grants do not divide
space between components. Exhaustion rejects the entire attempt.

See [`examples/key_only.rs`](examples/key_only.rs) and
[`examples/composed_host.rs`](examples/composed_host.rs).

## Applications composed from several libraries

Each selected component supplies its complete request list. The host combines
those lists into one `SealedDeclarationSnapshot`, admits disjoint namespaces
for their named owners and supplies explicit physical
exclusions for unmanaged `MemoryManager` clients. All components draw from the
same remaining pool. The application bootstraps the combined layout once.

Libraries adopting an already bootstrapped host can verify that all of their
requirements were included without rerunning bootstrap or replacing the host's
policy:

```rust
fn adopt_host<M: ic_memory::Memory>(
    runtime: &ic_memory::MemoryRuntime<M>,
) -> Result<(), Box<dyn std::error::Error>> {
    let requests = vec![ic_memory::MemoryRequest::new(
        "library_name", "library_name.rows.v1")?];
    let requirements = ic_memory::SealedDeclarationSnapshot::new(requests)?;
    runtime.verify_authority(
        &requirements,
        "library_name",
    )?;
    Ok(())
}
```

Duplicate keys, overlapping namespace grants, foreign owner claims and excluded
historical IDs fail before application stores open. Populated IDs without a
ledger record must be explicitly excluded by the host; bootstrap refuses to
infer custody from stored bytes.

The runtime reserves IDs `0..=9` and keys under `ic_memory.*` for its
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
the same store, keep its key and apply schema changes through the storage owner.

## Frequently asked questions

<p><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-memory/ic-faq-question.svg" alt="" width="22"> <strong>Does ic-memory move or migrate application data?</strong></p>

No. It validates allocation identity. Schema and data migrations remain the
application's responsibility.

<p><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-memory/ic-faq-question.svg" alt="" width="22"> <strong>Does it back up stable memory or recover corrupted application data?</strong></p>

No. Keep a separate backup and disaster-recovery plan. `ic-memory` fails closed
when its allocation metadata cannot be recovered safely.

<p><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-memory/ic-faq-question.svg" alt="" width="22"> <strong>What happens when validation fails?</strong></p>

Bootstrap returns an error and publishes no open capability. Fix the proposed
layout or policy; do not erase the ledger to bypass the conflict.

<p><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-memory/ic-faq-question.svg" alt="" width="22"> <strong>Can a retired memory location be reused?</strong></p>

No. Retirement is a permanent tombstone so a future version or rollback cannot
mistake unrelated data for the retired store.

<p><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-memory/ic-faq-question.svg" alt="" width="22"> <strong>Does every library bootstrap separately?</strong></p>

No. One owner bootstraps each concrete runtime. Libraries verify their
requirements against the host's committed layout and open only their keys.

<p><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-memory/ic-faq-question.svg" alt="" width="22"> <strong>What does each library declare?</strong></p>

Each library declares permanent keys and its owner label. The host admits the
namespace and owns physical exclusions; Memory assigns IDs and retains them.

<p><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-memory/ic-faq-question.svg" alt="" width="22"> <strong>Can an upgrade add a new store safely?</strong></p>

Yes, provided its key is new, the host admits its namespace and owner, the
common pool has space, and the complete layout passes admission and validation.

## Operations and advanced integration

Custom policies return `PolicyIdentity` directly from `runtime_bootstrap_identity`.
Construct it with `PolicyIdentity::new(host_token)`, where `host_token` is one
`[u8; 32]` identifying the complete prepared policy configuration and semantics.
Prepare any fallible host configuration first. Hosts own descriptive labels,
versions and token derivation; change the token when that policy changes. Memory
compares all token bytes for warm binding and does not persist or hash them.
Diagnostic JSON returns token arrays, including the doctor's direct tested token.

Production recovery and inspection use the host runtime's `diagnostic_export`,
`commit_recovery_diagnostic` and `doctor_report`. Raw ledger DTOs, bounded raw
commit-store readers and passive export construction are fixture-only under
`test-support`; recovered authority is internal. Bounded Cell payload inspection
remains available without granting allocation authority.

The current durable ledger uses checked numeric slots and a single `ICMSNAP1`
checksummed snapshot containing direct ledger CBOR. The 0.45.0 format cut retires
earlier images: use fresh/reset memory. Incompatible bytes reject without
initializing genesis or clearing memory. See the
[current ledger contract](../../docs/current-ledger.md).

For fixture runtimes, opt in to `test-support` and call `testing::fixture_runtime`
with explicit backing memory, owner, namespace, stable-key list and bucket
configuration. It returns `MemoryRuntime` for opening multiple allocations or
exercising refused opens, without initializing collections. Setup uses the
ordinary generic allocation policy and has typed `FixtureRuntimeError` failures;
no global manager or production admission policy is installed.

For fixture-only byte cells, opt in to `test-support` and construct
`testing::FixtureCell` with explicit backing memory, owner, namespace, key and
`MemoryManagerConfig`. The Cell handle retains its backing; the temporary
bootstrap runtime is dropped. No lifecycle handler or singleton is installed.
Use `try_new` for typed setup refusal of trusted images, or `create_bounded` /
`open_bounded` for explicit fresh/retained admission with an enforced byte limit.
Retained headers are checked before payload loading; malformed cells are refused
without initialization. `try_set` refuses oversized writes before mutation;
`into_memory` transfers the admitted handle for caller-owned raw inspection.
Existing `new` and `set` retain their panic convention. Callers keep codecs,
work budgets and restored-state/lifecycle validation; fresh empty bytes must
never substitute for a required retained journal. Bootstrap may update ledger
metadata before a later Cell refusal, and backing IO can still panic.

- [Operations and diagnostics](https://github.com/dragginzgame/ic-memory/blob/main/docs/operations.md)
  explains memory attribution, bucket configuration, and runtime reports.
- [Troubleshooting](https://github.com/dragginzgame/ic-memory/blob/main/docs/troubleshooting.md)
  maps common symptoms to safe recovery steps.
- [Advanced ic-memory](https://github.com/dragginzgame/ic-memory/blob/main/ADVANCED.md)
  covers explicit composition, custom policies, recovery, and runtime-owned persistence.
- [Safety invariants](https://github.com/dragginzgame/ic-memory/blob/main/SAFETY.md)
  records the guarantees future changes must preserve.
- [Documentation index](https://github.com/dragginzgame/ic-memory/blob/main/docs/README.md)
  separates current guidance from historical engineering evidence.
- [Release guide](https://github.com/dragginzgame/ic-memory/blob/main/RELEASING.md)
  is for maintainers preparing and publishing a release.

`ic-memory` is pre-1.0 infrastructure extracted from Canic. Current releases use
the current API and durable format only; earlier pre-1.0 wire formats are not a
compatibility target.
