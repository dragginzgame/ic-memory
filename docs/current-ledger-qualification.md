# Current ownership ledger qualification

The original 0.27.0 hard-cut evidence was captured after released
`62e15131c6145fa66a77528d96b00dcbf64a892c` (0.26.2), while the manifest still
declared 0.26.2. Later reviews and experiments identify their own source below.
These are local Linux source, behavior and footprint observations; they do not
establish publication or retained-installation qualification. The
[current contract](current-ledger.md) owns consumer API and deployment requirements.

## Post-release physical-slot audit

Read-only implementation review of released 0.27.0 at
`f2aefd140bc24c7befa6f170c2e2e25df2af667c`, on 2026-10-06. This review changes no
runtime code or persisted format and does not relabel the earlier measurements.

- `physical.rs::select_authoritative_slot` checks both complete payload checksums
  and rejects any present invalid slot. It never restores an older valid snapshot
  after corruption. `commit_payload_with_generation` replaces the inactive slot
  after checking the predecessor; the old complete payload remains serialized.
- `runtime/mod.rs::bootstrap_unbootstrapped` synchronously stages the whole record
  and persists it with one `Cell::new` before confirming/publishing allocations.
  Upstream Cell 0.7.2 serializes that complete value and writes the header and value;
  it does not separately commit the two slots. The
  [IC interface specification](https://docs.internetcomputer.org/references/ic-interface-spec/canister-interface/#entry-points)
  owns trap rollback, including the complete upgrade sequence. Returning an ordinary
  error is distinct from trapping, and custom native Memory backends must not be
  assumed transactional.
- `stable_cell.rs` reads the entire encoded value and serde owns both payload
  buffers before authoritative logical recovery. Retaining the predecessor
  therefore adds serialized bytes, allocation/read work and checksum scanning.
  The current valid-two-slot fixture is 444 bytes: its predecessor slot value is
  210 bytes, including a 146-byte logical envelope; its current slot is 211 bytes.
  These are byte counts from the canonical fixture, not a measured replacement
  format or a claim of 210 bytes saved by a final design.
- No distinct atomicity or fallback-recovery benefit for two slots was found in
  the default runtime. The advanced public DTOs, slot diagnostics and corruption
  errors remain observable contracts. Arbitrary framework-owned persistence is a
  separate boundary; a single snapshot would not make partial native writes safe.

The smallest plausible next runtime change is one checksummed current snapshot,
retaining the checked counter, bounded decoding, logical integrity, permanent
ownership/tombstones and persistence-before-publication. It would require a
**0.28.0 hard cut**, current callers/fixtures/diagnostics, an explicit incompatible
physical-format identity and retained-installation disposition. No dual reader,
rollback history or configurable slot count is justified.

Before accepting that cut, compare 0/1/64/255-record encode/reopen/commit workloads
for heap allocations, encoded bytes, stable IO and matched Wasm artifacts; then
qualify corruption, counter overflow, growth refusal, stale proofs and installed
upgrade rollback. IC instruction savings still require installed measurements.
Ordinary small ledgers already fit one virtual page, and manager buckets round
physical allocation: lower encoded bytes do not establish fewer allocated pages
or buckets. The existing bounded two-slot implementation remains supported until
a replacement is separately implemented and qualified.

## Single-snapshot prototype measurements

2026-10-06, against released 0.27.1 at
`6e98b07882ed43d180195f5f915eafaef05f25db`. Both controls were exported from that
commit into isolated source directories under
`target/qualification/single-snapshot/`, with the same selected root lockfile
copied explicitly. The maintained library, manifests, lockfiles and current
fixtures are unchanged. This experiment is not a prepared release.

The candidate changes four source modules: `physical.rs`, `ledger/mod.rs`,
`lib.rs` and `constants.rs`. One `SnapshotCommitStore::current` replaces the
two-slot DTO and selection logic. It keeps checked counters, the bounded opaque
payload, one checksum, logical-envelope/integrity recovery and the existing
persist-before-publication runtime flow. The candidate physical marker is
`ICMESNAP`, distinct from the released `ICMEMCOM`; the logical envelope is
unchanged. The enclosing decode ceiling becomes 64 KiB + 4 KiB. Slot-specific
diagnostics/errors are replaced with a current-record diagnostic/corruption error.
There is no earlier-format reader or optional storage mode.

### Native encoding, recovery and stable IO

Linux x86-64, Rust 1.99.0, standard release profile, locked/offline, one test
thread. Both sources use the identical disposable `snapshot_measurements.rs`
harness and the existing metered VectorMemory. Controls have two successful
commits with the same retained records, counters 1 and 2, maximum schema versions
and reserved lifecycle state. These are steady-state comparisons: a newly
initialized two-slot store has less predecessor data. The 255-record long-key
control uses 128-byte keys; the others use `app.storeNNN.v1` keys.

| Records / key control | Released encoded bytes | Candidate encoded bytes | Released reopen peak heap | Candidate reopen peak heap |
| --- | ---: | ---: | ---: | ---: |
| 0 | 266 | 144 | 374 | 198 |
| 1 | 466 | 244 | 774 | 398 |
| 64 | 13,150 | 6,586 | 29,534 | 14,778 |
| 255 / short | 51,732 | 25,877 | 117,268 | 58,645 |
| 255 / maximum | 109,872 | 54,947 | 240,944 | 120,483 |

Reopen includes the bounded Cell read, outer decode and logical recovery. Heap
counts are System allocation requests on this thread, excluding existing inputs
and backing memory; every measured result is dropped and live measured bytes
return to zero. They are not RSS, Wasm linear-memory consumption or IC costs.
Removing one opaque buffer saves one allocation per reopen, not half the number
of allocations: the 255-record short-key case is 301 -> 300 allocations, with
reallocations 6 -> 3. Owned strings and logical record validation still dominate
allocation count. Canonical encoding uses one allocation in both controls.

The reopen-and-commit phase reads and recovers the stored record, commits a prepared
counter-3 ledger and writes it through `Cell::new`. It excludes declaration
admission and creation of the prepared ledger. Peak requested heap for the
255-record short-key case is 133,652 -> 81,920 bytes; maximum keys use
240,968 -> 131,096 bytes. This phase still performs two stable reads and four
writes. Short-key read bytes and write bytes each fall from 51,740 to 25,885;
maximum keys fall from 109,880 to 54,955. Every same-width replacement fits its
existing capacity, with no measured growth. No timing claim is made.

Full rows: [native allocations and IO](measurements/single-snapshot-native.csv).

### Page and bucket controls

Separate fresh MemoryManager controls persist the same canonical records into
ID 0 with bucket sizes 1 and 128 pages. This measures the substrate's allocation
rounding; it is not an admitted application runtime or an installed canister.

| 255-record control | Bucket pages | Released virtual pages | Candidate virtual pages | Released physical pages | Candidate physical pages |
| --- | ---: | ---: | ---: | ---: | ---: |
| Short keys | 1 | 1 | 1 | 2 | 2 |
| Short keys | 128 | 1 | 1 | 129 | 129 |
| Maximum keys | 1 | 2 | 1 | 3 | 2 |
| Maximum keys | 128 | 2 | 1 | 129 | 129 |

The maximum-key record saves one virtual page and, with one-page buckets, one
physical page on fresh allocation. Both controls still allocate one default
128-page bucket. Existing grown memory cannot be shrunk by writing a smaller
record, so these observations do not establish reclamation on retained stores.
Full rows: [bucket controls](measurements/single-snapshot-buckets.csv).

### Matched Wasm artifact

Both sources use Rust 1.99.0, the identical runtime-integration probe, selected
lockfile, standard release profile and `--emit=asm`. Independent build directories
prevent reuse of an artifact compiled from the other source.

| Source | Raw Wasm bytes |
| --- | ---: |
| Released 0.27.1 control | 374,986 |
| Single-snapshot candidate | 371,104 |
| Reduction | 3,882 (1.04%) |

This is one matched raw probe, not the full Wasm budget gate or a consumer binary.
It does not establish IC instruction/cycle savings. The earlier 0.27.0 result
above retains its own source binding; it is not relabelled as this baseline.

### Boundaries, limitations and reproduction

The two native workloads pass on Rust 1.99.0 and MSRV 1.88.0. Four candidate
integration tests also pass on both toolchains: payload/counter refusal leaves
the snapshot unchanged; corruption or the released marker cannot initialize or
commit; physical/logical counter disagreement rejects; and the current record
round-trips while the actual released Cell fixture is refused. Repeated current
commits retain only the current record. These are focused prototype checks.

The candidate's existing owner-local tests and fixture producers have not been
converted to the new contract. Full library/caller/compile-fail qualification,
growth refusal and publication behavior, installed upgrade rollback, supported
native macOS execution and IC costs remain unqualified for this candidate. The
prototype is not suitable for direct adoption. Retained installations and
external consumers have not been changed or retired.

These measurements support a tightly scoped **0.28.0 hard cut** replacing the
physical DTO, diagnostics, current fixtures and outer bound together. Keep the
logical ledger, tombstones, checksum and checked counter; introduce no migration
engine, compatibility reader or storage configuration. Production adoption must
finish that propagation and qualification and resolve retained-data disposition.
The current 0.27 contract remains supported. This investigation introduces no
pending release entry or package version change.

For each `variant` of `baseline` and `candidate`, from the owning repository:

```bash
CARGO_TARGET_DIR="$PWD/target/qualification/single-snapshot/$variant-build" \
  cargo +1.99.0 test --locked --offline --release \
  --manifest-path "target/qualification/single-snapshot/$variant/Cargo.toml" \
  --test snapshot_measurements -- --test-threads=1 --nocapture
CARGO_TARGET_DIR="$PWD/target/qualification/single-snapshot/$variant-build" \
  cargo +1.99.0 rustc --locked --offline --release \
  --manifest-path "target/qualification/single-snapshot/$variant/Cargo.toml" \
  --target wasm32-unknown-unknown \
  --example wasm-runtime-integration-size-probe -- --emit=asm
```

The candidate additionally runs `--test snapshot_boundaries`. MSRV native checks
use `+1.88.0` and the separate `candidate-msrv-build` directory. All source copies,
the candidate source diff, harnesses, logs, binaries and Wasm artifacts are
retained under `target/qualification/single-snapshot/`. These are local evidence,
not package artifacts or release receipts. Failed lockfile/harness attempts are
preserved. An initial candidate attempt sharing the baseline build directory
reused the baseline binary; its identical rows are discarded. All candidate rows
above come from its independent build directory.

Tree hashes use sorted relative `src` paths and their SHA-256 lines, hashed again.

| Input / artifact | SHA-256 |
| --- | --- |
| Baseline source tree | `ddf8eeaa5892b6dd23f775f0de8202b95034eb59ffd4b8cf9ea718bd31fa44f2` |
| Candidate source tree | `6bf68d2ca8c99d90b5d4f0f90fd1fa4a9174d690379bd973f48e57e7786551e2` |
| Unchanged manifest | `68387104f7bc35ca55eee123fea79a528f10bf7bc85f57bd1d964dc8ba96c4c9` |
| Selected lockfile, both controls | `83627663ed770d5dc84a7c4f58fb5121491e2588f68cace773ba41e80f792b60` |
| Candidate source diff | `980955d930068e17150e0702f9866158f76317b1dde2da061921d600a50d26be` |
| Common measurement harness | `4c74901640e9bf10f40c92b161d63c55da44a64c73b4b79e030ef32d269e183b` |
| Candidate boundary harness | `27a3325ca9daf34a92edd00a5154080de3392b4337043e8c37153f47d0a2991b` |
| Runtime-integration probe | `4767d041e455b89e2ddbdd76b4129b2f9a64ed029f237f54aeb15d0eeef2b8bc` |
| Baseline native measurement binary | `e6b3c73e43a1c09e3be9dc2edb06835abbada627a244b61ea8d019ebb3d73420` |
| Candidate native measurement binary | `e888df2b551f65c213d645328a7777c869cb1e55babf3c65f3151e0f969de677` |
| Baseline Wasm | `55449f4947345131a63e54cf5bc2f90967a7770d414ffdf207056c69e13ffb33` |
| Candidate Wasm | `faf32342e8d636ff86623d46e5ddd58bf87047b612e52dc87dc655041f675e98` |
| Native CSV | `a7a1d054100afe29e12bb8d5915e4dd2eb0baec11893aa0d287d85b62ea09c37` |
| Bucket CSV | `64c200cf89d2ac8f372e1ca74a00acd4251ad170290282c8af776a550be91191` |

## Overengineering review and resulting cut

| Mechanism | Evidence and disposition |
| --- | --- |
| Per-bootstrap generation trail | Former `ledger/record.rs` appended parent, count, timestamp and runtime-fingerprint data on every cold bootstrap. `claim.rs` and `registry.rs` resolve ownership from retained key/ID/state records, not that trail. Delete the vector, DTO, accessors, chain validator and generation-count ceiling. |
| Per-schema audit trail | Former `AllocationRecord::observe_schema` appended schema changes. Admission consumed only the final schema entry. Store one `SchemaMetadata`; refresh it in `stage.rs`. |
| Observation/retirement generations | First/last observation and retirement generation fields served audit validation; ownership conflicts require the retained identity and lifecycle state. Remove the fields and use unit retirement state. |
| Runtime fingerprint and commit time | These were optional audit inputs propagated through declaration/capability/bootstrap layers. They granted no authority. Remove their APIs, field validation and timestamp plumbing. Keep the separate sealed-declaration fingerprint: it binds a live runtime to current declarations. |
| History wrapper and two integrity modes | The wrapper primarily grouped records with an audit vector. Strict integrity enforced a contiguous audit chain after structural checks. Put records directly on the ledger and keep one current ownership/genesis validator. DTO recovery remains a real independent trust boundary. |
| Ownership records and tombstones | `claim.rs` rejects moved keys, reused slots and retired identities. `registry.rs` marks every retained record occupied, including omitted stores. Keep these facts; dropping them would make existing bytes reusable under a new meaning. |
| Commit counter and protected slots | Counter equality rejects stale proofs; `physical.rs` checks next commits, checksums and ambiguous/corrupt slots. Keep the established bounded physical protocol in this cut. Both slots live inside one Cell: they are neither independent atomic writes nor an audit/rollback facility. IC message rollback remains the write-atomicity owner. |
| 32,768-entry bucket table | This is the pinned upstream MemoryManager's bucket ownership table, not a list of stores or upgrades. Reading it validates manager metadata. The pending local array replaces a temporary heap buffer; it adds no persistent table. |

The resulting model adds no mode, registry, cache, migration engine or optional
legacy decoder. Bootstrap still stages, persists and publishes in the same order.
Cold bootstrap still advances the counter for unchanged declarations; introducing
a separate no-op protocol is outside this cut. Bounded lookups remain simple;
there is no proven need for another index over 255 identities.

The two-slot substrate remains an advanced public recovery contract. A single
checksummed snapshot would be a separate protocol change, not merely deleting an
audit vector. The current slots should not be justified as independently atomic
storage. Ordinary small ledgers fit one page with both snapshots; removing one
snapshot would not necessarily save a page or manager bucket.

## Current format and bounds

The logical payload carries `ICMEMLED`, marker `ICMS`, version 1 and a 24-byte
header. Logical CBOR contains `current_generation` and `records`; each record has
`stable_key`, `slot`, unit `state` and current `schema`. Earlier discriminators
fail before logical decoding; initialization cannot replace incompatible storage.

Logical CBOR is bounded to 65,536 bytes, each opaque envelope to 65,560 bytes,
and the outer stable-cell value to 135,168 bytes. These replace 16 MiB logical and
32 MiB + 4 KiB outer ceilings. The lower ceilings are refusal bounds, not claimed
per-canister savings. There are at most 255 retained identities. A worst-case
set with 128-byte keys, maximum nonzero schema versions, longest state encoding
and near-maximum counters fits the writer/reader limits, including both snapshots.

No application collection, MemoryManager format, dependency selection, toolchain
or raw stable-memory addressing primitive changes. The former logical ledger is
incompatible. External consumers and retained deployments still need the explicit
disposition described in the current contract; no deployed canister was reset.

## Encoded fixture footprint

Compare the representative envelope fixtures in the pre-cut snapshot with the
regenerated current fixtures. Key, ID, lifecycle, latest schema and counter are
preserved for these four controls; the obsolete audit fields disappear.

| Envelope | Before bytes | Current bytes |
| --- | ---: | ---: |
| Empty genesis | 87 | 54 |
| One active store, counter 1 | 334 | 146 |
| One reserved store, schema 1 | 343 | 155 |
| One retired store, counter 2 | 435 | 147 |

The one-active protected store is 423 → 234 bytes. All current fixture bytes
come from `LedgerCommitStore::commit` and its canonical writer, rather than an
independent encoder. Current fixture tests recover and re-encode them. The
regenerated two-slot controls cover newer authority and corrupt-newer refusal;
they are not presented as matched footprint controls.

Repeated schema churn and commits cross counter 65,536 while retaining one record
and the latest schema. The regression runs 1,024 public bootstrap commits, verifies
their counters/current schema after recovery, and keeps each logical envelope below
200 bytes and the final two-slot record below 600 bytes. CBOR integer width can
grow by at most eight bytes per counter; no event is appended. Serialization and
recovery now depend on record count, not upgrade count. No timing or IC instruction
speedup is inferred from that complexity change.

## Native allocation measurements

Linux x86-64, Rust 1.99.0, selected root lockfile, standard Cargo release profile,
one test thread. The [maintained harness](../crates/ic-memory/tests/allocation_measurements.rs)
counts this thread's System allocation requests. Existing input/fixture memory is
excluded; measured results are dropped and the counter verifies zero live bytes
at each boundary. This is requested heap, not RSS or IC instructions/cycles.

Current workloads use 0, 1, 64 and 255 records with maximum schema versions, at
counters 1, 1,024 and `u64::MAX`. Recovery/validation allocation counts and peak
heap stay unchanged across those counters: one-record recovery uses 3 allocations,
zero reallocations and 181 peak bytes per call; one-record validation uses one
allocation and 104 peak bytes. Empty-record recovery/validation allocate nothing.
Encode sizes may vary slightly with counter width. Work still grows with retained
record count. Timings are noisy and are not a performance claim.

The diagnostics controls reproduce zero heap allocations for numeric summaries
on empty/populated managers before/after bootstrap. Detailed reports allocate
3 times before bootstrap and 13 after; peak requested heap remains 36,748 and
36,838 bytes. The [buffer prototype evidence](allocation-diagnostics-qualification.md)
records the matched heap-to-stack tradeoff separately.

The malformed-text controls now use stable keys, not retired fingerprint fields.
Valid 128-byte keys recover; 129-byte keys fail grammar validation. Larger
advertised text fails CBOR preflight before owned-string allocation. This exercises
the maintained rejection path; it is not a comparison with the earlier fingerprint
workload. Full rows are in
[current-ledger-native-allocations.csv](measurements/current-ledger-native-allocations.csv).

## Focused Wasm comparison

Use the exact command of the retained baseline: Rust 1.99.0, selected root lockfile,
`wasm32-unknown-unknown`, standard release profile and assembly emission. The
unchanged runtime-integration probe exercises bootstrap, memory IO and diagnostics.

| Source | Raw Wasm bytes | Delta from released 0.26.2 |
| --- | ---: | ---: |
| Released 0.26.2 | 451,284 | — |
| Buffer-only 0.26.3 prototype | 450,697 | −587 |
| Current 0.27.0 cut | 374,988 | −76,296 (16.9%) |

The ledger cut contributes 76,296 − 587 = 75,709 fewer bytes relative to the
buffer-only prototype in this matched probe. This is a raw artifact comparison;
it is not the full `wasm-size` profile/budget gate or a consumer binary measurement.
No installed IC instruction/cycle cost is measured.

An intermediate plain `cargo build` produced 380,672 bytes. A same-source control
reproduced it; its emission flags differ from the retained baseline command.
The earlier 70,612-byte subtraction used that plain output. Use the matched
assembly-emitting row above for the recorded comparison. Both artifacts and
commands are retained; do not mix their hashes or pretend the commands are identical.

```bash
cargo +1.99.0 rustc --locked --offline --release --target wasm32-unknown-unknown \
  --example wasm-runtime-integration-size-probe -- --emit=asm
```

## Focused behavior and limits

- Rust 1.99.0 and MSRV 1.88.0 pass the library behavior suite and selected metadata,
  runtime and composed-host callers. Same-format cold reopen preserves application
  data. Omission, key movement, slot reuse, reservation and permanent retirement
  retain their typed behavior.
- Worst-case current metadata round-trips; malicious counts, malformed text,
  corrupted/ambiguous slots, physical/logical mismatch and unknown nested fields
  fail closed. Unsupported discriminators reject before logical decode and cannot
  initialize over existing storage.
- New validated-counter overflow coverage proves `u64::MAX` rejection without
  mutating the commit store; stale validation remains the first refusal. Existing
  reservation/retirement overflow tests remain. Fresh and existing growth refusal
  publish no mapping and retry deterministically. The existing-commit fixture now
  contains enough current records for a genuine page-growth refusal.
- Compile-fail fixtures are updated and rerun without overwrite. They still test
  private proof construction, precommit/admission authority and pending-commit use,
  rather than prohibiting retired names. README doctests pass with warnings denied.
- Strict library/caller/test Clippy, current formatting and vendored snapshot
  checks pass. Selected manifests and lockfiles remain unchanged.

Full validation, full Wasm budgets, package/release gates, installed PocketIC/IC
behavior, native macOS execution and external Canic/IcyDB caller/deployment
qualification are not established by these local checks. A current-format native
reopen does not prove an earlier-format deployment transition. No upstream rules
or tooling snapshot were rewritten.

## Reproduction and retained evidence

```bash
cargo +1.99.0 test --locked --offline --lib --test diagnostic_metadata \
  --test memory_runtime --test compile_fail --example composed_host -- --test-threads=1
cargo +1.88.0 test --locked --offline --lib --test diagnostic_metadata \
  --test memory_runtime --example composed_host -- --test-threads=1
cargo +1.99.0 test --locked --offline --release --test allocation_measurements \
  -- --ignored --test-threads=1 --nocapture
cargo +1.99.0 clippy --locked --offline --lib --tests -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo +1.99.0 test --locked --offline --doc
make verify-shared-tooling fmt-check
```

Local logs retain prefix `/tmp/ic-memory-0.27.0-`, including failed fixture,
compile, growth-fixture, lint and malformed-text attempts and their successful
retries. The pre-cut source, tests, examples and fixture snapshot remains at
`/tmp/ic-memory-0.27.0-before`. The matched raw Wasm is retained as
`/tmp/ic-memory-0.27.0-wasm-runtime-probe-matched.wasm`; the flags control is
`/tmp/ic-memory-0.27.0-wasm-runtime-probe-flags-control.wasm`. The baseline and
buffer-prototype artifacts retain their earlier qualification bindings. These
local temporary paths are evidence, not package artifacts or release receipts.

| Final input/artifact | SHA-256 |
| --- | --- |
| Sorted src tree | `a5bd933a320421df3de173a8910984673980997f1dd1df73a3070393fb8818a5` |
| Cargo.toml | `246c49df609c2e3e8ade9c0b8f1156bbf4b19194163984ff3468c2889b2ebe73` |
| Selected Cargo.lock | `68c9efd0e2b98c57451bf41b83c973a8bf86eaca4a66fa8d30373225dd86e655` |
| Native measurement harness | `75154b133d3ff2b3c53803f6cc5f8962795f87890de446c46ec709f8a789ab5a` |
| Native measurement binary | `6998ba3b182802df36d9cf8dd2ec7aa50daf50c64bab9ad8383d92ad6f8c93e1` |
| Probe source | `4767d041e455b89e2ddbdd76b4129b2f9a64ed029f237f54aeb15d0eeef2b8bc` |
| Matched probe artifact | `a0ef62b4c87a4bf979a1aa89b5e64595c1f6f1cfa91d5357b144a249238d2135` |

The source-tree digest concatenates each recursively sorted `src` file's relative
path, NUL, contents, and NUL. The explicit file list and bindings are retained in
`/tmp/ic-memory-0.27.0-source-bindings.json`. Earlier logs are not relabelled as
final-source evidence.

## Removed symbol inventory

This lists exact former names against the pre-cut source snapshot, including
private helpers and tests. The replacements column distinguishes renamed tests
and methods from deletion of the superseded audit mechanism. Methods with the
same name and fewer arguments are changed contracts, not deleted symbols.

| Former file | Removed name | Reason / current replacement |
| --- | --- | --- |
| `src/capability.rs` | `ValidatedAllocations::runtime_fingerprint` | Retired audit metadata; no audit replacement. Current records/latest schema retain allocation safety. |
| `src/capability.rs` | `CommittedAllocations::runtime_fingerprint` | Retired audit metadata; no audit replacement. Current records/latest schema retain allocation safety. |
| `src/cbor.rs` | `deserialize_history` | Retired audit metadata; no audit replacement. Current records/latest schema retain allocation safety. |
| `src/declaration.rs` | `DeclarationSnapshot::with_runtime_fingerprint` | Retired audit metadata; no audit replacement. Current records/latest schema retain allocation safety. |
| `src/declaration.rs` | `DeclarationSnapshot::runtime_fingerprint` | Retired audit metadata; no audit replacement. Current records/latest schema retain allocation safety. |
| `src/declaration.rs` | `DeclarationSnapshot::into_parts` | DeclarationSnapshot::into_declarations (returns declarations only) |
| `src/declaration.rs` | `validate_runtime_fingerprint` | Retired audit metadata; no audit replacement. Current records/latest schema retain allocation safety. |
| `src/declaration.rs` | `tests::snapshot_rejects_unbounded_runtime_fingerprint` | Retired audit-only invariant; current ownership, limits, corruption and overflow tests remain. |
| `src/ledger/integrity.rs` | `AllocationLedger::validate_staging_bounds` | AllocationLedger::validate_bounds and checked counter advancement |
| `src/ledger/integrity.rs` | `AllocationLedger::validate_committed_integrity` | AllocationLedger::validate_integrity (one integrity contract) |
| `src/ledger/integrity.rs` | `validate_record_integrity` | Retired generation/schema-history validation; current validate_integrity checks ownership and genesis. |
| `src/ledger/integrity.rs` | `validate_schema_history_integrity` | Retired generation/schema-history validation; current validate_integrity checks ownership and genesis. |
| `src/ledger/integrity.rs` | `tests::reference` | Retired audit-only invariant; current ownership, limits, corruption and overflow tests remain. |
| `src/ledger/integrity.rs` | `tests::generation_validation_matches_reference_for_orderings_and_overlapping_failures` | Retired audit-only invariant; current ownership, limits, corruption and overflow tests remain. |
| `src/ledger/mod.rs` | `tests::allocation_history_accessors_expose_read_only_views` | tests::allocation_record_accessors_expose_read_only_views (renamed; current contract tested) |
| `src/ledger/mod.rs` | `tests::generation_record_constructor_rejects_empty_runtime_fingerprint` | Retired audit-only invariant; current ownership, limits, corruption and overflow tests remain. |
| `src/ledger/mod.rs` | `tests::cbor_ledger_codec_rejects_unknown_nested_history_fields` | Retired audit-only invariant; current ownership, limits, corruption and overflow tests remain. |
| `src/ledger/mod.rs` | `tests::cbor_ledger_codec_rejects_unknown_nested_generation_fields` | Retired audit-only invariant; current ownership, limits, corruption and overflow tests remain. |
| `src/ledger/mod.rs` | `tests::cbor_ledger_codec_requires_generation_optional_fields` | Retired audit-only invariant; current ownership, limits, corruption and overflow tests remain. |
| `src/ledger/mod.rs` | `tests::stage_validated_generation_records_schema_metadata_history` | tests::stage_validated_generation_replaces_current_schema_metadata (renamed; current contract tested) |
| `src/ledger/mod.rs` | `tests::stage_validated_generation_records_runtime_fingerprint` | Retired audit-only invariant; current ownership, limits, corruption and overflow tests remain. |
| `src/ledger/mod.rs` | `tests::new_committed_requires_strict_generation_history` | Retired audit-only invariant; current ownership, limits, corruption and overflow tests remain. |
| `src/ledger/mod.rs` | `tests::new_committed_preserves_structural_rejection_before_history_checks` | Retired audit-only invariant; current ownership, limits, corruption and overflow tests remain. |
| `src/ledger/mod.rs` | `tests::validate_integrity_rejects_retirement_before_last_observation` | Retired audit-only invariant; current ownership, limits, corruption and overflow tests remain. |
| `src/ledger/mod.rs` | `tests::validate_integrity_rejects_retirement_before_allocation` | Retired audit-only invariant; current ownership, limits, corruption and overflow tests remain. |
| `src/ledger/mod.rs` | `tests::validate_integrity_requires_schema_history_at_first_generation` | Retired audit-only invariant; current ownership, limits, corruption and overflow tests remain. |
| `src/ledger/mod.rs` | `tests::validate_integrity_rejects_schema_history_after_last_observation` | Retired audit-only invariant; current ownership, limits, corruption and overflow tests remain. |
| `src/ledger/mod.rs` | `tests::validate_integrity_rejects_non_increasing_schema_history` | Retired audit-only invariant; current ownership, limits, corruption and overflow tests remain. |
| `src/ledger/mod.rs` | `tests::malformed_generation_fingerprints_reject_as_codec_errors_and_preserve_slots` | Retired audit-only invariant; current ownership, limits, corruption and overflow tests remain. |
| `src/ledger/mod.rs` | `tests::validate_committed_integrity_requires_current_generation_record` | Retired audit-only invariant; current ownership, limits, corruption and overflow tests remain. |
| `src/ledger/mod.rs` | `tests::committed_integrity_rejects_allocation_references_to_genesis` | tests::integrity_rejects_nonempty_genesis_before_commit (renamed; current contract tested) |
| `src/ledger/mod.rs` | `tests::validate_committed_integrity_rejects_generation_history_gaps` | Retired audit-only invariant; current ownership, limits, corruption and overflow tests remain. |
| `src/ledger/mod.rs` | `tests::committed_integrity_rejects_a_skipped_parent_in_contiguous_history` | Retired audit-only invariant; current ownership, limits, corruption and overflow tests remain. |
| `src/ledger/mod.rs` | `tests::history_boundary_round_trips_and_rejects_next_generation_without_mutation` | Retired audit-only invariant; current ownership, limits, corruption and overflow tests remain. |
| `src/ledger/mod.rs` | `tests::record_count_boundary_and_malicious_history_reject_on_recovery` | tests::record_count_boundary_and_malicious_count_reject_on_recovery (renamed; current contract tested) |
| `src/ledger/mod.rs` | `tests::ledger_collection_limits_preserve_refusal_order_and_committed_slots` | Retired audit-only invariant; current ownership, limits, corruption and overflow tests remain. |
| `src/ledger/mod.rs` | `tests::schema_collection_limit_accepts_exact_history_and_rejects_the_next_record` | Retired audit-only invariant; current ownership, limits, corruption and overflow tests remain. |
| `src/ledger/mod.rs` | `tests::encoded_byte_limit_rejects_before_commit_mutation` | Retired audit-only invariant; current ownership, limits, corruption and overflow tests remain. |
| `src/ledger/record.rs` | `AllocationHistory` | Retired audit metadata; no audit replacement. Current records/latest schema retain allocation safety. |
| `src/ledger/record.rs` | `SchemaMetadataRecord` | Retired audit metadata; no audit replacement. Current records/latest schema retain allocation safety. |
| `src/ledger/record.rs` | `GenerationRecord` | Retired audit metadata; no audit replacement. Current records/latest schema retain allocation safety. |
| `src/ledger/record.rs` | `AllocationHistory::from_parts` | Retired audit metadata; no audit replacement. Current records/latest schema retain allocation safety. |
| `src/ledger/record.rs` | `AllocationHistory::records` | AllocationLedger::records (replacement on the ledger; not a move) |
| `src/ledger/record.rs` | `AllocationHistory::generations` | Retired audit metadata; no audit replacement. Current records/latest schema retain allocation safety. |
| `src/ledger/record.rs` | `AllocationHistory::is_empty` | Retired audit metadata; no audit replacement. Current records/latest schema retain allocation safety. |
| `src/ledger/record.rs` | `SchemaMetadataRecord::new` | Retired audit metadata; no audit replacement. Current records/latest schema retain allocation safety. |
| `src/ledger/record.rs` | `SchemaMetadataRecord::generation` | Retired audit metadata; no audit replacement. Current records/latest schema retain allocation safety. |
| `src/ledger/record.rs` | `SchemaMetadataRecord::schema` | Retired audit metadata; no audit replacement. Current records/latest schema retain allocation safety. |
| `src/ledger/record.rs` | `GenerationRecord::new` | Retired audit metadata; no audit replacement. Current records/latest schema retain allocation safety. |
| `src/ledger/record.rs` | `GenerationRecord::generation` | Retired audit metadata; no audit replacement. Current records/latest schema retain allocation safety. |
| `src/ledger/record.rs` | `GenerationRecord::parent_generation` | Retired audit metadata; no audit replacement. Current records/latest schema retain allocation safety. |
| `src/ledger/record.rs` | `GenerationRecord::runtime_fingerprint` | Retired audit metadata; no audit replacement. Current records/latest schema retain allocation safety. |
| `src/ledger/record.rs` | `GenerationRecord::declaration_count` | Retired audit metadata; no audit replacement. Current records/latest schema retain allocation safety. |
| `src/ledger/record.rs` | `GenerationRecord::committed_at` | Retired audit metadata; no audit replacement. Current records/latest schema retain allocation safety. |
| `src/ledger/record.rs` | `deserialize_runtime_fingerprint` | Retired audit metadata; no audit replacement. Current records/latest schema retain allocation safety. |
| `src/ledger/record.rs` | `AllocationRecord::first_generation` | Retired audit metadata; no audit replacement. Current records/latest schema retain allocation safety. |
| `src/ledger/record.rs` | `AllocationRecord::last_seen_generation` | Retired audit metadata; no audit replacement. Current records/latest schema retain allocation safety. |
| `src/ledger/record.rs` | `AllocationRecord::schema_history` | AllocationRecord::schema (latest metadata only) |
| `src/ledger/record.rs` | `AllocationRecord::observe_schema` | record_declaration / record_reservation replace current schema |
| `src/ledger/record.rs` | `AllocationLedger::new_committed` | AllocationLedger::new (one integrity contract) |
| `src/ledger/record.rs` | `AllocationLedger::allocation_history` | AllocationLedger::records (direct records; return contract changes) |
| `tests/allocation_measurements.rs` | `ledger` | current record-count workloads constructed in ledger_encoding_and_validation_allocations |
| `tests/allocation_measurements.rs` | `ledger_with_fingerprint` | current record-count workloads constructed in ledger_encoding_and_validation_allocations |
| `tests/allocation_measurements.rs` | `fingerprint_store` | key_text_store (renamed; malformed key fixture replaces fingerprint) |
| `tests/diagnostic_metadata.rs` | `snapshot_decode_checks_fingerprint_invariants` | Retired audit-only invariant; current ownership, limits, corruption and overflow tests remain. |
| `tests/diagnostic_metadata.rs` | `snapshot_decode_checks_collection_invariants_before_fingerprint` | snapshot_decode_checks_collection_invariants (renamed; collection checks retained) |
| `tests/diagnostic_metadata.rs` | `generation_decode_checks_fingerprint_metadata` | Retired audit-only invariant; current ownership, limits, corruption and overflow tests remain. |
| `tests/diagnostic_metadata.rs` | `generation_decode_preserves_current_shape_and_metadata_bounds` | Retired audit-only invariant; current ownership, limits, corruption and overflow tests remain. |
