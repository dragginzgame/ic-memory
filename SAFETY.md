# Safety Invariants

`ic-memory` is stable-memory allocation-governance infrastructure. Future
changes must preserve these invariants on every recovery, validation, staging,
commit, and session-opening path.

These invariants are about allocation ABI safety. They do not prove
store-schema compatibility, application-level data validity, controller
authorization, or endpoint safety.

## Non-Negotiable Allocation Invariants

- `stable_key -> allocation_slot` is forever.
- `allocation_slot -> stable_key` is forever.
- Once a stable key has been assigned to a slot, that key must never point to a
  different slot.
- Once a slot has been assigned to a stable key, that slot must never be reused
  for a different key.
- Retired allocations cannot revive.
- Retirement is a tombstone, not a free-list operation.
- Omitted historical declarations are preserved, not implicitly retired.
- A reserved allocation can become active only after full declaration
  validation.

## Generation Invariants

- Validated sessions are bound to exactly one committed ledger generation.
- Staging a validated generation must reject stale validated sessions whose
  generation no longer matches the current ledger.
- Durable generation counters must never silently saturate or wrap.
- Physical commit generation must equal logical ledger generation.
- Committed generation history must form a strict parent-linked chain.
- A committed ledger with a nonzero current generation must contain the matching
  generation record.

## Physical / Logical Binding

- The physical protected commit slot and the logical allocation ledger must
  describe the same generation.
- A payload committed at physical generation `N` must decode to a ledger whose
  current generation is also `N`.
- A non-next logical generation must not be committed over the current ledger.
- Public physical DTOs and committed byte payloads are untrusted until they pass
  the recovery and ledger-integrity validation paths.

## Recovery Invariants

- Corrupt physical state fails closed.
- Ambiguous physical state fails closed.
- Dual-slot recovery must not select an authoritative generation when the slots
  disagree in a way the recovery rules cannot prove safe.
- Identical duplicate physical slots at the same generation are recoverable
  deterministically.
- A newer corrupt slot cannot override an older valid slot.
- Recovered ledgers are untrusted until compatibility and committed-integrity
  checks succeed.

## Retirement Invariants

- A retired stable key cannot be declared again.
- A retired slot cannot be claimed by a different stable key.
- Retirement requires the stable key and slot to match the historical allocation
  record.
- Retired records must carry a retired generation, and non-retired records must
  not carry one.
- Tombstones are preserved for rollback safety, diagnostics, and historical ABI
  integrity.

## Integrity Boundary

The protected physical checksum is only torn-write and accidental-corruption
detection. It is non-cryptographic and does not provide adversarial tamper
resistance, authenticity, or authorization.

Public durable structs are DTOs. Decoded, deserialized, diagnostic, or manually
constructed values are untrusted until the relevant recovery, compatibility,
integrity, validation, or commit path has accepted them.
