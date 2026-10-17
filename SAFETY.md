<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-memory/ic-memory-readme-header.svg" alt="IC Memory — Stops upgrades from mixing up stored data" width="100%">
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
- Schema admission, supported versions and data semantics belong to each storage
  owner. Allocation metadata does not carry a second schema-version authority.

## Runtime and lease custody

- One host-owned runtime manages each backing memory. Hosts must exclude other
  runtimes, managers and raw access to aliased backing until that runtime and
  all its handles have been dropped. Generic backing aliases are not detectable.
- A committed application allocation has at most one live `RuntimeMemory` lease
  within that runtime. Repeated opens refuse with a typed error before IO;
  authority checks and warm bootstrap do not clear outstanding leases.
- Handles are non-cloneable. Moving a handle into or out of a collection transfers
  its lease; dropping it releases the slot, including on construction failure
  or native unwinding. Forgetting a handle does not release it. A detached handle
  retains manager/backing custody after its runtime is dropped.
- Production collection integrations use the governed IC Storage facade. The
  public backing interfaces do not grant permission to create competing managers
  or wrap a lease in an alternate shared mutable route. This is a cooperative
  host contract, not a sandbox against arbitrary linked code.
- Lease state is volatile. No lease bits, new IDs or format fields are persisted.
  Cold reopening after all prior owners are gone retains the current stored bytes.

## Runtime IO Bounds

- Every runtime read, raw read and write must fit its current virtual extent,
  independently of bucket slack, cache state or build profile.
- Offset/length arithmetic must not wrap. Invalid spans panic on native hosts
  or trap in Wasm before accessing the backing memory.
- Empty IO is valid at offset zero in empty memory and at the end of grown
  memory. An empty span beyond the extent is still invalid.
- Raw reads retain the caller's pointer-safety obligations and initialize the
  destination on success. Backing failures may leave a partial destination;
  the wrapper must not add a zero-fill or intermediate buffer.

## Generation Invariants

- Validated capabilities are bound to exactly one committed ledger generation.
- Validation and staging share one owned recovered ledger; a checked claim must
  not be detached and applied to another ledger.
- Durable generation counters must never silently saturate or wrap.
- Physical commit generation must equal logical ledger generation.
- Counter zero is empty genesis. Nonzero counters need no audit record.
- A ledger retains at most 255 ownership records. Omitting or retiring an identity never frees its slot.

## Physical / Logical Binding

- The checksummed current snapshot and the logical allocation ledger must describe
  the same generation.
- A payload committed at physical generation `N` must decode to a ledger whose
  current generation is also `N`.
- A non-next logical generation must not be committed over the current ledger.
- Public physical DTOs and committed byte payloads are untrusted until they pass
  the recovery and ledger-integrity validation paths.

## Recovery Invariants

- Corrupt physical state fails closed.
- Ambiguous physical state fails closed.
- The one current snapshot must pass marker and checksum validation. Invalid
  present state cannot fall back to genesis or an older generation.
- Decoded ledger DTOs are untrusted until the explicit current-format
  discriminator and committed-integrity checks succeed.
- Stable-cell ledger storage used by every `MemoryRuntime` must pass fallible
  Cell-header and record decoding before recovery and admission, so corruption is
  classified as a bootstrap error instead of escaping as a decode panic.
  Capacity-checked writes use `ic-stable-structures::Cell` after successful
  validation. Fresh-cell initialization writes an empty protected store only.
- Each runtime's internal `ic_memory.*` governance allocations must stay
  recoverable in the durable ledger, but must not be published or opened through
  public application-memory helpers.
- Maintained recovery must enforce encoded-byte, collection and nesting
  limits before the corresponding untrusted allocation or decoding. Writers and
  readers must admit the same current bounded shape; oversized or corrupt state
  must never be replaced with an empty ledger.

## Validation-Before-Open Invariant

Storage integrations must validate layout before opening stable-memory handles:

1. Construct the runtime only after its raw backing memory is classified as
   empty or as the current `ic-stable-structures` `MemoryManager` layout.
2. Supply sealed key requests and the explicit host namespace/pool policy
   expected by the current binary, and recover the persisted allocation ledger.
3. Run host/consumer admission against bounded recovered metadata, then resolve
   requests while retaining existing assignments.
