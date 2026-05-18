# Safety Invariants

`ic-memory` is persistent allocation-governance infrastructure. Future changes
must preserve these invariants on every recovery, commit, validation, and
staging path.

## Allocation Invariants

- `stable_key -> allocation_slot` is forever.
- `allocation_slot -> stable_key` is forever.
- Retired allocations cannot revive.
- Omitted historical declarations are preserved, not retired.
- A reserved allocation can become active only after full declaration
  validation.

## Generation Invariants

- Validated sessions are bound to exactly one ledger generation.
- Durable generation counters must never silently saturate.
- Physical commit generation must equal logical ledger generation.
- Committed generation history must form a strict parent-linked chain.

## Recovery Invariants

- Corrupt physical state fails closed.
- Ambiguous physical state fails closed.
- Identical duplicate physical slots at the same generation are recoverable
  deterministically.
- A newer corrupt slot cannot override an older valid slot.

## Integrity Boundary

The protected physical checksum is only torn-write and accidental-corruption
detection. It is not cryptographic integrity and does not provide adversarial
tamper resistance.

Public durable structs are DTOs. Decoded or manually constructed values are
untrusted until the relevant compatibility and integrity validation has
succeeded.
