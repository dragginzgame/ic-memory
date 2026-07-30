# Operational Guidance

Use `ic-memory` when a canister has multiple stable stores, generated stores,
framework-owned stores, or plugin-provided stores that may evolve across
releases.

Keep stable keys stable. If the durable store is the same, preserve the key and
update schema metadata. Changing the key declares a new allocation identity.

The normal integration pattern is:

1. declare ranges with `ic_memory_range!`,
2. register application stable structures through `ic_memory_key!`,
3. call `ic_memory::bootstrap_default_memory_manager()` from `init` and
   `post_upgrade` before any application stable structure is touched.

`ic_memory_key!` is safe in a `thread_local!` definition because the actual
stable-memory open happens when the value is first touched. The macro returns a
typed open result; the TLS initializer should propagate or explicitly handle
that result. Bootstrap must run before the first touch.

Exactly one layer should bootstrap a given ledger store. Framework stacks
should compose declarations into that owner, or use distinct ledger stores and
allocation domains.

The canonical owner is `MemoryRuntime<M>`. It owns the `MemoryManager`, ledger
cell, bootstrap lifecycle, committed capability, opens, and diagnostics for one
backing memory. Linked crates share only one immutable
`SealedDeclarationSnapshot`; every runtime independently recovers and persists
its own ledger.

The default convenience runtime is thread-local. Native threads therefore have
independent default memory instances and must each bootstrap their own runtime.
IC Wasm execution is single-threaded, so the default TLS runtime naturally has
canister-instance lifetime. Bootstrap means once per runtime, not once per
process, and there is no public reset API.

The default runtime reserves `MemoryManager` IDs `0..=9` and stable keys under
`ic_memory.*` for allocation-governance records. The internal ledger allocation
is retained in durable history for recovery, but it is not published as
application-openable memory.

Omitting a historical declaration does not retire or free its key or slot.
Explicit retirement creates a tombstone and still does not make the slot
reusable for a different stable key.

Schema metadata is optional diagnostic metadata. Use it to record the in-place
store schema version that a generation declared, but keep application migration
logic outside `ic-memory`.

For operator diagnostics, `MemoryRuntime::doctor_report(&snapshot)` reports
stable-cell status, protected commit recovery state, recovered ledger export,
registered declarations, range authority, validation preflight, and live
memory sizes for that runtime when recovery succeeds. The default-runtime
wrapper returns a typed TLS access error if it is re-entered. Failure states
include stable diagnostic codes for automation as well as human-readable
messages.
