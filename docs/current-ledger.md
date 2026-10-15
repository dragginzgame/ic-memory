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
record is limited to 64 KiB + 4 KiB. Recovery and serialization scale with record
count, not the number of upgrades. Stable allocation remains page/bucket rounded.

One checksummed current snapshot is retained. The physical `current` field and
`ICMESNAP` marker identify the enclosing snapshot. Missing, corrupt or
incompatible current bytes never silently initialize a ledger. Recovery checks
the marker, checksum, logical envelope, ownership integrity and physical/logical
generation binding. A commit replaces the current record only after validating
its predecessor and exact successor. No previous payload or rollback history is
stored. The runtime persists the owning Cell before publishing allocation authority.
IC message rollback owns atomic stable writes; arbitrary native backends must
provide their own transaction boundary.

`CommitStoreDiagnostic.current` reports `CommitSnapshotDiagnostic::{Empty, Valid,
Invalid}`. Its `recovery` result carries the checked physical generation or its
error; logical decoding remains a separate boundary. `InvalidSnapshot` reports
an untrusted observed generation. It never authorizes fallback or a reset.

Current measurements and rejection/upgrade checks are recorded in
[single-snapshot qualification](single-snapshot-qualification.md).

## Hard cut and retained installations

The 0.41.0 hard cut encodes `MemoryManagerSlot` as a checked integer instead of
nested tagged maps, and removes transient declaration display labels. The
logical family magic remains `ICMEMLED`, with distinct marker `ICMN` and version
1. Earlier `ICMS` logical payloads are incompatible; their discriminator is never
reused to interpret the new layout. The physical `current`/`ICMESNAP` checksum
representation introduced in 0.39.0 is unchanged, as are the pinned manager
layout and application collection bytes. Changed logical payloads necessarily
change snapshot lengths and checksums.

Current fixture producers, readers, examples and source callers use numeric
slots. Update raw diagnostic DTO consumers to read numeric slot fields and
remove declaration label arguments; the ordinary key-request/pool contract is
unchanged. Canic's source callers and retained-data review are tracked in
[Canic #573 — Adopt Memory numeric slot and label API hard cut after publication](https://github.com/dragginzgame/canic/issues/573).
Historical [logical qualification](current-ledger-qualification.md) and
[physical qualification](single-snapshot-qualification.md) retain their original
source/format bindings and do not qualify this cut. There is one current decoder,
with no earlier-layout reader, automatic reset or migration engine.

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
