<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-memory/ic-memory-readme-header.svg" alt="IC Memory — Stops upgrades from mixing up stored data" width="100%">
</p>

# Released 0.24.6 Canic memory qualification — 2026-10-04

Canic **0.110.52** passes 14 focused native memory regressions and default-feature
`canic-core` Wasm compilation against published ic-memory **0.24.6**.

The consumer snapshot was extracted with `git archive` from release commit
`d3b1f4ffbbbec8402f3bcef513c7a3db514f93d5`. Its original lockfile selects
ic-memory 0.24.2. Only the isolated snapshot lockfile was updated, replacing that
package with 0.24.6; every retained package record stayed unchanged. No local
source patch was used. The graph retains ic-stable-structures 0.7.2, ic-query
0.45.6, ic-timers 0.10.6 and ic-cdk-timers 1.0.0.

The published producer package identifies release commit
`b89f8fc7eb33f802e6aebdc4ae4cdc0d89197f3a`; all 46 Rust source files match that
commit byte for byte. This qualification covers released 0.24.6, not the open
0.24.7 candidate.

Commands ran with Rust 1.99.0 in `/tmp/ic-memory0247-canic.HBqUEB`, using that
snapshot's own build directory:

| Cargo arguments | Result |
| --- | --- |
| `test --locked -p canic-core --lib ops::runtime::memory::tests -- --test-threads=1` | 14 passed; 1,361 unrelated tests filtered out |
| `check --locked --offline -p canic-core --target wasm32-unknown-unknown` | Passed |

Native tests cover configured default bootstrap and authority, declaration
retention, read-only allocation accounting, unknown-owner attribution, current
diagnostic fields and response mapping, ledger/application growth refusal with
retry, and bucket-exhaustion conservation with typed errors.

The snapshot's archived inputs remained unchanged except for its qualified
lockfile. Its manifest SHA-256 is
`4ec779e41733839f4b60c69dabd70dd4d856991e90f896c52d208816bdfe3867`;
its qualified lockfile SHA-256 is
`8f535a8020bc5705e9b6f0491c1f22f1f45619c51442723cb5735168842ee156`.
Raw logs, captured manifests and input metadata are retained locally in the
Git-ignored `target/qualification/0.24.6/canic/` directory.

The active Canic checkout was neither edited nor used for compilation. Its
ongoing edits and selected 0.24.5 graph are outside these results. This is focused
memory integration evidence, not complete Canic CI, installed lifecycle proof,
or live deployment qualification. Consumer MSRV, other feature combinations,
Wasm-size and execution-cost comparisons were not checked. No consumer version,
generated fixture, GitHub issue, commit, tag, push or deployment was changed.
