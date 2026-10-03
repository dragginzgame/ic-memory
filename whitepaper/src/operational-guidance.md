# Operational Guidance

Use `ic-memory` when a canister has multiple stable stores, generated stores,
framework-owned stores, or plugin-provided stores that may evolve across
releases.

Keep stable keys stable. If the durable store is the same, preserve the key and
update schema metadata. Changing the key declares a new allocation identity.

The normal integration pattern is:

1. declare explicit host ranges with `ic_memory_range!`; use `mode = Allowed`
   when they supply fresh logical placements,
2. register logical requests with `ic_memory_declaration!`, or fixed stores
   through `ic_memory_key!`,
3. compose consumer preparation in the host's policy when recovered identity or
   historical journal selection is needed,
4. bootstrap that host from `init` and `post_upgrade` with its selected policy
   and bucket configuration before any stable store is touched,
5. verify consumer requirements against committed host authority, then open
   stores by key or expected fixed ID.

For a standalone host using the built-in policy and default bucket geometry,
`ic_memory::bootstrap_default_memory_manager()` performs bootstrap. A configured
host uses `bootstrap_default_memory_manager_with_config(config, &policy)`.
Libraries adopt the already bootstrapped host through authority verification;
they do not bootstrap a second time with a different policy.

`ic_memory_key!` is safe in a `thread_local!` definition because the actual
stable-memory open happens when the value is first touched. The macro returns a
typed open result; the TLS initializer should propagate or explicitly handle
that result. Bootstrap must run before the first touch.

Exactly one layer should bootstrap a given ledger store. Framework stacks
should compose declarations into that owner, or use distinct ledger stores and
allocation domains.
Distinct domains require genuinely distinct backing memory. A second manager
over the same IC stable memory is not an independent allocation domain.

The canonical owner is `MemoryRuntime<M>`. It owns the `MemoryManager`, ledger
cell, bootstrap lifecycle, committed capability, opens, and diagnostics for one
backing memory. Linked crates share only one immutable
`SealedDeclarationSnapshot`; every runtime independently recovers and persists
its own ledger.

The default convenience runtime is thread-local. Native threads therefore have
independent default memory instances and must each bootstrap their own runtime.
Each worker performs configured host bootstrap before consumer adoption and
before touching any thread-local stable-store initializer. Database readiness
alone does not initialize unrelated host stores. The
[composed-host example](https://github.com/dragginzgame/ic-memory/blob/main/examples/composed_host.rs)
exercises this order and two cold reopens over one backing.
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
An omitted key also loses current open authority. Historical inspection requires
explicit declaration or host-authorized selection during admission, before
commitment; diagnostics never authorize an open.

Schema metadata is optional diagnostic metadata. Use it to record the in-place
store schema version that a generation declared, but keep application migration
logic outside `ic-memory`.

For operator diagnostics,
`MemoryRuntime::doctor_report(&snapshot, &policy)` reports stable-cell status,
protected commit recovery state, recovered ledger export, registered
declarations, range authority, validation under the tested policy, and live
memory sizes for that runtime when recovery succeeds. It also compares the
tested policy identity and declaration fingerprint with the binding established
by successful bootstrap. Size failures are reported per allocation without
discarding successful measurements. The default-runtime wrapper returns a typed
TLS access error if it is re-entered. Failure states include stable diagnostic
codes for automation as well as human-readable messages.
All default observations leave a missing runtime untouched; diagnostics return
typed `NotBootstrapped` without selecting bucket size or sealing declarations.
For recovery inspection before configured bootstrap, construct an owned runtime
with the chosen geometry and use its doctor/recovery methods. Doctor validation
does not run consumer preparation or certify admission.

For recurring metrics, use `memory_allocation_summary`; protected per-ID
inspection can use `memory_allocations`. Both read 34,848 bytes of validated
manager metadata without decoding ledger history, writing or growing memory.
Physical allocation, virtual extent, bucket slack and unmanaged residuals are
distinct from payload occupancy, which remains unavailable.

Direct `RuntimeMemory::grow` returns `Result<u64, RuntimeGrowError>` and reserves
backing capacity before bucket assignment. The substrate `Memory` trait adapter
alone converts failure to `-1`. Handle typed refusal at the owning layer;
native backing panics and partial writes do not provide IC message rollback.

Monitor durable history as well as physical memory. Cold bootstraps, reservations
and retirements consume generations; matching warm calls do not. The 65,536
generation ceiling gives over 179 years at one generation per day, while byte
limits and schema churn can exhaust the ledger earlier. There is no compaction
or tombstone reuse; exhaustion is typed and requires an explicit future format
or import decision.
