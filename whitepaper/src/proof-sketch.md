# Proof Sketch

The proof sketch states what the protocol must preserve. The Lean model in the
next chapter encodes these facts over a compact abstraction, but it does not
prove that the Rust implementation preserves the facts across every code path.
That obligation is carried by the Rust recovery, validation, staging, commit,
and test suite.

## Stable Key Has A Unique Slot

Assume a ledger `L` satisfies no stable-key movement. If records `r1` and `r2`
are both in `L` and `r1.key = r2.key`, then `r1.slot = r2.slot`.

This is direct application of the no stable-key movement invariant. The
interesting implementation obligation is therefore not the theorem itself, but
ensuring every recovery, validation, reservation, staging, and commit path
preserves the invariant before producing authority.

## Physical Slot Has A Unique Stable Key

Assume a ledger `L` satisfies no physical-slot reuse. If records `r1` and `r2`
are both in `L` and `r1.slot = r2.slot`, then `r1.key = r2.key`.

This is direct application of the no physical-slot reuse invariant. The result
prevents the swapped-ID failure: two distinct keys cannot both own the same
stable-memory slot.

## Open Authority Is Ledger-Backed

Let `A` be a post-commit open authority tied to committed ledger `L`. If `A`
permits opening stable key `k` at slot `s`, then `L` contains an active
allocation record for `k` at `s`.

By model construction, a post-commit open authority contains only declarations
whose matching allocation records are active in the committed ledger. Opening
resolves a stable key to one of those declarations, so the corresponding active
ledger record exists.

## Generation Advancement Is Monotone

If staging succeeds at generation `g`, the staged generation is `g + 1`, and:

$$
g < g + 1
$$

The generation counter advances by successor in the model. The Rust
implementation has additional failure cases for stale validated generations,
declaration-count limits, broken parent links, unknown record-generation
references, and `u64` overflow; this theorem should not be read as a
verification of those checks.
