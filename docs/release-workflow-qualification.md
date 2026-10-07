# Shared release workflow qualification

## 0.27.3 publication network scope

Prepared separately from clean released source
`c1e7dc657c0994421258f9924255f4d0349067ad` (0.27.2), on Linux x86-64,
2026-10-06. The main checkout remains clean for completion of the maintainer's
0.27.2 publication; the fix is reviewable as a patch and isolated checkout.
Pending notes select compatible 0.27.3; neither manifest nor lock is bumped.

The 0.27.2 launcher exported `CARGO_NET_OFFLINE=true` into the helper. Its
`cargo publish --locked --registry crates-io` subprocess inherited that setting
and refused an HTTP request. This is a consumer implementation error. Shared
Tooling requires offline validation and separately authorized publication; it
does not require an offline publication process.

The correction keeps `RUSTUP_AUTO_INSTALL=0` and all existing Cargo
`--locked --offline` compilation/discovery flags, while preserving the caller's
network environment. Current Rust evidence/tag/source checks and publication
arguments are unchanged. No implicit network retry, dependency selection,
format change or replacement qualification evidence is introduced.

Focused Make substitutes check that normal and bootstrap Cargo invocations
remain locked/offline and disabled for toolchain auto-installation, while
publication children receive the caller's unset/false/true setting. Make adapter
checks, strict tooling lint, both-workspace formatting and the 28-file snapshot
verification pass. A native minimal Cargo/helper control reproduces inherited
`true` through the original launcher and proves `false` through the direct Cargo
retry. The corrected launcher preserves all three caller states on both paths
and both Rust 1.99.0/1.88.0 (12 cases). The control only prints child environment;
it neither calls real publication nor contacts a registry.

Evidence remains under `target/qualification/publish-offline/`: native original,
workaround and fixed controls, isolated source and `fix-checks.log`. Qualified
launcher SHA-256 `e0135368223df5084e416eb153035c5807f9aa3ee8c0d282e9aa56f30cc1302b`; Make fixture SHA-256 `03082062e8bf4e7907c8853aadd19379abcd81d35d18098923045fd010292238`.
No function, method or type was removed. Library/runtime source, dependency and
receipt selections are unchanged. No full gate, real publication, commits,
tags or pushes were executed. Native macOS and matching CI remain outstanding
for the prepared fix; CI for 0.27.2 cannot qualify these edits.

## 0.27.2 tracked dependencies and local pinning

Local Linux x86-64 working-tree qualification on 2026-10-06, based on released
ic-memory `6e98b07882ed43d180195f5f915eafaef05f25db` (0.27.1). Pending notes select
0.27.2; package metadata remains 0.27.1. This tooling batch does not change the
runtime, library APIs or durable memory format.

The maintainer requested the current local Shared Tooling. Its pinning files
were initially uncommitted and temporarily exported with explicit working-tree
provenance. During qualification they became commit
`a7efade1a68e43f148252a1a73908a46c4cbe9e9` (local 0.1.5), matching the reviewed
pinning bytes. The final 28-file snapshot was exported through the canonical
refresh helper from an isolated checkout of that exact commit, with the original
upstream URL. The temporary manifest was removed; its evidence remains under
`target/qualification/dependency-pins/`. Final CI/release commands require no
sibling checkout. No sibling file was changed by this adoption.

Both lockfiles retain their original bytes and selected versions/features:

| Lockfile | SHA-256 |
| --- | --- |
| Root | `83627663ed770d5dc84a7c4f58fb5121491e2588f68cace773ba41e80f792b60` |
| Independent PocketIC workspace | `7bc872926fbeddd23706750c6205582ba9eae6c873612794e06e862503bb6be7` |

CI now fetches the tracked root graph with `--locked`, instead of generating a
new selection on each run. `make check-pins` checks parsed declarations and Git
tracking; `make test-pins` runs the unmodified canonical fixtures. The complete
release gate includes both. Scoped exceptions record the existing exact
stable-structures 0.7.2 layout boundary and PocketIC 16.0.0 server API. The
independent workspace remains excluded from the library's build/dependency gate.

Release surfaces now include `Cargo.lock`. `replace_lock_version` preserves all
other bytes and independently checks TOML graph equality after changing the
root package version. Validation lock bytes must equal the saved source's lock.
Preparation writes all other metadata, including the lock, before the manifest;
retries and rollback retain exact owned edits and preserve conflicting input.
The existing offline Cargo workspace refresh, command evidence, receipt schema,
commit/tag/push ownership and retained archives remain. A real Cargo 1.99.0
workspace-refresh control in a disposable source tree locked zero new packages
and preserved the expected version-edited lock byte-for-byte. This is not final
package or release qualification.

A follow-up bounded recovery review traced the canonical runner's prepare retry
through Make and the Rust adapter. It applied the current local Shared Tooling
audit contract/code-hygiene questions, under this repository's `AGENTS.md`
overlay and existing release qualification scope. The new `audits/` methods
were uncommitted beside baseline `a7efade1a68e43f148252a1a73908a46c4cbe9e9`,
not part of the recorded snapshot: `audits/README.md` SHA-256
`926d5a39ea62a3becd19ff0326c63987b2929d3a1a728e2a0ef8a7180824d231`,
`audits/code-hygiene.md` SHA-256
`518d890bf485bef06c316b4cf26ac6bf357ed52bcce995e8d9b1232a103d0ea3`.
Scope was the pending pin/CI adoption, tracked-lock writes, helper startup,
receipt admission and retry/refusal paths; runtime storage and installed
PocketIC behavior were excluded because this batch changes neither. This is
additional release evidence, not a whole-product audit or native release proof.

The review reproduced a bounded recovery defect: interruption after the lock
write but before the manifest caused preflight to reject the exact candidate
lock. A real locked Cargo helper invocation also refused before Rust could run.
The corrected preflight requires saved source-bound successful validation for
this partial state. The Make launcher builds current source against coherent
scratch metadata with an unchanged lock, then executes in the actual checkout.
Ordinary helper calls use the original workspace. No stale helper executable,
alternate dependency selection or release-effect dispatcher was introduced.
Bootstrap inputs/builds remain under Cargo's selected target directory.
The launcher disables automatic Rustup installation and forces Cargo offline;
consumer substitutes check both settings.

Focused results:

- Rust 1.99.0 and MSRV 1.88.0 pass all 22 release adapter tests. Tracked-lock
  coverage includes all increments, interruption retry, rollback/conflicting
  inputs, receipt/source graph binding, final-package HEAD movement and older
  selected-commit evidence. Command effects remain substituted.
- The interrupted-lock test fails before the correction and resumes after it
  without another validation or a Cargo fetch against mismatched metadata.
  Missing receipts, changed selection and changed compiler configuration refuse
  preflight while preserving the partial lock/base manifest. A native disposable
  checkout reproduces locked Cargo startup refusal, then the actual Make launcher
  returns its real base version on both Rust 1.99.0 and 1.88.0. These native
  startup checks do not execute a release gate or Git effects. Consumer Make
  substitutes also prove normal dispatch, scratch metadata isolation, selected
  target paths with spaces, command failure propagation and duplicate-root
  refusal. Native macOS startup remains for matching CI qualification.