4. Validate the completed declarations against retained ownership and current policy.
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

Validation and staging operate on one owned recovered ledger. A checked claim
is local to that attempt, not a transferable proof for another ledger. Failure
discards the attempt before protected commitment and persistence.

The runtime stores application bindings and their generation only after its
stable-cell write succeeds. Callers cannot manually confirm persistence or
publish allocation-open authority. Borrowed declarations are passive views;
production inputs remain key-only requests and cannot select numeric IDs.

Integrations open storage only through that runtime after current-format
recovery, committed-ledger integrity, declaration validation, generation staging,
commit and durable persistence. Diagnostics, durable DTOs, copied declarations
and staged state are not open authority.

## Runtime Ownership Invariant

Every fact derived from a backing memory belongs to one `MemoryRuntime<M>`:

- `MemoryManager<Rc<M>>` and ledger persistence;
- bootstrap lifecycle and recovery result;
- committed application bindings and generation;
- memory-open authority;
- diagnostic ledger and commit-recovery view; and
- live virtual-memory sizes.

Each bootstrap attempt decodes its ledger record from persisted memory. A failed
attempt must not retain a cached record that could bypass decoding on retry.
Capability publication follows successful persistence.

One runtime must never use another runtime's lifecycle or committed bindings.
The host explicitly supplies an immutable checked snapshot of component key
requests and metadata. There is no process-global declaration registry.
Bootstrap is once per concrete runtime, not once per process.

The host explicitly constructs and retains one runtime for each concrete
backing memory. Memory owns no singleton or TLS lifecycle. Each native worker
must bootstrap its own host before component adoption or touching thread-local
stable stores. Hosts pass the same runtime or opened handles to components;
observations cannot construct a missing runtime or choose its bucket geometry.
There is no public reset operation: process flags cannot replace durable facts.

A successful bootstrap binds the runtime to the exact canonical requests,
allocation pool and `PolicyIdentity`. Matching warm bootstrap does not replay
admission or write another generation; changed bindings reject. Doctor checks
the same exact binding without publishing authority. These are in-memory
lifecycle facts, not a durable ownership history or hash protocol.
The policy identity is a host-issued 32-byte token covering its complete prepared
semantics/configuration. Hosts own derivation, labels and fallible setup; every
token byte participates in equality. It is not authentication or persisted data.

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
34,848 bytes of validated manager metadata, without decoding retained ownership.
They distinguish current, ledger and unknown bindings and preserve physical,
bucket, virtual, slack and unmanaged-byte conservation. Virtual extent is not
payload occupancy; reported pool eligibility is not historical ownership or access
authority.

## Retirement Invariants

- A retired stable key cannot be declared again.
- A retired slot cannot be claimed by a different stable key.
- Retirement requires the stable key and slot to match the historical allocation
  record.
- Retirement retains the stable key and slot. No
  per-retirement event or generation history is required.
- Tombstones are preserved for rollback safety, diagnostics, and historical ABI
  integrity.

## Integrity Boundary

The one checksummed current snapshot is serialized inside
`ic-stable-structures::Cell`. IC message execution provides atomic stable-memory
commit and rollback; native backing memory must provide its own transaction
boundary. A present invalid snapshot fails closed. The checksum detects
accidental corruption and provides no adversarial tamper resistance,
authenticity or authorization.

Public durable structs are DTOs. Decoded, deserialized, and diagnostic values
are untrusted until the relevant recovery, current-format, integrity,
validation, or commit path has accepted them.

Serde decoding alone does not grant allocation authority. `StableKey`,
`MemoryManagerSlot` enforce their identity syntax and usable bounds during
construction and decoding. Keys remain canonical and slot IDs exclude sentinel
255. Allocation declarations combine checked keys and slots.
`MemoryAuthority` checks its printable ASCII owner and valid namespace prefix,
including on decode. `MemoryAllocationPool` canonicalizes checked exclusion IDs into one fixed mask
and rejects overlapping namespaces; governance is always excluded.
The internal resolved snapshot checks declaration count and unique keys and
slots; external hosts supply checked logical request snapshots. Retained claims, current namespace grants, physical
eligibility and application policy still require their validation boundaries
before influencing authority. Previous owner labels are not durable policy.

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
