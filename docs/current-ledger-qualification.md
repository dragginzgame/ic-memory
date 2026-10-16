# Current ownership ledger qualification

The original 0.27.0 hard-cut evidence was captured after released
`62e15131c6145fa66a77528d96b00dcbf64a892c` (0.26.2), while the manifest still
declared 0.26.2. Later reviews and experiments identify their own source below.
These are local Linux source, behavior and footprint observations; they do not
establish publication or retained-installation qualification. The
[current contract](current-ledger.md) owns consumer API and deployment requirements.

## Deferred IC Wire review (2026-10-16)

Keep Memory's bounded Serde collection decoder local for now. The inspected
IC Wire checkout describes an unpublished investigation and sets `publish = false`;
its overlapping sequence visitor alone does not justify a new dependency.
Revisit consolidation when a published, stable, codec-independent Serde boundary
has demonstrated use beyond its originating investigation. Compare rejection
before excess-element decoding, allocation counts, Memory's staging-capacity hint
and matched Wasm size before proposing adoption. Keep Memory's CBOR preflight,
numeric limits and ledger semantics local. This records a deferred review;
it selects no dependency, changes no codec and adds no release gate or schedule.

## 0.45.0 resolved construction and report ownership

This batch compares the incoming working tree saved at
`target/qualification/cleanup0450d/before/`, based on released
`f4630f7dbe90c2c92f5029479f8420c1a1076a89`, with three additional cuts.
Both maintained lockfiles/manifests, toolchain, catalogs, current stored fixtures
and snapshot remain byte-unchanged. The package still declares 0.44.0;
the public error/report cuts extend the pending 0.45.0 minor.

Fixture setup removes its repeated authority verification after successful
bootstrap of that exact request/grant. Logical exports carry generation and
records; the fixed ledger-ID constant replaces anchor fields and the doctor
retains physical recovery status, including after logical recovery failure.
The explicit physical diagnostic method remains available independently.

Resolution now owns production construction of its canonical output and accepts
recovered ledger evidence. Sealing and historical admission retain key uniqueness,
selection bounds, disjointness and poisoning; the free-ID iterator retains unique
placement. Canonical sorting remains, without a second general snapshot validator.
Persisted integrity, host grants, policy callbacks, historical claim checks and
persistence-before-authority remain independent. Synthetic staging construction
is confined to unit fixtures. No new stored format or reset is introduced.

Matched Rust 1.99 raw `wasm-size` builds use the same selected lockfile/profile
and `wasm32-unknown-unknown`, without Binaryen:

| Probe | Before bytes | After bytes | Delta |
| --- | ---: | ---: | ---: |
| Core | 158,922 | 158,478 | -444 |
| Diagnostics | 185,234 | 183,959 | -1,275 |
| Admission | 162,823 | 162,334 | -489 |
| Runtime integration | 162,992 | 162,479 | -513 |
| IO fixture | 173,939 | 173,256 | -683 |

All 86 existing native measurement rows retain allocation/reallocation counts
and peak heap. Native timing is not an IC instruction or cycle measurement.
Focused runtime, ledger, fixture-cell, request, validation, constructor, report
and public integration/UI tests pass. A full-pool cold-recovery regression
combines direct and selected historical requests across all 245 application IDs,
preserves every mapping and refuses exhaustion without changing committed bytes.
Strict all-target `test-support` Clippy, actual Rust 1.91 all-target checking,
Wasm checking, warning-denied Rustdoc and two-workspace formatting pass.
Full release validation, installed-runtime and native macOS qualification were
not repeated. Sources, commands, failed attempts and comparisons remain under
`target/qualification/cleanup0450d/`.