- The real Git index fixture reuses local history and creates an uncommitted
  tree object with a tracked lock. It accepts only original/prepared lock bytes
  and refuses dependency drift, arbitrary added text, unrelated staging and
  metadata mode changes. It touches only its own index; no commits/tags are made.
- The complete consumer declaration check passes in a disposable checkout with
  the current adoption files and both locks staged there. The real checkout's
  check correctly refuses the still-untracked root lock. The maintainer must
  commit adoption before that gate can pass; the agent did not alter the real index.
- Both actual workspace graphs pass `cargo +1.99.0 metadata --locked --offline
  --format-version 1`, with unchanged lock hashes. These cheap graph checks are
  not independent host-package compilation or installed IC qualification.
- Canonical pinning fixtures, the consumer Make adapter/attempt-retention tests,
  canonical runner substitutes, hook fixtures, manifest/Rust formatting and
  strict tooling lint pass. Strict Rust Clippy passes for the helper/tests.
- Independent consumer fixtures clear inherited Make flags and logger checkout
  identities, following the current release rule. The old hook fixture failed
  under an inherited `FORMAT_CARGO=true`; the updated fixture passes. Actual
  parent Make calls with release selections and unrelated logger identities pass
  the adapter/hook fixtures, without invoking a real release or full gate.
- The selected yq 4.47.2 Linux executable was found in a local cache, verified
  against the consumer SHA-256 and version, then copied into owned qualification
  tooling. No network download was made. A curl substitute supplied those bytes
  through the actual consumer selection adapter/canonical installer; corruption
  refusal preserved the previously verified executable. The local canonical
  installer tests pass with synthetic payloads and platform substitutions. These
  checks do not prove real macOS execution or live release-asset downloads.

Logs, fixture sources/indexes, metadata JSON, cached parser and failed/inconclusive
attempts remain in `target/qualification/dependency-pins/`. The initial lint
failure for the consumer setup wrapper's ShellCheck source annotation and its
successful correction are retained, as are the interruption reproductions and
corrected launcher/adapter checks (`partial-lock-*.log`). Earlier local-export checks retain their
original scope; they are not relabelled as committed snapshot adoption.

| Qualified file | SHA-256 |
| --- | --- |
| `Makefile` | `dc478376835c6683d9f1c4cc14fe01468f7dc985d9cfa80c2abce231073f8497` |
| `examples/repo_tool/mod.rs` | `ce3914f67f62d8f4cc19e983ff1802977b3570522e2c18e775a59fd433b0c9d8` |
| `examples/repo_tool/tests.rs` | `7eb09b126929fe191c33fe9f8334333143026a97a69d16536b2ff92e0c2a28dc` |
| `.shared-tooling.snapshot` | `557d47e2c7e5711c6569037fc7844fbf47c46d19c191b334bdc6b4d02c5c6323` |
| `.github/workflows/ci.yml` | `50e40b8fd633e7a03d8a96066347e1c0db2391277c0864ebc4be2a15a0ec474c` |
| `ci-tool-versions.env` | `f7015ec217daa9a78f0deb80297e6e314fc02cd23be1e2439d7bb1367baa9cea` |
| `scripts/ci/test-release-adapters.sh` | `cae5f008c594193b9f64a3e427722fd009cccca9667fe69bab5a1dc08de776d5` |
| `scripts/ci/test-git-hooks.sh` | `6293a3caed33cdbfc04cbd4aeaf37f8eddef8469758d9e80a7198db31165a750` |
| `scripts/dev/install-yq.sh` | `939c06aa48fa0e2861db668046ad1d9eb1e7254855588f6d15adfed656a2eaac` |
| `scripts/dev/run-repo-tool.sh` | `a6263e1a3bf25d004a81644810823b1f5b7d1faad2dfbc6425a5bd1bcd4e832c` |

No function, method or type was removed or renamed. Full validation, Wasm budgets,
package/release gates, live pushes/publication and installed IC tests were not
executed. Supported native macOS and remote CI qualification for this batch
remain outstanding until matching source runs complete. Old source commits
without a tracked root lock cannot satisfy the new source binding: complete any
earlier unfinished release with its qualified source/tooling before adoption,
preserving its plans and evidence. See [maintainer steps](../RELEASING.md).

### Committed-lock hook fixture correction

On 2026-10-06 the maintainer's gate on source
`03e25c5ce1bc7bdf1af48d3a2e6b1993fffed26b` stopped in `test-hooks`, before
version preparation. The saved log remains at
`target/release-validation/attempts/verify.8TNLkL`. A focused trace reproduced
the failure at the no-lockfile assertion: the fixture exported HEAD's newly
tracked root and independent lockfiles before replacing their manifests with
synthetic packages. The earlier fixture pass used HEAD without tracked locks
and did not prove this committed-source case.

The consumer-owned fixture now removes only its inherited locks from its own
working tree/index before constructing synthetic manifests. The check that
formatting creates no locks or build output remains. The vendored hook and
installer are unchanged. Unexpected fixture failures report the failed command
and working directory, print captured output and retain the fixture.

Linux focused checks pass for hook selection/refresh, partial-stage refusal,
formatter-failure isolation, installation and logical path aliases, including
parent Make release selections and unrelated logger identities. Snapshot
verification, strict tooling lint and both-workspace formatting checks pass.
An intentional formatter refusal preserves status 2, diagnostics and fixture
files; a before/after checksum proves the real index is untouched. Both actual
lock hashes remain those recorded above. Logs and failed fixtures are retained
under `target/qualification/hook-tracked-locks/`, including the pre-fix trace,
initial lint failure and corrected checks. Qualified fixture script SHA-256:
`b097f44f230fff2628bf46a2bd7e005d69e5ab943a7e4b2d8d2ffd0592299856`.

This is a compatible correction within pending 0.27.2. No function, method or
type was removed. No full gate, release, publication or native macOS execution
was run by the agent; committed-source and matching CI qualification remain
maintainer steps. The real index, source/dependency versions and failed gate
logs are preserved.

## 0.27.1 selected-commit recovery

Local Linux x86-64 working-tree qualification on 2026-10-06, based on released
ic-memory `f2aefd140bc24c7befa6f170c2e2e25df2af667c`. Pending notes select 0.27.1;
package metadata remains 0.27.0. This section records the new batch separately
from the historical 0.26.0 adoption evidence below.

The 23-file snapshot exports committed Shared Tooling
`cb86188c5956866564de4fb6ec6be67b27981ab9` (0.1.4) through a clean temporary local
clone with the original upstream URL. The live sibling's dirty changelog and
distribution-test edits are preserved and are not attributed to that revision.
The maintenance rule is now part of the snapshot. Canonical runner/hook changes
are vendored unchanged; consumer adapter changes remain local and unstaged.

