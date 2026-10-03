# Released 0.20.0 consumer qualification — 2026-10-03

Published ic-memory **0.20.0** passes the focused IcyDB admission, lifecycle,
and logical-memory qualification below. Canic's focused memory tests and native
and Wasm core checks also pass. These results establish the selected memory
integration paths; they do not establish that the wider Canic application works.
The maintainer's reported Canic failure remains outside this qualification.

## Inputs and scope

The producer release is `6fc8e2d390b652896a1a7db63796087f3f4d4c19`.
The cached crates.io package's `.cargo_vcs_info.json` names that same commit,
and all 46 Rust files under `src/` match the release checkout byte for byte.
Consumers resolve the published registry package, without a local source patch.

| Consumer checkout | Selected dependency graph | Qualification boundary |
| --- | --- | --- |
| IcyDB `d35d8ffda192c52d2dfda183b55251fcbc5a0cee`, workspace 0.264.6, with existing working-tree edits | ic-memory 0.20.0; ic-testkit 0.13.0; ic-timers 0.9.2; ic-cdk-timers 1.0.0 | Public memory admission/error mapping, native participation, and six maintained generated-actor tests |
| Canic `e327ed6f01a07f27f030eaf7e0dc04802e01c17c`, with existing working-tree edits | `canic-core` resolves only ic-memory 0.20.0; ic-timers 0.9.2; ic-cdk-timers 1.0.0 | Default-feature core compilation and its 13 native memory tests |

Both current graphs pass offline `cargo metadata --locked --format-version 1`.
Canic's workspace lockfile also contains ic-memory **0.18.0** through its
optional published IcyDB 0.264.6 test consumer. That package is outside the
selected `canic-core` dependency closure. Canic's repository instructions identify
this composition as optional and outside deployed Canic dependencies. It was
not qualified, adapted, or synchronized with local IcyDB.

The final workspace lockfile SHA-256 values are:

- IcyDB: `ac2addb4909fb115cc531192681efb55bb638b3b43ce1f83b8a14d264c928539`.
- Canic: `d6df5fa7aa7e794879525d6fb02a5c9af6f659973943a33e94cfc9daaa785309`.

Dependency updates occurred during the initial checks. After the maintainer
confirmed they were finished, IcyDB's installed tests and native/error checks
were run on the resolved current graph. Both consumers' workspace manifests
and lockfiles remained unchanged through final evidence collection. IcyDB's
`crates/icydb-core/src/db/executor/planning/continuation/scalar.rs` then changed
at 13:29 UTC, after all passing IcyDB commands completed by 13:27:32 UTC. That
later query implementation is not covered by these receipts; they do not certify
the final whole checkout. The memory and qualification test inputs were
unchanged. Canic's core sources remained unchanged across its checks;
concurrent edits to two Canic testing fixture baselines do not qualify those
fixtures.

The toolchain was Rust 1.99.0. Installed tests used PocketIC server 16.0.0 and
the maintained fixture build/cache helpers. The lifecycle test installs its
debug fixture and builds production-profile upgrade artifacts as its existing
test specifies. These are current-format actors on both sides of each upgrade;
this is not an earlier pre-1.0 format compatibility test.

## Focused results

Every Cargo command used `--locked`, `CARGO_NET_OFFLINE=true`, and a separate
`CARGO_TARGET_DIR` under `/tmp/ic-memory020-qualification/<consumer>`.
Commands below ran in the corresponding consumer checkout.

