# Allocation Invariants

## No Stable-Key Movement

For all records `r1` and `r2` in `L`:

$$
r_1.key = r_2.key \Rightarrow r_1.slot = r_2.slot
$$

Once a stable key is assigned to a slot, that key cannot later name another
slot.

## No Physical-Slot Reuse

For all records `r1` and `r2` in `L`:

$$
r_1.slot = r_2.slot \Rightarrow r_1.key = r_2.key
$$

Once a physical slot is assigned to a stable key, the slot cannot be reused for
a different stable key.

## Retirement Is A Tombstone

For all `k` in `K` and `s` in `S`:

$$
\mathsf{RetiredAt}(L,k,s) \Rightarrow
\neg \mathsf{ActiveAt}(L,k,s)
$$

Retirement is not a free-list operation. It preserves historical ABI facts for
rollback safety and diagnostics.

## Post-Commit Open Authority

In the abstract model, a memory handle is opened through a post-commit
authority tied to a committed ledger. If that authority permits opening `k` at
`s`, the committed ledger contains an active record for `k` at `s`:

$$
\mathsf{MayOpen}(A,k,s) \Rightarrow \mathsf{ActiveAt}(L,k,s)
$$

For the Rust default runtime, this corresponds to publishing
`CommittedAllocations` only after the staged ledger generation has been
persisted. Manual integrations carry this as an ordering obligation.

## Generation Monotonicity

Successful staging advances the committed ledger generation by exactly one in
the abstract model:

$$
g' = g + 1 \quad \text{and therefore} \quad g < g'
$$

The Rust implementation refines this with explicit checks for stale validated
generations, declaration-count bounds, and `u64` overflow.

## Committed Generation Chain

Committed generation records form a strict parent-linked chain:

$$
generation_i = i \quad \land \quad parent_i = i - 1
$$

The first real staged generation uses parent `0`. A committed ledger whose
current generation is nonzero must contain the generation record for that
current generation, and every allocation record generation reference must point
to a known generation record.

## Schema Metadata Is Diagnostic

Schema metadata attached to declarations, reservations, and allocation records
must validate before it is persisted. It may help a framework diagnose or route
migration decisions, but `ic-memory` does not use it as proof that application
bytes can be decoded under a new schema.
