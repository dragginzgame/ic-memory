# Safety Invariants

`ic-memory` is stable-memory allocation-governance infrastructure. Future
changes must preserve these invariants on every recovery, validation, staging,
commit, and allocation-opening path.

These invariants are about allocation ABI safety. They do not prove
store-schema compatibility, application-level data validity, controller
authorization, or endpoint safety.

## Non-Negotiable Allocation Invariants

- Once a stable key is committed to a physical allocation slot, future binaries
  must either reopen that same stable key on that same slot or declare a new
  stable key.
- The same active stable key cannot move to a different physical slot.
- The same active physical slot cannot be reused by a different stable key.
- Once a stable key has been assigned to a slot, that key must never point to a
  different slot.
- Once a slot has been assigned to a stable key, that slot must never be reused
  for a different key.
- Retired allocations cannot revive.
- Retirement is a tombstone, not a free-list operation.
- Omitted historical declarations are preserved, not implicitly retired.
- A reserved allocation can become active only after full declaration
  validation.
- A reservation is policy/diagnostic staging only until it is declared as an
  active allocation. Refreshing a matching reservation is allowed; reserving an
  already active or retired allocation is rejected.
- Schema metadata attached to declarations, reservations, and committed schema
  history must pass `SchemaMetadata::validate()`.

## Generation Invariants

- Validated capabilities are bound to exactly one committed ledger generation.
- Staging a validated generation must reject stale validated capabilities whose
  generation no longer matches the current ledger.
- Durable generation counters must never silently saturate or wrap.
- Physical commit generation must equal logical ledger generation.
- Committed generation history must form a strict parent-linked chain.
- A committed ledger with a nonzero current generation must contain the matching
  generation record.

## Physical / Logical Binding

- The checksummed commit slot and the logical allocation ledger must describe
  the same generation.
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
- Identical duplicate commit slots at the same generation are recoverable
  deterministically.
- Every present commit slot must pass marker and checksum validation. Recovery
  must not discard an invalid slot and fall back to an older generation because
  doing so could forget committed allocation history.
- Recovered ledgers are untrusted until the explicit current-format
  discriminator and committed-integrity checks succeed. A recognized obsolete
  format is rejected as unsupported; it is never decoded through a legacy path.
- Stable-cell ledger storage used by the default runtime must be preflighted
  before opening it through `ic-stable-structures::Cell`, so envelope or record
  corruption is classified as a bootstrap error instead of escaping as a decode
  panic.
- The default runtime's internal `ic_memory.*` governance allocations must stay
  recoverable in the durable ledger, but must not be published or opened through
  public application-memory helpers.

## Validation-Before-Open Invariant

Storage integrations must validate layout before opening stable-memory handles:

1. Recover the persisted allocation ledger.
2. Declare the stores expected by the current binary.
3. Validate those declarations against ledger history and framework policy.
4. Commit the new allocation generation.
5. Only then open stable-memory handles using committed allocation authority.

Opening stable-memory handles before validation defeats the purpose of this
crate.

## Capability Boundary

`ValidatedAllocations` is an opaque, pre-commit validation result. It must not be
deserializable, default-constructible, or publicly constructible, and it must
not be accepted by allocation-opening APIs.

`CommittedAllocations` is the in-memory open capability. It must not be
deserializable, default-constructible, or publicly constructible. The default
runtime may produce it only after its stable-cell write succeeds. Generic
persistence owners may confirm it only after durably writing the pending
`PendingBootstrapCommit` state.

Integrations may open storage only from `CommittedAllocations` produced after
current-format recovery, committed-ledger integrity, declaration validation,
generation staging, commit, and durable persistence. Diagnostics, durable DTOs,
and `ValidatedAllocations` are not open authority.

## Retirement Invariants

- A retired stable key cannot be declared again.
- A retired slot cannot be claimed by a different stable key.
- Retirement requires the stable key and slot to match the historical allocation
  record.
- The retired lifecycle state carries its committed retirement generation
  directly; a retired record without that generation is unrepresentable.
- Tombstones are preserved for rollback safety, diagnostics, and historical ABI
  integrity.

## Integrity Boundary

The redundant commit slots are serialized together inside the default
`ic-stable-structures::Cell`; they are not independently atomic physical
writes. ICP message execution provides atomic stable-memory commit and rollback.
If the enclosing record remains decodable, the slot checksum can detect
accidental corruption. Any present invalid slot fails closed; the other slot is
not a rollback path. The checksum is non-cryptographic and does not provide
adversarial tamper resistance, authenticity, or authorization.

Public durable structs are DTOs. Decoded, deserialized, and diagnostic values
are untrusted until the relevant recovery, current-format, integrity,
validation, or commit path has accepted them.

Serde decode is not validation. Constructor-backed invariants such as stable-key
grammar and `MemoryManager` slot descriptor rules must be rechecked by the
validation boundary before decoded values influence allocation authority.

Invariant-bearing DTO fields are intentionally private where feasible. Callers
should use checked constructors and accessors instead of fabricating durable
allocation state directly.

## Non-Goals

`ic-memory` does not provide:

- cryptographic tamper resistance
- malicious-controller protection
- endpoint authorization
- application schema migration correctness
- stable data semantic validation
- IC management-canister lifecycle safety
- full disaster recovery
