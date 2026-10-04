<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/ic-memory/main/images/ic-memory-readme-header.svg" alt="IC Memory — Internet Computer helper library" width="100%">
</p>

# Released 0.24.3 IcyDB qualification — 2026-10-04

Published ic-memory **0.24.3** passes all **27** focused IcyDB admission,
adoption, error-projection, lifecycle, and logical-memory tests below. This
receipt records the completed local qualification; the documentation follow-up
checked its saved logs without rerunning the tests.

## Inputs and scope

The producer release is `69a90f1cc241aa3af7dd8520c5305d49f1fbe0a2`.
The cached crates.io package's `.cargo_vcs_info.json` names that commit, and
all 46 Rust files under `src/` match the release source byte for byte.

The IcyDB checkout was clean at
`3472892653ec7a7718da2e20c11c365c50725c68`. Its selected workspace packages
declare **0.264.7**; the commit subject says 0.264.8. The resolved graph contains
one ic-memory package, published **0.24.3**, without a local source patch. It
also selects ic-testkit 0.14.1, ic-timers 0.10.6, ic-cdk-timers 1.0.0, and
PocketIC 16.0.0.

The input captures record these SHA-256 values:

- Workspace manifest: `1ba0ebe2d80b1b67e186b7eecfdea5ec58b8b940359d4f2b1630bdb7acd5c7d7`.
- Workspace lockfile: `8c8bfa7f83d06854b938027e71cbc0abe7aaac78c84a9f0033c4ea1a716acc67`.
- Capture of the 1,704 tracked Rust source-file hashes:
  `97bb7f7a2ad3afb57c75f651a1147f1fe8ef463bcc574951ee63226989ac58f8`.

The checkout HEAD, status, manifest, lockfile, and captured tracked Rust inputs
remained unchanged across qualification. This identifies the checked inputs;
it does not certify subsequent downstream edits.

## Focused results

Commands ran in the IcyDB checkout with Rust 1.99.0, `--locked`, offline Cargo
resolution, and `CARGO_TARGET_DIR=/tmp/ic-memory020-qualification/icydb`.
That existing target cache was reused; the selected package and fresh build logs
identify ic-memory 0.24.3 despite the directory's historical name.

| Cargo arguments | Result |
| --- | --- |
| `test --locked -p icydb --test memory_admission --test default_memory_manager --test lifecycle_participant -- --test-threads=1` | 16 passed: 14 admission tests, one default-manager test, one native participant test |
| `test --locked -p icydb --lib error::tests::bootstrap -- --test-threads=1` | Four public bootstrap/adoption error-mapping tests passed |
| `test --locked -p icydb --lib error::tests::database_bootstrap_preserves_typed_cause_until_public_projection -- --test-threads=1` | One typed-cause projection test passed |
| `test --locked -p icydb-testing-integration --test lifecycle_participant --test logical_memory -- --test-threads=1 --nocapture` | Three lifecycle and three logical-memory tests passed |

Native tests cover host authority and declaration drift, retained IDs, historical
journal selection, revoked grants, geometry mismatch, pool exhaustion, typed
growth refusal, and participant reentry/retry. Error tests preserve public
classifications, bounded grant evidence, and typed causes until projection.

Installed lifecycle tests cover ordering before deferred database work, retained
rows, exclusion of participant functions from the complete ingress surface, and
trapped-upgrade rollback followed by clean retry. Logical-memory tests cover
empty omitted-store retirement without relocating survivors, journal-debt
rejection with original-actor recovery, and pending-marker rejection after
journal folding.

Installed tests used IcyDB's maintained shared-server runner and fixture build
helpers with PocketIC 16.0.0 on local loopback. The lifecycle and logical-memory
tests used the current format throughout. They do not establish compatibility
with an earlier pre-1.0 format. Both qualification-owned servers were stopped
after the checks; no external deployment was performed.

## Installed lifecycle measurements

| Phase | Participant instructions |
| --- | ---: |
| Init | 1,767,433 |
| Empty post-upgrade | 4,364,842 |
| Populated post-upgrade | 4,431,550 |
| Converged post-upgrade | 4,454,129 |

All phases pass the unchanged **12,750,000** instruction ceiling and report
three deferred runs. Empty and populated stable extents are both
**23,134,208 bytes**, with extent preservation checked through upgrade.
These are current-run measurements. Changes in the consumer and dependency
graph prevent attributing a difference from earlier receipts to ic-memory.
Consumer Wasm-size and cycle deltas were not measured.

## Evidence and remaining qualification

Raw evidence is retained locally under `target/qualification/0.24.3/` in the
ic-memory checkout. This Git-ignored directory contains `result.json`, resolved
metadata, manifest/lockfile and tracked-source input captures, and
`icydb-native.log`, `icydb-errors.log`, `icydb-installed-build.log`, and
`icydb-installed.log`. This receipt preserves the scope and results in tracked
documentation; the raw logs remain local artifacts.

This is focused downstream qualification, not complete IcyDB CI. Consumer MSRV,
Clippy, unrelated database tests, and live deployments were outside its scope.
No consumer source, manifest, lockfile, version, or generated source was edited.

Canic was neither inspected nor modified for this qualification. Its deployment
problems remain unresolved by these IcyDB results. The next Canic check requires
a settled dependency graph and qualification of its actual selected memory and
lifecycle integration paths. The historical
[0.20.0 receipt](consumer-qualification-0.20.0.md) remains unchanged. No GitHub
issue disposition or comment was updated.
