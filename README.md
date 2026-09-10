# ic-memory

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/ic-memory/main/images/under-construction.gif" alt="Animated warning banner" width="400">
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
  <img src="https://raw.githubusercontent.com/dragginzgame/ic-memory/main/images/balloon-meme.jpg" alt="Meme showing ic-memory keeping ic-stable-structures stable memory allocations from drifting" width="375">
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
ic-memory = "0.12.3"
ic-stable-structures = "0.7.2"
```

Declare the MemoryManager IDs your crate owns. A shared compile-time constant
keeps the explicit authority identical across the range and each key:

```rust,ignore
const MEMORY_AUTHORITY: &str = "icydb.test_db";

ic_memory::ic_memory_range!(authority = MEMORY_AUTHORITY, start = 120, end = 129);
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
            authority = MEMORY_AUTHORITY,
            key = "icydb.test_db.users.data.v1",
            ty = UsersStore,
            id = 120,
        )
        .expect("committed users memory")
    ));
}
```

Bootstrap once per concrete memory runtime before touching stable data:

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

The default runtime API is exported from the crate root. It is one
thread-local `MemoryRuntime<DefaultMemoryImpl>`, so every native thread owns an
independent backing memory, lifecycle, committed capability, and diagnostic
view. On IC Wasm, execution is single-threaded and the same TLS object naturally
has canister-instance lifetime.

Use helpers such as
`ic_memory::bootstrap_default_memory_manager()`,
`ic_memory::bootstrap_default_memory_manager_with_policy(...)`,
`ic_memory::committed_allocations()`,
`ic_memory::open_default_memory_manager_memory(...)`, and the macros shown
above; implementation modules are private.

The no-argument bootstrap helper uses ic-memory's built-in versioned
`PolicyIdentity`. A custom policy implements both `AllocationPolicy` and
`RuntimeBootstrapPolicy`. Its bounded identity contains a policy-family name,
a nonzero semantic version, and an optional caller-computed 32-byte
configuration digest. Change the version when policy semantics change and use
the digest when effective runtime configuration changes.

## Multi-Crate Composition

Every crate registers into the same linked declaration registry. Crates do not
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
            .expect("committed users memory")
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
            .expect("committed orders memory")
        ));
    }
}
```

The linked program seals one immutable, canonical declaration snapshot.
Bootstrap supplies that snapshot to the calling thread's default runtime,
recovers and commits that runtime's allocation ledger, and publishes committed
allocations into that runtime only. TLS-backed stores open when your code first
touches the `thread_local!`.

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

The committed allocation state is an in-memory capability published into one
runtime only after that runtime's stable-cell persistence succeeds. It is not a
serde payload and should not be treated as configuration.

## Explicit Runtimes

Frameworks and tests that own backing memory directly should use
`MemoryRuntime<M>` as the canonical API:

```rust,ignore
use ic_memory::{MemoryRuntime, sealed_declaration_snapshot};

let declarations = sealed_declaration_snapshot()?;
let mut runtime = MemoryRuntime::new(backing_memory)?;
runtime.bootstrap(&declarations, &policy)?;

let rows = runtime.open_memory("app.rows.v1", 120)?;
let diagnostics = runtime.diagnostic_export()?;
```

Construction is fallible. Empty backing memory is initialized as an
`ic-stable-structures` `MemoryManager`; nonempty backing memory must already
pass validation of the current `MGR` header, bucket table, and virtual/physical
extents. Foreign, unsupported, or corrupt metadata returns a typed error before
manager initialization can write. The read-only layout adapter is coupled to the
exact `ic-stable-structures = "=0.7.2"` dependency.
A pre-grown blank memory is nonempty and is rejected rather than assumed
disposable.

Each runtime owns all facts derived from `backing_memory`: recovery, ledger
cell, lifecycle, committed allocations, opens, diagnostics, and live sizes.
Multiple runtimes share only the immutable linked declaration snapshot. A
failed bootstrap publishes no capability, and repeated bootstrap on the same
runtime object is idempotent only when the snapshot and
`RuntimeBootstrapPolicy::runtime_bootstrap_identity()` match the successful
bootstrap. A changed snapshot or policy identity returns a typed error without
touching the ledger. Policy implementations should change their identity
whenever policy configuration or semantics change. This binding is
intentionally in-memory lifecycle and diagnostic state; it is not upgrade audit
history and is not persisted in the allocation ledger.

There is intentionally no public reset API. Native tests should construct a new
explicit runtime or use the naturally independent default TLS runtime; changing
global flags cannot reset a concrete stable-memory instance safely.

## Bounded physical allocation attribution