| Consumer | Cargo arguments | Result and evidence |
| --- | --- | --- |
| IcyDB | `test --locked -p icydb --test memory_admission --test default_memory_manager --test lifecycle_participant -- --test-threads=1` | 16 passed: 14 admission tests, one default-manager test, one native participant test |
| IcyDB | `test --locked -p icydb --lib error::tests::bootstrap -- --test-threads=1` | Four public bootstrap error-mapping tests passed |
| IcyDB | `test --locked -p icydb-testing-integration --test lifecycle_participant -- --test-threads=1 --nocapture` | Three passed: lifecycle ordering and retained rows, complete ingress exclusion, trapped-upgrade rollback and clean retry |
| IcyDB | `test --locked -p icydb-testing-integration --test logical_memory -- --test-threads=1 --nocapture` | Three passed: empty omitted-store retirement without relocating survivors, journal-debt rejection with original-actor recovery, pending-marker rejection after journal folding |
| Canic | `check --locked -p canic-core --lib` | Passed |
| Canic | `test --locked -p canic-core --lib ops::runtime::memory::tests -- --test-threads=1` | 13 passed; 1,361 unrelated tests filtered out |
| Canic | `check --locked -p canic-core --lib --target wasm32-unknown-unknown` | Passed |

IcyDB's native admission tests exercise authority, retained allocation IDs,
historical journal selection, revoked grants, geometry mismatch, pool exhaustion,
and typed growth refusal without publishing candidate authority. The native
participant test covers reentry rejection, retry, and completed duplicates.
The error tests preserve public conflict/internal classifications and bounded
grant evidence.

Canic's memory tests cover configured bootstrap after early default access,
unopened allocations, measured versus unmeasurable memory projections, exact
commit diagnostic mapping, ledger/application growth refusal and retry, and
receipt-capacity exhaustion with a typed operations failure. Installed Canic
participants, complete deployed roles, auth/topology, and wider application
behavior were not exercised.

The installed IcyDB tests required local loopback access and used
`POCKET_IC_BIN=/home/adam/.cache/canic/pocket-ic-server-16.0.0-pocket-ic-x86_64-linux/pocket-ic`.
No canisters were deployed to an external network.

## Installed lifecycle measurements

The maintained lifecycle fixture reported:

| Phase | Participant instructions |
| --- | ---: |
| Init | 2,256,438 |
| Empty post-upgrade | 4,521,715 |
| Populated post-upgrade | 4,587,988 |
| Converged post-upgrade | 4,610,556 |

All four phases remain under the unchanged **12,750,000** instruction ceiling.
Each phase reports three deferred runs. Empty and populated stable extents are
both **23,134,208 bytes**, with the fixture checking extent preservation through
upgrade. These are current-run measurements, not matched release-to-release
speedups, cycle savings, or consumer Wasm-size comparisons.

## Evidence and limits

Full command logs, resolved metadata, and source-input fingerprints are retained
locally under `target/qualification/0.20.0/` in the ic-memory checkout. This
directory is ignored by Git; the logs are not committed documentation artifacts.
The passing receipts are `icydb-native-current.log`, `icydb-errors-current.log`,
`icydb-lifecycle-qualified.log`, `icydb-logical-qualified.log`,
`canic-core.log`, `canic-memory.log`, and `canic-wasm.log`. Input captures
`inputs-current.json` and `inputs-qualified.json` distinguish the qualification
inputs from subsequent consumer edits.

Initial unsuccessful lifecycle attempts are retained separately. They stopped on
an in-progress lockfile update, an incorrect PocketIC executable path, and a
sandbox loopback restriction. Correcting those prerequisites allowed the
maintained tests to pass; those attempts do not establish an ic-memory runtime
failure.

This is focused acceptance on dirty consumer checkouts, not complete consumer
CI, an immutable downstream release receipt, or a rerun of consumer MSRV,
Clippy, raw Wasm budgets, or all generated Canic roles. No sibling source,
manifest, lockfile, generated source artifact, or version was edited for this
work. Builds and caches were isolated under `/tmp`. No commit, tag, push,
external deployment, or GitHub comment was made.

The existing [IcyDB acceptance thread on #8](https://github.com/dragginzgame/ic-memory/issues/8#issuecomment-5969145448)
remains the historical upstream record. This local qualification adds fresh
0.20.0 evidence without changing the closed dispositions in the
[issue reconciliation](issue-reconciliation.md) or claiming that Canic's
separate reported failure is resolved.
