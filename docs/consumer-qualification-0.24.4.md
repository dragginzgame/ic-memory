# Released 0.24.4 IcyDB qualification — 2026-10-04

Published ic-memory **0.24.4** passes **27** focused IcyDB admission, adoption,
error-projection, lifecycle, and logical-memory tests in an isolated source
snapshot. No files in the active IcyDB checkout were edited by this qualification.

## Inputs and scope

The producer release is `12238fa9cd3191d5b1a7ccb234df89135c070f15`.
The cached crates.io package's `.cargo_vcs_info.json` names that commit, and
all 46 Rust files under `src/` match the release source byte for byte.

The snapshot was extracted from IcyDB commit
`3472892653ec7a7718da2e20c11c365c50725c68` using `git archive HEAD`, with
the workspace lockfile copied separately. It excludes the active checkout's
uncommitted migration-page-limit test and generated Candid edits. The snapshot's
1,704 Rust source files match the committed inputs used for the
[0.24.3 qualification](consumer-qualification-0.24.3.md).

Only the snapshot lockfile was updated, selecting published ic-memory 0.24.4
instead of 0.24.3; every other package remained unchanged. The selected workspace
packages declare **0.264.7**, although the commit subject says 0.264.8. The graph
contains one ic-memory package without a local source patch, plus ic-testkit
0.14.1, ic-timers 0.10.6, ic-cdk-timers 1.0.0, and PocketIC 16.0.0.

The input captures record these SHA-256 values:

- Workspace manifest: `1ba0ebe2d80b1b67e186b7eecfdea5ec58b8b940359d4f2b1630bdb7acd5c7d7`.
- Original copied lockfile: `8c8bfa7f83d06854b938027e71cbc0abe7aaac78c84a9f0033c4ea1a716acc67`.
- Qualified snapshot lockfile: `7c4f9cac25774bcfe1fcfd68f66134adab86bb38914ece692ca5fd4539eb8ac7`.

All captured snapshot Rust sources, the manifest, and the qualified lockfile
remained unchanged across the checks. During qualification, the active checkout
advanced independently to `02d451ebabc9cf1d3acadead47c741d6ce6ca705`, with a
different lockfile also selecting 0.24.4. These results certify the isolated
inputs above, not that subsequent commit or dependency graph.

## Focused results

Commands ran in `/tmp/ic-memory0244-qualification/icydb` with Rust 1.99.0,
`--locked`, offline Cargo resolution, and
`CARGO_TARGET_DIR=/tmp/ic-memory020-qualification/icydb`. The historical target
directory supplied a build cache; metadata and fresh logs identify the selected
0.24.4 package.

| Cargo arguments | Result |
| --- | --- |
| `test --locked -p icydb --test memory_admission --test default_memory_manager --test lifecycle_participant -- --test-threads=1` | 16 passed: 14 admission, one default-manager, one native participant |
| `test --locked -p icydb --lib error::tests::bootstrap -- --test-threads=1` | Four bootstrap/adoption error-mapping tests passed |
| `test --locked -p icydb --lib error::tests::database_bootstrap_preserves_typed_cause_until_public_projection -- --test-threads=1` | One typed-cause projection test passed |
| `test --locked -p icydb-testing-integration --test lifecycle_participant --test logical_memory -- --test-threads=1 --nocapture` | Three installed lifecycle and three logical-memory tests passed |

Native checks cover host authority, declaration drift, retained IDs, historical
journal selection, revoked grants, geometry mismatch, pool exhaustion, typed
growth refusal, participant reentry/retry, and public error projection.

Installed checks cover lifecycle ordering before deferred database work, retained
rows, participant exclusion from ingress, trapped-upgrade rollback and clean
retry, empty omitted-store retirement without survivor relocation, journal-debt
rejection with original-actor recovery, and pending-marker rejection after
journal folding. Both sides of the upgrades use the current format; these tests
do not establish compatibility with earlier pre-1.0 formats.

PocketIC 16.0.0 ran on local loopback through IcyDB's maintained shared-server
runner and fixture helpers. No qualification-owned server remains running, and
no external deployment was performed.

## Installed lifecycle measurements

| Phase | Participant instructions |
| --- | ---: |
| Init | 1,767,123 |
| Empty post-upgrade | 4,364,506 |
| Populated post-upgrade | 4,431,833 |
| Converged post-upgrade | 4,454,357 |

All phases pass the unchanged **12,750,000** instruction ceiling and report three
deferred runs. Empty and populated stable extents are both **23,134,208 bytes**,
with extent preservation checked through upgrade. These are current-run
measurements, not a claimed performance improvement. Consumer Wasm-size and
cycle deltas were not measured.

## Evidence and remaining qualification

Raw evidence is retained locally in the Git-ignored
`target/qualification/0.24.4/`: result and metadata JSON, snapshot/source input
captures, process checks, and native, error-projection, build, and installed logs.
This tracked receipt preserves the scope and results independently of those
local artifacts.

This is focused downstream qualification, not complete IcyDB CI. Consumer MSRV,
Clippy, unrelated database tests, the newer active checkout, and live deployments
remain outside its scope. No active consumer source, manifest, lockfile, version,
or generated artifact was edited.

Canic was neither inspected nor modified. Its deployment problems remain
unresolved by these results; its actual selected dependency graph and lifecycle
paths still need qualification once settled. No GitHub issue or comment was
updated.