Use `runtime.memory_allocations()` or
`default_memory_manager_memory_allocations()` for an owned `MemoryAllocations`
report. Collection reads exactly 34,848 bytes of validated manager metadata and
returns all 255 usable IDs in order, including zero-size IDs and the ledger at
ID 0. It never decodes ledger history, initializes stores, writes, grows memory,
or advances a generation. The default helper refuses to construct a missing
runtime; an existing unbootstrapped runtime can report physical allocation with
unknown current bindings.

The report measures the actual persisted bucket size, physical and virtual
extents, assigned buckets, manager metadata, known current stable-key/owner
bindings, and unknown/unmanaged residuals. Virtual bytes are addressable extent,
not payload occupancy. `payload_bytes` is unavailable. Bucket slack is only
assigned bucket capacity beyond virtual extent. Conservation is explicit:

```text
physical bytes = manager metadata + assigned bucket bytes + unmanaged bytes
assigned bucket bytes = sum(per-ID bucket bytes)
                      = known binding bytes + unknown binding bytes
                      = virtual bytes + bucket slack
```

A current range claim does not prove historical ownership or grant access.
Retired/absent keys are explicitly unknown; the ledger's reserved ID is included
without reading its payload. Keep operator/controller authorization in the
integrating application. Full doctor/ledger diagnostics below still decode
history and are not substitutes for this bounded report.

## Bucket policy

Fresh runtimes retain the 128-page (8 MiB) default. `MemoryRuntime::new` honors
an existing same-release memory's actual setting. For an explicit setting use
`MemoryRuntime::new_with_config(memory, MemoryManagerConfig::new(pages)?)`; all
nonzero `u16` page counts are supported. Existing memory must match exactly or
construction fails before effects. Configuration is immutable for that runtime.

For a default runtime, select configuration through
`bootstrap_default_memory_manager_with_config(config, &policy)` before any
operation that constructs the runtime. Repeated explicit configuration must
match the established manager, independently of the allocation policy identity.
No bucket setting shrinks existing memory or migrates the durable format.

Open operations and macros return `RuntimeMemory<M>`, implementing `Memory` and
`Clone` without requiring `M: Clone`. Stable store type annotations must use
`ic_memory::RuntimeMemory<DefaultMemoryImpl>`. The runtime retains one private
shared backing for read-only attribution and owns exactly one manager.

The [CANIC-162 handoff](docs/canic162-memory-attribution.md) contains the exact
Canic integration example, reproducible measurements, capacity tradeoffs, and
limitations. Fixture evidence supports configurable smaller buckets, but does
not justify changing the default or selecting a Toko policy without live
attribution and a capacity assessment.

## Diagnostics

Use `default_memory_manager_doctor_report()` for operator-facing preflight and
runtime diagnostics. It returns a typed error if the default TLS runtime is
re-entered. Otherwise it can be called before or after bootstrap and reports the
stable-cell status, protected commit recovery state, recovered ledger export,
registered declarations, range authority, validation preflight, and live
`MemoryManager` slot sizes when they can be recovered. This no-argument entry
point evaluates the built-in policy. Integrations that bootstrap with a custom
policy should call
`default_memory_manager_doctor_report_with_policy(&policy)`, or call
`runtime.doctor_report(&declarations, &policy)` on an explicit runtime.

Doctor output includes the tested policy identity and sealed-declaration
fingerprint, the binding established by successful bootstrap, and a typed
binding comparison. Live size measurement is also per allocation: one invalid
slot is reported as a `DiagnosticMemorySizeOutcome::Failed` value without
discarding successful measurements for other slots.
Diagnostic failures carry stable `DiagnosticCode` values alongside their
human-readable messages for operator automation.

Use `default_memory_manager_commit_recovery_diagnostic()` when you only need
commit-slot presence and validity, the selected authoritative generation, and
any corruption or ambiguity error.

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
seal linked declarations
bootstrap once per memory runtime
only then open stable memory
```

Framework authors and policy adapters should read
[ADVANCED.md](https://github.com/dragginzgame/ic-memory/blob/main/ADVANCED.md).
The non-negotiable invariants are recorded in
[SAFETY.md](https://github.com/dragginzgame/ic-memory/blob/main/SAFETY.md). The
protocol whitepaper lives in
[whitepaper/src/SUMMARY.md](https://github.com/dragginzgame/ic-memory/blob/main/whitepaper/src/SUMMARY.md)
and builds as an mdBook with `make maintainer-build`.

`ic-memory` is early infrastructure extracted from Canic. It owns allocation
governance, not schema migration, endpoint routing, authorization, or data
semantics.
