# Operational Guidance

Use `ic-memory` when a canister has multiple stable stores, generated stores,
framework-owned stores, or plugin-provided stores that may evolve across
releases.

Keep stable keys stable. If the durable store is the same, preserve the key and
update schema metadata. Changing the key declares a new allocation identity.

The normal integration pattern is:

1. declare ranges with `ic_memory_range!`,
2. register application stable structures through `ic_memory_key!`,
3. call `bootstrap_default_memory_manager()` from `init` and `post_upgrade`
   before any application stable structure is touched.

`ic_memory_key!` is safe in a `thread_local!` definition because the actual
stable-memory open happens when the value is first touched. Bootstrap must run
before that first touch.

Exactly one layer should bootstrap a given ledger store. Framework stacks
should compose declarations into that owner, or use distinct ledger stores and
allocation domains.

Omitting a historical declaration does not retire or free its key or slot.
Explicit retirement creates a tombstone and still does not make the slot
reusable for a different stable key.

Schema metadata is optional diagnostic metadata. Use it to record the in-place
store schema version that a generation declared, but keep application migration
logic outside `ic-memory`.
