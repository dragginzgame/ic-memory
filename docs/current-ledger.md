# Current ownership ledger

The ledger retains what is needed to prevent stores from opening one another's
memory: each stable key, permanently claimed ID, current lifecycle state and
latest diagnostic schema metadata. It has at most 255 records, including the
runtime's governance allocation. Omitted identities retain their records;
retirement tombstones the identity and ID permanently. Changing a memory ID is
rejected rather than interpreted as a migration.

No per-bootstrap event, schema history, first/last observation, retirement time,
commit timestamp or linked-code fingerprint is stored. These did not contribute
to ownership validation. The single `current_generation` counter still advances
on cold bootstrap, reservation and retirement. It binds validation to a commit
and rejects stale proofs; it is not an upgrade log. Matching warm bootstrap
continues to return its established capability without writing.

`AllocationLedger::records()` exposes retained records. `AllocationRecord::schema()`
returns latest metadata. `AllocationState::Retired` has no event payload.
`AllocationLedger::new(counter, records)` validates the one current shape;
protected recovery still establishes the physical/logical binding before any
open authority. Bootstrap and staging functions no longer take timestamps, and
`DeclarationSnapshot` contains declarations only. The separate sealed-declaration
fingerprint remains a transient binding between a runtime and its declarations.
Its values change with the simplified snapshot shape; regenerate diagnostic
expectations rather than treating an earlier value as a current binding.

There is one bounded codec and integrity validator. Logical CBOR is limited to
64 KiB; each opaque payload adds its 24-byte envelope. The enclosing protected
record is limited to 128 KiB + 4 KiB. Recovery and serialization scale with record
count, not the number of upgrades. Stable allocation remains page/bucket rounded.

Two checksummed commit slots remain. They detect corrupt/ambiguous persisted
state and preserve the current commit protocol. They are not an application audit
log or rollback facility: any present corrupt slot fails closed. Both snapshots
are serialized inside one stable Cell. IC message rollback owns atomic stable
writes; arbitrary native files are not made crash atomic by this protocol.

## Hard cut and retained installations

Pending 0.27.0 changes public APIs, diagnostic JSON and the durable logical ledger.
The family magic remains `ICMEMLED`; the current format marker is `ICMS`, version
1. The former discriminator is unsupported before its logical data is decoded.
There is no earlier-layout reader, automatic reset or migration engine.

Current fixture producers, readers, examples and source callers are updated
inside this repository. External framework/application callers must remove
history and timestamp APIs, consume current record/schema fields, and regenerate
their own current DTO fixtures. The underlying pinned MemoryManager layout and
application collection bytes are unchanged by this cut, but that does not make
an existing ledger directly readable.

Before installing on a canister with an earlier ledger, the maintainer must
identify the retained installation and choose its explicit data disposition.
A fresh install has no earlier ledger. A whole-canister reinstall/reset discards
application data too and requires its owner's authorization and backup/recovery
plan. Preserving existing application data requires a separately reviewed,
application-owned transition that carries every retained key/ID/tombstone into
current state and preserves application lifecycle obligations. The generic import
boundary accepts checked current DTOs only; this release implements no converter.
Keep the earlier binary and recovery evidence until that transition is qualified.

Do not erase ID 0 or initialize genesis over an incompatible ledger: that would
forget ownership while leaving collection bytes behind. An unsupported-format
or bounded-decoding error is the expected safe result until the installation's
disposition is resolved. Large earlier records can exceed the new outer ceiling
before their inner format is inspected. No retained installation or downstream
deployment has been qualified as part of this local change.
