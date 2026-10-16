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
and enforces exact successors; it is not an upgrade log. Matching warm bootstrap
continues to return its established capability without writing.

`AllocationLedger::records()` exposes retained records. `AllocationRecord::schema()`
returns latest metadata. `AllocationState::Retired` has no event payload.
`AllocationLedger::new(counter, records)` validates the one current shape;
protected recovery still establishes the physical/logical binding before any
open authority. Bootstrap and staging functions no longer take timestamps, and
resolved numeric declarations remain internal to the runtime. Hosts supply
`SealedDeclarationSnapshot` logical requests. Warm bootstrap and doctor compare
the exact canonical requests, allocation pool and policy identity through one
binding checker. No declaration fingerprint or diagnostic hash protocol remains.

The stable Cell value encodes `LedgerCommitStore` directly. Fallible
`decode_stable_cell_commit_store` and
`decode_stable_cell_commit_store_from_memory` return passive DTO state;
protected recovery remains internal to runtime bootstrap. The bounded
publication-byte type is separate from decoded state. There is no additional
Cell record DTO or serialized `store` wrapper.

Bootstrap validates and stages against one owned recovered ledger, applying each
checked claim once. No detached validation proof can be rebound to another
same-generation ledger. Doctor shares the per-declaration checks without staging;
protected commitment and persistence still precede allocation authority.

There is one bounded codec and integrity validator. Logical CBOR is limited to
64 KiB; the protected byte string contains that logical CBOR directly. The
enclosing protected record is limited to 64 KiB + 4 KiB. Recovery and serialization scale with record
count, not the number of upgrades. Stable allocation remains page/bucket rounded.

One checksummed current snapshot is retained. The physical `current` field and
`ICMDIRCT` marker identify the enclosing snapshot. Missing, corrupt or
incompatible current bytes never silently initialize a ledger. Recovery checks
the marker, checksum, bounded logical CBOR, ownership integrity and
physical/logical generation binding. A commit replaces the current record only after validating
its predecessor and exact successor. No previous payload or rollback history is
stored. The runtime persists the owning Cell before publishing allocation authority.
IC message rollback owns atomic stable writes; arbitrary native backends must
provide their own transaction boundary.

`CommitSnapshotDiagnostic::{Empty, Valid, Invalid}` reports the physical snapshot
directly. `Valid` carries the checksum-checked physical generation; `Invalid`
carries an untrusted observed generation. Logical decoding remains a separate
boundary. An invalid diagnostic never authorizes fallback or a reset.

Current measurements and rejection/upgrade checks are recorded in
[current qualification](current-ledger-qualification.md#0440-exact-binding-and-flattened-cell-qualification).

## Hard cut and retained installations

The maintainer authorized the 0.44.0 direct-CBOR hard cut and retirement of old
stable-memory images: fresh/reset memory is acceptable for this delivery.
The `ICMDIRCT` marker replaces the earlier physical identity; the nested logical
header, Cell record wrapper, format-version constant and envelope error API are removed. Unknown
markers return `CommitRecoveryError::UnsupportedFormat` before logical decoding.
This approval changes the supported source/fixture contract; no deployed
canister, real stable memory or backup is reset by this code change.

The manager/Cell layout and numeric slot encoding are unchanged. There is one
current CBOR decoder, no automatic reset, earlier-layout reader or migration engine.
Opaque payloads accept bounded byte strings; JSON output is diagnostic only and
its numeric arrays are not an alternate durable input format.
Current examples and fixtures use only the direct-CBOR contract. Consumers must
rebuild and select the new minor release together; historical qualification and
earlier images do not establish compatibility with this format.

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
