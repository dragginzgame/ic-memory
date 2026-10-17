# Current ownership ledger

The ledger retains each permanent stable key, claimed numeric ID and current
reserved/active/retired state. It holds at most 255 records, including governance.
Omitted and retired identities retain their records and slots. Store schema
versions, migrations and schema admission belong to each storage owner, such as
IcyDB's schema subsystem; Memory carries no parallel schema metadata.

One checked `current_generation` counter binds logical ownership to physical
commitment and advances on cold bootstrap. Matching warm bootstrap returns the
established capability without another write or admission replay. There is no
per-bootstrap event, timestamp, ownership-label history or upgrade log.

Fixture-only `testing::AllocationLedger::new(counter, records)` checks count,
unique key/ID ownership and empty-genesis rules under `test-support`. Decoded
ledgers and `testing::LedgerCommitStore` are passive DTOs; recovered authority
remains internal. Production callers use their runtime's recovery/diagnostics.
The stable Cell contains the commit store directly. Its required `current` field
holds the snapshot without an intermediate physical-store object.
The private commit-store readers admit bounded current bytes; protected recovery
still establishes integrity and counter binding. Raw readers are exposed only
under `testing` for explicit fixtures and do not grant open authority.

Bootstrap owns one recovered ledger through validation and staging. It protects
the staged commit, persists the owning Cell, then constructs committed allocation
authority. There is no intermediate persistence-confirmation token or public
manual commit path. Any failure before persistence publishes no new authority.

Logical ownership is direct bounded CBOR, limited to 64 KiB. The enclosing
protected record is limited to 64 KiB + 4 KiB. The one `current` snapshot uses
`ICMSNAP1`; marker, counter, payload length and bytes are checksum-bound. Opaque
payloads are byte strings, and JSON is diagnostic output rather than an alternate
snapshot input. No additional logical envelope is stored.

Recovery checks the marker/checksum, bounded logical CBOR, ownership integrity
and physical/logical counter binding. Present invalid or incompatible state
cannot initialize genesis or fall back to another generation. Publication checks
exact successors. IC message execution owns stable-write rollback; arbitrary
native backing needs its own transaction boundary.

Physical diagnostics remain `CommitSnapshotDiagnostic::{Empty, Valid, Invalid}`.
Only `Valid` carries a checksum-checked generation, and logical ledger recovery
remains a separate boundary. Diagnostics do not grant allocation authority.

## Host configuration and component adoption

Components contribute `MemoryRequest::new(authority, key)` and use the host's
committed runtime. Warm binding and adoption check keys, current owner grants,
policy token and the canonical allocation pool. Hosts supply a 32-byte token
identifying all policy semantics/configuration and prepare fallible policy setup
before bootstrap. Labels, versions and token derivation stay with the host; Memory
compares the complete bytes without allocating a policy name. This token is not
persisted. These checks do not check application schemas or reinterpret stored collection bytes. The policy has one
`validate_allocation(key, slot)` callback for each external declaration; recovered
admission remains separate and is never replayed by doctor or warm bootstrap.

`MemoryAllocationPool::new(grants, excluded_ids)` accepts usable numeric IDs,
checks the domain, includes governance 0..=9, and stores one immutable mask.
`excluded_ids()` projects canonical ascending IDs; serialized pools contain
`excluded_ids`, not ranges. Equivalent orderings and duplicates have the same
binding. Canic or another host owns range grouping for presentation/configuration.
This pool policy is not durable ownership and changes no retained key/ID mapping.

## Hard cut and retained installations

The maintainer approved retirement of earlier ledger images for the schema-free
0.45 cut: fresh/reset memory is acceptable. `ICMSNAP1` replaces `ICMDIRCT` for the
incompatible record shape. Incompatible enclosing shapes fail during record
decoding; otherwise an unsupported marker rejects with
`CommitRecoveryError::UnsupportedFormat`. Numeric slots and the pinned upstream
manager/Cell layout remain unchanged. There is one current reader and fixture
set; no old-format decoder, converter or automatic reset is included.

This approval changes source/fixture support. It resets no deployed canister,
real stable memory or backup. Consumers must rebuild on the new minor, adopt the
key-only request and numeric exclusion APIs, and retain their own schema checks.
Each application owns its retained-data disposition and installation qualification.

Before installing over earlier bytes, identify the installation and its explicit
disposition. A whole-canister reinstall also discards application data and needs
its owner's authorization. Preserving retained data requires a separately
reviewed application transition carrying every key/ID/tombstone and lifecycle
obligation into the current contract. Never clear only ID 0 while leaving
collection bytes behind. Keep earlier binaries and recovery evidence until the
application's disposition is qualified.

See [current qualification](current-ledger-qualification.md) for source-bound
measurements and installed checks. Earlier-format results do not establish
compatibility with this cut.
