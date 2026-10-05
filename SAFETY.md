<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/ic-memory/main/images/ic-memory-readme-header.svg" alt="IC Memory — Internet Computer helper library" width="100%">
</p>

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
  history contains only absent or nonzero versions, checked at construction
  and decoding. This metadata does not validate application schemas.

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
- Decoded ledger DTOs are untrusted until the explicit current-format
  discriminator and committed-integrity checks succeed.
- Stable-cell ledger storage used by every `MemoryRuntime` must pass fallible
  envelope and record decoding before recovery and admission, so corruption is
  classified as a bootstrap error instead of escaping as a decode panic.
  Capacity-checked writes use `ic-stable-structures::Cell` after successful
  validation. Fresh-cell initialization writes empty protected slots only.
- Each runtime's internal `ic_memory.*` governance allocations must stay
  recoverable in the durable ledger, but must not be published or opened through
  public application-memory helpers.
- Maintained recovery must enforce encoded-byte, collection, nesting and history
  limits before the corresponding untrusted allocation or decoding. Writers and
  readers must admit the same current bounded shape; oversized or corrupt state
  must never be replaced with an empty ledger.

## Validation-Before-Open Invariant

Storage integrations must validate layout before opening stable-memory handles:

1. Construct the runtime only after its raw backing memory is classified as
   empty or as the current `ic-stable-structures` `MemoryManager` layout.
2. Supply the sealed declarations, logical requests and explicit host grants
   expected by the current binary, and recover the persisted allocation ledger.
3. Run host/consumer admission against bounded recovered metadata, then resolve
   requests while retaining existing assignments.
4. Validate the completed declarations against ledger history and current policy.
5. Stage and durably persist one new allocation generation.
6. Publish committed authority, then open application stable-memory handles.

`MemoryRuntime::new()` performs the first step and is fallible. It rejects
nonempty foreign or unsupported manager bytes before calling
`MemoryManager::init()`, because that dependency initializes a new manager
header when its magic is absent.

Runtime policy implementations also provide an explicit
`RuntimeBootstrapPolicy` identity. Repeated bootstrap is accepted only when
that identity and the sealed declaration snapshot match the successful
bootstrap, preventing a later policy argument from being silently ignored.
Matching warm bootstrap and consumer authority verification must not replay
admission, advance the generation or replace the host's policy or bucket size.
Historical selections of unknown or retired keys, or without current grant and
policy authorization, must reject before persistence. Naming a recovered key
never grants an open capability.

Opening stable-memory handles before validation defeats the purpose of this
crate.

## Capability Boundary

`ValidatedAllocations` is an opaque, pre-commit validation result. It must not be
deserializable, default-constructible, or publicly constructible, and it must
not be accepted by allocation-opening APIs.

`CommittedAllocations` is the in-memory open capability. It must not be
deserializable, default-constructible, or publicly constructible. A
`MemoryRuntime<M>` may store it only after that runtime's stable-cell write
succeeds. Generic persistence owners may confirm it only after durably writing
the pending `PendingBootstrapCommit` state.

Integrations may open storage only from `CommittedAllocations` produced after
current-format recovery, committed-ledger integrity, declaration validation,
generation staging, commit, and durable persistence. Diagnostics, durable DTOs,
and `ValidatedAllocations` are not open authority.

## Runtime Ownership Invariant

Every fact derived from a backing memory belongs to one `MemoryRuntime<M>`:

- `MemoryManager<Rc<M>>` and ledger persistence;
- bootstrap lifecycle and recovery result;
- committed allocation capability;
- memory-open authority;
- diagnostic ledger and commit-recovery view; and
- live virtual-memory sizes.

Each bootstrap attempt decodes its ledger record from persisted memory. A failed
attempt must not retain a cached record that could bypass decoding on retry.
Capability publication follows successful persistence.

One runtime must never use another runtime's lifecycle or committed capability.
The only process-global authority is the immutable canonical snapshot of linked
declarations, ranges, and metadata. Bootstrap is once per concrete runtime, not
once per process.

The default convenience layer is one thread-local runtime. On native targets,
each thread therefore has independent default memory and runtime state. On
single-threaded IC Wasm, the TLS runtime naturally has canister-instance
lifetime. There is no public reset operation because resetting process flags
cannot reset or replace the concrete backing memory that owns durable facts.
Each native worker must bootstrap its host before consumer adoption or touching
thread-local stable stores. Default readiness, open, ID-resolution, adoption and
diagnostic helpers must observe existing state without constructing an absent
runtime or choosing its bucket geometry.

A successful runtime bootstrap also binds that runtime in memory to a validated
`PolicyIdentity` and deterministic sealed-declaration fingerprint. Those values
prevent a repeated call from silently substituting different policy semantics
or declarations, and doctor reports expose the same binding. They are
diagnostic lifecycle metadata, not persisted ledger authority or upgrade audit
history. The snapshot fingerprint is non-cryptographic.

## Growth and Physical Accounting

`RuntimeMemory::grow` must reserve physical backing capacity before the manager
assigns buckets. Typed refusal preserves virtual extents and manager metadata
and permits retry; only the required substrate `Memory::grow` adapter translates
failure to `-1`. A manager refusal or unexpected previous extent after successful
preflight violates the private growth-accounting invariant and must panic. Native
backing panics or partial writes are outside this refusal guarantee. All handles
share the runtime's assigned-bucket count.

Zero-page growth must check the shared reservation for reentry, then return the
current extent without backing IO or manager mutation.

Physical reports and numeric summaries must remain read-only and bounded to
34,848 bytes of validated manager metadata, without decoding ledger history.
They distinguish current, ledger and unknown bindings and preserve physical,
bucket, virtual, slack and unmanaged-byte conservation. Virtual extent is not
payload occupancy; a reported range claim is not historical ownership or access
authority.

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

Serde decoding alone does not grant allocation authority. `StableKey`,
`MemoryManagerSlot` and `MemoryManagerIdRange` enforce their identity syntax and
usable bounds during construction and decoding. Keys remain canonical, slot IDs
exclude sentinel 255, and ranges remain ordered with usable ends. `SchemaMetadata`
contains an absent or nonzero schema version, established by construction and
decoding. `AllocationDeclaration` checks optional printable ASCII labels at
construction and decoding, including their 256-byte bound. Fingerprints, range
metadata, duplicate claims, policy and ledger history still require their
validation boundaries before influencing authority.

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
