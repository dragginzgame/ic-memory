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