| Qualified consumer file | SHA-256 |
| --- | --- |
| `Makefile` | `a0abe82db42eec9a8d64a32aea96811221ec15cf9eafd35156f410c35142acba` |
| `examples/repo_tool/mod.rs` | `8a3e4ac97b850a8b4f5ae564023dac0ae63791923a0d8adbb739c1beb43d3f80` |
| `examples/repo_tool/tests.rs` | `fd9ec01c8562d513c2f3543f87832d2a33da9dbeb732a367098e39a9f3699aea` |
| `scripts/ci/test-release-adapters.sh` | `f22f4e1fdf87bcb104a5000ff3b72c30b9ca2a28e958e486914359ef5f41077c` |

Selected root lockfile SHA-256:
`56a2d80654983e476b1ebfec8d640a316649af4df5fd455822e67061cab2b4f6`.
Independent runtime-qualification lockfile SHA-256:
`7bc872926fbeddd23706750c6205582ba9eae6c873612794e06e862503bb6be7`.
No dependency, toolchain, package-version or receipt-schema change was made.

Focused checks pass:

- `make test-tooling` on Rust 1.99.0 and
  `make test-tooling VALIDATION_TOOLCHAIN=1.88.0`: all 20 adapter tests. The new
  recovery case keeps a newer committed callback fix at HEAD, verifies the exact
  older release and retained package, preserves receipt/archive bytes, and
  dispatches neither replacement packaging nor a full gate. Wrong selected commit,
  parent, ancestry or tag, corrupted archives and missing final evidence refuse.
  Compiler/configuration/lock binding and publication rejection tests remain.
- A follow-up regression first reproduced incorrect acceptance when HEAD moved
  to a newer descendant during final packaging. The final-package path now
  rechecks exact HEAD, separately from the ancestor check used for historical
  recovery. Rejection creates no final receipt or tag and preserves the prepared
  receipt/archive. This uses substituted effects, without real commits or packaging.
- The new real-index fixture clones existing local history, initializes only its
  own index and stages files without creating commits/tags. It rejects unrelated
  index entries and arbitrary metadata even after restoring working files, admits
  exact prepared bytes and rejects whitespace-altered bytes and mode changes.
  The same index guard runs before release commit admission.
- `make test-release-adapters`: actual consumer Make dispatch, all three release
  kinds, explicit resume, eight selection variables, distinct historical
  `RELEASE_COMMIT` forwarding on late callbacks and retained failed/successful gate
  logs. Its helper and entry-point runner are substitutes.
- `make test-release-runner`: canonical command-substitute fixtures cover older
  interrupted minor releases plus descendant fixes, subsequent patch/minor/major,
  same-kind/explicit retries, lost push replies, fresh validation failure/retry,
  exact original tag/evidence retention, selected atomic push and real Make callback
  dispatch. Git effects and consumer receipts are substitutes; this is distinct
  from the actual Rust adapter evidence above.
- `make verify-shared-tooling test-hooks fmt-check`, pinned Actionlint/ShellCheck
  through `make lint-tooling`, and strict
  `cargo +1.99.0 clippy --locked --offline --example repo-tool --tests -- -D warnings`.

Logs, including failed attempts, remain under
`target/qualification/shared-tooling-0.27.1/`. Initial strict Clippy caught an
oversized test; removing a redundant negative assertion fixed it. Extending the
guard to commit admission exposed missing staged-blob behavior in the command
substitute and a missing working file in the real-index fixture. Both fixtures
were corrected; both toolchain suites pass. Combining identical substitute match
arms then satisfied strict Clippy. No function, method or type was removed or
renamed in this batch.

The reproduced failure is retained in `head-change-before-fix.log`. After fixing
the guard, both complete adapter test suites and strict Clippy pass in the
`head-change-*` logs. Earlier 19-test logs retain their original source scope;
they are not relabelled as proof of this guard. The table above identifies the
final guarded adapter/test source.

Older-commit recovery reuses intact final evidence; it never manufactures missing
qualification from newer source. If the selected older commit lacks its final
receipt, it must be qualified at that exact source before recovery. Changed
lockfile/compiler/configuration inputs remain genuine refusal conditions. These
limits preserve the existing evidence contract; no historical receipt check is
bypassed and no parallel decoder or new recovery mode is added.