A small Canic response-projection patch is prepared against its committed
Memory 0.44 integration at `47d35215a2ba2246da355d46e588296fd80760fb`.
Its 15 focused Memory projection tests and formatting pass with that selected
published graph; strict Core Clippy fails at two unchanged Auth record reexports.
The patch uses the fixed ID constant and the checked generation of a successful
logical export, preserving Canic's Candid response without another ledger read.
Remote Canic main still selects Memory 0.35, so no PR carrying unrelated unpublished
integration commits was opened. Full Memory 0.45 adoption and installation remain
owned by [Canic #573 — Adopt Memory numeric slot and label API hard cut](https://github.com/dragginzgame/canic/issues/573).
The normal sibling checkout remains unchanged.

## 0.45.0 output-only reports and immutable-storage experiment

This follow-on starts from the incoming working tree saved at
`target/qualification/cleanup0450c/before/`, still based on released
`f4630f7dbe90c2c92f5029479f8420c1a1076a89`. Report-only types no longer implement
`Deserialize`; their serialized fields remain unchanged. Ledger, slot, host
configuration and policy-identity decoding remain selected. This introduces no
additional stored-format change. Both maintained lockfiles, catalogs, versions
and tools remain unchanged by this follow-on.

The proposed `Arc<[T]>` replacement for both immutable `Arc<Vec<T>>` snapshots
was measured and rejected. Raw Rust 1.99 `wasm-size` probes increased by 401 to
569 bytes; changing only committed capabilities still added 297 to 317 bytes.
For tightly sized native request vectors, shared slices saved 24 retained bytes
but raised sealing peak heap by up to 12,168 bytes at 254 requests. Overallocated
input vectors retained substantially less memory after conversion, but that
benefit does not justify the measured Wasm and transient-heap cost for the
selected small, bounded snapshots. Both production representations are retained.
The native sealing experiment covers empty, single, 64- and 254-request
snapshots with tight and excess input capacity. Its harness is retained only
with experiment evidence, rather than adding maintained code for a rejected change.

Final raw Wasm sizes equal the incoming baseline for all five probes: core
158,922; diagnostics 185,234; admission 162,823; runtime integration 162,992;
IO fixture 173,939 bytes. All 86 existing matched native workload rows retain
their allocation/reallocation counts and peak heap. These are Linux native heap
and raw-Wasm observations, not IC instruction, cycle or application-binary results.
Focused diagnostic tests, six existing native measurement tests plus the sealing
experiment, strict
all-target `test-support` Clippy, actual Rust 1.91 all-target checking, Wasm
checking, warning-denied Rustdoc and the two-workspace formatter pass. Full
release validation and installed-runtime qualification were not repeated for
this report-only cut. Sources, failed attempts and comparisons are retained at
`target/qualification/cleanup0450c/`.

The retention extraction already exists in
[Host PR #104 — Digest-verified durable artifact copying](https://github.com/dragginzgame/ic-host-tooling/pull/104).
It is refreshed against released Host 0.23.1 without a second retention/cache
abstraction. Caller-selected names, destination reuse, independent retained-file
rehash and execution-boundary rechecks stay with Memory. The published consumer
adoption remains owned by
[#92 — Adopt published Host verified copying](https://github.com/dragginzgame/ic-memory/issues/92).
Host PR commit `f8fdea23b15c6806de1527881a7eadf75083b66c` passes 79 focused
durable-filesystem tests, strict filesystem Clippy, Rustdoc, Rust 1.88 checks
and formatting on Linux. The prospective consumer patch passes all 57 helper
tests and strict Clippy in a disposable copy selecting those Host sources.
Its path-patched graph is rehearsal evidence only. The patch and failed setup
attempts remain with this batch's evidence; native hosted acceptance,
publication and registry adoption are separate requirements.

## 0.45.0 flat snapshot and single policy callback

This follow-on compares the incoming dirty tree saved at
`target/qualification/cleanup0450b/before/`, based on released
`f4630f7dbe90c2c92f5029479f8420c1a1076a89`, with the additional wrapper and policy
cuts. It excludes the earlier pending 0.45 work and deferred IC Wire note.
Both maintained manifests/lockfiles, package versions, tool catalogs, toolchain
and vendored snapshot remain unchanged. The package still declares 0.44.0;
0.45.0 remains the pending breaking release.

The stored commit now owns its required `current` snapshot directly. Physical
marker/checksum admission, exact successors, logical integrity and counter
binding retain their checks. `ICMSNAP1` identifies the incompatible enclosing
shape. The approved fresh/reset disposition applies to the pending cut, including
unpublished preceding fixture bytes; no older reader or real reset is included.
One allocation-policy callback replaces the paired key/slot callbacks, retaining
host grants before policy, historical checks after policy and governance bypass.
Recovered admission remains separate; doctor and warm binding do not replay it.

Matched Rust 1.99 raw `wasm-size` builds use the same root lockfile/profile and
`wasm32-unknown-unknown`, without Binaryen for this comparison:

| Probe | Before bytes | After bytes | Delta |
| --- | ---: | ---: | ---: |
| Core | 160,944 | 158,922 | -2,022 |
| Diagnostics | 187,091 | 185,234 | -1,857 |
| Admission | 164,853 | 162,823 | -2,030 |
| Runtime integration | 164,910 | 162,992 | -1,918 |
| Installed IO fixture | 175,849 | 173,939 | -1,910 |

All 86 matched native measurement rows retain allocation/reallocation counts.
Each measured snapshot encodes ten fewer bytes. Peak measured heap falls by ten
bytes for empty doctor and 0/1-record snapshot reopening; other measured peaks
are unchanged. Native timing is not an IC instruction or cycle measurement.

Installed qualification at `target/qualification/runtime/attempt.NJY2aA/` passes
original, `-O3`, `-Os` and `-Oz` artifacts with unchanged Testkit 0.25.4, PocketIC
client 16.0.0 and server 16.1.0. Each variant passes 12 invalid-IO traps with
neighbor/prior-write preservation, failed-upgrade/geometry-conflict rollback and
clean retry advancing exactly one generation. A separate measurement of the
saved incoming original Wasm uses the identical qualifier and prepared tools;
all nine original-artifact 1,000-IO instruction rows are unchanged. This is
fresh/current-format and same-format upgrade evidence, not earlier-image or
application-installation acceptance.

Focused checks pass 187 library tests, 12 native integration/compile-fail harness
tests, 57 release-helper tests, four doctests and six native measurement tests.
Strict all-target `test-support` Clippy, actual Rust 1.91 native all-target and
Wasm checks, warning-denied Rustdoc, formatting and four Wasm budgets pass.
Full release validation, clean-source packaging and native macOS acceptance have
not run. Commands, saved sources, failed attempts and comparisons are retained
at `target/qualification/cleanup0450b/`.

A bounded, read-only Cell inspection proposal was prepared against
upstream stable-structures `b5d1650463f477affb3d358e86b1113953fc9937`. Seven Cell
tests pass on Rust 1.85/1.99, four Cell doctests pass and warning-denied Wasm
library documentation passes. Native documentation fails at unchanged links to
Wasm-only types; upstream Rust 1.84 and book tools are unavailable locally.
The maintainer decided to retain Cell header admission in Memory. The prototype
is retained as experiment evidence only; no upstream PR was opened or fork
adopted. The local guard and exact 0.7.2 dependency remain selected. This is an
API fit decision, not a demonstrated Cell corruption defect.

Consumers must combine their former key and slot checks into the new callback
and preserve any application-specific error ordering. Coordinated publication,
API adoption and application-owned installation/data decisions remain with
[Canic #573 — Adopt Memory slot and allocation API cuts](https://github.com/dragginzgame/canic/issues/573)
and [IcyDB #355 — Generate explicit component memory requests](https://github.com/dragginzgame/icydb/issues/355).
Their normal sibling checkouts are unchanged.

## 0.45.0 schema-free ledger and fixed pool qualification

This task compares the incoming dirty tree saved at
`target/qualification/cleanup0450/before/`, based on released
`f4630f7dbe90c2c92f5029479f8420c1a1076a89` (0.44.0), with this additional cleanup.
The incoming bounded fixture-cell work is preserved and excluded from this task's
change totals. Both maintained manifests/lockfiles, package versions, toolchains,
tool catalogs and vendored snapshot are byte-unchanged by this task. The package
still declares 0.44.0; 0.45.0 is the next breaking release, not a publication claim.

The runtime stages, protects and persists in one owned flow, drops the staged
ledger before Cell encoding, and constructs open authority only after persistence
succeeds. Pool exclusions use one fixed mask and project sorted numeric IDs;
hosts own range presentation. Durable allocation records contain key, slot and
state, while applications retain schema admission. The distinct `ICMCLAIM` marker
identifies the incompatible format. The maintainer approved retiring earlier
images in favor of fresh/reset memory and running installed qualification.
No deployed canister, real stable memory or backup was reset; no compatibility
reader, converter or automatic clearing is included.

Matched raw Wasm uses Rust 1.99.0, the same selected root lockfile and `wasm-size`
profile, targeting `wasm32-unknown-unknown` without Binaryen for this comparison:

| Probe | Before bytes | After bytes | Delta |
| --- | ---: | ---: | ---: |
| Core | 164,547 | 160,944 | -3,603 |
| Diagnostics | 192,276 | 187,091 | -5,185 |
| Admission | 168,641 | 164,853 | -3,788 |
| Runtime integration | 169,270 | 164,910 | -4,360 |
| Installed IO fixture | 179,192 | 175,849 | -3,343 |

The matched release-mode native measurement has 86 workload rows. Every row's
allocation/reallocation counts and peak measured heap either stay unchanged or
fall. Fresh bootstrap removes one allocation at 1, 32 and 245 external requests;
the 245-request peak falls from 189,319 to 183,415 bytes. Doctor removes two
allocations per invocation. The 255-record short-key snapshot falls from 19,981
to 12,841 encoded bytes, with reopen peak heap falling from 52,749 to 32,449 bytes.
These are System allocation requests on the measured thread, excluding existing
inputs/backing; every operation releases its measured allocations. They are not
RSS, Wasm linear-memory use, stable-page savings or IC instruction/cycle deltas.

Focused checks pass 187 library tests, 12 native integration/compile-fail harness
tests, 57 release-helper tests, four doctests and six explicit native measurement
tests. Strict all-target `test-support` Clippy, actual Rust 1.91 native all-target
and Wasm checks, warning-denied Rustdoc, formatting and all four Wasm budgets pass.
Current fixtures preserve their keys, numeric IDs, lifecycle states and counters
through regeneration with the current types/encoder. Exhaustive mask membership,
canonical projection, malformed/corrupt data, ownership, failure ordering and
retry checks retain their focused coverage.

Installed evidence at `target/qualification/runtime/attempt.bbPhwm/` passes
original, `-O3`, `-Os` and `-Oz` artifacts with unchanged Testkit 0.25.4, PocketIC
client 16.0.0 and server 16.1.0. Each artifact passes 12 invalid-IO traps with
neighbor/prior-write preservation, post-persistence upgrade and geometry-conflict
rollback, and clean retry advancing exactly one generation. The original
artifact matches the measured input. This qualifies fresh/current-format startup
and same-format upgrades, not earlier-image upgrades or consumer installations.
There is no matched installed baseline for an IC instruction comparison here.

Commands, saved sources, failed attempts and comparisons remain under
`target/qualification/cleanup0450/`. Full release validation, clean-source
packaging and native macOS acceptance were not run. A source-only Canic range
projection patch is prepared against the observed file hash; it is not a clean
upstream contribution or qualified adoption. The published minor, complete API
adoption and application-owned data/install checks remain with
[Canic #573 — Adopt Memory slot and allocation API cuts](https://github.com/dragginzgame/canic/issues/573)
and [IcyDB #355 — Generate explicit component memory requests](https://github.com/dragginzgame/icydb/issues/355).
Neither normal sibling checkout was edited.

## 0.44.0 direct-CBOR and single-pass staging qualification

This local working-tree batch starts from `dd0f77564215d8e1891dbfb8ccddeae3b6b8eebd`
plus the saved incoming edits, including the selected IC Build 0.8.0 graph and
prior request/admission cleanup. Both selected lockfiles, toolchains and package
versions are unchanged. The package still declares 0.43.0; 0.44.0 is the pending
breaking release, not a publication claim. Before/after sources, commands,
artifacts and failed attempts are retained under
`target/qualification/cleanup0440/`.

The new `ICMDIRCT` physical marker identifies a bounded direct-CBOR payload.
The nested logical header is removed. Marker/checksum admission, logical
integrity, physical/logical counter binding and Cell persistence before capability
publication remain. Each declaration's policy and historical claim are checked
once against the owned staging ledger. Doctor shares that checker without
staging. The maintainer authorized retirement of old images and fresh/reset
memory for this cut; no installation or stable memory was reset.

Matched raw Wasm uses Rust 1.99.0, the selected root lockfile, the committed
`wasm-size` profile and `wasm32-unknown-unknown`, without Binaryen optimization:

| Probe | Before bytes | After bytes | Delta |
| --- | ---: | ---: | ---: |
| Core | 168,650 | 166,895 | -1,755 |
| Diagnostics | 201,970 | 198,885 | -3,085 |
| Admission | 173,094 | 170,976 | -2,118 |
| Runtime integration | 177,229 | 175,470 | -1,759 |

The matched release-mode native declaration-resolution measurement runs ten
fresh bootstraps each at 1, 32 and 245 external requests. Allocation totals are
unchanged; peak measured heap falls by 64, 512 and 4,120 bytes respectively.
Removing the initial envelope buffer adds two buffer reallocations per fresh
bootstrap. Capacity-hint experiments reduced that churn but increased Wasm or
peak heap; the simpler, smaller writer was selected. Native timings do not
establish an IC instruction or cycle delta.

Focused strict Clippy, Rust 1.91, current-wire recovery, malformed input,
claim-conflict/retirement rejection, failed local staging, generation overflow,
consumer/compile-fail and generic example checks pass. Installed qualification
uses the unchanged PocketIC 16.0.0 client and Testkit 0.25.4-selected 16.1.0 server.
The first sandboxed server attempt could not bind localhost; the authorized
local-network retry passed original, `-O3`, `-Os` and `-Oz` variants. Each variant
passed 12 invalid-IO traps with neighboring/prior bytes preserved, post-persistence
upgrade rollback, geometry-conflict rollback and clean retry advancing exactly
one generation. Evidence is under `target/qualification/runtime/attempt.7T0XAg/`.
This qualifies fresh and same-format installed behavior, not old-image upgrades,
retained deployments or a full release matrix.

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

## 0.44.0 exact binding and flattened Cell qualification

This follow-on compares the saved incoming pending-0.44 tree at release HEAD
`dd0f77564215d8e1891dbfb8ccddeae3b6b8eebd` with the additional exact-binding,
fixture-construction and direct-Cell changes. It excludes earlier changes in
that dirty tree. The current Cell value is `LedgerCommitStore` itself; its
seven-byte `store` wrapper and duplicate Cell-value fixture are removed.
The maintainer-approved fresh/reset disposition for 0.44 still applies; no
previous-layout reader or real stable-memory reset is included.

Matched Rust 1.99 raw `wasm-size` builds use the same lockfile and profile,
without Binaryen for these size comparisons:

| Probe | Before bytes | After bytes | Delta |
| --- | ---: | ---: | ---: |
| Core | 166,895 | 164,958 | -1,937 |
| Diagnostics | 198,885 | 194,984 | -3,901 |
| Admission | 170,976 | 168,932 | -2,044 |
| Runtime integration | 175,470 | 173,549 | -1,921 |
| Installed IO fixture | 183,408 | 181,471 | -1,937 |

All 86 native measurement rows preserve allocation counts. Reallocation counts
are unchanged except that each extreme-counter Cell encoding avoids one buffer
reallocation. Cell reopening and doctor peaks fall by seven bytes; those encoding
cases also avoid capacity doubling. Numeric summaries remain zero-allocation.
These native heap observations do not establish an IC instruction speedup.

The final 200 library tests, 12 selected native integration/compile-fail harness
tests and four doctests pass. Strict all-target `test-support` Clippy, actual
Rust 1.91 all-target checks, Wasm test compilation, warning-denied Rustdoc,
formatting and all four Wasm budgets pass. The old fixture engine's dedicated
transition API tests are removed; production staging is exercised against
reserved, active and retired starting records, retaining ownership/integrity
and corruption coverage.

Final installed qualification at `target/qualification/runtime/attempt.w4o79N/`
passes original, `-O3`, `-Os` and `-Oz` artifacts with the selected Testkit 0.25.4,
PocketIC client 16.0.0 and server 16.1.0. Every variant checks 12 invalid-IO traps,
neighbor/prior-write conservation, post-persistence upgrade and geometry-conflict
rollback, and clean retry advancing exactly one generation. The final original
artifact is byte-matched to the retained input. This proves current-format
startup and same-format upgrade behavior, not an upgrade from retired images.
Both maintained lockfiles, package versions and tool selections remain unchanged.

Evidence, matched inputs and command logs are retained at
`target/qualification/cleanup0440b/`. Full release validation, packaging and
native macOS acceptance were not run. Canic's three direct-record projection
fixture edits are prepared separately; its registry-selected 0.44 adoption and
application-owned retained-data decision remain under
[Canic #573](https://github.com/dragginzgame/canic/issues/573#issuecomment-6213455806).

## 0.44.0 explicit host runtime and primary report facts

This follow-on compares the saved incoming pending-0.44 tree at release HEAD
`dd0f77564215d8e1891dbfb8ccddeae3b6b8eebd` with the host-runtime, primary-report
and opaque-CBOR cleanup. It excludes earlier work in that dirty tree. Both
maintained lockfiles, dependency selections, package versions, toolchain and
vendored snapshot remain unchanged by this task. The selected Host 0.22.1
helper call now supplies explicit replacement/0o666 options.

Memory no longer owns singleton/TLS state. Wasm fixtures explicitly retain one
host-owned runtime; exported operations, backing geometry and IO checks remain.
Reports store counts/pages/geometry and derive byte totals through accessors.
Their serialized output changes accordingly; Canic owns expanded API projections.
Opaque payload input uses only bounded CBOR byte strings, preserving the current
CBOR bytes while retiring JSON-array input. There is no further stored-format
change beyond the already approved 0.44 cut.

Matched Rust 1.99 raw `wasm-size` builds, with identical lockfile/profile and
without Binaryen for this table:

| Probe | Before bytes | After bytes | Delta |
| --- | ---: | ---: | ---: |
| Core | 164,958 | 164,644 | -314 |
| Diagnostics | 194,984 | 192,275 | -2,709 |
| Admission | 168,932 | 168,746 | -186 |
| Runtime integration | 173,549 | 169,361 | -4,188 |
| Installed IO fixture | 181,471 | 179,283 | -2,188 |

All 86 native measurement rows retain allocation/reallocation counts. Detailed
report peak heap falls from 22,440 to 16,320 bytes in all four empty/populated,
bootstrapped/unbootstrapped cases; numeric summaries remain allocation-free.
These measurements do not establish application instruction savings.

Installed qualification at `target/qualification/runtime/attempt.I1pFkA/`
passes original, `-O3`, `-Os` and `-Oz` artifacts with the selected Testkit 0.25.4,
PocketIC client 16.0.0 and server 16.1.0. Every artifact passes 12 invalid-IO
traps, neighbor/prior-write preservation, post-persistence upgrade and geometry
conflict rollback, and clean retry advancing exactly one generation. This is
current-format startup and same-format upgrade evidence, not earlier-image
compatibility or application-owned composition.

There is a measured IO instruction tradeoff. The saved baseline Wasm is
byte-identical to the earlier installed baseline in `attempt.w4o79N`; the
qualifier, server and optimizer identities also match. In the unchanged 1,000-IO
loops, the original artifact adds 2 instructions per safe read/write and 26 per
raw read; optimized artifacts add 2 and 25 respectively. These are bounded
fixture measurements, not a speedup claim or application cycle estimate. The IO
implementation and bounds checks are unchanged; the smaller host/report build
changes generated code.

Focused qualification passes 191 library tests, six explicit allocation
measurements, current native integration/compile-fail tests, 57 release-helper
tests and four public doctests. Strict all-target `test-support` Clippy, actual
Rust 1.91 all-target checks, Wasm test compilation, warning-denied Rustdoc,
formatting and all four Wasm budgets pass. README/operations/admission examples
now use the explicit instance. Facade-only tests are retired; explicit custom
policy, host independence, injected-handle lifetime, admission, corruption,
conservation and installed IO/upgrade coverage remain.

Evidence and matched inputs are retained at `target/qualification/cleanup0440c/`.
Full release validation, packaging and native macOS acceptance were not run.
[Canic #573](https://github.com/dragginzgame/canic/issues/573#issuecomment-6214373359)
and [IcyDB #355](https://github.com/dragginzgame/icydb/issues/355#issuecomment-6214373692)
own the coordinated consumer changes after publication;
their normal checkouts remain untouched. No downstream application build or
retained installation has been qualified by this task.