No maintainer release, real-index staging, commit/tag/push, publication, full
validation/Wasm/package gate or installed IC qualification was executed. Native
macOS ARM64/Intel and remote CI for these dirty edits still need matching evidence.
The release-commit and existing CI results do not qualify this working-tree batch.
This work implements the current local consumer scope of
[#10](https://github.com/dragginzgame/ic-memory/issues/10) and adopts the correction
from [Shared Tooling #5](https://github.com/dragginzgame/shared-tooling/issues/5).
Neither issue was modified. See [RELEASING.md](../RELEASING.md) for recovery and
the [physical-slot review](current-ledger-qualification.md#post-release-physical-slot-audit)
for the separate, unimplemented runtime simplification candidate.

## Source and scope

This record covers unstaged working-tree adoption of the common runner on
2026-10-05, based on ic-memory source `4d7881a`. It does not identify a release
commit or claim publication. Pending notes select **0.26.0** for the changed
maintainer CLI and mandatory receipt schema; package versions remain 0.25.14.
Library APIs and stable-memory formats are outside this batch.

The 22-file vendored snapshot records Shared Tooling revision
`f52c0e2476aee094359ed21de91c468540d3969f`. The shared source was clean at refresh.
Snapshot verification checks exact hashes/modes, independently of workflow tests.
Later local shared edits, including the user-triggered maintenance rule, apply
through the approved AGENTS overlay but are not attributed to that revision.
No sibling files were edited by this work.

Consumer files qualified on Linux x86-64:

| File | SHA-256 |
| --- | --- |
| `Makefile` | `f44052ec090650dcf8507c3047887d55a223a4a93a03536dd599d3746fece63c` |
| `examples/repo_tool/mod.rs` | `73b3fdcb323629993ba1729925334a7892230c7590b853ee7e6d55767654facf` |
| `examples/repo_tool/tests.rs` | `b103ae03f11c5edd859e33dc76abf4cc6bdbfc8b48db79ef79ac6508b151e4ff` |
| `scripts/ci/test-release-adapters.sh` | `718af94789af60a52cdefd73009410af9734bb46a4e07ef37cf726e987a5ea60` |

Selected lockfiles were preserved:

- Root: `a330429dc9941eb36fb1fb8d8ebc4fffe2c73e9b98fd1d6247b9c6b8e37b87af`.
- Isolated qualification workspace:
  `7bc872926fbeddd23706750c6205582ba9eae6c873612794e06e862503bb6be7`.

## Focused results

- Rust 1.99.0 and 1.88.0: all 17 `repo-tool` tests pass. Git, Cargo mutation,
  publication and full-gate commands are substituted. Real fixture boundaries
  use SHA-256 and the read-only canonical increment helper. Tests cover all three
  candidates, dated root/detail metadata, original dependency selection, saved
  validation before mutation, missing/mismatched evidence, compiler replacement
  refusal, preparation rollback, interruption after partial metadata/manifest/lock
  writes, failed packaging, corrupted retained artifacts and final package retry.
  Completed prepared checks dispatch neither packaging nor another gate.
- `make test-release-adapters`: actual consumer Make recipes dispatch all three
  maintainer entry points and optional resume to a harmless runner substitute.
  Named adapters forward all seven selections. Multiple release selections stop
  before dispatch. Failed and successful gate attempts retain separate complete
  stdout/stderr logs under a target path containing spaces. This does not exercise
  actual Rust qualification or Git effects.
- `make test-release-runner`: unchanged canonical command-substitute suite passes.
  It covers increments/order, exact staging and atomic push scope, locks, fresh
  preflight/validation after early failures, saved-intent selection and recovery
  after uncertain Git effects. This suite substitutes consumer Make adapters;
  it does not prove their implementation. These three test layers are distinct.
- Strict `cargo +1.99.0 clippy --locked --offline --example repo-tool --tests --
  -D warnings` passes. This compiles helper and root test targets, without running
  the full suite or qualification gate.
- The moved generation-integrity reference test and the adjusted four-GiB IO
  boundary test each pass as selected root library tests.
- Snapshot integrity (22 files), both-workspace formatter checks, Actionlint,
  ShellCheck and whitespace checks pass. Lint uses separately installed pinned
  Actionlint 1.7.12 and ShellCheck 0.11.0; no tool installation occurred.
- Actual consumer hook fixture checks pass, including aliased checkout setup,
  index selection, partial staging refusal and failed formatter isolation. These
  fixtures never create commits or tags.

Reproduction commands:

```sh
cargo +1.99.0 test --locked --offline --example repo-tool
cargo +1.88.0 test --locked --offline --example repo-tool
cargo +1.99.0 clippy --locked --offline --example repo-tool --tests -- -D warnings
cargo +1.99.0 test --locked --offline --lib generation_validation_matches_reference_for_orderings_and_overlapping_failures
cargo +1.99.0 test --locked --offline --lib byte_io_crosses_four_gib_with_discontiguous_buckets
make test-release-adapters test-release-runner
make verify-shared-tooling test-hooks fmt-check
# With the separately installed pinned lint tools on PATH:
make lint-tooling
git diff --check
```

Logs, including failed attempts, are retained locally under
`target/qualification/release-adapters/`. Initial helper run: 10 passes/3 fixture
failures (nested detail-directory setup, remote inspection sequence and retired
consumer tag-collision ownership). Third run: 16 passes/1 failure exposed a
non-idempotent Cargo-update substitute that changed a dependency after root refresh.
Correcting that substitute made interrupted refresh recovery observable.
Initial strict Clippy found two helper issues plus two pre-existing test-only
issues; a later adapter split needed the same semicolon-style correction. Final
strict Clippy passes. Earlier logs remain labeled as failed evidence.

## Ownership and retirement

The old Rust `next_version` function is replaced by the canonical shell helper.
`Repository::prepare` is replaced by separately validated selected-intent adapters;
`Repository::stage`, `Repository::commit` and `Repository::push` are removed because
Git effects belong to the common runner. These former methods were in
`examples/repo_tool/mod.rs`. `ReleaseEvidence` is restructured/renamed to
`PackageEvidence`, containing `ValidationEvidence` and `ReleaseSelection`;
this is a mandatory schema cut, not a compatible type alias.

In `examples/repo_tool/tests.rs`, rename
`release_versions_and_drafts_preserve_dependency_versions_and_history` to
`release_versions_and_pending_notes_preserve_dependency_versions_and_history`, and
`dirty_sources_remote_collisions_and_changed_validation_inputs_prevent_version_edits`
to `dirty_sources_and_changed_validation_inputs_prevent_version_edits` to describe
current maintained contracts. Consumer fixtures now refuse mutating Git commands;
test-owned effect models and canonical substitutes cover that handoff.
The two integrity-test functions move unchanged within the same module to satisfy
Clippy. The IO test omits an unnecessary clone; production behavior is unchanged.

## Qualification limits

No actual release target, full validation/Wasm/package gate, network publication,
Git commit/tag/push or index staging was executed. The test layers above are not
an assembled live release or evidence of remote adoption. Native macOS 15 ARM64
and x86-64 require matching CI evidence, separately from this Linux pass. The
existing native CI gate includes these focused release tests through the Makefile;
configured jobs alone do not establish passing qualification.

No runtime dependency, toolchain, durable format or library API changed. Two
library source files have only test lint fixes/movement. No Wasm, instruction,
stable-memory or runtime performance reduction is claimed. Recovery avoids repeated
full gates and completed package work in substitute tests; no wall-time savings
are measured. Finish outstanding earlier releases with their original tooling
before this hard cut; preserve all earlier receipts and artifacts.


## Committed-adoption hook fixture correction

On 2026-10-05, the maintainer's full release gate at source
`d1a914870ce4515c66296cdb08607a6fc5e329ca` passed snapshot, Rust helper, consumer
Make adapter and canonical runner tests, then failed in `test-hooks`. This was
before version preparation; the canonical manifest remains 0.25.14 and the
pending candidate remains 0.26.0. This failed gate is not release qualification.
Its original log is retained unchanged at
`target/release-validation/attempts/verify.hx3N8v`, SHA-256
`23838ff2284e91e94f2e62a296eea20fbc1526d3865389d66896a45e7d43aec6`.

All 22 snapshot entries still match the current committed Shared Tooling revision
`f52c0e2476aee094359ed21de91c468540d3969f`, including hook/installer/runner bytes.
Local shared maintenance-policy edits remain dirty, active via AGENTS, and outside
that recorded revision. No shared executable update is required for this failure.

The consumer `new_fixture` function used global loop variables. It overwrote the
outer partial-staging case's `path` with `scripts/dev/install-git-hooks.sh`, so
those cases edited the installer instead of their intended source/configuration
files. Once adoption was committed, that installer also no longer differed from
HEAD, leaving it unselected. The hook correctly preserved the unrelated edit;
the fixture incorrectly expected rejection. The earlier working-tree hook pass
masked both faults and did not prove the four intended partial-staging cases.

The correction keeps fixture loop variables local, explicitly adds a staged edit
to each case and verifies its selection before adding the unstaged edit. Cases
exercise `src/lib.rs`, `Cargo.toml`, `Makefile` and `ci-tool-versions.env`
independently of whether consumer tooling already matches HEAD. An unexpected
success reports its fixture directory and operation. No function/type was removed
or renamed; the vendored hook and installer are unchanged.

Corrected script SHA-256:
`967b16da0202798accc080fd0d480809c02aa857a5ab70aa2527dcb0c4f783fd`.
Focused Linux checks pass:

- `make test-hooks`.
- `make --no-print-directory test-hooks VALIDATION_TOOLCHAIN=1.99.0` with all
  seven release selections passed as Make variables, matching the failed
  release's kind/versions/source/date/remote/branch. This is only a focused hook
  check, not a rerun of the full gate or maintainer release command.
- Pinned ShellCheck 0.11.0 on the corrected script, `bash -n`, 22-file snapshot
  verification and whitespace checks.

Failure trace and passing focused logs are retained under
`target/qualification/release-adapters/hook-repair-d1a9148/`. The root and isolated
qualification lockfile digests remain those recorded above. All edits are
unstaged; no release, publication, original-index staging, full gate or artifact
cleanup was run by the agent. Native macOS and a successful maintainer release
still require their own observations.

## 0.28.1 shared release-command adoption

This focused Linux review uses source
`6096c2ed597dee248823b011811a48bc00dce05d` plus the pending snapshot, adapter,
instruction and documentation edits. The unchanged 46-file Shared Tooling 0.1.7
snapshot selects `47cd2ccaf0e8b428f06e6db0262df76cfc1581de`. Every exported byte,
digest and executable state was compared with that committed source. The active
code-hygiene method is unchanged; its local source reference was refreshed.

The [shared checker](../scripts/ci/check-release-commands.sh) now owns generic
Make dispatch. The consumer passes only its reviewed parse-time inputs,
`rust-toolchain.toml` and `ci/tool-versions.env`. Local fixtures retain selection
forwarding, gate-attempt retention and locked launcher/publication-environment
checks. The [helper contract](verification-helpers.md) defines the boundary.

Focused checks passed with Bash 5.2.21, GNU Make 4.3, Actionlint 1.7.12 and
ShellCheck 0.11.0:

- `make verify-shared-tooling test-release-adapters`.
- `make test-tools` and `make lint-tooling`.
- The frozen upstream `test-file-digests.sh`, `test-release-commands.sh` and
  `test-release-metadata.sh`, including GNU/Perl digests, nested Make isolation,
  retained failures and common changelog-finalizer rejection/history behavior.
  These are upstream fixtures, not consumer release qualification.
- Unchanged selected manifests/lockfiles and Makefile, preserved published
  changelog history, and unchanged captured qualification inputs after the checks.

Source identity, input hashes and full logs are retained under
`target/qualification/shared-release-checker-0.28.1.syrqcuz5/`. Tests use substitute
release effects or disposable indexes and existing history; they create no
commits, tags, pushes or publication. No named function, method or type was removed.
The local adapter shrank by 17 lines; added shared exports remain upstream-owned.
Library code and dependency selection are unchanged. No full gate, Wasm-size
measurement, installed IC qualification or native macOS execution was performed;
matching remote CI remains separate evidence for the pending edits.

## 0.28.2 independent Make fixtures

Focused Linux evidence uses source
`086b63a95ab9c5581115f56b68788f961e22c8e4` plus the two fixture environment-reset
edits and pending documentation. The shared snapshot remains at 0.1.7,
`47cd2ccaf0e8b428f06e6db0262df76cfc1581de`, matching the clean local checkout and
the latest observed upstream commit. Its checker already clears all five Make
controls; no shared-source change is needed.

Before the fix, a dry-run parent Makefile exported `GNUMAKEFLAGS=-n`, `MAKEFILES`
pointing to a rejecting include, and foreign validation logger identities. Its
recursive recipe executed the independent fixtures. The shared dispatch checker
passed, but the local release fixture then read the parent's include and failed.
The hook fixture likewise reached that include during its disposable hook check.
Both failures and the hook's retained fixture are preserved.

After clearing the two additional inherited Make controls, these focused checks
passed:

- Normal `make verify-shared-tooling test-release-adapters` and `make test-hooks`.
- The same parent Makefile invoking each fixture with dry-run/include/logger
  contamination; both completed their maintained positive and rejection checks.
- `make lint-tooling`; selected lockfiles, manifests, snapshot, Makefile, real hook
  and captured qualification inputs remained unchanged.

The parent fixture, before/after logs and input hashes are retained under
`target/qualification/make-fixture-isolation-0.28.2.6ank19sr/`. Release effects are
substitutes; hook/index operations use disposable repositories and existing
history. No original-index staging, commits, tags, pushes, publication, dependency
fetching or full gate was performed. No function, method or type was removed.
Library/Wasm source is unchanged. Native macOS and matching remote CI for these
pending edits remain separate qualifications.

## 0.28.3 shared Cargo and installer adoption

Focused Linux evidence uses consumer HEAD
`51a28a666b8b2000e0f63ed664ea47d00127d2ca` plus the pending adoption and the
maintainer's pre-existing root Cargo edits. The 50-file snapshot exports committed
Shared Tooling 0.1.8 at `d957d1f8801885c5b69e4a9ef900155f5f2a8a9d` from a clean
private checkout. A first refresh refused that private clone's local origin
before replacing any export; correcting only its origin URL permitted the normal
refresh. No dirty upstream bytes or vendored-file patches were used.

These focused checks passed:

- Snapshot integrity, the actual `make check-pins` caller and both shared pin/
  Cargo metadata fixtures. The metadata fixture is now part of `make test-pins`
  and therefore the existing native CI/release gate.
- A disposable export of the actual consumer sources and current manifests/
  lockfiles: the real Make caller passed, rejected an independent build dependency
  separately in the root and runtime-qualification manifest with
  `cargo-inheritance` findings, then passed after restoration. Both selected
  graphs' manifests and lockfiles were preserved byte for byte.
- `make test-tools`, `make test-release-adapters`, strict tooling lint and
  non-mutating formatting checks for both workspaces.
- The pin/metadata, tool and release-adapter checks under GNU Bash 3.2.57 on
  Linux, with that Bash selected for child commands too. This tests the shell
  baseline, not native macOS userland.
- The committed upstream installer fixture under Bash 5 and Bash 3.2.57, covering
  asset mapping, checksum/version refusal, candidate retention and preservation
  of installed executables. These are substitutes, not official asset downloads.

Logs, exact dependency-input hashes, source identity and the disposable consumer
check are retained in
`target/qualification/shared-cargo-installers-0.28.3.BNFCxs/`. The maintainer's
root `ic-host-tools` 0.2 selection and both original workspace lockfiles were
preserved; this adoption did not resolve dependencies or qualify that dependency
upgrade through compilation. The two old installer entry points contained
`usage`, `resolve_platform` and `main`; those six functions are removed from the
entry points in favor of the upstream-owned common installer. Its 88 lines plus
the two five-line wrappers replace 246 lines, a 148-line reduction in that surface.
No library API, durable layout, Wasm source or product Rust implementation changes.
No Wasm-size or instruction-cost measurement is claimed.

Native qualification remains outstanding. Shared Tooling
[CI run 37484175750](https://github.com/dragginzgame/shared-tooling/actions/runs/37484175750)
tests the selected `d957d1f` source: Linux and lint pass, while both macOS portable
jobs fail after IC fixture completion and before the host fixture reports
completion. Official host installation/offline checks and the installer fixture
passed earlier in both jobs. Suppressed host-fixture output does not identify the
exact assertion; this is not proof of an official asset or installer defect.
The upstream failure is already recorded in
[shared-tooling #17](https://github.com/dragginzgame/shared-tooling/issues/17#issuecomment-6019280848).
The sibling remains read-only under this repository's instructions.

The earlier successful
[0.28.2 CI run](https://github.com/dragginzgame/ic-memory/actions/runs/37466450294)
does not qualify these dirty adoption inputs or the maintainer's Cargo edits.
Matching consumer native CI and upstream host-fixture resolution are required
before treating [#13](https://github.com/dragginzgame/ic-memory/issues/13) as
qualified. No full gate, compilation, live installation, release command,
original-index staging, commit, tag, push or publication was performed.

### Pending upstream host-fixture repair

On 2026-10-06 the upstream repair and formatter prerequisite helper remain
uncommitted after `d957d1f`. Their selected working-tree scripts were copied into
private qualification inputs; they were not installed into the consumer snapshot.
Input hashes, before/after logs and the reproduction scripts are retained under
`target/qualification/shared-host-repair-0.28.3.D74GNJ/`.

The reproduction delegates archive creation/extraction to GNU tar, changing only
the valid gzip modification-time header on each creation. This simulates a
non-reproducible archive header without changing the tar/executable payload. The
committed old fixture fails under both Bash 5 and Bash 3.2.57; the repaired fixture
passes under both. The old fixture's retained repacked archive remains valid gzip
and extracts the identical executable, but its SHA-256 differs from the originally
authenticated archive. Restoring the saved archive bytes avoids that mismatch;
the repaired cases also prove that version/PCRE2 rejection actually executes the
authenticated payload. This is a targeted Linux simulation, not reproduction or
qualification on native macOS.

The pending shared formatter fixture passes under both shells, including failed
status with apparently correct version output and unavailable rustfmt. Its actual
offline probes of this consumer's prepared Rust 1.99.0/cargo-sort 2.1.4 also pass.
No formatter helper is adopted from dirty upstream source. A maintainer commit
containing these changes is required before refreshing the snapshot and wiring
the formatter caller; matching native CI remains separately required. The
maintainer's root Cargo edits and both selected workspace lockfiles remain
unchanged. This qualification does not add a release gate or authorize Git effects.

## 0.28.4 portable fixtures and formatter admission

On 2026-10-06, prepare the compatible tooling batch over released consumer source
`099dbeab7d63a8255ab59cc5e43129c24c8dd4d1`. The 52-file snapshot selects committed
Shared Tooling `21f3ec3dd97f2968c9f0b08924451bb2f71770d1` (0.1.10). A clean private
checkout of that exact commit supplied the canonical refresh helper and exports.
The live sibling's later uncommitted changes were not copied or attributed to
that commit. The earlier pending-source evidence above retains its original
identity; this is a new consumer adoption attempt.

The common host fixture now restores saved authenticated archive bytes rather
than repacking them, and verifies that its version/PCRE2 refusals execute the
authenticated payload. The Make formatter prerequisite delegates to the shared
checker with `RUSTUP_TOOLCHAIN=1.99.0` and cargo-sort 2.1.4. Both maintained
workspaces keep their actual sort/fmt commands. The shared prerequisite fixture
runs before the consumer hook fixture; the disposable index includes the newly
adopted checker. No original-index hook execution or activation was performed.
The separate Rust changelog-parser consolidation is deferred.

Focused Linux execution passes:

- `make verify-shared-tooling check-pins test-pins test-tools`, including actual
  two-workspace Cargo inheritance inspection and the repaired host fixture.
- `make test-hooks fmt-check`, including wrong/failed version probes, missing
  rustfmt, actual both-workspace sorting and Rust formatting, selected-file
  refresh, partial staging refusal and unrelated-edit/failure isolation.
- `make test-release-runner test-release-adapters` with substituted effects,
  including the refreshed saved-identity/finalized-changelog fixture cases.
- `make lint-tooling` and the committed upstream installer fixture. The latter
  uses substituted assets; it does not install official tools into this checkout.

The snapshot, pin, tool, runner, adapter, hook and formatting checks also pass
with GNU Bash 3.2.57 selected on PATH for child scripts. The upstream installer
fixture passes under that shell too. These are Linux shell-baseline checks,
not native macOS execution. Local documentation-path and diff-whitespace checks
pass. Full logs and the exact dependency-input hashes are retained in
`target/qualification/portable-tooling-0.28.4.AuXLJb/`. Both manifests and both
selected lockfiles remain byte-for-byte unchanged. The package remains 0.28.3;
only the next undated changelog candidate is 0.28.4.

Native evidence is narrower than a complete green gate:

- The released consumer's
  [run 37490382402](https://github.com/dragginzgame/ic-memory/actions/runs/37490382402)
  tests `099dbeab`: Linux, lint and all three MSRV jobs pass; both macOS native
  gates fail at the old host-tool fixture. Hook and Cargo fixtures passed before
  that failure. The failure remains recorded and does not qualify these edits.
- Shared Tooling
  [run 37491682760](https://github.com/dragginzgame/shared-tooling/actions/runs/37491682760)
  tests the selected `21f3ec3` commit. Both macOS architectures pass the repaired
  host-tool fixture and formatter prerequisite checks, then fail later in
  `test-fixture-retention.sh`. That separate fixture is not exported here.
  Upstream Linux and lint jobs pass; the complete upstream run remains failed.

Matching consumer Linux/macOS CI after the maintainer's commit and push remains
required for native adoption qualification of
[#13](https://github.com/dragginzgame/ic-memory/issues/13) and
[#14](https://github.com/dragginzgame/ic-memory/issues/14). Configured native CI
already runs the maintained hook and formatting targets; no workflow expansion
is needed. Shared ownership and native fixture observations do not substitute
for this consumer's full gate.

No library API, durable layout, dependency selection, product Rust code or
canister source changes. No functions, methods or types are removed in this
batch; the replaced Make admission recipe retains its target. No compilation,
Wasm-size measurement, full gate, live installation, release command, original
index staging, commit, tag, push, publication or GitHub write was performed.

## 0.28.5 consumer release-fixture retention

On 2026-10-06, inspect released source
`d56af42b0bd9ac5a794de2248e31b335945a1e1c` (0.28.4). The consumer shell adapter
fixture's unconditional EXIT cleanup and Rust `Fixture::drop` both removed
unsuccessful test inputs. The shared command/runner fixtures already retain
failure evidence. Change these two local cleanup owners without changing the
52-file snapshot at `21f3ec3dd97f2968c9f0b08924451bb2f71770d1`.

Rust unwinding now preserves the fixture and attempts to save the already
serializable substitute state. Borrow/serialization/write errors are reported
without introducing a second panic; the original failed assertion remains the
test failure. Successful runs retain their existing scratch cleanup. The shell
trap preserves nonzero status, retains its fixture and reports its path.

Focused Linux execution passes:

- `make test-tooling`, including the new unwinding regression. It confirms
  validation receipts, selected lock bytes and substituted command history
  survive failure; a successfully dropped fixture is removed.
- `make test-release-adapters` under Bash 5 and GNU Bash 3.2.57. A private Make
  substitute injects exit status 7 at the consumer committed-check call after
  earlier calls succeed. Both shells retain the actual fixture and its command
  trace with the failed status; the uninjected adapter runs pass.
- Strict Clippy for the repo-tool example and test targets, ShellCheck of the
  changed consumer adapter script, and formatting checks for both workspaces.

Logs, injected failure fixtures and exact dependency/input hashes are retained
under `target/qualification/consumer-fixture-retention-0.28.5/`. Both manifests
and both lockfiles remain byte-for-byte unchanged. The manifest stays at 0.28.4;
the next undated changelog candidate is 0.28.5. No functions, methods or types are
removed. This change is confined to test execution and does not affect shipped
Rust code, Wasm, memory layout or release receipts. No full gate, network setup,
live release operation, original-index staging, commit, tag, push or publication
was performed.

The released source's
[CI run 37498603294](https://github.com/dragginzgame/ic-memory/actions/runs/37498603294)
tests `d56af42b`, separately from these pending changes. At inspection, Linux
and ARM macOS full gates, all three MSRV checks and tooling lint pass; Intel
macOS's hook and formatting checks pass while its full gate is still running.
This is incomplete native evidence and does not qualify the pending retention
fix. Earlier failures remain recorded above.

A subsequent inspection on 2026-10-06 confirms that the same run completed
successfully: all seven jobs pass, including the Intel macOS full gate. This
qualifies the released 0.28.4 source on all three declared native CI hosts;
the pending 0.28.5 retention fix still requires its own matching native run.
The maintainer-authorized report is now
[#15](https://github.com/dragginzgame/ic-memory/issues/15); no local artifacts or
filesystem paths were published with it.

## 0.29.0 shared owners and CI diagnostics

On 2026-10-06, qualify source `d56af42b0bd9ac5a794de2248e31b335945a1e1c`
(released 0.28.4) plus the pre-existing pending fixture-retention changes and
this maintainer-authorized shared-owner adoption. The earlier 0.28.5 evidence
above retains its original scope; its unpublished notes now belong to the
complete pending 0.29.0 batch. The minor candidate reflects the expanded offline
host-set admission: existing jq/yq-only setups must run `make install-host-tools`.
The package manifest remains 0.28.4; no release effect is part of qualification.

The canonical refresh exports Shared Tooling 0.1.12 at
`33c2a6f0018a94915f819ff219e270500ed5b73b` from a clean private checkout. The
54-file snapshot includes the exact-commit CI helper and the newly linked
governance roster. The first local-link check found that roster absent from
the old file selection; adding its committed export resolves the consumer gap.
The sibling checkout remains read-only. The baseline and maintenance rule are
refreshed together; the local maintainer-owned Git effects, broad-gate boundary,
live local-policy exception and independent-workspace scope remain explicit.

The governance walkthrough uses this authorized local repair and focused checks.
For the upstream-owned finding, the already-reported host-fixture failure and
[Shared Tooling #17](https://github.com/dragginzgame/shared-tooling/issues/17)
retain their original source/evidence above; the committed owner correction is
present in this refreshed snapshot. Issue reporting, upstream acceptance and
consumer qualification remain distinct. No new upstream finding is invented
and no sibling repair is performed.

Review width is 30 changed/new files, approximately 1,000 added and 240 removed
lines including the initial pending retention work. Most width comes from
canonical snapshot propagation, its new helper/roster, documentation and focused
fixtures. Local semantic areas are host setup, filesystem effects, dependency
selection and CI diagnostics. Production orchestration keeps its existing
state model while delegating file mechanics to their canonical owner.

Host ownership is now split: `ic-host-fs` owns durable file writes and hashing;
`ic-host-artifacts` owns digest identities. Both selected registry crates are
0.3.0 with unused default features disabled. The root lock adds those two crates
and removes the old monolith and eight unused transitive packages. Every retained
package keeps its version, source, checksum and dependency selection. The
runtime-qualification manifest and lock are unchanged. The new host crates,
sha2, rustix, flate2 and wasmparser are absent from the Wasm normal/build/dev graph.

Removed symbol: `examples/repo_tool/mod.rs::write_atomic`. Its staging, write,
sync, rename and cleanup implementation is replaced by
`ic_host_fs::durable::write_bytes` through the existing effect boundary. The
consumer still owns serialization, metadata ordering, saved intent, receipts
and rollback/retry. The process/file boundary supplies deterministic test
substitutes without a production test-only filesystem path or new runtime mode.

Focused Linux x86-64 qualification passes:

- Real pinned jq/yq/ripgrep installation and offline verification, including
  PCRE2 admission. The first sandbox attempt failed at DNS and retained
  `.tools/host-set.4MBe6V`; the authorized network-capable retry passed. These
  observations do not qualify native macOS installation.
- All 26 `repo-tool` tests on Rust 1.99.0 and 1.88.0. Real publication checks
  prove complete replacement while an open reader retains the old inode,
  rejected-directory evidence preservation and staging cleanup. Substituted
  failures before and after visible publication cover each release metadata
  surface, original-byte rollback, receipt/artifact preservation and retry
  using the same saved validation intent.
- Strict Clippy for the example and test targets; both-workspace formatting;
  actual consumer hook isolation and shared formatter prerequisites.
- Snapshot integrity, two-workspace dependency declarations and pin/Cargo
  metadata fixtures; host/IC/evidence fixtures; actual Make release adapters
  and canonical runner substitutes, including destination-drift refusals.
- The committed installer fixture and the shared CI-helper fixture executed
  against the consumer's exact exported helper bytes. These use command and
  asset substitutes and do not prove live GitHub CI or official installer assets.
- Workflow lint and ShellCheck. Executing the actual fixture-directory and
  validation-log steps with a failing Make substitute preserves exit status 23,
  both output streams and a retained receipt. Structured upload selection checks
  confirm failure-only collection, hidden evidence and 14-day retention. No live
  GitHub artifact upload is performed.
- Locked offline no-deps metadata for both workspaces and focused Wasm checks
  for `repo-tool` and the core size probe. These are compilation/graph checks,
  not new Wasm-size or IC instruction measurements.

Input hashes, source/environment identities, initial dirty edits, logs and
the CI substitute harness are retained under
`target/qualification/shared-owners-0.28.5.SFQuos/`. The directory retains the
candidate name used at the start of preparation; the source hashes and complete
pending notes establish this batch's actual identity.

Full gates, native macOS execution, installed PocketIC, live release/publication,
original-index hooks, staging, commits, tags and pushes were not performed.
Matching consumer native CI after the maintainer's commit and push is required
for the changed source; earlier green runs do not qualify it. Implementation
and adoption correspond to [#15](https://github.com/dragginzgame/ic-memory/issues/15),
[#16](https://github.com/dragginzgame/ic-memory/issues/16) and
[#17](https://github.com/dragginzgame/ic-memory/issues/17). Their committed-source
or native-qualification acceptance steps remain separate from this local result.

### Committed source and native CI confirmation

On 2026-10-07, inspect released 0.29.0 at
`7a20c685cc83949b7c0e6af9b16816c3a8e7cdd6` on the verified
`dragginzgame/ic-memory` main branch. The all-workflow listing for that exact
commit contains one applicable push run,
[CI 37580768769](https://github.com/dragginzgame/ic-memory/actions/runs/37580768769),
attempt 1, completed successfully. All seven jobs pass: tooling lint, full
pinned-toolchain validation on Ubuntu 24.04 and macOS 15 ARM/Intel, and MSRV
checks on those same three native hosts.

The job-step observations confirm successful repository-local host/IC setup,
both-workspace formatting, actual disposable hook isolation and the complete
configured validation gate on every supported host. That gate includes the
consumer release adapters, canonical runner substitutes, dependency and
metadata fixtures, and `repo-tool` tests. These results qualify the committed
shared-owner adoption and failed-fixture retention changes. They supersede the
pending native qualification stated above without changing its earlier local
evidence or implying a live release was exercised.

The adopted Shared Tooling 0.1.12 source at
`33c2a6f0018a94915f819ff219e270500ed5b73b` also has matching successful
[CI 37511845192](https://github.com/dragginzgame/shared-tooling/actions/runs/37511845192):
portable regression on Linux and both native macOS architectures, plus
lint/security. Upstream qualification and consumer adoption are separately
observed.

A focused local recheck verifies all 54 snapshot files. Disposable exports
accept unchanged bytes, reject checksum-helper-only corruption, and reject
combined checksum-helper/payload corruption without executing the corrupted
helper. The live checkout and its snapshot remain intact. This completes the
remaining integrity evidence requested by
[#18](https://github.com/dragginzgame/ic-memory/issues/18); release destination
and retry behavior are qualified through the substituted runner in the native
consumer gates, with no real Git effects.

CI observations and the three snapshot-check logs are retained under
`target/qualification/ci-adoption-0.29.0-2026-10-07/`. Failure-only artifact
uploads were skipped in this successful run, so live failure upload remains
unobserved. Installed PocketIC qualification, publication and deployment are
outside this evidence. No CI rerun, dispatch, original-index hook, commit, tag,
push or release command was performed by the agent.

## Workspace layout adoption for #19

On 2026-10-07, prepare the authorized
[#19](https://github.com/dragginzgame/ic-memory/issues/19) change against released
0.29.0 source `7a20c685cc83949b7c0e6af9b16816c3a8e7cdd6`. Preserve the preceding
uncommitted native-CI confirmation. The new numbered pending entry is 0.30.0:
checkout path dependencies and package-specific manifest references must select
`crates/ic-memory/`. Package versions remain 0.29.0 and 0.0.0; this preparation
does not execute a release or change dependency selections.

The maintained manifest inventory contains two approved roots and two packages:

| Owner | Former package manifest | New package manifest | Selected lockfile |
| --- | --- | --- | --- |
| Library workspace | `Cargo.toml` | `crates/ic-memory/Cargo.toml` | `Cargo.lock` |
| Independent installed qualification | `testing/runtime-qualification/Cargo.toml` | `testing/runtime-qualification/crates/ic-memory-runtime-qualification/Cargo.toml` | `testing/runtime-qualification/Cargo.lock` |

Both former manifests become virtual roots with explicit resolver 3, members,
default members, shared package metadata and dependency catalogs. The library
root retains its profiles and lint selections. Its source, examples, tests,
current wire fixtures and full package guide move together; the independent
binary's source moves under its own package. No old package, source tree or
maintained symlink alias remains. Repository scripts, operational documentation
and application identities retain their owners. The package's MIT license is an
unchanged copy of the repository-owned license for archive inclusion.

The canonical snapshot refresh exports 55 files from reviewed committed Shared
Tooling 0.1.13 at `e378671d90afa237ff63a4b0e3b9551eb2c222b6`, explicitly adding
`rules/rust-workspaces.md`. A clean private checkout supplies the export; the
active sibling's newer uncommitted script changes are excluded. Its matching
[upstream CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37581058940)
passed; that observation is separate from consumer qualification.

Product release parsing now selects `[workspace.package]` and requires the
named member to inherit its version. The private function
`examples/repo_tool/mod.rs::replace_package_version` is moved with its module and
renamed `replace_workspace_version`; its replacement edits the root workspace
field. No Rust function, method or type is otherwise removed. Release metadata
still publishes the root manifest last, uses the same selected lock/receipts,
and reconciles visible publication before rollback or retry. The canonical
dependency example moves to the package guide. A retained bootstrap projects
both manifests and source paths when the lock already selects the candidate.

Focused Linux x86-64 results:

- Both roots pass locked offline metadata and formatting. Metadata comparison
  preserves package identities, versions, effective dependencies/features,
  targets, publication selection, edition/MSRV and owning workspace/target
  roots. Both selected lockfiles are byte-identical to the initial inputs.
- All 27 `repo-tool` tests pass on Rust 1.99.0 and 1.88.0, including the named
  member inheritance boundary and existing interrupted preparation, visible
  publication failure, rollback, receipt and retry cases. Consumer Make-adapter
  and actual disposable hook fixtures pass with both virtual-root layouts,
  including partial-stage refusal for root and member manifests.
- A real Cargo launcher check with an intentionally advanced fixture lock
  compiles the projected workspace, returns the caller's 0.29.0 identity and
  preserves both caller manifest and lock bytes. Git/release effects are absent.
- Nine current wire-fixture tests and all nine maintained compile-fail cases
  pass. Library source, ordinary examples, public boundary tests and durable
  fixture bytes are unchanged by their move. The independent PocketIC binary
  compiles locked/offline against its separate existing graph.
- Strict example/test Clippy, focused core-probe/repo-tool Wasm checks, snapshot
  integrity, dependency inheritance, workflow lint and ShellCheck pass.
- Package inventories preserve all 96 former package-owned source/example/test/
  fixture/guide/license inputs. Root invocation selects the same sole package.
  Repository governance, CI and operational evidence cease to be incidental
  archive payload. This is inventory/source-path verification, not a full
  package verification or publication.
- Current navigation and pending notes pass 235 local link checks across 50
  documents. The initial whole-ledger check found a historical published link
  to `examples/composed_host.rs`; its original entry is preserved, and only the
  new pending ledger section is included in current-navigation qualification.

Inputs, metadata, package inventories, focused logs and the real Cargo bootstrap
fixture are retained in `target/qualification/workspace-layout-19/`. Current
navigation follows moved sources; historical release notes and recorded hash
tables retain their original identities. A sibling-manifest scan finds no
checkout path dependency on ic-memory requiring an owning-repository update.

The successful 0.29.0 consumer CI above does not qualify this dirty layout
change. Matching native Linux/macOS CI after the maintainer's commit and push
remains required before closing #19. No broad local gate, full packaging/release
qualification, installed PocketIC server, staging, commit, tag, push,
publication, deployment or maintainer release command is performed.
