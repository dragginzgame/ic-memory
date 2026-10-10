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

### Native confirmation for released 0.30.0

[Consumer CI run 37587252306, attempt 1](https://github.com/dragginzgame/ic-memory/actions/runs/37587252306)
completed successfully for exact released source
`2fdeec4582bbcffcaa09aac625f48a47b29e3194`. All seven jobs pass: tooling lint,
full validation on Ubuntu 24.04, macOS 15 ARM64 and macOS 15 Intel, and separate
MSRV checks on those three hosts. This closes the native qualification condition
for [#19](https://github.com/dragginzgame/ic-memory/issues/19).

The released lock selects `ic-host-artifacts` and `ic-host-fs` 0.3.1, with registry
checksums matching the official Cargo index. These CI results qualify that actual
released graph and the recorded 0.1.13 snapshot. The original preparation's
unchanged-lock comparisons above retain their 0.29.0 input scope. This run does
not qualify subsequent working-tree changes for #20 or prove publication,
installed PocketIC qualification or live release effects.

## Make execution adoption for #20

Prepared against released ic-memory 0.30.0 source
`2fdeec4582bbcffcaa09aac625f48a47b29e3194` for
[#20](https://github.com/dragginzgame/ic-memory/issues/20). This is local
working-tree evidence, separate from the native confirmation for #19 above.

The canonical refresh exports 56 files from a clean private checkout of reviewed
committed Shared Tooling 0.1.14 at
`25e7ce83149e081e4dcc52c55c33724e44153f2a`, adding
`scripts/ci/check-make-execution.sh` explicitly. Runner, hook, installer, common
runner fixtures and linked governance are unchanged upstream exports. Newer dirty
sibling rules remain active under the approved local exception but are not
attributed to this recorded revision. The apps/ governance allowance does not
require moving this consumer's existing crates/ packages.

The consumer hook fixture copies and stages the new helper in its disposable
source. It tests ignore-errors, dry-run, question, touch and version-only modes
with real Make, requiring refusal before a formatter marker, index refresh or
working-file change. A failing formatter writes partial bytes only inside its
export; original selected bytes and index remain unchanged. A parallel parent
Make invocation forwards its selected variable through the actual hook. Existing
both-workspace formatting, partial-stage, unrelated-edit and installation cases
remain covered without creating commits or tags.

The Rust validation adapter invokes the same shared checker before gate dispatch.
A substituted refusal test records no later process dispatch, preserves the prior
validation receipt and creates no archived successful attempt. Receipt schemas,
identity binding and recovery remain unchanged; the shared helper owns execution
mode admission rather than a new consumer flag parser.

Focused Linux x86-64 results:

- All 28 repo-tool tests pass locked/offline on Rust 1.99.0 and MSRV 1.88.0.
  Strict repo-tool/test Clippy passes on 1.99.0.
- Canonical runner tests use actual GNU Make for all three increments under the
  five rejected modes, with Git/release effects substituted. Rejection precedes
  state creation/preparation. Existing failure/recovery tests also pass.
- Consumer Make adapters and disposable hook fixtures pass on Bash 5 and genuine
  GNU Bash 3.2.57. The Bash 3.2 checks select that executable for nested shell
  invocations, including the parent Make hook case.
- Snapshot verification, pin declarations/inheritance and pin/metadata fixtures,
  both-workspace formatting, workflow lint and ShellCheck pass. The first lint
  attempt required a fixture-only annotation for an intentionally literal Make
  variable; its log is retained alongside the passing final check.
- Both selected lockfiles, host/IC pins and Rust toolchain bytes match the initial
  inputs. No manifest/version, package-layout, dependency or product-format
  change is made. Current changed-document navigation passes 87 local references;
  published changelog history is preserved.

Inputs, focused logs and released-source CI JSON are retained in
`target/qualification/make-execution-20/`. The pending changelog selects 0.30.1
for this compatible correction. Matching native Linux/macOS CI after the
maintainer's commit and push remains required before closing #20. The successful
0.30.0 CI run does not qualify this later working tree. No broad local gate,
full package/release qualification, original-index hook, staging, commit, tag,
push, publication, deployment or maintainer release command is performed. Sibling
repository files remain read-only.

## Archive retention adoption for #22

Prepared against released 0.30.0 source
`2fdeec4582bbcffcaa09aac625f48a47b29e3194`, preserving the #20 working-tree
adoption above, for [#22](https://github.com/dragginzgame/ic-memory/issues/22).
The consumer already selects registry `ic-host-fs` and `ic-host-artifacts` 0.3.1
with unused default features disabled. Their `write_with` and `copy_reader`
source matches the reviewed clean host 0.3.2 commit
`c7c0d85765054909c05d86f6d3fd2c9965510335` byte-for-byte; no dependency update
is required or performed.

`Repository::record_package` delegates unique staging allocation, synchronization
and atomic archive publication to `ic-host-fs::durable::write_with`.
`ic-host-artifacts::copy_reader` copies and identifies the accepted stream with
constant memory. The original source digest is compared inside the producer
callback before publication, and the retained file is independently hashed
before the consumer can write a receipt. Existing retained paths are verified
without replacement. The private `Execute::write_with` boundary forwards to the
shared owner and permits deterministic command-effect substitution in tests.
No Rust function, method or type is deleted; the superseded inline parent/
staging/copy/rename sequence is removed rather than retained as a second path.

Focused Linux x86-64 evidence:

- All 30 repo-tool tests pass locked/offline on Rust 1.99.0 and MSRV 1.88.0,
  including the current #20 Make admission and existing release recovery tests.
- New consumer tests inject publication errors before and after complete archive
  visibility. Saved validation bytes and unrelated temporary-file evidence are
  unchanged; no prepared receipt is written on refusal. Retry transfers only
  when the retained file is absent, verifies visible matching bytes without a
  second transfer, and refuses existing corrupt bytes without repair.
- A producer-boundary source mutation changes the real fixture archive after
  initial digest admission. The actual consumer callback refuses before archive
  publication or receipt creation. The changed source remains available for
  inspection, original metadata is restored, and a fresh package attempt passes.
- Actual `Processes` publication uses the shared stream callback in the native
  file test; an open descriptor continues to observe the former complete bytes.
  Substituted after-publication errors prove consumer reconciliation, not a new
  native filesystem sync-failure reproduction. Shared-owner publication contracts
  retain their separate native qualification.
- Strict repo-tool/test Clippy, both-workspace formatting, workflow lint,
  ShellCheck, snapshot integrity and actual Make-adapter fixtures pass.
  Both lockfiles, host/IC pins and the Rust toolchain are byte-identical to the
  recorded starting inputs. The initial unsupported `cargo fmt --example`
  invocation made no edits; formatting used the supported `cargo fmt --all`.
- Earlier #20 canonical-runner and Bash 3.2 hook/adapter evidence remains scoped
  to its unchanged shared exports and shell fixtures; it is not another execution
  attributed to this Rust retention change.

Logs and input hashes are retained in `target/qualification/archive-retention-22/`;
the separate review prototype remains in `target/qualification/host-reuse-review/`.
The root and minor-line notes extend the same compatible pending 0.30.1 entry
for #20 and #22, preserving published history. Manifests remain at released
0.30.0. The expanded cloc/common-command setup in #21 remains a separate follow-up,
awaiting a reviewed committed upstream source and compatible release planning.

Matching native Linux/macOS CI after the maintainer's commit and push remains
required for #20 and #22. Released 0.30.0 CI is green but does not qualify this
later working tree. No full local gate, live release/package qualification,
original-index hook, staging, commit, tag, push, publication, deployment or
sibling file edit was performed.

## Common tool command adoption for #21

Prepared on 2026-10-07 against released consumer HEAD
`2fdeec4582bbcffcaa09aac625f48a47b29e3194`, carrying the earlier uncommitted
#20 and #22 repairs. These earlier sections retain their original candidate and
dependency identities; this batch supersedes their pending-version selection.
The numbered draft is now **0.31.0**, because existing developers must refresh
their host installation to add required cloc. Package manifests remain at
released 0.30.0; neither selected lockfile changed during this adoption.

The 64-file snapshot is a canonical export of reviewed Shared Tooling 0.1.15,
`bfb50bd0884b5e6c5ee9592056531c6108f96d73`, from a clean private checkout.
[Exact-source upstream CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37593142226)
passed Linux, macOS Intel/ARM and lint/security. Dirty sibling reporting,
finalizer and sccache fixes are not attributed to that revision or exported.
This upstream evidence does not qualify the consumer working tree.

The unchanged `make/tools.mk` replaces copied installation/check recipes,
preserves the help default and selects checkout-local host/IC paths. Both host
setup and offline checks require ripgrep and cloc. Actual consumer release and
hook fixtures export the include, including the adapter's retained bootstrap.
CI keeps its existing common setup/check targets and runs the adopted command
and report fixtures through `test-tools` on each declared native host.

Fresh local Linux evidence:

- The previous jq/yq/ripgrep set fails offline admission without modification.
  Explicit `make install-host-tools` authenticates the complete pinned set,
  including cloc 2.10; subsequent `make host-tools-check` passes. The prior set
  and failed fixture candidates remain retained. Existing pins are unchanged.
- `make test-tools` passes common Make ordering/path/default/failure dispatch,
  host/IC installer refusal and old-set preservation, checksum helpers, root
  Rust reporting, sibling Rust reporting and tooling inventory fixtures.
  Installer fixtures substitute downloads; actual host setup is recorded
  separately. No live IC installation was requested or performed.
- Snapshot integrity, workflow/ShellCheck lint and consumer release-adapter
  substitutes pass. Pin declarations/metadata, both-workspace formatting,
  disposable hook isolation, common release-runner substitutes and all 30
  Rust repo-tool tests pass through the actual consumer Make targets.
- Common tool/report fixtures, actual release dispatch and hook integration
  also pass under genuine GNU Bash 3.2.57 selected for nested Bash invocations.
  This is Linux shell execution, not native macOS qualification.
- Actual `make cloc` reports only `ic-memory`, using locked offline root Cargo
  metadata. Actual `make cloc-tooling` reports the current sibling tooling
  inventory. Counts are snapshots of their declared scopes, not deletion or
  performance claims. Both selected lockfile hashes remain unchanged.

Logs, original snapshot/notes and input hashes are retained in
`target/qualification/common-tools-21/`. The current root lock selects host
filesystem/artifact 0.3.2; separate 30-test runs on Rust 1.99.0 and 1.88.0 and
strict Clippy for that selection remain in
`target/qualification/shared-0.1.15-review/`. Earlier 0.3.1 evidence is not
relabeled. Read-only review of IC Host Tooling 0.3.3 at
`3d18ca9a9ed0ac5935a16c5bac99694d8e9a7d0a` and its
[successful native CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37595113180)
found no additional equivalent consumer flow to replace: bounded process capture
would introduce another dependency and new output/deadline policy for the
small existing command boundary. Its borrowed error accessors have no current
consumer here. No host dependency selection changed during this batch.

The current documentation and pending-note links resolve; published root and
0.30 detail history are byte-identical to HEAD. An initial whole-ledger link
check also found the pre-existing published `examples/composed_host.rs` reference
in the historical root changelog, which predates this setup adoption. Its failed
log is retained; history was not rewritten to repair that unrelated reference.

The reviewed reporting contract excludes the independent runtime-qualification
workspace and Rust product/example trees from the respective root/tooling
reports. Independent selection remains tracked in
[Shared Tooling #41](https://github.com/dragginzgame/shared-tooling/issues/41);
custom snapshot-manifest and SSH-source reporting defects remain tracked in
[#39](https://github.com/dragginzgame/shared-tooling/issues/39). This consumer's
root manifest and HTTPS identity avoid those cases; pending upstream repairs
are not patched into the immutable snapshot.

[#21](https://github.com/dragginzgame/ic-memory/issues/21) remains open until
maintainer commit and matching native consumer CI. No full local gate,
package/release qualification, original-index hook, staging, commit, tag, push,
publication, deployment or sibling file edit was performed.

## Shared Tooling 0.1.16 adoption and LOC fixture isolation

Prepared on 2026-10-07 against released consumer HEAD
`2fdeec4582bbcffcaa09aac625f48a47b29e3194`, carrying the earlier #20/#21/#22
working-tree changes in the same numbered **0.31.0** draft. The preceding
sections retain their original input identities and qualification scopes.
Manifests remain at released 0.30.0; both selected lockfiles, existing host/IC
pins and the Rust toolchain are byte-identical to this batch's starting inputs.

The 66-file snapshot is a canonical export from clean private checkouts of
reviewed Shared Tooling 0.1.16,
`b69507367d45e3db9543359e689e1fcba0467ff4`. It includes the newly linked
canister audit addendum and `scripts/distribution/refresh-consumer.sh`, a required
dependency of the updated tooling inventory fixture. Every declared payload and
mode matches the committed source. Dirty 0.1.17 fixture repairs are excluded;
the sibling remains read-only. Product limits, Rust release receipts and the
consumer gate roster retain their local ownership.

The consumer Make boundary clears `CARGO_TARGET_DIR` only when invoking the
independent root and sibling LOC fixtures. Their own explicit custom-target
cases remain active; actual builds and reports retain caller selections.
The shared fixture files are unchanged. This addresses the consumer trigger in
[Shared Tooling #47](https://github.com/dragginzgame/shared-tooling/issues/47).

Fresh focused Linux evidence:

- Actual `make test-tools CARGO_TARGET_DIR="<checkout>/target/qualification/shared-0.1.16-adoption/caller target [selected]"`
  passes common Make dispatch, host/IC installer fixtures, checksums and both LOC
  fixtures, including explicit independent-workspace selection. The final tooling
  inventory fixture stops because current committed consumer HEAD does not yet
  contain `scripts/ci/check-make-execution.sh`; that file is still an unstaged
  part of the prepared #20 adoption. The failed suite log and fixture are retained.
  No commit or real-index mutation was used to manufacture this prerequisite.
- The canonical inventory fixture passes separately from the clean committed
  upstream checkout, including real exporter/verifier-backed custom manifest,
  nested-root and HTTPS/SSH cases. This qualifies the shared behavior, not the
  consumer's committed-history prerequisite. Adding the canonical exporter fixes
  the separately identified dependency omission in the consumer snapshot.
- Snapshot integrity, offline host admission, workflow/ShellCheck lint, actual
  consumer release dispatch, pins/metadata, both-workspace formatting, disposable
  hook isolation and canonical release-runner substitutes pass.
- Root/sibling LOC fixtures, consumer release dispatch/hooks and the canonical
  upstream inventory fixture also pass under genuine Bash 3.2.57. This is Linux
  shell-portability evidence, not native macOS qualification.
- Actual `make cloc` selects only the library root. Actual
  `make cloc CLOC_MANIFEST=testing/runtime-qualification/Cargo.toml`
  with a caller-selected target reports only `ic-memory-runtime-qualification`:
  86 runtime LOC. Graphs remain separate, selected target configuration is kept,
  and neither lockfile changes. Actual `make cloc-tooling` also completes.

Logs, prior snapshot/notes and input hashes remain in
`target/qualification/shared-0.1.16-adoption/`. Initial reproductions remain in
`target/qualification/shared-0.1.16-review/`. Existing Rust example/test source
and dependency selection are unchanged by this shell/reporting batch; prior
30-test Rust 1.99/1.88 and strict Clippy evidence remains scoped to its recorded
0.3.2 host selection, without being relabeled as a fresh run.

The 0.1.16 root LOC fixture still selects the enclosing Git root when scratch
is placed inside a checkout. This separate cause is tracked in
[Shared Tooling #48](https://github.com/dragginzgame/shared-tooling/issues/48).
The configured consumer CI uses runner scratch outside the checkout. Do not
claim inside-checkout scratch qualification or copy pending upstream manifest
repairs into the immutable export.

Remaining qualification: after the maintainer commits the prepared files,
rerun `make test-tools` against that committed history and obtain matching
native Linux/macOS consumer CI. The shared
[0.1.16 CI run](https://github.com/dragginzgame/shared-tooling/actions/runs/37598153506)
has Linux/lint success but queued macOS jobs at inspection; it is not complete
native evidence. [Consumer #21](https://github.com/dragginzgame/ic-memory/issues/21)
and upstream #47 remain open for their respective remaining work. No full local
gate, package/release qualification, original-index hook, staging, commit, tag,
push, publication, deployment, CI dispatch or sibling file edit was performed.

## Shared Tooling 0.1.17 canonical LOC fixture repair

During the authorized 0.1.16 adoption, the maintainer committed the directly
relevant canonical root-fixture correction as 0.1.17,
`88f1d70cdf671aefb9507d7a81411ed5daa358b3`. The final 66-file snapshot now
records a clean canonical export of that commit. The previous section and its
logs retain the actual intermediate 0.1.16 evidence rather than being relabeled.
The pending candidate remains **0.31.0**; manifests stay at released 0.30.0.

The shared root LOC fixture now clears its own inherited target selection and
uses explicit manifests. The temporary consumer root-fixture guard was removed
in the same change; `make test-tools` keeps `env -u CARGO_TARGET_DIR` only for
the independent sibling fixture, whose upstream entrypoint is unchanged.
Actual build/report selections are preserved. Exported files are not patched.

Fresh final-source evidence in
`target/qualification/shared-0.1.17-adoption/`:

- The canonical root fixture passes with both an inherited target containing
  spaces/globs and TMPDIR beneath this actual Cargo/Git checkout, under GNU
  Bash 5.2 and genuine Bash 3.2.57. Generated-output, root-package, relocated
  checkout and explicit independent-workspace assertions stay active.
- The unchanged sibling fixture passes with its inherited target cleared and
  ordinary outside-checkout scratch. A separate inside-checkout probe fails
  because its synthetic zeta package sees the consumer's enclosing Cargo
  workspace, despite having its own Git checkout. The failed log/fixture are
  retained; this distinct setup gap is reported in
  [Shared Tooling #53](https://github.com/dragginzgame/shared-tooling/issues/53).
  Configured consumer CI uses runner scratch outside the checkout.
- Final snapshot integrity, workflow/ShellCheck lint, both-workspace formatting
  and bare-Make help pass. Both selected lockfiles, host/IC pins and toolchain
  remain byte-identical to the recorded starting inputs. Published root and
  0.30 detail notes remain unchanged; current docs and pending links resolve.

Earlier successful 0.1.16 root/independent report commands, pins/metadata,
consumer dispatch/hooks, Bash 3.2 and canonical release/inventory checks retain
their recorded scopes. Their shared implementation bytes did not change in
0.1.17 except for the separately requalified root fixture. Rust adapter source
and selected dependencies are also unchanged by this batch; prior Rust test
evidence is preserved, not presented as a fresh execution.

The consumer tooling-inventory fixture remains blocked on its newly adopted
checker being absent from committed consumer HEAD. This is the acknowledged
upstream fixture gap in
[Shared Tooling #50](https://github.com/dragginzgame/shared-tooling/issues/50),
not a passing consumer gate. The canonical exporter dependency is now included,
and the same inventory fixture passes from committed upstream source. An
eventual reviewed fixture correction or maintainer commit can satisfy the
remaining consumer prerequisite; agents do neither a real commit nor index
mutation to manufacture it. Rerun `make test-tools` when that prerequisite is
satisfied, then bind native CI to the maintainer's actual committed source.

[Exact-source 0.1.17 CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37601115116)
has Linux/lint success with both macOS jobs queued at inspection. The older
0.1.16 run was cancelled after the new push; neither is claimed as complete
supported-host qualification. Consumer released-0.30.0 CI does not qualify this
later working tree. [#21](https://github.com/dragginzgame/ic-memory/issues/21)
and the relevant upstream issues remain open for their own remaining work.
No full gate, package/release qualification, real-index hook, staging, commit,
tag, push, publication, deployment, CI dispatch or sibling file edit occurred.

## Shared Tooling 0.1.18 fixture and reporting adoption

After the maintainer released 0.31.0, the checkout and annotated `v0.31.0`
tag identify `faa64e6500d0226411b5181b4023254d1f985ae6`. The maintainer's
continuation authorized the previously recommended committed fixture cleanup.
A clean private checkout of Shared Tooling 0.1.18,
`a3430b34b32a60f3b245a2b4f7e2f5321556fe56`, supplied all 68 canonical exports.
The source checkout is recorded in
`target/qualification/shared-0.1.18-adoption/source.path`; no sibling source
or dirty upstream bytes were changed or attributed to this revision.

Both LOC fixtures now own their Cargo manifests, standalone workspaces and
output configuration. The consumer's `env -u CARGO_TARGET_DIR` dispatch was
removed. The new context fixture checks inherited output selection under an
enclosing Git/Cargo workspace and ancestor Cargo configuration. Reporter
fixtures cover physical output reached through direct and ancestor symlink
aliases. Real builds and reports retain caller-selected target directories.

The inventory fixture runs from current consumer exports without requiring a
commit or the distribution exporter. The unused consumer
`scripts/distribution/refresh-consumer.sh` was deleted, including `usage`,
`fail`, `cleanup`, `validate_relative_path` and `check_consumer_parent`.
Canonical exporting and real exporter/verifier integration remain upstream-owned;
no replacement consumer exporter or test commit was introduced. These are
consumer deletions of an unused exported helper, not deletion of its upstream
owner. Follow-up belongs to [#23](https://github.com/dragginzgame/ic-memory/issues/23),
[Shared Tooling #47](https://github.com/dragginzgame/shared-tooling/issues/47),
[#50](https://github.com/dragginzgame/shared-tooling/issues/50) and
[#53](https://github.com/dragginzgame/shared-tooling/issues/53).

The updated include exposes explicit optional Rust-tool setup/check commands.
Their installer and substituted-Cargo fixture are declared companions; the
required host/IC aggregate and current separate exact cargo-sort formatter setup
remain. The two additive pins select cargo-sort-derives 0.13.0 and
candid-extractor 0.1.6; all previously selected pin lines are byte-identical.
No actual optional-tool installation, compilation or download was performed.
Toolchain preparation remains explicit, and offline checks never install tools.

Fresh Linux x86-64 evidence under
`target/qualification/shared-0.1.18-adoption/`:

- `make test-tools` passes in the uncommitted consumer tree with inherited
  `CARGO_TARGET_DIR` containing spaces and glob characters. The previously
  blocked tooling-inventory fixture now passes without index or history changes.
  Context tests retain expected counts, locks and deliberate custom targets.
- The same complete focused tool-fixture target passes with genuine GNU Bash
  3.2.57 selected for nested shell calls. Its version and full output are retained
  in `bash32-version.log` and `bash32-tools.log`. This is Linux portable-shell
  execution, not native macOS qualification.
- Snapshot integrity, offline host admission, dependency/inheritance and metadata
  fixtures, both-workspace formatting, Actionlint, ShellCheck, consumer Make
  release dispatch, canonical release-runner substitutes and disposable hook
  isolation pass (`focused-checks.log`). No fixture creates commits or tags.
- Actual root and explicitly selected independent-workspace Rust reports pass;
  the latter reports only `ic-memory-runtime-qualification`. Bare Make still
  prints help. Report logs and unchanged existing pin lines are retained.
- Input hashes prove both workspace manifests/lockfiles, Rust toolchain, IC pins
  and Rust adapter/test sources remain unchanged. No Rust tests were rerun in
  this shell/reporting batch, and earlier Rust evidence retains its own inputs.

Pending notes select compatible 0.31.1 from released 0.31.0. Product manifests
remain 0.31.0; public APIs, durable formats, package qualification receipts and
published changelog history are preserved. Current links and exact history/input
checks are recorded alongside the focused evidence.

Current pending/document navigation passes 67 local references across ten
documents. A full root-ledger link check still finds the previously recorded
historical `examples/composed_host.rs` link; its failure is retained in
`published-doc-links.failed.log`, and finalized notes remain unchanged. The first
history comparison mistakenly included the root title on only one side; the
corrected comparison selects the finalized 0.31.0-and-older sections on both
sides and passes byte-for-byte for root and detailed notes.

At inspection, [consumer 0.31.0 CI](https://github.com/dragginzgame/ic-memory/actions/runs/37604857958)
had successful lint, Linux full gate and Linux MSRV; macOS 15 ARM/Intel jobs
were queued. [Shared Tooling 0.1.18 CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37604299590)
had successful Linux, lint and macOS ARM checks with Intel still running.
Neither pending remote status is a full supported-host pass, and released-source
CI does not qualify these later uncommitted consumer edits. Keep #20–#22 open
for matching released-source native results, and #23 for maintainer commit and
matching consumer CI after this adoption. No full local gate, package/release
qualification, real-index hook, staging, commit, tag, push, publication, CI
dispatch or sibling source edit occurred.

## IC Host 0.4.1 dependency qualification

The pending compatible 0.31.2 batch starts from released 0.31.1,
`40773d7858dfe5cd4df5075f10b81a5af500f5a4`, with the maintainer-selected root
lockfile update from `ic-host-artifacts`/`ic-host-fs` 0.4.0 to 0.4.1. Registry
source records identify Host release commit
`ce2dd57cedc5000b44bb6a9ff5194f7d65a42c38`; both downloaded source trees match
the reviewed clean upstream checkout. Dependency requirements, effective features,
Rust source and toolchains are unchanged.

Focused Linux x86-64 evidence is retained under
`target/qualification/host-0.4.1-review/`. The first offline adapter test stopped
because the selected registry cache lacked the new archives; its failed log is
preserved. Separate `make fetch-dependencies` populated that cache with the
selected lockfile, then offline Rust 1.99.0 adapter tests passed all 30 cases.
Strict example Clippy and Rust 1.88.0 example compilation also passed. Adapter
tests exercise real durable writes while substituting Git, Cargo and publication
effects; they do not prove a live release or upload.

Preparation evidence under `target/qualification/0.31.2-preparation/` records
successful full locked offline metadata for both maintained workspaces, the
68-file snapshot check, declaration/inheritance pins and both-workspace formatting.
The independent PocketIC workspace has no dependency on these host crates and
requires no lockfile update. Input hashes bind both manifests/lockfiles, adapter
source/tests and the toolchain declaration. Published root/detail note sections
remain unchanged; both pending headings select 0.31.2.

At inspection, [Host 0.4.1 CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37614534197)
passed Linux and MSRV with both macOS jobs queued. The
[released consumer 0.31.1 run](https://github.com/dragginzgame/ic-memory/actions/runs/37611434339)
passed Linux, lint, macOS ARM validation and both macOS MSRV checks; Intel
validation remained queued. Neither run qualifies the changed consumer lockfile.
No full local gate, package/release qualification, staging, commit, tag, push,
publication or sibling source edit was performed for this batch.

Subsequent inspection on 2026-10-07 confirms that the released consumer run
above completed successfully at `40773d7858dfe5cd4df5075f10b81a5af500f5a4`:
Linux and macOS 15 ARM/Intel validation, all three MSRV jobs and tooling lint
passed. This qualifies the committed 0.31.1 tooling adoption and carried
Make/archive changes; it does not qualify the later 0.4.1 lockfile selection.
The earlier cancelled 0.31.0 run retains its own incomplete result.

Host 0.4.1 CI instead completed with both macOS validation jobs failing in
`scripts/ci/test-tool-commands.sh` before Rust tests. The first expected/actual
command comparison differs because a trailing slash in `TMPDIR` produces a
double separator while Make supplies normalized `CURDIR`. On this Linux host,
the unchanged consumer 0.1.18 fixture reproduces exit 2 with a trailing-slash
`TMPDIR`; its substitute PATH assertion stops before the comparison. Retained
logs and the failed fixture are under
`target/qualification/0.31.2-preparation/tmpdir-exposure/`; upstream native failed
logs are `target/qualification/host-0.4.1-review/upstream-macos-failed.log`.
This establishes fixture exposure, not a Rust library defect or native pass.

The canonical correction is owned by
[Shared Tooling #56](https://github.com/dragginzgame/shared-tooling/issues/56).
Its physical fixture-root normalization and context regressions are now prepared
in the uncommitted 0.1.19 source based on
`a3430b34b32a60f3b245a2b4f7e2f5321556fe56`. Consumer adoption remains with
[ic-memory #23](https://github.com/dragginzgame/ic-memory/issues/23), alongside
the Rust installer path-boundary correction in
[Shared Tooling #54](https://github.com/dragginzgame/shared-tooling/issues/54).
The current 68-file snapshot stays unchanged until a reviewed committed revision
is available. Neither prepared upstream correction is claimed in 0.31.2 notes.

## IC Host 0.4.2 copy-error convergence

The requested review of new IC Host crates selects clean released 0.4.2,
`6501d0e9fa7ba0439ec7a4010ca7bf0205e1d712`. Remote main and the peeled release
tag match that identity. Both cached registry source trees record that revision
and match the upstream Rust source. Narrow offline updates changed only the two
Host packages from the previously prepared 0.4.1 selection to 0.4.2. The root
artifact requirement is now the compatible range `0.4.2`, because the conversion
first exists there; default features, filesystem requirement, other selections,
product version and toolchain remain unchanged. Both maintained workspace graphs
pass full locked offline metadata; the independent PocketIC graph is unaffected.

`Repository::record_package` delegates native copy-error projection to
`From<CopyError> for io::Error` through `?`, removing the consumer's blanket
`io::Error::other` wrapping. Native reader/writer failures retain their kind,
OS code and cause; the shared owner retains non-I/O copy failures as typed
causes. The maintained consumer boundary needs an I/O error for durable writing,
rather than a separate input/output classification. This adopts the addition in
[Host #14](https://github.com/dragginzgame/ic-host-tooling/issues/14).

Focused Linux evidence under `target/qualification/host-0.4.2-convergence/`:
all 31 adapter tests pass on Rust 1.99.0 and MSRV 1.88.0; strict example/test
Clippy, both-workspace formatting, declaration pins and 68-file snapshot
verification pass. The new integration test replaces an admitted source with a
directory before copying, compares the propagated failure with that host's
native read error, and verifies absent publication, owned staging cleanup and
preserved unrelated evidence. Existing source-change, corruption and publication
recovery assertions remain. Processes use substitutions for Git/Cargo/release
effects while file copying and durable staging are real.

The bounded capture/admitted-tool engines do not replace `Processes::run`'s
inherited environment and live stdio without adding new output/deadline policy.
This consumer has no installed-tool hash/admit composition or ordered upload
chunk hashing to remove. Receipt/source/lock bindings and independent retained
archive hashing protect distinct release recovery boundaries and remain local.
No duplicate copy, hashing or publication engine remains in the reviewed adapter.
This review covers maintained Host callers and their qualification graph, not a
whole-library or deployed-runtime audit.

[Host 0.4.2 CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37624014360)
passes Linux and MSRV, but macOS ARM/Intel again stop before Rust tests at the
tool-command fixture comparison tracked in
[Shared Tooling #56](https://github.com/dragginzgame/shared-tooling/issues/56).
Both native job logs are retained beside the focused evidence. The first CLI
failed-log request returned an empty file; direct job-log reads supplied the
actual failures. No dirty upstream correction was adopted. New consumer source
still needs committed native CI; released 0.31.1 success does not qualify it.
No full local gate, package qualification, sibling edit, staging, commit, tag,
push, publication or CI dispatch occurred. Pending notes remain compatible
0.31.2 and published history is unchanged.

## IC Host 0.4.3 selected dependency review

The maintainer-selected root lockfile now contains `ic-host-artifacts` and
`ic-host-fs` 0.4.3. This check preserves that selection and the previous adapter
changes. Released Host source is `644d49c096ae05c2e17e1b6aacf14770988c5cf6`,
confirmed by remote main, the peeled tag and both registry VCS records. Both
downloaded Rust source trees match an export of that exact commit. During review,
the sibling acquired new uncommitted gzip helpers; comparison with the moving
checkout then differed. Those dirty additions are excluded from the selected
release and evidence, and gzip features remain disabled here.

The new typed streaming and descriptor-relative writers share the engine used
by this consumer's existing `write_with` and `write_bytes`. This adapter has no
directory confinement or separate typed-serialization publication pipeline to
replace. Adopting explicit modes/permissions here would add choices without
removing another engine. Archive digest admission, independent retained hashing,
receipt encoding and visible-byte reconciliation retain their existing owners.
No further Rust source change or symbol removal was justified by this review.

Evidence under `target/qualification/host-0.4.3-review/` records the initially
missing standard-cache archives and failed offline test attempt. Separate locked
`make fetch-dependencies` waited for the shared package-cache lock, then downloaded
only the two selected Host archives. Fresh offline adapter tests pass all 31
cases on Rust 1.99.0 and 1.88.0. Strict example/test Clippy, declaration pins,
both-workspace formatting, 68-file snapshot integrity and both workspaces' full
locked offline metadata pass. Input hashes confirm that neither maintained
manifest/lockfile, adapter/test source nor toolchain changed during these checks.
Pending 0.31.2 notes now identify the actual 0.4.3 selection; published history
and the consumer snapshot remain unchanged.

At inspection, [Host 0.4.3 CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37639415895)
has successful Linux and MSRV jobs with macOS ARM/Intel queued. Host's snapshot
now contains the committed Shared Tooling 0.1.19 TMPDIR and Rust-installer fixes
at `a06e4719e3839b8eefcfb88ec8923aa88eb63ccc`; queued native checks do not yet
establish that the earlier macOS failure is resolved in execution. Separate
ic-memory snapshot adoption remains on
[ic-memory #23](https://github.com/dragginzgame/ic-memory/issues/23).
Local Linux passes do not establish native consumer qualification. No full local
gate, package qualification, sibling edit, staging, commit, tag, push, publication
or CI dispatch occurred.

## Shared Tooling 0.1.20 candidate review

Read-only review selects clean committed `3ecc48e579f6cf6e6ab01a6645d8a250fc8c6934`,
whose `VERSION` is 0.1.20 and whose identity matches remote main. No 0.1.19/0.1.20
release tags were observed; committed snapshot availability does not require one.
This clarifies the earlier generic reference to that SHA as a released identity.
The existing consumer snapshot still selects 0.1.18.

An exact Git archive under `target/qualification/shared-0.1.20-review/source/`
passes focused Linux checks: tool commands with trailing-slash physical and
aliased TMPDIR roots, Rust installer path rejection/retry and evidence retention,
IC tool installation/check/activation, and release-runner recovery/finalizer
cases. Installations, downloads and Git release effects are fixture substitutes;
no live installation or release was exercised. Candidate logs and consumer input
hashes are retained beside that export. These passes establish the reviewed
upstream behavior, not adoption in this consumer or native macOS acceptance.

Refreshing the selected IC installer requires adding its new canonical pin
parser, `scripts/ci/ic-tool-pins.awk`. Governance links now require
`rules/contributions.md`. The consumer's explicit maintainer-owned Git effects
remain the approved local exception to the new general PR authority. Frontend
formatter and hosted artifact-retention qualification are upstream CI coverage;
they add no frontend dependency to this Rust consumer. PocketIC and disk checks
remain opt-in, with server custody and capacity thresholds locally owned.

At inspection, [exact-source CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37641211708)
passes Linux portable regression and lint/security; both macOS jobs remain
queued. The relevant snapshot follow-up stays on
[ic-memory #23](https://github.com/dragginzgame/ic-memory/issues/23).
No source repair, snapshot refresh, broad gate, real-index hook, staging, commit,
tag, push, publication or sibling edit occurred during this review.

## IC Host 0.4.5 selected dependency review

Review preserves the maintainer-selected `ic-host-artifacts` and `ic-host-fs`
0.4.5 lock entries and the existing adapter changes on consumer HEAD
`40773d7858dfe5cd4df5075f10b81a5af500f5a4`. Host release source
`93a905b048bcaa2a0aed4214ac2f13f065dc2905` matches remote main, the peeled
`v0.4.5` tag and both registry VCS records. Each downloaded Rust source tree
matches an exact Git archive. Earlier 0.4.2/0.4.3 evidence remains unchanged.

The flow review follows
[the shared method](https://github.com/dragginzgame/shared-tooling/blob/3ecc48e579f6cf6e6ab01a6645d8a250fc8c6934/audits/flow-convergence-and-duplication.md)
at Shared Tooling `3ecc48e579f6cf6e6ab01a6645d8a250fc8c6934`, with its live
local baseline changes and this repository's maintainer-owned Git exception.
Scope is the new Host 0.4.4/0.4.5 behavior and existing release-helper callers;
it adds no consumer states or Rust changes.

| Flow | Shared owner | Consumer obligation retained |
| --- | --- | --- |
| Package copy and digest | `copy_reader` and its native I/O conversion | Admit the source digest and independently verify retained raw archive bytes. |
| Durable file publication | `write_with` / `write_bytes` through one engine | Encode receipts and reconcile visible bytes after publication errors. |
| Gzip payload identity | Optional Host gzip helpers | No caller here: archive receipts identify compressed `.crate` bytes, not decoded payloads. |

Host 0.4.5 converts admitted permissions to Darwin's narrower mode with a checked
conversion. Its native CI now compiles and executes the filesystem tests, resolving
the original E0308 from [Host #18](https://github.com/dragginzgame/ic-host-tooling/issues/18).
The new gzip helpers remain disabled; replacing raw archive hashing with them
would change receipt identity. The existing streamed publication already shares
the owner engine; typed/descriptor writers add no needed convergence here.
No further source simplification or symbol removal is justified in this scope.

Evidence in `target/qualification/host-0.4.5-review/` records successful locked
offline cache preparation, all 31 adapter tests on both Rust 1.99.0 and 1.88.0,
strict example/test Clippy, both workspaces' full locked offline metadata,
declaration pins, formatting and 68-file snapshot integrity. Metadata confirms
no enabled artifact features. Input hashes verify that manifests, lockfiles,
adapter/test source and toolchain stayed unchanged throughout these checks.
Only the pending 0.31.2 notes and this evidence record changed in this review.

[Exact Host CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37645681743)
passes Linux/MSRV but fails both macOS jobs: 57 filesystem tests pass and the
non-UTF-8 filename publication unwrap at `durable/stream/tests.rs:205` receives
native EILSEQ (92), before publication. This filesystem-fixture assumption is
tracked separately in [Host #19](https://github.com/dragginzgame/ic-host-tooling/issues/19);
it does not establish another mode-conversion failure. Job summaries and failed
logs are retained with the focused evidence. Full native owner acceptance remains
outstanding; local Linux passes do not qualify dirty consumer changes on macOS.
The successful released [consumer CI](https://github.com/dragginzgame/ic-memory/actions/runs/37611434339)
still applies only to 0.31.1, not this new lock selection. No broad local gate,
package qualification, snapshot refresh, sibling edit, staging, commit, tag,
push, publication or CI dispatch occurred. Pending 0.31.2 remains compatible.

## Shared Tooling 0.1.20 consumer adoption for #23

The issue-work request authorizes the accepted snapshot refresh on
[ic-memory #23](https://github.com/dragginzgame/ic-memory/issues/23). The canonical
exporter copied 70 files from a clean private checkout of committed
`3ecc48e579f6cf6e6ab01a6645d8a250fc8c6934`, adding the IC pin parser and linked
contribution rule to the declared roster. No moving sibling bytes or in-place
vendored repairs were used. The newer 0.1.21 PR release engine is outside this
adoption; the existing direct runner, Make include and hook bytes are unchanged.
The active live-local policy exception and explicit maintainer-owned Git effects
remain in the local overlay. [#24](https://github.com/dragginzgame/ic-memory/issues/24)
requires a separate change to that user-supplied ownership rule.

This installs the reviewed Rust path-admission and tool-command scratch-root
repairs for [Shared Tooling #54](https://github.com/dragginzgame/shared-tooling/issues/54)
and [#56](https://github.com/dragginzgame/shared-tooling/issues/56). Canonical IC
matrix admission replaces its inline parser. The refreshed changelog fixture
covers preserved history/EOF bytes, while the consumer's existing Rust selector
and its identity/refusal checks remain unchanged. No product API, format, receipt,
tool pin, compiler, manifest or lock selection changes in this adoption.

Fresh Linux evidence under `target/qualification/shared-0.1.20-adoption/` passes
the full focused `make test-tools` target under Bash 5 and genuine Bash 3.2.57,
including enclosing-workspace/target and current-export LOC cases. Aliased
trailing-slash tool-command admission also passes under Bash 3.2. Installer
downloads/Cargo execution and release effects are substitutes. Consumer release
adapters, canonical runner fixtures, disposable hooks, pins/metadata fixtures,
lint, formatting, snapshot and offline host-tool admission pass. All 31 Rust
adapter tests and both full locked offline metadata graphs pass. Documentation
checks cover 142 local references across 31 active documents. Input and real-index
hashes confirm preserved selections and no staging.

An expanded trailing-slash run passes the changed tool-command/Rust/IC fixtures,
then fails the unchanged sibling LOC fixture's diagnostic grep: its unnormalized
`scratch//` path differs from the reporter's physical path. Prepared-PATH tracing
confirms the assertion failure; a separate first trace lacking cloc is retained
as a prerequisite failure. A one-line normalization proposal in a temporary exact
export passes both physical and aliased trailing-slash cases. The consumer export
is intact; [Shared Tooling #57](https://github.com/dragginzgame/shared-tooling/issues/57)
owns the new finding and proposal, with adoption acceptance retained on #23.

[Selected upstream CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37641211708)
now passes Linux, both macOS architectures and lint/security. Separately, released
[Host 0.4.6 CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37648086908)
at `0fb05f9e18f032425188d68e1d69317a0f0127d5` passes Linux, both Macs and MSRV,
resolving [Host #19](https://github.com/dragginzgame/ic-host-tooling/issues/19).
Its fixture correctly distinguishes lexical refusal before producer invocation from
native final-name refusal after staging/producer execution, with no visible final
output and cleanup on failure. The root lock still selects 0.4.5; no newer package
selection is implied by owner acceptance. Matching committed consumer native CI
remains required. Pending notes stay compatible 0.31.2, with published history
preserved. No full local gate, release/package qualification, tool installation,
sibling edit, real-index hook, commit, tag, push or CI dispatch occurred.

## Contribution authority adoption for #24

The maintainer explicitly selected [#23](https://github.com/dragginzgame/ic-memory/issues/23)
and [#24](https://github.com/dragginzgame/ic-memory/issues/24), authorizing removal
of the blanket local agent-commit prohibition. This supersedes the ownership
exception recorded in earlier reviews; those reports retain their original scope.
The 70-file Shared Tooling 0.1.20 snapshot already includes the committed common
contribution rule and its linked baseline, changelog, maintenance, release and
reviewable-change guidance. Its bytes remain unchanged by this documentation batch.

`AGENTS.md` now applies that authority directly. Release/setup instructions remove
conflicting unconditional ownership wording, and the root README links the shared
contribution guide for human contributors. Request interpretation was reviewed
against both the local overlay and the canonical rule:

| Request | Admitted scope |
| --- | --- |
| Ordinary fix or continuation | Local edits and focused checks; no implicit staging/commit/push. |
| Explicit commit | Scoped staging and commit; no implicit push. |
| Explicit PR | Topic branch, scoped commits, branch push and PR creation/update. |
| Merge or integration-branch push | Separate explicit authorization; protections/checks remain required. |
| Standard release | Explicitly selected documented release effects; publication/deployment stay separate. |

PR contributions do not select PR release delivery. Qualification still uses
substituted Git effects and disposable hook indexes; changing the rules does not
authorize a live commit, push, release or qualification-fixture history creation.
No public library, format, source, manifest, lock, compiler or release receipt
change is part of this batch. The pending compatible candidate remains 0.31.2.

Evidence in `target/qualification/issues-23-24/` passes snapshot integrity,
152 local references across 32 active documents, historical note byte comparisons
and whitespace checks. Input hashes preserve both workspace selections, Rust
adapter/tests and the exact snapshot; the real index hash is unchanged. Previous
focused executable evidence for #23 remains bound to its original inputs and was
not rerun for these documentation changes. No function, method or type was removed.
Both issues are prepared locally; committed delivery remains outstanding, and
#23 additionally requires matching native consumer CI. The separate trailing-slash
LOC context remains on [Shared #57](https://github.com/dragginzgame/shared-tooling/issues/57),
with no reviewed owner correction yet available at inspection. No broad gate,
compilation, real-index hook, commit, tag, push, publication, CI dispatch or sibling
edit occurred in this batch.

## Shared Tooling 0.1.21 and local LOC/CI candidates

Reviewed committed Shared Tooling 0.1.21
`45e34e92b43edb9543d5b7212774f87f8334079f` and separately captured its dirty
LOC and CI changes in `target/qualification/shared-0.1.21-review/`. The consumer
retains the exact 70-file 0.1.20 snapshot; no PR release policy or merged-source
receipt contract is selected here. Ordinary contribution authority is separate.

The committed direct runner command-substitute fixture passes with an enclosing
`RELEASE_DELIVERY=pr`; the fixture explicitly selects direct delivery. The new
PR fixture was inspected but not executed locally because it creates real Git
commits, tags and pushes in disposable repositories, which this overlay forbids
for qualification. Upstream execution remains distinct evidence.

[Upstream CI attempt 1](https://github.com/dragginzgame/shared-tooling/actions/runs/37652236506)
passes Linux, Apple Silicon and lint/security. Intel macOS started and was
cancelled during the portable regression step at 16:51:02 UTC; the direct runner
pass is logged, but no complete regression result follows. Native formatter/IC
qualification and final failure collection were skipped. Cancellation proves
incomplete qualification, not a source defect. The raw job log is retained.

For [Shared #57](https://github.com/dragginzgame/shared-tooling/issues/57), an
isolated exact-source copy plus the captured dirty LOC correction passes the
expanded enclosing Git/Cargo, inherited-output, physical trailing-slash and
symlinked TMPDIR contexts under Bash 5 and genuine Bash 3.2.57. ShellCheck passes.
The sibling fixture hash is
`c646cbb4efa4b885121d6a7290093b94dffef2cdf0790d26e1625e5c47549468`;
the context fixture hash is
`da3358ef5a7af2feb075afc5e3361dd35c4f325fc380a05a979a9bc1b02214e1`.
These identify dirty bytes, not released 0.1.21 or an adopted consumer export.

The separately captured CI candidate raises the portable job limit to 25 minutes
and gives its regression step a 15-minute limit, reserving collection time. Its
failure-retention fixture passes Bash 5/3.2, ShellCheck and actionlint; native
timeout/collection behavior still needs matching CI. Captured source hashes,
both workspace selections, consumer Rust inputs, snapshot and real index remain
unchanged. Consumer released-HEAD CI remains green at `40773d7`; dirty work has
no remote result. No broad gate, tool install, compilation, sibling edit, staging,
commit, tag, push, release or CI dispatch occurred.

## Shared Tooling 0.1.22 consumer adoption for #23

The maintainer pushed/tagged ic-memory 0.31.2 at
`df01ac27f47fa46ebcab3a43665c81bb58c22e40`, then requested continuation.
The committed contribution authority resolves
[#24](https://github.com/dragginzgame/ic-memory/issues/24); its earlier reports
retain their original review identities. Continued
[#23](https://github.com/dragginzgame/ic-memory/issues/23) with committed Shared
Tooling 0.1.22 `2687f26317952c43c685f7f799ed09288dc10a67`.

A clean private clone at that exact revision supplied the canonical exporter.
The reviewed manifest adds the canonical PR helper, matching the updated shared
governance roster, and refreshes all 71 declared files together. The exporter
verifies hashes/modes and preserves the real index. No sibling file or vendored
file was patched. The LOC fixture hashes match the previously captured #57
candidate; the existing consumer context fixture receives the expanded admission.

The new runner supports an explicitly selected PR policy, but this consumer's
receipt adapters remain direct. Make exports the direct default and refuses
other selections for release targets before dispatch. A disposable copy of the
actual Makefile tests environment and command-line refusal for patch/minor/major
and resume with a substitute runner; no runner effect occurs on refusal, and
ordinary direct dispatch succeeds. Existing adapter forwarding and unique failed
gate retention remain covered. The canonical direct fixture owns its delivery
selection. No PR merged-source receipt contract is claimed or selected.

Fresh Linux evidence in `target/qualification/shared-0.1.22-adoption/` passes:

- Complete focused `make test-tools` with physical trailing-slash TMPDIR under
  Bash 5, then aliased trailing-slash TMPDIR under genuine Bash 3.2.57. This
  repeats the earlier failing consumer reproduction against actual exports.
- Consumer release adapters and canonical direct runner command substitutes;
  the consumer adapter fixture also passes Bash 3.2 with enclosing PR selection.
- Snapshot integrity, dependency pins, both-workspace formatting, shell/workflow
  lint, 168 local references across 34 active documents, pending-note links,
  whitespace and published-note preservation. The whole historical root ledger
  still fails on its previously recorded `examples/composed_host.rs` link;
  `documentation-links-history.log` retains that failure without rewriting history.
- Input hashes for both workspace selections, Rust adapter/tests, toolchains,
  pins, shared Make include and hook, plus the unchanged real index.

Installer/Cargo and release dispatch in fixtures are substitutes. Rust source,
the hook and selected dependency graphs are unchanged; prior compiled evidence
is not relabelled, and no new compilation or full local gate ran. The PR fixture
is not adopted or run because its real disposable Git effects conflict with the
local qualification overlay. Package versions remain 0.31.2; compatible pending
notes select 0.31.3 and preserve every published root/detail entry.

At inspection, [0.31.2 consumer CI](https://github.com/dragginzgame/ic-memory/actions/runs/37661144098)
passes Linux full-gate/MSRV, tooling lint and Apple Silicon MSRV, with remaining
macOS checks queued. [Shared 0.1.22 CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37659875012)
passes Linux, Apple Silicon and lint/security; Intel is running. Neither is
claimed fully green. This dirty consumer adoption still needs committed delivery
and matching native CI. No tool installation, build, full gate, real-index hook,
staging, commit, tag, push, release, publication, CI dispatch or sibling edit
occurred. No function, method or type was removed.

## 0.31.3 issue and shared PR-query follow-up

The maintainer requested continued 0.31.3 issue/tooling work on consumer HEAD
`df01ac27f47fa46ebcab3a43665c81bb58c22e40` with the pending canonical 71-file
0.1.22 adoption. The snapshot, Make delivery selection/refusal and adapter fixture
hashes still match the preceding qualification; its passing tests are reused
only for those unchanged inputs, not presented as fresh executions. Pending
notes remain compatible 0.31.3 and package versions remain 0.31.2.

[Shared 0.1.22 CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37659875012)
at `2687f26317952c43c685f7f799ed09288dc10a67` is now successful across Linux,
Apple Silicon, Intel and lint/security. The retained Intel log explicitly records
the expanded LOC contexts, direct/PR runner fixtures and native artifact checks.
This supersedes the earlier incomplete observation and establishes upstream
acceptance of [#57](https://github.com/dragginzgame/shared-tooling/issues/57),
which is closed. [Consumer #23](https://github.com/dragginzgame/ic-memory/issues/23)
still owns committed delivery and matching CI of its actual adoption.

Separately reviewed the dirty 0.1.23 PR-query correction on
[Shared #42](https://github.com/dragginzgame/shared-tooling/issues/42), using the
code-hygiene boundary questions from the unchanged shared audit method. The
canonical `pr_query` owns paginated observation and identity admission;
`pr_publish`/`pr_review` consume its admitted identity before release effects.
Real gh 2.45.0 rejects the committed `--slurp` invocation with status 1. A real
GET-only query using `--paginate` and jq page aggregation succeeds. No remote PR
or release effect was attempted.

Captured dirty helper/fixture bytes in `target/qualification/0.31.3-followup/`
pass a narrow `pr_query` API-substitute harness under Bash 5 and genuine Bash
3.2.57. Both interpreters accept empty array pages and a matching later-page PR;
they reject missing/malformed/non-array pages, partial-query failure, duplicate
PRs across pages, disappearance, changed saved identity and invalid details.
Failures retain raw replies without publishing the identity sidecar. ShellCheck
passes the captured upstream helper/fixture. The known CLI defect is a bounded
upstream release blocker; the candidate has component proof, while full recovery
and native qualification remain with its owner. These results do not qualify
uncommitted bytes as 0.1.22 or a native/live PR release.

The full real-Git PR fixture remains unexecuted under this overlay, and direct
consumer delivery does not need that correction. Shared files remain read-only;
the actual snapshot is unchanged. Captured owner hashes, both workspace
selections and the real index remain stable. No Rust symbols were removed, and
no compilation, broad gate, installation, staging, commit, tag, push, release,
publication or CI dispatch occurred. Earlier failed and native-incomplete
observations remain preserved at their original evidence identities.

Final post-batch [consumer CI inspection](https://github.com/dragginzgame/ic-memory/actions/runs/37661144098)
now reports success for released 0.31.2 `df01ac27f47fa46ebcab3a43665c81bb58c22e40`:
Linux and both macOS full gates, all three MSRV jobs and tooling lint pass. This
supersedes the preceding running/queued observation for that release. It confirms
delivery of the earlier 0.1.20 baseline, not the still-uncommitted 0.31.3 refresh.

## 0.31.3 released snapshot acceptance

The maintainer pushed/tagged 0.31.3 at
`692fbc81d698f4b8253565ff3036f3ad3fb42cd2`, following source commit `c9866b3`.
Remote annotated tag identity and committed 71-file snapshot provenance agree.
[Exact-release CI](https://github.com/dragginzgame/ic-memory/actions/runs/37743519477)
passes Linux and both macOS full gates, all three MSRV jobs and tooling lint.
This completes [#23](https://github.com/dragginzgame/ic-memory/issues/23), which
is closed. Earlier failed/incomplete observations retain their source identities.
The subsequently selected Host dependency changes are separate dirty inputs.

## IC Host 0.5.0 selected dependency review

Reviewed released IC Host 0.5.0
`db637fac8b7a9ef62301e1d9009ffeb5ffcd0be7` against 0.4.6 and the maintainer's
existing dirty root selection. The annotated tag names that exact source.
Registry artifact/filesystem packages record the same VCS commit and have
byte-identical source trees and original manifests. Their archive SHA-256 values
match the root lock: artifacts
`872fa46c90cf915d7414be4ac7bbe0955b0934ffa1e8ac883fad122637dbc981`, filesystem
`96578ba90b978f0bafc48454f87f2869f74f3b6d6b3d06790cff47f9c95c7f98`.

Both requirements are compatible 0.5 ranges with defaults disabled and remain
non-Wasm development dependencies. The actual graph selects only these two
Host crates, each with no features. Existing `copy_reader`, `hash_file`,
`durable::write_bytes` and `durable::write_with` contracts remain applicable.
The common publication engine adds optional closed-writer admission while
preserving existing no-admission staging, identity, sync and cleanup behavior.
The consumer still hashes raw retained archives independently and preserves
native copy errors; executable admission does not apply to its archive files.

Reviewed the other new library surfaces without adopting them: owned-child
cleanup retains explicit caller IO/lifecycle obligations; IC reports compare
caller-selected, revision-bound resources rather than proving installability;
gzip/Wasm argument, field and error changes are breaking upstream contracts
outside this feature selection. The consumer has no matching process-group,
executable-installer or IC-limit policy to migrate. No duplicate mechanism was
introduced, and product Wasm budgets remain local.

[Exact Host release CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37744999108)
passes Linux, both Macs and MSRV. Both retained native Mac logs execute actual
closed-writer probes, rejection/identity/race/phase tests, all six child cleanup
cases and resource-report tests. Host #14/#20/#10 are resolved; #5 retains its
remaining consumer integration contracts. Draft PR #21 remains separate from
the subsequently tagged/published implementation and was not merged or closed.

Fresh Linux evidence in `target/qualification/host-0.5.0-review/` passes locked
offline cache admission, full metadata for both maintained graphs, all 31
repo-tool tests on Rust 1.99 and MSRV 1.88, strict selected Clippy, the focused
compile-fail trust/capability target, snapshot, pins and both-workspace formatting.
The current root lock also contains the maintainer-selected `serde_spanned`,
`toml`, `toml_datetime`, `toml_parser`, `toml_writer` and `trybuild` updates;
they are preserved and included in this actual qualification, not attributed to
Host or silently reverted. The independent manifest/lock, Rust source/tests,
compiler/tool pins, snapshot and real index hashes remain unchanged.

Pending notes select compatible 0.31.4 while package versions remain 0.31.3.
No public library, format or release-receipt change is part of this batch.
Current dirty dependency inputs still require committed delivery and matching
consumer native CI; released 0.31.3 and upstream green runs do not qualify them.
No broad gate, package/release qualification, installation, network fetch,
staging, commit, tag, push, publication, CI dispatch or sibling edit occurred.
No function, method or type was removed. Published note bytes remain preserved;
this review removes no prior evidence. Active/detail and pending links pass.
The earlier export's source-path record was unavailable, so the documentation
checker was recovered from exact committed Shared Tooling 0.1.22; the initial
setup failure is retained separately. The known old root example-link failure
remains distinct from active and pending documentation admission.

## Shared Tooling 0.1.23 direct release integrity adoption

Continued compatible 0.31.4 work from released consumer 0.31.3
`692fbc81d698f4b8253565ff3036f3ad3fb42cd2`, preserving the separately selected
Host 0.5.0 and TOML/trybuild dependency changes. Reviewed committed Shared
Tooling 0.1.23 `0ba0ad00ed94848e54ecc82629b6b7873b7284c0` against the selected
0.1.22 snapshot. Its canonical exporter, executed from an isolated clean local
clone at that exact revision, refreshed the existing 71-file roster. The runner,
PR helper, direct fixture and release guidance are the only changed exported
files. No vendored bytes were edited locally and the sibling stayed read-only.

[Shared #58](https://github.com/dragginzgame/shared-tooling/issues/58) owns the
confirmed runner gaps: final-hook mutations escaped its last common check, and
completed direct resume did not observe tags or destination ancestry. The
consumer Rust push hook already checks receipts, payload and tag before returning,
but completed recovery still requires the common destination checks. No incorrect
live IC Memory release was observed. The adopted runner now independently checks
the index, payload and exact annotated tag after the final hook, then verifies
local/remote tag objects and branch ancestry on completed resume. Known
descendants remain valid; conflicts or unavailable observations refuse completion
without replaying effects. Existing receipt ownership and direct-only Make
dispatch remain intact; no duplicate recovery engine or PR adapter was added.

[Matching upstream CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37746567888)
passes Linux, Intel, Apple Silicon and lint/security at the selected commit.
Retained native logs explicitly pass the direct command-substitute fixture and
the upstream PR merge/squash/rebase, recovery and conflict fixture. This also
qualifies the committed pagination correction tracked by
[Shared #42](https://github.com/dragginzgame/shared-tooling/issues/42), replacing
the earlier dirty-source observation. IC Memory exports the canonical helper
without selecting PR delivery. Its real-Git PR fixture was not executed locally
under the consumer overlay; native upstream fixtures remain separately attributed
and do not prove a live consumer release.

Fresh Linux evidence is retained in
`target/qualification/shared-0.1.23-adoption/`. The canonical direct fixture
passes final-hook tag-target, tag-object, index and working-file mutation
refusals, exact-version repair/retry, completed local/remote tag and branch
conflicts, remote-query failure, no-effect clean resume and known descendants.
Actual consumer Make dispatch passes unsupported-delivery refusal, selection
forwarding, launcher isolation and attempt retention. Both fixtures pass under
Bash 5 and genuine Bash 3.2.57; the final 3.2 run selects that interpreter on
PATH for nested scripts too. An earlier outer-interpreter-only run is retained
separately and is not used to claim nested 3.2 qualification.

All 31 Rust repo-tool tests pass on the selected Rust 1.99 inputs, including
receipt/archive corruption, older-release recovery and isolated real-index
admission. Release effects are substituted; the isolated index fixture reuses
existing history without commits or tags. Snapshot integrity, dependency pins,
ShellCheck, actionlint and 150 local references across 31 selected documents
pass. Root dependency inputs, the independent manifest/lock, compiler/tool pins,
Make dispatch, Rust adapter source/tests and the real checkout index retain their
captured hashes. Earlier Host/MSRV/Clippy evidence applies only to its unchanged
recorded inputs, rather than being presented as a fresh run.

Pending notes and recovery guidance now cover the full compatible 0.31.4 batch;
package versions remain 0.31.3. [Consumer #25](https://github.com/dragginzgame/ic-memory/issues/25)
owns committed delivery and matching native consumer CI. The current adoption
is unstaged and uncommitted; upstream and released-consumer green runs do not
qualify these dirty consumer bytes. No broad gate, live release, package
qualification, installation, network dependency fetch, real checkout staging,
commit, tag, push, publication, CI dispatch or sibling edit occurred. No function,
method or type was removed. Published notes and prior review records remain
preserved with their original source and evidence identities.

Post-batch inspection confirms the sole active consumer workflow still passes at
released 0.31.3; this is not CI for the current edits. Shared #58/#42 are resolved
at the reviewed committed/native acceptance boundary. Concurrent duplicate
[consumer #26](https://github.com/dragginzgame/ic-memory/issues/26) is consolidated
onto the already evidenced #25, which remains open for consumer delivery and CI.

## 0.31.4 streamed receipt publication

The requested simplification review traced receipt publication, archive retention,
process execution and Cargo/release metadata against sibling Host and Shared
Tooling. The current flow-convergence method and local overlay retain their
0.1.23 identities; this is a selected source review and focused qualification,
not a whole-library or performance audit. Consumer source remains released
0.31.3 with the pending 0.31.4 edits recorded above.

Validation and package receipts previously created separate complete encoded JSON
buffers. They now share `Execute::write_json`, using the existing Host typed
durable publisher, a fixed-size output buffer and explicit flush before production
success. Serde owns pretty encoding; Host owns staging, synchronization,
publication and typed producer/cleanup failures; the Rust consumer still owns
receipt fields and recovery admission. No mode, dependency, schema or recovery
axis was added. Original lock bytes remain part of the receipt; this removes an
additional complete encoded-document allocation, not every in-memory receipt value.

Intentional separations remain: historical receipt backups copy exact bytes,
archive identity checks before/during/after copying protect distinct boundaries,
and product version/lock/surface checks bind release intent. Host's admitted
process runner uses cleared environments, null stdin and bounded capture, so it
does not replace the adapter's caller-environment and inherited-IO execution.
IC resource reports do not own local raw-Wasm probe budgets. The approved
independent PocketIC workspace retains its separate graph. No further equivalent
shared owner justified a source change in this selected review.

During verification, a concurrent root-lock update selected Host 0.5.1. The
initial input-hash mismatch and earlier test logs are retained, rather than
attributing them to one stable dependency selection. Host 0.5.1 at
`81f9809861159def2fd0987fcb7961cda4afd969` changes release tooling only; its
crate source is unchanged from 0.5.0. Selected registry sources and original
manifests match the sibling, their VCS identities name that release, and archive
checksums match the root lock. Full locked metadata confirms only artifact/fs
Host dependencies with no features. The concurrent selection was preserved;
no dependency upgrade or fetch was executed by this review.

Final evidence in `target/qualification/0.31.4-receipt-streaming/` captures one
stable source/lock selection. All 32 repo-tool tests pass on Rust 1.99 and MSRV
1.88, including native filesystem comparison of all three receipt encodings,
partial-serialization rejection with the original typed producer error, unchanged
prior receipt and owned-stage cleanup. Existing publication-error, corruption,
rollback and older-release recovery proofs still pass. Strict selected Clippy,
both-workspace formatting, pins and snapshot integrity pass. Independent graph,
compiler/tool pins, snapshot and real-index hashes remain unchanged.

[Host 0.5.1 CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37750135927)
passes for its exact source, separately from local consumer execution.
[Consumer #27](https://github.com/dragginzgame/ic-memory/issues/27) tracks delivery
and native acceptance of the receipt change alongside #25. Pending notes remain
compatible 0.31.4; package version remains 0.31.3 and published history is preserved.
Current edits remain unstaged/uncommitted. No function, method or type was removed,
and no broad gate, package/release qualification, real release effect, publication,
CI dispatch or sibling edit occurred. Dirty consumer macOS qualification remains
outstanding; no memory or speed improvement was measured.

## 0.31.4 failure-evidence archive preparation

[Consumer #28](https://github.com/dragginzgame/ic-memory/issues/28) tracks the
requested correction for retained host-tool paths such as `Linux:x86_64`, which
the pinned upload action rejects. The consumer-owned selection adapter and
fixture are applied under `scripts/ci/`, but are not connected to CI or Make
gates yet: their canonical shared archive helper remains uncommitted beside
Shared Tooling 0.1.23. The immutable 71-file snapshot is unchanged by this batch.
No incomplete workflow or dirty shared executable was activated.

Captured source and evidence remain in
`target/qualification/0.31.4-failure-upload-adoption/`, including exact source
hashes, original upstream status, a prepared workflow, failure logs and an isolated
proposed correction. The shared helper and consumer fixture pass Bash 5 on Linux.
Genuine Bash 3.2.57, also selected on PATH for nested scripts, exposes an initial
empty-array expansion under `set -u`. The isolated one-line portable-array
correction passes the complete common fixture and final consumer fixture under
Bash 3.2; the final consumer fixture also passes Bash 5. This correction was
[reported to Shared #59](https://github.com/dragginzgame/shared-tooling/issues/59#issuecomment-6056660308),
without editing sibling source or attributing dirty bytes to a committed revision.
The shared owner subsequently applied that exact correction in the working tree;
byte comparison binds the corrected dirty helper and unchanged common fixture to
the qualified proposal. This still does not establish a committed export.

The consumer fixture deliberately fails the actual exported host-tool fixture
with substitute downloads and tool versions. Its retained colon-bearing tree
round-trips through a real tar/gzip archive, together with the exact selected
temporary logs, qualification and release-validation trees, and host/IC candidate
bundles. Checks preserve original log bytes and nonzero gate status, newline/colon
filenames, permissions, executable files and unfollowed symlinks. Unrelated paths
stay outside the selection. Separate retry archives preserve prior output;
no-input collection produces no artifact, and a substituted failed tar retains
its partial output and original evidence. ShellCheck, the prepared workflow's
actionlint check, current tooling lint and current snapshot verification pass.
The pinned uploader's inspected pure validator accepts the actual collected
`evidence.tar.gz` filename, whose contents include the retained colon/newline
paths. No live upload was performed. The real index remains unchanged.

Activation still requires a reviewed corrected shared commit and canonical export,
then connecting the fixtures and prepared workflow, updating supported retention
guidance and the pending 0.31.4 notes, and rerunning focused checks against the
actual exports. Native macOS and actual upload/download acceptance remain
outstanding. No full gate, real tool installation, live CI dispatch, release,
publication, commit, tag, push or sibling edit occurred. No function, method or
type was removed. This is prepared consumer integration, not a completed CI fix;
existing compatible 0.31.4 release notes retain their previously completed scope.

## 0.31.4 example-test Clippy correction

The maintainer's check of committed consumer source
`304839ff34160473e9c3ac29dd25b64034b0693d` exposed two
`clippy::items_after_statements` errors in the streaming-receipt test. The earlier
selected example Clippy command covered its executable, not its test target.
Move the existing `RejectedValue` struct and serialization implementation to the
start of the test scope. This preserves the same assertions and typed rejection,
without adding a lint allowance or changing runtime behavior.

Fresh Rust 1.99 focused checks pass: `cargo clippy --locked --offline --example
repo-tool --tests -- -D warnings`, the selected streaming-receipt test, and
`make fmt-check`. Evidence is retained in
`target/qualification/0.31.4-test-item-order/`. No function, method or type was
removed; the two local items were moved within their existing scope. Pending
0.31.4 notes retain their compatible scope. No full gate or Git delivery effect
was executed.

## 0.31.5 archived CI failure evidence

Prepared locally on Linux from released consumer 0.31.4
`bb61562cd00ba96651d975bd166ccd5181a5d2e0` for
[#28](https://github.com/dragginzgame/ic-memory/issues/28). The production failure
collector now uploads one archive instead of exposing retained Unix filenames to
the pinned artifact transport. It retains the same selected roots, failure
condition, artifact name and 14-day policy. The separate MSRV log route is unchanged.

The canonical exporter ran from a clean isolated checkout of exact Shared source
`eeb72e741199bd8574280eacb3542d8379b912f6`. Its commit message says 0.1.25, while
VERSION remains 0.1.24; neither dirty sibling changes nor a finalized version are
attributed to this source. The existing custom-manifest option records the common
archiver and fixture in `.shared-tooling.archives.snapshot`, together with the
two unchanged bootstrap helpers. Those bootstrap records refer to the same
physical files/hashes already declared in the primary snapshot. Both independent
manifests verify without a sibling checkout. This adds two shared files, without
changing the 71 primary exports or importing the unresolved newer tracking runner
and real-Git fixture from [Shared #62](https://github.com/dragginzgame/shared-tooling/issues/62).

Fresh consumer fixtures pass with Bash 5 and genuine Bash 3.2.57 selected on PATH
for nested scripts. They force the actual exported host-tool fixture to fail
using substituted downloads/tool versions, then verify its retained colon-bearing
tree, exact selection/logs/nonzero status, unusual names, permissions, executable
files and unfollowed links through real tar/gzip. The committed common fixture
also refuses overlapping/missing inputs, symlink parents, occupied regular/link/
FIFO outputs, and archives inside their selection; newline/option-like root and
output boundaries round-trip correctly. The consumer preserves a newline-ending
root during selection instead of undoing the shared correction.

Retries retain separate outputs; missing selections produce no archive, and a
substituted failed tar retains partial output and original inputs. The consumer
payload verifier checks a separately retained archive digest before
extraction, then compares original logs and selected bytes/modes/links. Corrupted
payloads refuse before creating an extraction directory. Existing retained-fixture
destinations refuse before overwriting their evidence. No named function, method
or type was removed; payload assertions moved into the verifier used by local
and hosted checks.

The native CI matrix now prepares a retained fixture, uploads its archive with
the existing pinned uploader, downloads only the returned artifact ID with the
reviewed pinned downloader, and runs that same verifier. A locally executed
retained-fixture/output-file control and copied-file verification pass; the copy
is explicitly not a hosted transport result. Consumer native upload/download
acceptance remains pending matching CI for these unstaged/uncommitted edits.
[Exact shared CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37762726615)
passes at the selected source, separately from this consumer's source acceptance.

Evidence is retained under `target/qualification/0.31.5-failure-evidence/`, with
clean source identity, exporter logs, before/final input hashes, both shell fixture
logs, retained control/oracle and copy-verification logs. An initial ShellCheck
array-comparison warning was corrected; its failure log is preserved. Final
snapshot checks, ShellCheck, actionlint and declaration/Cargo-inheritance pins pass.
Current guidance and pending-entry links resolve. The full root changelog scan
finds a pre-existing published link to the retired `examples/composed_host.rs`;
the base commit has the same missing target. Its result is recorded separately,
and published history is preserved.
Primary exports, runner/fixture bytes, both lockfiles/manifests and toolchain/tool
pins retain their initial hashes. The raw index hash changed during final Git
inspection, after earlier identical-hash checks; indexed blobs/modes still equal
the unchanged HEAD and the cached diff is empty. No staging command was used,
and the initial hash/mismatch observation is preserved rather than reset.
Published 0.31.4-and-earlier notes
are preserved; one undated compatible 0.31.5 entry describes the completed change.
No Rust build, broad gate, real tool installation, dependency selection/fetch,
commit/tag/push, release, publication, CI dispatch or sibling edit was performed.


## 0.31.5 evidence wrapper path admission

Follow-up review at consumer base `bb61562cd00ba96651d975bd166ccd5181a5d2e0`
reproduced exit 127 before archive creation when a relative collector entry
inherited `CDPATH`. Its `cd` printed a directory into command substitution before
`pwd`, corrupting the helper root. Original failure logs remained untouched.

The collector, retained-fixture producer and payload verifier now anchor their
script operand before physical `cd`, using a non-newline suffix to preserve path
bytes. Relative retained destinations are anchored to their caller before the
fixture changes directory. No new helper library, mode or fallback is introduced.
The consumer owns script bootstrap and evidence selection; the unchanged common
archiver owns archive admission/creation; the verifier owns digest admission and
payload assertions. Existing failure, retry and retention responsibilities remain
at those boundaries.

Fresh Bash 5 and genuine Bash 3.2.57 runs with inherited `CDPATH` pass; the fixture
uses relative collector/verifier entry points. A relative retained destination
passes, as does a copied collector/verifier in a physical checkout ending in a
newline. `make test-failure-evidence verify-shared-tooling lint-tooling check-pins`
passes. Before/final hashes and reproduction/qualification logs are retained in
`target/qualification/0.31.5-wrapper-paths/`. The three consumer wrappers change;
no named function, method or type is removed. Library contracts and the compatible
pending 0.31.5 identity remain unchanged.

The canonical shared archive fixture has the same bootstrap defect at reviewed
Shared source `672ab4b8af50c75ed21a359ca5968682de83be94`, with bytes identical to
the selected archival export. Its relative invocation under inherited `CDPATH`
also exits 127; this is reported in
[Shared #67](https://github.com/dragginzgame/shared-tooling/issues/67).
Immutable exports and sibling source remain untouched. Consumer progress is
recorded on [#28](https://github.com/dragginzgame/ic-memory/issues/28).
Native consumer acceptance remains pending CI for these dirty bytes. No broad
gate, Rust build, dependency change, real installation, staging, commit, push,
release or CI dispatch occurred in this follow-up.


## 0.31.6 Host 0.7 and shared tooling qualification

Consumer base is released 0.31.5 `5cc8c6dc817843bd453b01a34b76159eb8d6797d`.
The maintainer selected registry Host 0.7.0 in the root manifest/lock, together
with the existing selected zerocopy refresh. Both `ic-host-fs` and
`ic-host-artifacts` remain host-only development dependencies with disabled default
features. Exact Host source `491fc0e231b9650526f5f57b9ab7b1f62f02218c` changes no
filesystem/artifact source relative to the qualified Host 0.5.1 revision. The new
process communication and cancellation contracts are outside this graph; the
consumer retains its configured inherited IO and command execution policy.

Fresh selected `make test-tooling` runs all 32 repo-tool tests successfully and
strict example/test Clippy passes offline with Rust 1.99.0. The consumer-owned
release-adapter fixture also reproduced a helper bootstrap failure under inherited
`CDPATH`; anchoring its script path and preserving directory bytes fixes the
relative entry point. Fresh fixture executions pass with Bash 5 and genuine
Bash 3.2.57 selected on PATH for nested scripts. They substitute release effects;
no real consumer Git effect or full gate was executed. No function, method or
type was removed. Evidence is retained in
`target/qualification/0.31.6-host-review/`.

The maintainer explicitly authorized Shared Tooling source repairs for
[#67](https://github.com/dragginzgame/shared-tooling/issues/67) and
[#66](https://github.com/dragginzgame/shared-tooling/issues/66), overriding the
sibling read-only rule only for those fixes. A second explicit one-time exception
permitted commits/tags/local pushes solely in disposable Shared test repositories.
Future policy conflicts are to be reported through owning GitHub issues; this
record does not extend either exception to future work or real delivery.
The Shared portable suite passed with Bash 5 and genuine Bash 3.2.57, inherited
CDPATH and real isolated Git
recovery fixtures. An external commit advanced Shared to
`b866d41041a1986eeec95bde9af4c6ba0853d2e3` during qualification, including the prepared
fixes and other owner's #7/#68/#69 work. No real-repository commit was performed by
this agent. Consumer adoption subsequently uses a clean private checkout of
that exact commit and its canonical exporter. The first export refused a remote
URL mismatch before mutation; matching the existing manifest URL allowed the
76-file export. Its one manifest replaces the supplemental archive manifest,
including both former duplicate verifier records. No vendored byte was patched.

Shared selection qualification uses actual synthetic host/IC installations and
the real action collector. Failed checks and changed/unknown active selections
retain full bundles; successful complete active sets retain caller pins/check
logs and IC receipts. A single local Linux measurement using the already installed
Shared bundles produced 398,794,175 bytes in 21 seconds with full retention and
25,591,711 bytes in 2 seconds with compact retention. This is one sequential sample,
including verification/collection, rather than a portable performance guarantee.
Tiny synthetic host bundles grew because their diagnostic metadata exceeded the
omitted payload. The Shared native upload/download control remains separate from
these local observations. Source-bound Linux CI at the selected committed source
uploaded and downloaded the compact archive, then failed its final stale comparison
to `portable-regression.log`. The current producer writes four IC/Rust install/check
logs. A scoped dirty Shared follow-up now compares those actual logs; a regression
executes the actual downloaded-payload verifier and rejects corruption of each.
Missing fixture companion declarations are also added at their source owners.
These follow-ups are not attributed to the committed consumer snapshot.
The fresh full Bash 5/CDPATH portable suite passes after the follow-ups, with
focused retention also passing in Bash 3.2. ShellCheck and actionlint pass.
A separate disposable Shared exporter fixture proves both installer fixtures
refuse missing evidence helpers before consumer mutation; the helper refuses its
missing selector, and the explicit complete 76-file selection passes. Its private
fixture commit is covered only by the one-time test exception and is not upstream
delivery. Logs are retained in `/tmp/shared-66-followup-portable.log`,
`/tmp/shared-66-followup-bash32-retention.log` and
`/tmp/shared-66-export-edges.ww5ikF/`.

Package version remains 0.31.5; one undated compatible 0.31.6 notes entry covers
this completed consumer batch. The maintainer's lockfile subsequently selected
Host 0.7.1; source `410fee7c309e781edf6a361f0e480d71b7c11e5a` changes no selected
filesystem/artifact code from 0.7.0. Fresh 32-test repo-tool and strict offline
Clippy checks pass against 0.7.1. The root manifest and independent qualification
manifest/lock retain their recorded input hashes; the new root lock hash is
`493c669a8976f7c9c2776508a3845cba5f851abc7eac8f873f86a092cf6f14d6`.

Consumer `verify-shared-tooling`, `check-pins`, `lint-tooling`, `test-tools`,
`test-pins`, `test-release-adapters` and `test-hooks` pass. Archive selection also
passes with genuine Bash 3.2.57; compact unknown bundles remain complete and an
invalid mode refuses collection. Product log selection stays local; the shared
selector owns fresh tool verification and classification. Full retention remains
the command default, with explicit compact selection in CI. The exported native
tracking fixture is qualified by the Shared suite; its real disposable Git
effects were not executed again through Memory. The stricter consumer fixture
boundary remains an upstream follow-up in
[Shared #70](https://github.com/dragginzgame/shared-tooling/issues/70).

Released Memory 0.31.5 native archive/upload/download/payload steps all pass on
Linux, macOS Intel and Apple Silicon in run 37772779312. Host 0.7.0 run
37773664766 is green; 0.7.1 run 37776708008 remains queued at inspection.
Those results do not qualify this dirty Memory batch or dirty Shared follow-ups;
their matching native CI remains pending in
[#29](https://github.com/dragginzgame/ic-memory/issues/29). Released
[#28](https://github.com/dragginzgame/ic-memory/issues/28) is closed after matching
transport evidence. Library contracts, compiler/tool pins
and the independent runtime graph are unchanged. No broad consumer gate, network
preparation, installation, staging, release or CI dispatch occurred.


## 0.31.7 committed installer follow-ups and hook bootstrap

Released source is `15cfca2c575ad9708902c5abf828bf36f55356d3`, matching remote main
and `v0.31.6`. The matching run 37788652605 is queued at initial inspection;
publication alone does not complete native acceptance for
[#29](https://github.com/dragginzgame/ic-memory/issues/29).

A clean private checkout of committed Shared follow-up
`db039347d2372b877c1c46dcdd2b5c3aa9412009` canonically refreshes the same 76-file
selection. It imports the reviewed installer operand corrections, fixture
companion declarations and documented selection behavior. Shared's corrected
workflow oracle is part of that owning source but is not a consumer workflow
export. No dirty sibling bytes or vendored patch are used.

The consumer-owned hook fixture separately reproduces exit 128 before source
lookup under inherited `CDPATH`. Its relative initial cd printed an extra path
into ROOT. Anchoring BASH_SOURCE and preserving physical directory bytes repairs
that entry point. The native matrix now explicitly exercises the existing hook
suite with CDPATH; no new fixture framework or Git delivery effect is added.
The suite borrows an existing committed object graph read-only and touches only
its disposable index, not the real repository index/history.

Fresh `verify-shared-tooling`, `check-pins`, `lint-tooling`, `test-tools`,
`test-release-adapters` and `test-hooks` all pass with inherited CDPATH. The
actual hook suite also passes with genuine Bash 3.2.57 selected for nested scripts.
Tool downloads and installations in the installer fixtures are substitutes;
hook formatting uses the prepared Rust/cargo-sort tools. No real provisioning,
native Git release fixture, full Memory gate, dependency fetch, stage/commit/push
or CI dispatch occurred. Both manifests/lockfiles retain their initial hashes.
Evidence is retained in `target/qualification/0.31.7-tooling-review/`.

The compatible pending notes select 0.31.7 without changing package version
0.31.6. No function, method or type is removed. Direct release delivery, receipt
contracts and the independent runtime graph remain unchanged. Consumer delivery
and matching native checks are tracked in
[#30](https://github.com/dragginzgame/ic-memory/issues/30); the future consumer
release-fixture profile remains owned by
[Shared #70](https://github.com/dragginzgame/shared-tooling/issues/70).

The post-batch exact-source inspection confirms Memory 0.31.6's Linux native,
MSRV and lint jobs pass in run 37788652605; macOS ARM native is running, with
Intel native and macOS MSRV queued. Shared db039's Linux native job in run
37787910279 passes the corrected compact upload/download/payload control;
both macOS jobs remain queued. Concurrent dirty Shared profile/budget edits
are excluded from this export. Host 0.7.2 source
`8236b506307d33f7c34f56d6e0ba9db4300792d7` changes no selected filesystem/artifact
code from 0.7.1; its run 37789259299 is queued and Memory's lock stays at 0.7.1.

A separate read-only installer review reproduces false success for a literal
newline-bearing active host link in a private copy of the authentic installed
bundle. Shared's direct checker trims that link while reconstructing its bundle
path, even though the active bin path is missing. The selector's literal-link
guard keeps full evidence for this case. The owning finding and smallest repair
are recorded in [Shared #75](https://github.com/dragginzgame/shared-tooling/issues/75);
no installer source or real active tool set was changed. Original inputs/logs
remain in `/tmp/shared-literal-active-link.hhJ9Yv/`.


## 0.31.7 clean-source diagnostics

The existing pending 0.31.7 batch remains based on released consumer
`15cfca2c575ad9708902c5abf828bf36f55356d3`. Before this change, actual
`make ensure-clean` returned a generic refusal that omitted every changed path.
The same read-only command now refuses with the actual Git porcelain entries,
preserving staged/working columns and quoted unusual names. `Repository::clean`
uses its existing command output directly; no path parser, new observation or
shared state owner is introduced. Failed Git observations propagate unchanged.
The separate prepared-version-surface guards retain their own behavior.

The new boundary fixture clones an existing committed graph locally and changes
only its private working files/index. It admits a clean checkout, reports staged
Cargo.lock, unstaged Cargo.toml and a newline-bearing untracked path, and preserves
the staging tree and file bytes on refusal. Fault injection verifies that status
observation failure dispatches no other process and changes no package version.
No commit, tag or push is used, including in the fixture. All 33 repo-tool tests
and strict example/test Clippy pass offline on Rust 1.99.0; `make fmt-check`
passes for both maintained workspaces. Original/final command status and logs are
retained in `target/qualification/0.31.7-source-admission/`.

The change is tracked in [#31](https://github.com/dragginzgame/ic-memory/issues/31)
and reported as consumer evidence to the related
[Shared #74](https://github.com/dragginzgame/shared-tooling/issues/74). It does not
claim an upstream guidance fix or a new diagnostic path for every preflight guard.
Package version remains 0.31.6; one compatible pending 0.31.7 entry includes this
work. Both dependency graphs, compiler/tool pins, library contracts and durable
formats are unchanged. No function, method or type is removed. No broad gate,
network preparation, sibling edit, real index/history mutation, release or
publication occurred. Native qualification remains tied to future candidate CI.
At the post-batch inspection, released 0.31.6 Linux and Apple Silicon native jobs,
Linux MSRV and lint pass in run 37788652605. Intel native and both macOS MSRV jobs
remain queued, so #29 remains open. These observations do not qualify the dirty
candidate's source-admission change.

The maintainer subsequently changed the root catalog/lock from Host 0.7.1 to
0.8.0 during this turn, after the first test/Clippy logs were complete. Those
earlier logs remain 0.7.1 evidence. New input hashes and separate `*-host080.log`
checks bind fresh qualification to the selected 0.8.0 graph; this agent performed
no dependency selection or fetch. Exact Host source
`fc74f679c7503ef9dd2db8fbd893c5c5907c72c4` has identical filesystem/artifact code to
released 0.7.1. Its breaking process cleanup/error changes are outside the selected
host-only development crates/features, so public Memory contracts stay compatible.

Fresh 33-test tooling, strict example/test Clippy, both-workspace formatter and
pin checks pass offline with 0.8.0. Locked offline metadata projection succeeds
for both maintained workspaces; this is declaration/metadata evidence, not a
runtime-qualification build. The independent qualification manifest/lock remain
unchanged, and the new root manifest/lock hashes are recorded in
`host080-inputs.sha256`. Host's exact-source run 37792592340 is queued; new native
consumer acceptance remains pending. The compatible pending version stays 0.31.7.

## 0.31.7 Host 0.8.1 consumer review

The maintainer-selected root lock now uses published `ic-host-fs` and
`ic-host-artifacts` 0.8.1. Host release tag and remote main identify
`973f00a029dd873c242a4d06e9f8d2d5172ff0df`. Review of its committed changes from
0.8.0 found bounded geometric growth for owned artifact reads/chunk digests and
an additive nonblocking regular-file lock API. Memory uses streaming copying,
hashing and the existing durable publishers; those called implementations are
unchanged. No consumer API adaptation, new lock protocol or allocation-performance
claim follows from this review. The selected graph contains only the two Host
crates with empty feature sets, excluding process and IC-report mechanics.

Fresh locked/offline checks on Rust 1.99.0 pass: all 33 repo-tool tests, strict
example/test Clippy, both-workspace formatting, declaration pins and the 76-file
snapshot verifier. Root full-graph metadata and independent qualification
workspace metadata projection also pass offline. The latter is not a runtime
build. Inputs and separate `*-host081.log`/metadata files remain under
`target/qualification/0.31.7-source-admission/`; all recorded manifest, lock,
source and snapshot hashes remain unchanged after checks. Earlier 0.7.1 and
0.8.0 evidence is preserved with its original selection. No dependency fetch or
reselection, package-version change, sibling edit or full local gate occurred.

Host's sole active CI workflow returned no run for the exact 0.8.1 release SHA
at inspection. The preceding 0.8.0 run
[37792592340](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37792592340)
passes Linux/MSRV but fails both macOS native jobs when its cleanup fixture
spawns absent `/bin/true`. The released 0.8.1 fixture retains that operand;
the concurrent dirty 0.8.2 replacement with `/bin/sh -c 'exit 0'` is separately
owned under [Host #5](https://github.com/dragginzgame/ic-host-tooling/issues/5).
It was neither edited nor qualified here. This process-test failure is outside
Memory's selected graph, and absent exact-source CI does not qualify 0.8.1.

Released Memory 0.31.6's sole CI workflow now completes successfully at
`15cfca2c575ad9708902c5abf828bf36f55356d3`, attempt 1 of
[37788652605](https://github.com/dragginzgame/ic-memory/actions/runs/37788652605).
Linux and both macOS native/MSRV jobs plus lint pass. Each native job's hook
isolation and failure-evidence archive/upload/download/payload steps pass,
completing [#29](https://github.com/dragginzgame/ic-memory/issues/29). That source
still selects Host 0.7.1; it does not qualify the dirty 0.31.7 changes or Host
0.8.1. Pending 0.31.7 remains compatible and requires matching native acceptance.

## 0.31.7 Shared 0.1.28 adoption

The pending consumer batch canonically adopts committed Shared Tooling
`1872ed2c20f6c70689bb2249050b1d673c60bfa0` from a clean private checkout. The
previous 76 files remain selected; fourteen explicit additions complete the
baseline's linked maintenance catalog, optional coordinator and scheduler
templates. The resulting 90-file snapshot verifies exact hashes and modes.
The common MSRV and std/no_std guidance is recorded without changing compiler
selection. No task, live agent, schedule or installation is activated by copying
these inputs. Shared's real checkout remains unchanged.

Host/IC installers now admit literal active-link bytes, rejecting malformed
newline-bearing targets before any probe, download, lock or replacement.
[Shared #75](https://github.com/dragginzgame/shared-tooling/issues/75) owns this
repair. Consumer release-runner qualification is simulation-only again:
its native Git delegate accepts only non-writing `hash-object --stdin`, and a
sticky record detects attempted repository-operation escapes even in negative
cases. The real-Git tracking suite retains its existing owner scenarios but is
not selected or executed here
([Shared #70](https://github.com/dragginzgame/shared-tooling/issues/70)). No prior
fixture-effect exception is reused, and no commit, tag or push is created.

Focused snapshot, pin, action/shell lint, consumer Make/release adapters and
simulation-runner checks pass. Host/IC suites pass under CDPATH on Bash 5 and
genuine Bash 3.2.57, including nested Bash 3.2 calls and literal one/two-newline
refusal with unchanged link bytes and no tool/setup effects. The simulation
runner also passes under nested Bash 3.2. The immutable owner's optional
coordinator fixture passes on both shells with a substitute CLI, covering due
admission and failed/empty reports; that is not live agent or timer evidence.
Consumer tooling LOC/source/snapshot ownership checks pass for the expanded
selection. Inputs and separate logs remain in
`target/qualification/0.31.7-shared028-review/`.

All 67 catalog/AGENTS/minor-note local references pass. The wider full-root
changelog check fails at an already released 0.15.4 example link, which still
points to its pre-workspace path. Its failed log is retained, and
[#32](https://github.com/dragginzgame/ic-memory/issues/32) owns the proposed
historical-link correction. Both published changelog suffixes remain byte-for-byte
unchanged; no historical text or redirect file was introduced. Current root and
detail entries, snapshot provenance and the existing
[#30](https://github.com/dragginzgame/ic-memory/issues/30) adoption tracker agree.

Both workspace manifests/locks, tool pin inventories and repo-tool Rust sources
match their pre-adoption hashes. Earlier Host 0.8.1 Rust tests/Clippy retain
their original scope rather than being relabeled as new script qualification.
Package version stays 0.31.6, with compatible pending notes at 0.31.7. No public
API, durable format or production release-runner behavior changes; no function,
method or type is removed. The native tracking block moved to its canonical
upstream fixture rather than being discarded. No broad local gate, dependency
fetch/reselection, real index/history mutation, release or publication occurred.

At exact-source inspection,
[Shared CI 37799837183](https://github.com/dragginzgame/shared-tooling/actions/runs/37799837183),
attempt 1, passes Linux portable regression and lint/security. Linux native
failure-evidence and compact-tool upload/download/payload controls pass; both
macOS native jobs remain queued. The preceding db039 owner run now succeeds
across its full matrix, but does not qualify this new source. Released Memory
0.31.6's completed matrix remains distinct from dirty 0.31.7; this adoption
requires matching native candidate acceptance after delivery.

Concurrent dirty Shared policy edits observed at final inspection narrow standing
issue authority to `dragginzgame/*`, requiring explicit authority for external
GitHub writes. The live-local exception applies that rule now; local overlay
wording reflects the scope. Those uncommitted bytes are excluded from the
1872ed2 snapshot. All issue actions in this batch used verified `dragginzgame`
owners, and no external destination or schedule was activated.

## 0.31.8 Host 0.8.2 and example-link review

The maintainer delivered Memory 0.31.7 at
`bcb908ff6e29ab541cd4bf95b2f7ec0744ad6c33`, matching GitHub main and annotated
tag `v0.31.7`. Its exact-source
[CI 37802948348](https://github.com/dragginzgame/ic-memory/actions/runs/37802948348),
attempt 1, passes Linux native/MSRV and lint, including native CDPATH hook
isolation and archive/upload/download/payload controls. Both macOS native/MSRV
selections remain queued at inspection. Delivery is recorded separately from
complete native acceptance for
[#30](https://github.com/dragginzgame/ic-memory/issues/30) and
[#31](https://github.com/dragginzgame/ic-memory/issues/31).

The maintainer's new root lock selects registry `ic-host-fs` and
`ic-host-artifacts` 0.8.2 with empty feature sets. Their published Rust sources
match Host release `92bd2fecc71124b562e227a32a67644e1e5e34b7` and are identical
to 0.8.1. The only Host Rust change is the portable process cleanup fixture's
`/bin/sh -c 'exit 0'` replacement, outside Memory's selected graph. No consumer
API adaptation, new buffer or lock owner is justified. Host's new
[#27](https://github.com/dragginzgame/ic-host-tooling/issues/27) concerns Testkit's
shared/observed locking composition; Memory does not use that path and no sibling
repair was attempted.

Fresh locked/offline checks pass on Rust 1.99.0: all 33 actual repo-tool tests,
strict example/test Clippy, both-workspace formatting, declaration pins and the
90-file snapshot verifier. Root full-graph metadata and independent qualification
workspace metadata projection pass; the latter is not a runtime build. Both
workspace manifests/locks, Rust sources, snapshot and pin inventories retain
their recorded pre-check hashes. The already-selected root lock remains 0.8.2;
this agent performed no dependency selection or fetch. Separate inputs/logs are
in `target/qualification/0.31.8-host-review/`.

Under the request to work through issues,
[#32](https://github.com/dragginzgame/ic-memory/issues/32) is fixed locally by
changing only the published 0.15.4 example hyperlink's destination to
`crates/ic-memory/examples/composed_host.rs`. No historical prose, version,
date or order changed. Before the correction, the canonical committed Shared
local-link checker refuses the old destination; afterwards all root/detail note
links pass. The failed result remains in `links-before.log` with its status.
History comparison admits exactly this one destination correction in root and
requires the entire released detail suffix to remain byte-for-byte unchanged.
No obsolete example copy or compatibility redirect is added.

The new compatible pending 0.31.8 root/detail notes cover this lock selection and
link correction. Package version stays 0.31.7; public APIs, stable-memory formats,
compiler/tool pins and independent runtime qualification inputs are unchanged.
No function, method or type is removed. No broad local gate, real index/history
mutation, release, publication or CI rerun/dispatch occurred.

Host's exact 0.8.2
[CI 37801871391](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37801871391)
passes Linux native and Rust 1.88; both macOS native jobs remain queued. That does
not yet prove native acceptance of the repaired process fixture or nonblocking
locks. The new Memory candidate's local qualification is separate from released
0.31.7 CI and requires matching native evidence after delivery.

## 0.31.9 read-only source observation

Reviewed released base `4f48e2626a5d672e0bd12237f5303b35dd932be4`, matching
remote main and annotated `v0.31.8`. The delivered composed-host example link
resolves in that tree; all 72 local references across root/detail notes and this
document pass. This completes delivery for
[#32](https://github.com/dragginzgame/ic-memory/issues/32).

A disposable checkout using existing committed history reproduces a remaining
[#31](https://github.com/dragginzgame/ic-memory/issues/31) preservation gap:
after `read-tree HEAD` and `checkout-index --all --force`, ordinary Git 2.43.0
status reports clean source but refreshes stat-cache entries in `.git/index`.
The same query with `--no-optional-locks` reports clean source while preserving
the index byte-for-byte. No fixture commits, tags or pushes are needed. Original
and corrected index pairs/status logs are retained under
`/tmp/ic-memory-status-index.jRh9x2/`.

The local fix adds that Git option only to `Repository::clean`'s existing
observation. It does not disable required locks, change other Git commands,
parse paths again or replace the Rust adapter with a shell helper. The existing
real-Git fixture now compares exact index bytes after both clean admission with
empty stat-cache entries and dirty refusal, alongside its staged/unstaged/quoted
untracked paths, preserved file/tree bytes and unchanged observation-error checks.

Focused locked/offline qualification passes on Rust 1.99.0: the targeted boundary
test, all 33 repo-tool tests, strict example/test Clippy and both-workspace
formatting. Logs and source/input hashes belong to
`target/qualification/0.31.9-source-admission/`. Public APIs, durable formats,
both manifests/lockfiles, tool/compiler pins and the 90-file Shared snapshot at
`1872ed2` are unchanged. No function, method or type is removed. Pending root
and detailed notes select compatible 0.31.9 without changing package versions.

Released 0.31.8
[CI 37807169733](https://github.com/dragginzgame/ic-memory/actions/runs/37807169733)
has successful Linux and Intel macOS native jobs, including Intel CDPATH hook
and native archive/upload/download/payload controls. ARM native and some MSRV
jobs remain queued at inspection. The older 0.31.7 run is cancelled; its earlier
successes are not relabelled. This dirty 0.31.9 repair has no matching remote CI
or delivery yet. No broad local gate, real index/history mutation, release,
publication or CI rerun/dispatch occurred.

## 0.31.9 whitespace-path admission

Review of the same release-admission boundary found that trimmed Git file lists
can conceal unrelated whitespace-only names. Git 2.43.0 emits a space-only name
as `20 0a`; the adapter's generic Git-output trimming converts it to an empty
string. This affects the untracked-file check, source-difference comparison and
staged-file guard. The raw reproduction is retained in
`/tmp/ic-memory-0319-path-review.udHovA.untracked`.

The local correction uses the existing raw command-output method for those
three file-list observations. Identity/version queries retain their existing
trimming. No path parser, new abstraction or dependency is introduced. A native
Git fixture builds a source tree object and private index without committing;
it first admits unchanged surfaces, then refuses untracked, working and staged
whitespace-only paths. Refusals preserve the relevant file bytes and index tree.
This extends the pending
[#31](https://github.com/dragginzgame/ic-memory/issues/31) admission work.

The new regression fails before correction and passes afterward. All 34
repo-tool tests and strict example/test Clippy pass locked/offline with the
selected Host 0.8.3 crates. Both-workspace formatting passes after correcting
one new test-line wrap. Original failure logs, successful checks and source/input
hashes are retained in `target/qualification/0.31.9-path-admission/`; earlier
0.31.9 evidence is preserved. Public APIs, durable formats, package versions,
dependency selections, pins and the Shared snapshot are unchanged by this fix.
Root and detail notes retain the same compatible 0.31.9 candidate.

Released 0.31.8
[CI 37807169733](https://github.com/dragginzgame/ic-memory/actions/runs/37807169733)
now passes all three MSRV jobs, Linux native, Intel macOS native and tooling lint.
ARM macOS native has passed setup, formatting and hook isolation and is running
toolchain qualification; archive acceptance is still outstanding. No remote run
tests these dirty 0.31.9 changes. No broad local gate, real index/history effect,
sibling source edit, release, publication or CI dispatch/rerun occurred.

## 0.31.9 Host 0.8.3 review

The maintainer's subsequent root lock selection advances only registry
`ic-host-fs` and `ic-host-artifacts` from 0.8.2 to 0.8.3. Root compatible 0.8
requirements and disabled default features are unchanged. Fresh locked/offline
metadata selects both crates with empty feature sets; the independent runtime
workspace retains its manifest and lock. Earlier 0.31.9 source-admission logs
and hashes remain evidence for their original 0.8.2 selection.

Host's delivered source `67d031222073f23ad437b45053156e229f86a016` adds
`open_regular_lock_file_with_parents` by promoting the existing regular-file
opener and moving acquisition to its callers. Admission and locked callers
retain their behavior; Memory's typed durable writes do not need adaptation.
All 19 artifact Rust source files and all 18 filesystem Rust source files in
each cached published crate match this committed tree and its recorded Cargo
VCS identity. Artifact Rust code is unchanged from 0.8.2; the filesystem change
is confined to lock opening/callers and their fixtures. Host's process output
observer is outside Memory's selected dependency graph.

Fresh 0.8.3 qualification passes on Linux/Rust 1.99.0: all 33 repo-tool tests,
strict example/test Clippy, both-workspace formatting, declaration pins and the
90-file snapshot verifier. Both locked metadata projections pass; independent
runtime metadata is not an installed PocketIC build. Source/input hashes and
logs are retained separately in
`target/qualification/0.31.9-host083-review/`. Pending 0.31.9 notes now include
this compatible host dependency refresh; package version remains 0.31.8.

[Host CI 37810259864](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37810259864)
passes Linux native and Rust 1.88 MSRV; both macOS native jobs are queued.
Shared Tooling remains at committed 0.1.30
`4e274a2219c0b0cc3af68ec65658b373253518fb`, with only its subsequent VERSION
preparation dirty at inspection.
[Shared CI 37809818114](https://github.com/dragginzgame/shared-tooling/actions/runs/37809818114)
passes Linux portable and lint; both macOS jobs are queued. Its separate manual
Cargo-install assessment has no matching run. The reported dashboard/assessment
path bugs and PocketIC guidance gap remain unchanged in that committed source.
No sibling source was repaired or adopted. Memory's Shared snapshot, tool pins,
PocketIC 16.0.0 boundary and independent qualification inputs are unchanged.
No dependency reselection/fetch, broad local gate, real Git delivery, release,
publication or CI rerun/dispatch occurred.

## 0.31.9 remaining-scope review

A subsequent bounded review found no additional actionable defect in the pending
batch. The remaining trimmed commit-stage file-list comparisons follow the
corrected prepared-source and index guards, which already reject unrelated
whitespace-only paths. The source/index fixtures also retain exact staged-byte,
file-mode and saved-intent checks. No source or dependency selection changed in
this inspection, so the existing 34-test, Clippy and formatting evidence remains
applicable; no broad gate or redundant build was run.

Released 0.31.8 source `4f48e2626a5d672e0bd12237f5303b35dd932be4` now has a
fully successful seven-job
[CI run 37807169733](https://github.com/dragginzgame/ic-memory/actions/runs/37807169733).
The completed ARM native job passes host/IC setup, formatting, CDPATH hook
isolation, pinned-toolchain qualification and native archive/upload/download/
payload controls. Together with Linux/Intel native and all MSRV jobs, this
completes the remaining consumer acceptance criterion in
[#30](https://github.com/dragginzgame/ic-memory/issues/30). The dirty 0.31.9
admission follow-ups in [#31](https://github.com/dragginzgame/ic-memory/issues/31)
still require their own delivery and matching native qualification.

The clean Host sibling now has delivered 0.8.4 source
`97187b2a46d6f8a6964224a36a133d858ef0d223`, matching remote main and the peeled
annotated release tag. Its library correction skips temporary creation/sync when
admitting an existing lock file. Memory uses hashing and durable publication,
not that lock opener; those selected call paths are unchanged. Host's
[matching CI 37818647474](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37818647474)
is queued, so native acceptance is not established. Memory retains selected
Host 0.8.3; Shared remains committed at `4e274a2` with its previously reviewed
local governance edits. No sibling repair or snapshot refresh occurred.

## 0.31.10 CI retention

The maintainer delivered 0.31.9 at
`6343f62a36636cb999fa404e4b8769159ca9f4cb`, matching remote main and the peeled
annotated `v0.31.9` tag. Its
[CI run 37892022002](https://github.com/dragginzgame/ic-memory/actions/runs/37892022002)
now passes all seven jobs, including native Linux, Intel macOS and ARM macOS,
all three MSRV selections and tooling lint. Both macOS native jobs explicitly
pass CDPATH hook isolation and archive/upload/download/payload controls. This
completes the remaining delivery/native criterion in
[#31](https://github.com/dragginzgame/ic-memory/issues/31). The delivered lock
selects Host 0.8.4; earlier local 0.31.9 evidence for Host 0.8.3 is not relabelled.

For [#33](https://github.com/dragginzgame/ic-memory/issues/33), the local workflow
now groups push runs by workflow/ref/source SHA and preserves them; PR runs retain
one group per PR ref and may replace earlier revisions. A SHA group is necessary
to preserve pending runs as well as active jobs. Only the two concurrency fields
and explanatory comments change; gates, hosts, pins, permissions, timeouts and
evidence controls are unchanged. No new fixture or scheduling framework is added.

The actual changed workflow passes actionlint 1.7.12 and the maintained tooling
lint target with ShellCheck 0.11.0. Both-workspace formatting, declaration pins,
the exact 90-file snapshot and whitespace checks also pass. Logs and source/input
hashes are retained in `target/qualification/0.31.10-ci-retention/`. Published
root/detail notes remain intact, with one undated compatible 0.31.10 candidate.
This dirty workflow has no matching remote CI. Actual Memory scheduler acceptance
requires delivery and naturally overlapping pushes; a static lint pass is not
proof that GitHub preserved older running/queued jobs.

The maintainer's new root lock selects Host filesystem/artifact 0.8.5. Their Rust
sources are unchanged from 0.8.4, and the existing compatible requirements and
disabled default features remain. Clean Host source
`1cad3253096b6eb67be5187209e7fb606593c501` matches remote main and peeled
annotated `v0.8.5`; its
[CI run 37893479726](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37893479726)
is successful. That owner result does not qualify Memory's new locked graph.
Initially, locked/offline Memory metadata and adapter tests stopped because
`ic-host-artifacts 0.8.5` was absent from the local Cargo cache. The failed test log
is preserved; no online retry or dependency reselection occurred. At that point,
dependency preparation was needed before focused Host 0.8.5 adapter qualification.

Before any network preparation was authorized or executed, the selected cache
became available independently. A fresh locked/offline metadata observation now
succeeds against the same input hashes and selects both Host crates with empty
feature sets. All 34 repo-tool tests and strict example/test Clippy pass offline;
their successful logs are separate from the initial cache-miss failure. All 18
filesystem and 19 artifact Rust source files in each cached registry copy match
Host's committed release tree and Cargo VCS identity. The earlier fetch question
was no longer needed for those checks; no dependency preparation or online retry
had run at that point.
The initial failure remains evidence for its original cache state, not a current
qualification blocker.

The maintainer subsequently authorized the locked fetch explicitly.
`make fetch-dependencies` succeeds and reports no downloads; both lockfiles and
all recorded source/input hashes remain unchanged. Its log is retained with this
batch. No repeated build was needed because the qualified selection stayed intact.

Memory's manifests, independent runtime graph, tool/compiler pins, public APIs,
durable formats and Shared snapshot remain unchanged. Shared's current committed
0.1.32 governance applies through the live-local policy exception, but no sibling
source was edited or adopted. No symbol was removed. No broad gate, staging,
commit/tag/push, release, publication or workflow dispatch/rerun occurred.

## 0.31.10 Host 0.8.6 and fleet fixture selection

The maintainer's root lock now selects Host filesystem/artifact 0.8.6 while
retaining compatible 0.8 requirements and disabled default features. Released
Host `9f3d9a83def91030056c78e44c9efaa489be7d12` matches remote main and peeled
annotated `v0.8.6`. Its additive `CommunicationLimits` selects optional deadlines
for caller-owned child communication through the existing I/O engine; capture
and executable admission keep finite `OutputLimits`. Memory does not select that
process crate. All filesystem/artifact Rust sources are unchanged from 0.8.5.
Each cached published copy's 18 filesystem and 19 artifact Rust files matches
this committed release tree and its Cargo VCS identity.

For [#34](https://github.com/dragginzgame/ic-memory/issues/34), caller review found
three upstream-owned regression programs dispatched only by consumer
`test-tools`: `test-cloc-siblings.sh`, `test-cloc-tooling.sh` and
`test-cloc-fixture-contexts.sh`. The context fixture also invokes the sibling
fixture, so these retire together with their Make dispatch and snapshot records.
Pinned cloc 2.10 measures 267 code lines removed, excluding comments/blanks;
these were shared copies, not an independent product implementation. None of
the removed programs declares a production function or type.

The snapshot now selects 87 files from the same `1872ed2` source revision;
every retained hash/mode is unchanged. No vendored file was patched or dirty
sibling source imported. The local `test-cloc.sh`, setup/check commands, pinned
cloc and checksum/verifier companions remain selected. Existing documented fleet
report commands remain an intentional selection for this patch; their retirement
requires a separate scope decision and the committed optional-selection update
owned by [Shared #83](https://github.com/dragginzgame/shared-tooling/issues/83).
That current upstream Make/guide change is uncommitted and is not adopted here.
Local AGENTS and host-support guidance now describe the selected fixture roster.

Fresh locked/offline Memory checks pass: all 34 repo-tool tests, strict
example/test Clippy and full metadata with empty Host feature sets. The actual
tool-command dispatch fixture and retained local LOC fixture pass; real
`make cloc` reports the root workspace and an explicit independent qualification
manifest reports only that graph. This is LOC/metadata evidence, not installed
PocketIC qualification. Exact snapshot, formatting, tooling lint, declaration
pins, documentation links and whitespace checks accompany this selection.

Logs, selected-input hashes, cached-source comparisons and removed-fixture LOC
belong to `target/qualification/0.31.10-host086-fleet-fixtures/`. Earlier 0.31.10
Host 0.8.5/90-file logs retain their original scope. Package versions, independent
runtime inputs, compiler/tool pins and public/durable contracts remain unchanged.
The same compatible 0.31.10 notes cover the existing CI retention fix and this
completed batch. No broad gate, dependency fetch/reselection, staging, commit,
release, sibling edit or CI dispatch/rerun occurred. Dirty 0.31.10 still needs
matching delivery and native acceptance; native Host evidence is separate.

At the final inspection, delivered Host 0.8.6
[CI 37896909262](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37896909262)
passes Linux/Intel macOS/ARM macOS native jobs and MSRV. Released Memory 0.31.9
still has its complete matching CI success; neither run tests this dirty
0.31.10 fixture selection. All 100 selected documentation references resolve.

## 0.31.10 released source and Host 0.8.8

Remote main and the peeled annotated `v0.31.10` agree with Memory release
`b42cc73a1a848bea509e7a5ad4791877d984ba3e`. Its actual root lock selects Host
filesystem/artifact 0.8.8. Earlier notes and qualification above describe the
0.8.6 preparation; they are retained as historical evidence, rather than relabelled
as qualification of the subsequently selected release inputs.

Fresh locked/offline metadata selects both Host crates without enabled features.
Their cached Cargo VCS identities and 18 filesystem/19 artifact source files
match Host release `ccfd7724dd31c14cfbb8ae434f683babfeabf906`. All 34 repo-tool
tests and strict example/test Clippy pass against Memory's released selection.
Input hashes and logs are retained under `target/qualification/0.31.10-host088/`;
both manifests, both locks and the 87-file snapshot remain unchanged. This is
focused Linux consumer qualification, not installed PocketIC qualification.

Host's matching [CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37901415314)
has passed. Memory's separate exact-source
[CI](https://github.com/dragginzgame/ic-memory/actions/runs/37903537053) owns native
consumer acceptance for the delivered cache bootstrap, concurrency and fixture
selection. Neither prior consumer runs nor native Host results replace it.

## 0.31.11 preparation restoration failures

Local pending correction for [#36](https://github.com/dragginzgame/ic-memory/issues/36),
on released source `b42cc73a1a848bea509e7a5ad4791877d984ba3e` plus the adapter
and regression changes. The common release runner and both selected lockfiles
are unchanged. The root graph still selects Host filesystem/artifact 0.8.8 with
empty feature sets. The correction belongs to Memory's metadata-restoration
boundary; individual durable writes remain owned by Host.

A substituted package failure (`UnexpectedEof`), restoration-write failure
(`PermissionDenied`) and restoration-read failure (`NotFound`) reproduce the
old early exit and lost preparation error. The corrected result retains each
native error and failed path, completes the other owned restorations and
preserves a foreign README edit, validation receipt and package artifact.
No fixture commits, tags, pushes or package publication occur.

Focused evidence is retained in `target/qualification/0.31.11-rollback/`:
`before-isolated.log` records the reproduced defect, `tests.log` records all 35
adapter tests passing, and `clippy.log` records strict example/test Clippy with
Rust 1.99.0. `before.log` is an earlier inconclusive fixture-directory collision;
its retained directory was preserved, and subsequent checks use a fresh private
`TMPDIR`. These are local Linux checks with substituted release effects, not
full-gate, release or native macOS qualification. Matching CI is pending delivery.

## 0.31.11 Host 0.8.9

The maintainer-selected root lockfile now selects `ic-host-fs` and
`ic-host-artifacts` 0.8.9. Locked offline metadata confirms empty features for
both crates. All 37 packaged Rust source files match released Host commit
`0464db5146be910a0f078447831fa2807072c75a` and are unchanged from 0.8.8;
Host's release-adapter change is outside Memory's library dependency surface.
Memory's compatible requirements, both manifests, independent runtime lockfile
and Shared Tooling snapshot remain unchanged.

`target/qualification/0.31.11-host089/` retains input hashes, metadata, feature
selection and source inventories, plus passing logs for all 35 adapter tests,
strict Rust 1.99.0 example/test Clippy and a focused Rust 1.88.0 example check.
These qualify the local pending recovery correction with the new selected
libraries on Linux. They do not replace complete release gates or native macOS
qualification; the preceding 0.8.8 recovery evidence retains its original scope.

## 0.31.12 retained fixture collisions

Local pending correction for [#37](https://github.com/dragginzgame/ic-memory/issues/37)
on released source `d40a99919eb21378edd2e3305171c45957ae0e0c`, changing only the
Rust release-fixture constructor and its regression. A preexisting candidate file
reproduces the old native `AlreadyExists` panic. The corrected fixture skips
occupied directories/files through exclusive creation and preserves their
payloads after cleanup; the global `NEXT_FIXTURE` counter is no longer needed.
Other filesystem failures are not retried.

Evidence remains in `target/qualification/0.31.12-fixture-collisions/`:
`before.log` records the reproduced failure, `tests.log` records all 36 adapter
tests passing with the ordinary temporary directory, and `clippy.log` records
strict Rust 1.99.0 example/test Clippy. The failed reproduction remains under
`/tmp/ic-memory-fixture-collision-before.1sWAvy/`. These are local Linux checks
with substituted release effects; no fixture commits, tags, pushes or package
publication occur. Native macOS qualification awaits delivery. Manifests, both
lockfiles, tool pins, production adapter code and the Shared snapshot are unchanged.

## 0.31.12 Host 0.8.10

The maintainer-selected root lockfile selects Host filesystem/artifact 0.8.10.
Locked offline metadata confirms empty feature sets for both crates. All 37
packaged Rust source files match released Host commit
`e944f114f7542d27ead5df8996d1b2df04e06c31` and are unchanged from 0.8.9.
Host's shared checker/hook adoption affects its tooling, not Memory's selected
library source or immutable Shared Tooling snapshot.

`target/qualification/0.31.12-host0810/` retains input hashes, metadata, feature
selection, source inventories and passing logs for all 36 Rust adapter tests
and strict Rust 1.99.0 example/test Clippy. These are focused local Linux checks
of the pending fixture correction with the new selected libraries, not complete
release or native macOS qualification. Both manifests, the independent runtime
lockfile and Shared snapshot remain unchanged; no dependency fetch was needed.

## 0.32.0 Shared Tooling hard cut

Pending local adoption for [#38](https://github.com/dragginzgame/ic-memory/issues/38)
on released Memory `d40a99919eb21378edd2e3305171c45957ae0e0c`. The canonical
committed exporter refreshed the reviewed selection to Shared Tooling 0.2.0
`8140e3dd1b44409d682c721889ab702f438c6a17`, adding only the linked read-only
release-source helper to complete the governance roster. All 88 selected files
pass snapshot integrity. No vendored implementation was patched. Later dirty
Shared work for [#87](https://github.com/dragginzgame/shared-tooling/issues/87)
is not part of this export; Memory's selected catalog has its final newline.

The five-tool IC selection removes PocketIC completely from the shared catalog,
validator and installer branches. Actual Linux setup and offline checks pass.
The former six-tool offline check refuses before activation; explicit setup
selects a new bundle. The old pin-file and receipt bytes still match their saved
copies, and all seven receipt-covered files verify afterward. No retained bundle
or failed evidence is deleted. The initial checksum invocation used the wrong
working directory and was retained separately from the corrected bundle-root
checks; that attempt is not a successful verification.

PocketIC now comes from published Testkit 0.25.4's exact registry CLI, installed
through the canonical selected Cargo-tool installer with a release profile and
Cargo's published lockfile. Actual owner setup downloads and authenticates its
16.1.0 Linux server; the offline CLI receipt/byte check and server check return
the admitted absolute path. The separate runtime client remains locked to
16.0.0. `test-runtime` builds the supplied Wasm and runner before Testkit's
managed launch, clears inherited binary/URL selections, and uses the sole
`IC_TESTKIT_POCKET_IC_URL` contract. It retains input/artifact/CLI hashes,
runtime diagnostics and server stdout/stderr under unique attempt directories.
The ordinary library and release gates do not install or require a server.

Actual Linux installed qualification passes all 12 invalid-IO traps with
neighboring bytes and preceding mutation preserved, post-persistence upgrade
failure and geometry-conflict rollback, and clean retry advancing exactly one
generation. Nine valid-IO measurements also complete; the supplied Wasm is
229,074 bytes. The final command passes again after adding retained diagnostics.
A separate controlled Cargo/CLI dispatch confirms a failed runtime command's
status 23 is reported by Make and its diagnostics/hashes remain retained; stale
inherited server selections are cleared. That dispatch is a substitute proof,
not installed runtime qualification. Its first malformed substitute invocation
is retained and excluded from the passing result.

The root manifest/lock selection changed externally during this batch from Host
0.8.10 to 0.9.0. The selected filesystem/artifact features remain empty; all 37
packaged Rust files match exact Host release
`715854b47b888eefb6fc0fca76d9c99f55099150` and are unchanged from 0.8.10.
Both independent runtime manifest/lock inputs stay byte-identical. Passing
focused checks include all 36 Rust adapter tests, strict Rust 1.99.0
example/test Clippy, Rust 1.88.0 example admission and strict independent-runner
Clippy. The preceding 0.31.12/Host evidence records earlier provisional inputs
and is not relabelled as this new selection.

`target/qualification/0.32.0-shared0200/` retains before/current hashes, locked
metadata, Host source inventory and command logs. Snapshot, actual declaration
and inheritance admission, pin/metadata fixtures, tool/evidence fixtures,
consumer hook isolation, simulated release runner/adapters, formatting and
tooling lint pass. Tool, hook and pin fixtures also pass under genuine Linux-built
Bash 3.2. The new hook cases isolate Cargo's home as well as PATH, admitting
checkout-local prepared formatters and preserving the index/source on missing
or wrong versions. Initial fixture isolation failures are retained; global
formatter discovery and missing fixture `cargo-fmt` shims were corrected without
changing product formatting. Documentation admission passed 239 local references
across 37 documents before this record was added. The real staged payload remains
empty; raw index hashes changed later, so byte-identical real-index preservation
is not claimed. No commits, tags, pushes or publication occurred.

Testkit's published owner setup/check/managed launch passes all three native hosts
in [exact 0.25.4 CI](https://github.com/dragginzgame/ic-testkit/actions/runs/37901828971).
Memory now has separate Linux/Intel macOS/Apple Silicon runtime CI jobs for this
consumer pairing; these uncommitted jobs have not run remotely. Shared 0.2.0's
[exact CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37916384666)
passes Linux portable/lint jobs with both macOS jobs queued at inspection. Memory
0.31.11's [CI](https://github.com/dragginzgame/ic-memory/actions/runs/37910600351)
passes Linux native/MSRV/lint, with Apple Silicon native running and the other
three macOS jobs queued; it cannot qualify these pending edits. Full Memory gates,
package/release qualification and native consumer macOS execution remain unrun.
The pending candidate is 0.32.0 because the IC setup requires reinstall and the
qualification URL contract changes; manifests are not bumped by note selection.

### Final Shared Tooling 0.2.1 selection

Shared 0.2.1 `06b2e22f6bd213f1a590eb2a8797aee34c42dd69` became available
while this batch was finishing. The final 88-file snapshot adopts that committed
release through the canonical exporter from a disposable clean local clone:
the sibling already contained unrelated new dirty work, which was not copied.
No new commits were created. The preceding 0.2.0 record and logs keep their
original input identity.

The reviewed follow-up fixes [Shared #87](https://github.com/dragginzgame/shared-tooling/issues/87):
installation and offline version admission now consume a final pin row without
a newline. All three simulated hosts cover installation of that last tool and
refusal of its wrong version even with a matching modified receipt. The canonical
IC fixtures pass under Bash 5 and genuine Bash 3.2. Actual installed five-tool
and Testkit server offline checks, snapshot/declaration admission, formatting and
tooling lint pass. The actual managed Memory runtime qualification also passes
again with the final snapshot hash retained in its attempt inputs. Root manifests,
both locks, selected tool versions and server bytes stay unchanged by this
follow-up. Evidence is in `target/qualification/0.32.0-shared0201/`.

The final documentation check covers 258 local references across 39 documents.
The [exact Shared 0.2.1 CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37918240103)
is queued at inspection, with no completed native acceptance attributed to it.
Memory's newly configured native runtime jobs and complete release gates still
require their own execution after delivery; no dirty upstream work is part of
this consumer export.

## 0.33.0 fleet retirement and CI installer publication

On released 0.32.0 `cc725ef15585d9dd34eb8531bc1620acf195b55f`, the maintainer
selected full fleet-reporter retirement for [#34](https://github.com/dragginzgame/ic-memory/issues/34).
The two copies and manifest records are deleted: cloc 2.10 measures 398 code
lines (423 physical lines). Shared Tooling remains the implementation owner.
Local root/independent-workspace reports, tool fixtures and the omitted optional
report's owner diagnostic pass; Memory never dispatches a sibling implicitly.

The 86-file snapshot selects committed Shared 0.2.2
`ee48bb37c98c771e77b92fd891f0757d8c1c8b99`, adopting [Shared #88](https://github.com/dragginzgame/shared-tooling/issues/88).
Canonical installer fixtures pass with Bash 5 and genuine Linux-built Bash 3.2,
including late-directory refusal with candidate retention, late-symlink target
preservation and missing-Perl refusal. This uses substituted downloads/hosts,
not native macOS execution. Snapshot, consumer tool/evidence fixtures, declaration
admission, formatting, tooling lint and local documentation checks pass. Evidence:
`target/qualification/0.33.0-fleet-shared0202/`. No dependency/tool selection,
product source, build evidence or Git history changes. The minor candidate
reflects removal of the local fleet-report offering; package versions stay at
the released value. New consumer native CI and complete gates remain unrun.

## 0.33.1 untouched metadata rollback

On released 0.33.0 `1cbecf601fd24afc64d1fcfccb29609537be36be`, a focused
Linux regression for [#39](https://github.com/dragginzgame/ic-memory/issues/39)
reproduces untouched-file replacement after failure before the first metadata
publication. The original rollback changes file identities and broadens `0600`
permissions to `0644`. The correction skips publication when current bytes
already equal the backup; changed-file ownership and restoration-error handling
remain in the existing consumer transaction.

All 37 release-adapter tests and strict example/test Clippy pass with locked,
offline Rust 1.99.0 and the existing Host filesystem/artifact 0.9.1 selection.
Their registry VCS identities match Host release
`4a016053525fa710bc13f3aedbe85a471b78f6ed`. Formatting checks cover both
maintained workspaces. Evidence, including the failed regression and initial
zero-test filter attempt, is retained in
`target/qualification/0.33.1-untouched-rollback/`; the failed fixture remains
`/tmp/ic-memory-tooling-14-0`. These tests substitute release effects; no fixture
commit/tag/push or live release occurs. The compatible 0.33.1 correction is local
and uncommitted. Both dependency graphs, manifests, tool selections, snapshot and
product contracts stay unchanged. Full gates and new native CI remain unrun.

## 0.33.1 Shared 0.2.3 and Host 0.9.2

The maintainer's updated root lock selects Host filesystem/artifact 0.9.2;
the prior 0.9.1 rollback evidence above retains its original identity. Their
37 Rust sources are unchanged from 0.9.1. All 74 files across the two cached
copies match Host release `c5decaefd17809829bfa969966729d672f609c49`, including
registry VCS identity. Locked/offline metadata confirms empty feature sets.
All 37 adapter tests, strict Rust 1.99.0 example/test Clippy, declarations and
both-workspace formatting pass with the new selection.

The exact 86-file snapshot advances to clean, committed Shared 0.2.3
`ac4549c5ebde497f7db0da5d05d32835112e51de`. Only four selected documentation
files change, covering optional installer companions and CI queue diagnosis.
All selected executable bytes and modes remain unchanged; the optional CI
installer suite and its extra wrappers are not added. Canonical refresh and
snapshot admission pass. Evidence: `target/qualification/0.33.1-shared0203-host092/`.
No further dependency selection or source repair was needed. The independent
workspace inputs and both manifests are unchanged during this review.

[Host CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37924013633)
passes Linux native/MSRV; [Shared CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37925303425)
passes Linux portable/lint/security. Both macOS pairs remain queued at inspection.
The optional, manually dispatched Shared Cargo-install workflow has no matching
run and was not invoked during this review.
This review does not run full gates or qualify the dirty consumer batch on macOS.

## 0.33.1 note content and release identity

The consumer regression for [#40](https://github.com/dragginzgame/ic-memory/issues/40)
reproduces an `empty changelog entry` refusal before metadata preparation on
released 0.33.0 `1cbecf601fd24afc64d1fcfccb29609537be36be`, with the earlier
pending fixes and selected Host 0.9.2 retained. Removing the prose-only guard
lets correctly numbered root and detail entries pass simulated preparation,
commit qualification and dry-run publication admission. Historical bytes,
including a missing final newline, remain unchanged. Existing version/date,
duplicate identity, actual-heading replacement, source/index/payload and receipt
checks stay active. No function or type is removed.

All 38 adapter tests, strict Rust 1.99.0 example/test Clippy and both-workspace
formatting pass with the locked/offline selection. Evidence and the failed
regression remain in `target/qualification/0.33.1-note-content/` and
`/tmp/ic-memory-tooling-221-0`. Both manifests/locks and the 86-file snapshot
are unchanged during this correction. These are local Linux checks with
substituted release effects, not native macOS or live release qualification.
The compatible 0.33.1 batch remains uncommitted; full gates remain unrun.


## 0.33.2 Shared 0.2.4 fixture companions

The canonical exporter refreshes Memory's unchanged 86-file selection from a
clean isolated checkout of committed Shared Tooling 0.2.4
`ffbf665b8481c36b2d9f4d988abec557c3485fa6`. Eight selected fixture headers and
adoption guidance change; all selected bytes and executable modes match that
commit. Every declared companion is already selected. Production helpers,
hooks, pins, package manifests and both lockfiles remain unchanged. Dirty
upstream hook/installer edits are excluded from the committed snapshot.
[Shared #73](https://github.com/dragginzgame/shared-tooling/issues/73).

A focused initial export selecting the Cargo-metadata fixture without its version
reader refuses before creating selected files or a manifest, preserving unrelated
bytes. Two earlier attempts stopped at required verifier prerequisites; those
inconclusive logs are retained separately. No fixture commits, tags or pushes
are created, and the upstream integration suite is not copied or invoked.
Actual consumer snapshot, pin declarations/fixtures, tool fixtures, formatter
prerequisites, both-workspace formatting and local documentation links pass.
The broader documentation check also repairs two composed-host links left at
the former package location in existing reports; their historical claims remain
unchanged. The failed link-check output is retained.
Evidence: `target/qualification/0.33.2-shared0204/`.

This is local compatible adoption for pending 0.33.2, with no full gate or release
effects. Upstream exact-source Linux portable and lint/security jobs pass;
macOS jobs remain queued in [Shared CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37927306875).
Memory 0.33.1 is delivered at `17372c0d1d71fc3516f415ff8c30057c4441bc31`,
matching remote main and the peeled annotated tag. Its Linux native/MSRV,
installed runtime and lint jobs pass, while six macOS jobs remain queued in
[Memory CI](https://github.com/dragginzgame/ic-memory/actions/runs/37931572683).
These observations confirm delivery of [#39](https://github.com/dragginzgame/ic-memory/issues/39)
and [#40](https://github.com/dragginzgame/ic-memory/issues/40), without claiming
complete native acceptance or qualification of the new dirty adoption.


## 0.33.2 Shared 0.2.5 literal hook paths

The same compatible pending 0.33.2 draft now selects committed Shared Tooling
0.2.5 `04e07b4bf54e7aeb03eb7804a845cee27b7305df`. The canonical refresh uses
an isolated clean checkout and retains the existing 86-file roster. All selected
bytes and modes match that revision; manifests, both locks, the real Git index
and local Git configuration remain byte-identical to the incoming state.
The adopted production changes are the shared hook and installer; their linked
rule also changes. The preceding 0.2.4 evidence retains its original identity.
[Shared #89](https://github.com/dragginzgame/shared-tooling/issues/89).

Memory's new consumer regression first fails with the old installer in a
newline-ending checkout (`cd` loses the final newline). The failed fixture remains
`/tmp/ic-memory-hooks.TidTec`; the before log is retained. With the adopted files,
actual repeated setup and hook execution pass, formatting selected sources in
both maintained workspaces with the real cargo-sort/rustfmt targets. Neither
lockfile nor build directory is created in the fixture. Existing partial-staging,
failed-formatter, unrelated-edit and prepared-local-tool cases still pass.

The exact owner's focused hook suite also passes on local Linux Bash 5, including
literal conflicting local/inherited settings, failed Git observations and preserved
configuration. Consumer snapshot/pin admission, formatting, Actionlint, ShellCheck
and local documentation links pass. Logs and input hashes are retained in
`target/qualification/0.33.2-shared0205/`. Fixture effects reuse existing source
commits; no new commits, tags or pushes, actual clone activation, full gates or
release effects are performed.

[Exact upstream CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37937705371)
has successful Linux portable and lint/security jobs; both macOS jobs are queued.
The older native local-tool fixture failure remains tracked in
[Shared #85](https://github.com/dragginzgame/shared-tooling/issues/85), and the separate
qualification helper's newline-root gap remains in
[Shared #90](https://github.com/dragginzgame/shared-tooling/issues/90). Memory does
not select that helper. Local Linux evidence is not native macOS acceptance.


## 0.33.2 Host 0.9.3 review

The incoming maintainer-selected root lock uses `ic-host-fs` and
`ic-host-artifacts` 0.9.3, released at
`545e7236b91d84e190c80931b784f72cc4fafb11`. Their Rust source is byte-identical
to 0.9.2; this release adopts Shared Tooling's already-reviewed hook correction.
Both cached registry copies match the exact released source/VCS identity
(74 source files checked), and cached archive hashes match the selected lock.
Locked/offline metadata confirms empty feature sets for both packages. All
38 Memory release-adapter tests and strict Rust 1.99.0 example/test Clippy pass.
Manifests, both selected locks and the snapshot remain unchanged during review.
Evidence: `target/qualification/0.33.2-host093/`.

Remote main and the peeled annotated `v0.9.3` tag match the reviewed commit.
[Exact Host CI](https://github.com/dragginzgame/ic-host-tooling/actions/runs/37938930037)
has successful Linux native and MSRV jobs; both macOS jobs remain queued.
No new applicable Host defect or source simplification is identified in this
release. The consumer qualification stays in pending 0.33.2; no sibling edits,
source repairs, dependency reselection, full gates or Git/release effects occur.


## 0.33.2 publication tag identity

The released 0.33.1 publication adapter checks the tag initially, but omits it
from late admission and completion checks. The focused regression first shows
a successful adapter result after substituted Cargo removes the tag. The failed
fixture remains `/tmp/ic-memory-tooling-225-1`; the before log is retained.
[#41](https://github.com/dragginzgame/ic-memory/issues/41).

The existing tag admission now returns its observed annotated object identity.
Publication holds that identity and its selected release tuple through late
admission and completion. The regression covers pre-dispatch tag removal with
zero Cargo publication dispatches, plus removal, retargeting and annotation
replacement during Cargo with exactly one dispatch and unchanged receipt bytes.
The failure never replays external effects or regenerates qualification evidence;
a post-dispatch error cannot establish that the external publication failed.

All 39 adapter tests, strict Rust 1.99.0 example/test Clippy, both-workspace
formatting and local documentation links pass with the incoming Host 0.9.3 lock.
Input hashes confirm unchanged manifests, both locks and the selected snapshot.
Logs: `target/qualification/0.33.2-publication-tag/`. This compatible local fix
extends pending 0.33.2. Tests substitute Git/publication effects; no real commits,
tags, pushes or publication, full gates, or native macOS acceptance are claimed.


## 0.33.3 Shared 0.2.6 release routing

The canonical exporter selects clean committed Shared Tooling 0.2.6
`ce13a5314916891fd239d9b199b4a91b04775054`, adding only `make/release.mk`
to Memory's existing snapshot (87 files). All selected bytes/modes match that
commit; manifests, both locks, real index and local Git configuration remain
byte-identical to the incoming state. Production runner/hook bytes are unchanged.
[#42](https://github.com/dragginzgame/ic-memory/issues/42),
[Shared #91](https://github.com/dragginzgame/shared-tooling/issues/91).

The actual consumer fixture verifies standard arguments, selected destinations,
resume, conflicting goals, runner failure propagation and unsupported delivery
refusal with substituted release effects. All four entrypoints retain forced
cache preparation even for conflicting caller settings. The isolated Makefile
copies and actual hook index export include the new shared input. Default Make
still prints help. Existing two-workspace formatting, partial-stage refusal,
failed formatter isolation, newline-root and prepared-tool cases pass.

Snapshot/pin admission, formatting, Actionlint, ShellCheck, documentation links
and exact-owner release/hook suites pass on local Linux Bash 5. Evidence:
`target/qualification/0.33.3-shared0206/`. The root-only formatter include is not
selected, preserving the independent workspace and explicit compiler coverage.
The root Makefile loses eight net lines; the new shared include adds 19 vendored
lines. No functions/types or supported release targets are removed. Full gates,
fixture commits/tags/pushes, actual hook activation and release effects are unrun.

Memory 0.33.2 is delivered at `f9095c96077c01c7f63b6d8ef9e744e800ca445e`,
matching remote main and the peeled annotated tag. Its Linux native/MSRV,
installed runtime and lint jobs pass; six macOS jobs remain queued in
[exact-release CI](https://github.com/dragginzgame/ic-memory/actions/runs/37944024295).
[Shared 0.2.6 CI](https://github.com/dragginzgame/shared-tooling/actions/runs/37944389294)
has passed Linux portable and lint/security; both macOS jobs remain queued.
These do not qualify Memory's dirty compatible 0.33.3 adoption.


## 0.33.3 literal bootstrap target

The released launcher strips terminal newlines from Cargo's selected target path
when recovering a manifest/lockfile mismatch. The focused fixture first creates
a trimmed sibling bootstrap directory and fails helper dispatch. Before evidence
remains in `/tmp/ic-memory-release-adapters.nqy3zQ` and
`target/qualification/0.33.3-bootstrap-target/before.log`.
[#43](https://github.com/dragginzgame/ic-memory/issues/43).

The launcher now preserves the complete parsed path, removing only the output
record terminator. Its sentinel is appended only after the full metadata pipeline
succeeds. Actual consumer Make/launcher checks pass with a target ending in two
newlines, unchanged manifest/lock bytes and no trimmed destination. A parser
printing a plausible path before failing with status 23 stops without cache or
helper dispatch. Existing offline/cache preparation and Shared release-routing
checks still pass; Actionlint and ShellCheck pass. Logs and input hashes:
`target/qualification/0.33.3-bootstrap-target/`.

This compatible fix extends the existing uncommitted 0.33.3 draft. Manifests,
both selected locks, snapshot and pending Makefile routing remain unchanged.
Cargo/parser and release effects are substituted; native macOS acceptance and
full gates are unrun. No function/type is removed or real Git/release effect used.

## 0.33.3 checkout-local release routing

Review of the pending Shared 0.2.6 adoption reproduces a changed routing boundary:
an inherited `SHARED_TOOLING_ROOT` selects an external substitute runner instead
of the one copied into the consumer fixture. It exits 23, and Make refuses only
after that external dispatch. The failed fixture is retained at
`/tmp/ic-memory-release-adapters.hKPcgP`; its log is
`target/qualification/0.33.3-release-root/before.log`.
[#42](https://github.com/dragginzgame/ic-memory/issues/42).

Memory now binds only its four standard release targets to `$(CURDIR)` through a
target-specific override/export. The shared include still owns the recipes, and
other tooling-root selections remain unchanged. All four targets select the
local substitute runner under both environment and command-line external roots,
with exact increment/resume and destination forwarding. The complete consumer
adapter fixture also passes when its parent Make exports an external root; that
sentinel never executes. This restores the released relative-recipe behavior.

Local Linux Make checks, the actual two-workspace hook suite, Actionlint,
ShellCheck and the 87-file snapshot verifier pass. Manifests, both selected locks
(including the incoming Host 0.9.4 selection), snapshot and real index retain
their incoming bytes. Input hashes and logs are retained under
`target/qualification/0.33.3-release-root/`. No function/type is removed. Native
macOS, full gates, real release and Git delivery effects remain unrun.

Shared Tooling committed 0.2.7 at
`47d6ae6488b8007323fa7c2e22a6efa11d77ae63` during this pass. Its generic smoke
fixture now binds its disposable root ([Shared #7](https://github.com/dragginzgame/shared-tooling/issues/7)),
and its new Make companion rejects unsafe execution modes before recipes.
An isolated candidate using those exact committed files passes ordinary local
release dispatch, but both external-root controls execute the external substitute
admission probe before refusing with status 2. Parse-time admission precedes
Memory's target-specific binding. The snapshot therefore remains 0.2.6;
the new companion is not adopted or patched downstream. Bind the admission probe
to its selected reviewed companion at the source owner, then qualify a committed
fix before refreshing Memory. [Shared #30](https://github.com/dragginzgame/shared-tooling/issues/30).
Candidate logs and sentinel records are retained alongside the local fix evidence.

This bounded review applies the current Shared `audits/code-hygiene.md` method
to release Make dispatch and fixture isolation, against Memory release
`f9095c96077c01c7f63b6d8ef9e744e800ca445e` plus the pending adoption. Runtime
storage, installed IC behavior and independent Host dependency qualification are
outside this tooling repair. The local routing defect is corrected; upstream
admission remains an evidenced adoption gap. No sibling source is changed.

## 0.33.4 Shared 0.2.8 Make admission

The clean canonical exporter refreshes Memory's selection to committed Shared
0.2.8 `b2646cde9abbc8861857a4379c683a0c19eba43e`, explicitly adding
`make/execution.mk` (88 files). Uncommitted Shared 0.3.0 changes are excluded.
Actual consumer Make/index fixtures now include the companion and probe.
[#42](https://github.com/dragginzgame/ic-memory/issues/42).

The consumer adapter and two-workspace hook suites pass on Linux with Bash 5.2 /
GNU Make 4.3 and genuine Bash 3.2.57 / GNU Make 3.81. All four release targets
refuse direct/inherited unsafe modes before substitute-runner dispatch. External
root selections never invoke an unselected admission probe or runner; parallel
recursive Make with two Makefiles preserves selected routing/cache settings.
Exact-source upstream release fixtures pass on both profiles. Snapshot/pins,
formatting, Actionlint and ShellCheck pass. Real effects are substituted; no
fixture commit/tag/push, actual hook activation or full gate runs.

GNU Make 3.81 first refuses Memory's combined target-specific `override export`
syntax with `multiple target patterns`, even for `help`. Reclassifying caller
values before portable target exports restores parsing while preserving forced
release-only root/cache settings under conflicting caller values.
[#45](https://github.com/dragginzgame/ic-memory/issues/45).

A separate retained observation confirms the known command-line `MAKEFLAGS`
override gap in the unchanged Shared guard on both profiles. A normal failing
substitute runner gives status 2; `-i` alone refuses before dispatch. With `-i`
and `MAKEFLAGS=`, it dispatches and gives false success (status 0). Replaced
flags also admit dispatch under the tested `-n`, `-t` and `-q` invocations.
Existing passing fixtures do not cover this boundary. No consumer flag parser or
vendored repair is added. Canonical repair and native acceptance remain under
[Shared #30](https://github.com/dragginzgame/shared-tooling/issues/30).

Evidence, copied inputs, binary identities and per-profile logs are retained in
`target/qualification/0.33.4-shared0208/`. The real index/configuration, manifests
and both incoming locks remain byte-identical. This is uncommitted compatible
0.33.4 work; Linux portability checks do not qualify native macOS or a live release.

## 0.33.4 Host 0.9.7 review

Preserve the incoming maintainer-selected root lock's `ic-host-fs` and
`ic-host-artifacts` 0.9.7. Their registry VCS identity is Host release
`ca62e661918db2f4320743b9042a4993a5fff2aa`; cached archive SHA-256 values match
the selected lock. Neither selected crate has file changes from Host 0.9.4
`4e3daebd5df07c6449279535668436024a45c02b`. Host response-streaming changes
belong to the unselected tools crate.

All 39 repo-tool tests pass locked/offline on Rust 1.99 and MSRV 1.88; strict
selected example/test Clippy passes on 1.99. Cheap locked/offline metadata admits
both maintained workspaces. Dependency requirements/features, both lock bytes
and compiler selection remain unchanged. Separate evidence:
`target/qualification/0.33.4-host097/`. Native macOS and full gates are unrun.

## 0.33.4 Host 0.10 adoption

The maintainer's subsequent root manifest/lock selection advances only
`ic-host-fs` and `ic-host-artifacts` to published 0.10.0. Registry VCS identity,
remote main and peeled annotated `v0.10.0` agree on
`98562bea26a98993d93b80ed908bea4876c32a91`; archive checksums match the lock.
The earlier 0.9.7 results remain bound to their original selection.
[#46](https://github.com/dragginzgame/ic-memory/issues/46).

Initial compilation fails at the retired typed-writer name and two-argument
streamed calls. Receipt and archive writers now use Host's current three-argument
`write_with`, with one local replacement/permission selection preserving prior
defaults. Byte writers retain the complete new error through `?`, as do streamed
receipts and archives. Substitutes reuse those production boundaries.

The first test run exposes an obsolete direct-I/O assertion for archive-copy
failure. Its native I/O cause now remains inside `NamedWriteError::Producer`;
the corrected check verifies its kind, OS identity, absent cleanup failure and
unpublished destination. Existing native publication checks also assert typed
byte-write refusal and rejected partial stream production, with prior bytes and
directory entries preserved. JSON failure and recovery/rollback checks remain.

All 39 repo-tool tests pass locked/offline on Rust 1.99 and MSRV 1.88; strict
selected example/test Clippy passes on 1.99. Both workspace metadata projections,
formatting and focused tooling admission pass. Evidence, including both failed
attempts and corrected logs, stays in `target/qualification/0.33.4-host0100/`.
Incoming manifests/locks, shared snapshot and real index/configuration remain
byte-identical. No functions, methods or types are removed; no compatibility
alias, full gate, real release or Git delivery effect is introduced. The public
library API, storage and release recovery contracts keep this in compatible 0.33.4.

Host's parent-creation race correction is still uncommitted, pending 0.10.1
([Host #43](https://github.com/dragginzgame/ic-host-tooling/issues/43)); it is
excluded from registry qualification. Native macOS and matching delivery CI
remain outstanding for this working tree.

## 0.33.4 Shared snapshot version metadata

The new upstream main commit `f77fcb1f623c2e5f14b3fbe96ef68d24e0c20771`
adds source-version recording and fleet-report changes. Memory's clean canonical
refresh keeps the same 88-file selection, changing only the selected verifier,
snapshot-consumption guidance and manifest. Fleet reporters/fixtures remain
upstream-owned. The source's committed `VERSION` is 0.2.8 despite the 0.2.9
commit label and pending ledger; the exported annotation accurately records
0.2.8 and the new exact revision. Neither label proves a release/tag.

Retained disposable checks pass on Linux Bash 5 and genuine Bash 3.2: current
annotations verify, repeated refresh is byte-identical, an unannotated snapshot
verifies as unrecorded and refresh gains the exact source annotation. Duplicate,
noncanonical and extra-field annotations refuse in both verifier/exporter before
changing any selected files. Fixtures reuse existing history without commits,
tags or pushes. Evidence, including the first fixture-setup refusal, is under
`target/qualification/0.33.4-shared-f77fcb1/`.

The Make guard, release include, probe and smoke checker have no changes from
the previously qualified 0.2.8 inputs. Their existing consumer wiring/portability
evidence remains applicable; the hidden-`MAKEFLAGS` gap remains open under
[Shared #30](https://github.com/dragginzgame/shared-tooling/issues/30).
Snapshot/pins, formatting, tooling lint and documentation links pass. Manifests,
both locks, all incoming consumer Rust/Make/fixture files and real index/config
remain byte-identical. This extends the same uncommitted compatible 0.33.4 draft.
No named functions/methods/types are removed, fleet tooling adopted, full gates
or real release effects run. Matching native CI remains outstanding.

## 0.34.0 Shared Tooling and Make admission

Reviewed committed Shared **0.2.10** at
`43a0dc46cdc3c77e70a68e192561642ed50a3e0f` is exported through the canonical
refresh helper from a clean exact-revision checkout. The 89-file manifest adds
`scripts/ci/run-formatting.sh`; its hash and mode are verified. Dirty local
shared policy remains separately authorized and is not export provenance.

Memory keeps exact formatter prerequisites, compiler selection and both workspace
commands. The wrapper captures their combined output once, emits one success
line, preserves failing status and retains full logs. Actual consumer hook tests
copy the helper into their disposable selection; product failure selection and
archive round-trip checks include `formatting.*`. No fixture commit or real staged-entry
change was used.

The consumer parse guard admits `MFLAGS` as well as `MAKEFLAGS` before recipes.
Real Make tests substitute all release effects and reject four entrypoints under
nine direct/inherited unsafe modes, with original, empty and harmless explicit
`MAKEFLAGS` values. Normal direct/recursive dispatch and failing helper validation
remain qualified. Canonical shared repair remains upstream in
[Shared #30](https://github.com/dragginzgame/shared-tooling/issues/30).
[#47](https://github.com/dragginzgame/ic-memory/issues/47) owns this consumer fix.

Evidence under `target/qualification/0.34.0-tooling/` includes modern admission,
actual two-workspace hook and evidence tests. `portable-2.log` qualifies those
three targets with Linux Bash 3.2.57 and GNU Make 3.81. The earlier `portable.log`
failed before admission because Bash 3.2 rejects an empty array expansion under
nounset; the new fixture uses a nonempty argument vector and retains that failure.
It is not native macOS evidence. Snapshot/format/workflow/shell checks remain
focused; no maintainer release target or full validation gate was executed.

The existing pinned artifact download action is supplied a read-only token,
explicit repository/run and exact uploader artifact IDs. Digest mismatch is an
error, followed by the existing byte/mode/link comparison. This implements
[Shared #93](https://github.com/dragginzgame/shared-tooling/issues/93)'s authenticated
readback boundary. Local workflow lint proves syntax, not hosted availability.
Hosted acceptance of the working snapshot, wrapper, guard and readback remains
pending; the green 0.33.4 run is evidence only for released 0.33.4.

## 0.34.1 canonical Make admission

Prepared on delivered 0.34.0, `958080df899ebfa7bb9c2d4c93bb8664bd23575d`.
The canonical exporter ran from a clean disposable checkout of reviewed Shared
**0.2.11**, `83efac446348dea024798a331d77933b24b429dc`, whose
[all-host upstream run](https://github.com/dragginzgame/shared-tooling/actions/runs/38034912323)
passed. The unchanged 89-file selection now records that revision/version;
Memory's duplicate flag parser is removed. Active live-local shared policy
remains separately authorized; subsequent 0.2.12/0.2.13 source is not attributed
to this export. [#42](https://github.com/dragginzgame/ic-memory/issues/42),
[#47](https://github.com/dragginzgame/ic-memory/issues/47),
[#48](https://github.com/dragginzgame/ic-memory/issues/48).

Evidence under `target/qualification/0.34.1-shared0211/`:

- `focused-modern.log`: snapshot verification, actual Make dispatch with
  substitute release effects, two-workspace hook isolation, evidence-retention
  round trips and formatting pass. Unsafe direct/inherited modes still refuse
  under original, cleared and replaced `MAKEFLAGS`; new cases refuse erased
  command-line/Makefile `MFLAGS`. Normal recursive/parallel dispatch and failure
  propagation pass without the local parser.
- `focused-portable.log`: the same adapter, hook and evidence fixtures pass with
  GNU Make 3.81 and Bash 3.2.57 built on Linux. This proves tool portability on
  Linux, not native macOS execution.
- `repo-tool-host0102.log`: all 41 Rust release-tool tests pass with the incoming
  selected Host filesystem/artifact 0.10.2 lock. `clippy-host0102.log` and
  `msrv-host0102.log` record strict example Clippy and Rust 1.88 compilation.
- `pins-lint.log`: declaration/metadata fixtures, actionlint and ShellCheck pass.

The root manifest, incoming selected root lock and independent runtime lock match
captured inputs. No fixture commit/tag/push/publication, real-index staging, full
validation, package/release command, installed runtime or sibling source edit
was invoked. There are no removed Rust functions, methods or types in this
adoption; the deleted local parser was a Make variable/expression.

This is uncommitted compatible 0.34.1 preparation. Its own native hosted
acceptance requires delivered source. The completed 0.33.4 all-host run qualifies
older fixes only; 0.34.0 CI remains separate from these dirty changes. Coordinated
downstream composition is still tracked by #44 rather than this tooling evidence.

## 0.34.1 Shared 0.2.13 and Host 0.11.0

This supersedes the earlier 0.2.11/Host 0.10.2 preparation above without relabelling
its evidence. The maintainer requests current Shared Tooling and IC Host. The
incoming root catalog and lock already select Host filesystem/artifact 0.11.0;
Memory uses neither Host's process crate nor its changed process-limit API.
Its existing durable writer retains original, cleanup and publication-state
errors, including Host's improved cleanup diagnostics. Canister API and durable
formats are unchanged; the candidate remains compatible **0.34.1**.

The canonical exporter selects committed Shared **0.2.13**,
`5864f468d39f8f9d1bd26fca1afe0e20f25f1b5e`, through a clean exact-revision disposable
checkout. The first refresh refused the sibling after new concurrent dirty edits;
those bytes remain excluded from the 89-file export. Live local policy still
applies separately under the maintainer's existing exception. The new verifier
refuses supplied/resolved line-break directory paths and accepts ordinary aliases.
The selected Cargo installer reports exact missing/invalid executable identities.

Coherent admitted release preflight prepares the existing host-tool selection,
then checks host/formatter admission offline before the gate. Interrupted
metadata recovery checks the retained selection without replaying setup.
Standalone validation separates prerequisite checks from builds/tests, so parallel
Make cannot race them. Runtime admission names its existing explicit setup target
and retains failure status. Development validation packages working-tree edits;
release validation and the separate maintainer package target still require clean
source. No new dependency selector or installer is introduced.

Evidence: `target/qualification/0.34.1-shared0213-host0110/`.

- `validate.log`: full `make validate` passes, including shared integrity and
  tooling/runner/hook fixtures, strict Clippy, library/public/compile-fail/doc
  tests, rustdoc, Wasm tests and five raw-size budgets, development package
  verification and Rust 1.88 all-target compilation.
- `repo-tool-2.log`: all 42 focused Rust tooling tests pass against Host 0.11.0.
  The new rejection/ordering case preserves source metadata on setup/check
  failure before validation or version mutation.
- `adapters-runtime-admission.log`: actual Make-to-adapter fixtures qualify
  early missing-tool refusal under parallel Make; existing selected Testkit CLI
  and PocketIC server pass offline admission. This does not run installed IO or
  coordinated downstream lifecycle qualification.
- `portable.log`: release-adapter, hook and retained-evidence fixtures pass under
  genuine GNU Make 3.81/Bash 3.2.57 on Linux; this is not native macOS acceptance.
- `path-admission.log`: supplied and resolved invalid directory aliases refuse
  before manifest selection; a normal spaced alias verifies all 89 files.
- `root-metadata.json`, `runtime-metadata.json`: both cheap locked graph checks
  pass. The independent runtime graph does not contain Host and needs no update.
  `lint.log` records passing actionlint and ShellCheck.

The root manifest, root lock and both independent manifest/lock inputs match
captured incoming bytes. Compiler, formatter, CLI/server and stable-structures
selections remain unchanged. No commit, push, release, publication, fixture Git
history creation or sibling edit was performed. Qualification documentation was
updated after the successful code gate; it adds no executable changes.
Upstream new-source CI and native Memory acceptance remain independent of this
local Linux result. [#42](https://github.com/dragginzgame/ic-memory/issues/42),
[#48](https://github.com/dragginzgame/ic-memory/issues/48),
[Shared #95](https://github.com/dragginzgame/shared-tooling/issues/95),
[Shared #96](https://github.com/dragginzgame/shared-tooling/issues/96).


## 0.34.2 Shared 0.2.14 CI log evidence

Continue from delivered Memory **0.34.1**,
`fde94900b37ac91f47e5c18c97065fbe323a7e40`, with a clean incoming tree. The
canonical exporter selects committed Shared **0.2.14**,
`fd11692f31e7dfd44dcc2ca56634eaeab3569825`, from a clean exact-revision disposable
checkout. Its 90-file selection adds the upstream CI-helper fixture alongside
its existing production companion. Shared's concurrent uncommitted 0.3.0 toolset
work is excluded; live local policy still applies under the maintainer exception.

The adopted helper refuses unavailable failure evidence, preserves CLI status and
retains partial logs/retrieval errors. Completed successful, neutral or skipped
runs can have no failed-step logs. Memory includes the selected upstream fixture
in its existing tooling gate rather than duplicating the helper or its tests.
The canister API, durable format and setup targets are unchanged, so the pending
candidate is compatible **0.34.2**.

Evidence: `target/qualification/0.34.2-shared0214/`.

- `refresh.log`: canonical export and exact 90-file verification pass.
- `gh-ci.log`, `gh-ci-bash32.log`: selection, missing/partial-log refusal,
  legitimate empty output, retrieval-status preservation and retained observation
  checks pass on Bash 5 and genuine Bash 3.2.57 on Linux. CLI/Git effects are
  substitutes; this is not native macOS acceptance.
- `gh-ci-live-success.log`: the actual read-only helper accepts legitimately
  absent failed-step logs for successful Memory 0.34.0 CI run 38038532089 at
  `958080df899ebfa7bb9c2d4c93bb8664bd23575d`. This does not exercise a live failed
  log fetch or qualify dirty source remotely.
- `validate.log`: full `make validate` passes, including strict Clippy, Rust
  tooling/public/compile-fail/doc tests, hook/release/evidence fixtures, rustdoc,
  Wasm tests and size budgets, development packaging and Rust 1.88 all targets.
- `lint.log`: workflow and shell lint pass. Updated document links and diff
  whitespace are checked after qualification documentation is added.

Captured root/independent manifest and lock inputs remain byte-identical. Host
0.11.0, compiler, formatter, stable-structures and independent runtime/Testkit
selections are unchanged. No staging, commit, push, release, publication, runtime
provisioning, fixture Git history or sibling source edit occurred. This final
record is documentation added after the passing code gate, not a new executable
change. Matching-source native acceptance requires delivered Memory source.
[Shared #97](https://github.com/dragginzgame/shared-tooling/issues/97) owns the
upstream log repair; [Memory #50](https://github.com/dragginzgame/ic-memory/issues/50)
separately owns the pending complete-toolset adoption.


## 0.35.0 Shared 0.3.0 complete common tools

The requested tooling continuation carries the unreleased 0.34.2 CI-log fix into
pending **0.35.0**. The canonical exporter selects committed Shared **0.3.0**,
`88a73139a0f083344c41a6f6f4b5c3a8aca7dc1d`, from a clean disposable checkout.
The 90-file selection excludes the sibling's concurrent dirty fleet-helper work.
Live local policy remains separate under the maintainer exception. The earlier
0.34.2 record retains its original inputs rather than being relabelled.

Common setup/check now runs all twelve host, IC and Cargo tools sequentially.
Memory release preflight prepares the aggregate for coherent admitted source,
then checks it offline; interrupted metadata recovery does not replay setup.
Standalone validation checks the complete set before builds. CI uses local Cargo
tools rather than another global formatter installation, separates setup/check
commands, and retains each new check/setup log through the existing collector.
PocketIC setup stays explicit and outside the library gate.

The removed host-installer options and expanded common setup contract require a
minor release. Root and detail pending notes move together to 0.35.0; published
0.34 history, package versions and both dependency graphs are unchanged. There
is no canister API or durable-format change.

Evidence: `target/qualification/0.35.0-shared030/`.

- `refresh.log`: exact 90-file committed export verifies.
- `setup.log`: real local setup reuses prepared host/IC bundles and installs the
  selected Cargo executables. Cargo build/receipt evidence remains in `.tools/rust`.
- `tools-check.log`, `setup-reuse-offline.log`: complete offline admission and
  setup reuse pass; no new tool/version selection or compiler installation occurs.
- `runtime-admission.log`: existing selected Testkit CLI/server passes offline
  admission. No installed IO/upgrade or downstream composition run is claimed.
- `portable.log`: complete-host/aggregate ordering, release adapters and actual
  new-log archive round trips pass under genuine GNU Make 3.81/Bash 3.2.57 on
  Linux. Asset/Cargo/release effects in these fixtures are substitutes; they do
  not establish native macOS or live release acceptance.
- `evidence.log`: full and compact archives preserve new setup/check logs, byte
  identity, failure status and prior retry evidence.
- `validate.log`: full `make validate` passes, including all 42 Rust tooling tests,
  strict Clippy, maintained library/public/compile-fail/doc tests, rustdoc, hook/
  release/evidence fixtures, Wasm tests and five budgets, development packaging,
  and Rust 1.88 all-target compilation.
- `lint-2.log`, `fmt-check-2.log`: final workflow/shell lint and two-workspace
  formatting pass. Initial `lint.log` and `fmt-check.log` failures remain retained;
  the workflow redirection style and Rust formatting are corrected before gates.

Captured root and independent manifests/locks remain byte-identical. Host 0.11.0,
Rust 1.99 development/Rust 1.88 MSRV, formatter pins, stable-structures and Testkit/
PocketIC selections are preserved. No source staging, commit, tag, push, release,
publication, fixture Git history creation or sibling source edit occurs. Final
qualification prose is added after the passing code gate and checked separately.
Consumer delivery/native acceptance stays in [#50](https://github.com/dragginzgame/ic-memory/issues/50);
upstream toolset ownership is [Shared #98](https://github.com/dragginzgame/shared-tooling/issues/98).

## 0.35.1 Shared 0.3.1 setup preflight

On delivered Memory **0.35.0**, `13c664158e652bb7b42a26da3cac072aa1467141`,
the compatible pending **0.35.1** batch refreshes the existing 90-file selection
through the canonical exporter from committed Shared **0.3.1**,
`fa452afaa5012866eb1c20820dfa8038c106e7ec`. The source checkout is clean;
the consumer's working changes remain unstaged/uncommitted.

The existing IC and Rust installers supply silent read-only platform/catalog
and checkout-selected Rust/Cargo preflight before aggregate installation starts.
Host failures report tool/version/path/reason/repair details while authenticating
bytes before execution. No local replacement wrapper, new gate or tool pin is
added. Narrow setup commands, offline admission, formatter coverage and explicit
Testkit setup retain their existing owners.

Evidence is retained under `target/qualification/0.35.1-shared031/`:

- `refresh.log` verifies the exact committed export; `admission.log` checks all
  twelve installed common tools offline. `setup-reuse-offline.log` passes actual
  aggregate setup reuse with `CARGO_NET_OFFLINE=true`, including both preflights.
- `portable.log` passes aggregate ordering/refusal, host/IC/Rust installation,
  consumer release dispatch and actual archive round trips on genuine GNU Make
  3.81/Bash 3.2.57 on Linux. Preflight-owner fixtures prove unsupported/missing/
  unavailable prerequisites stop before installation or product extensions,
  and successful preflight preserves retained receipts/build evidence.
  Asset, Cargo and release effects in these fixtures are substitutes.
- `validate.log` passes full `make validate`: strict Clippy, 241 library tests,
  public/compile-fail/tooling/doctests, release/hook/evidence fixtures, rustdoc,
  Wasm checks/five size budgets, development packaging and Rust 1.88 all targets.
- `lint.log` passes workflow/ShellCheck lint. `docs-2.log` passes the seven
  changed documents' local references. The earlier `docs.log` retains an
  unavailable checker invocation; the corrected run uses the committed shared
  documentation checker rather than introducing another local helper.
- `source-head.txt`, `qualified.diff` and `inputs.sha256` bind executable
  qualification to the delivered base plus actual working changes. Final
  qualification prose and issue links are added afterward and checked separately.

Both root and independent manifests/locks remain byte-identical to captured
inputs. Host filesystem/artifact 0.12.0, development Rust 1.99/MSRV 1.88,
formatter pins, stable-structures and Testkit/PocketIC selections are unchanged.
No staging, commit, tag, push, release, publication, fixture Git history or sibling
source mutation occurred. Local Linux/portable fixtures do not establish native
macOS, installed runtime composition or live release acceptance.
[#51](https://github.com/dragginzgame/ic-memory/issues/51) owns consumer delivery
and native acceptance; [Shared #101](https://github.com/dragginzgame/shared-tooling/issues/101)
owns the reusable repair.

## 0.35.2 Shared 0.3.2 parallel Cargo and fixture completion

On delivered Memory **0.35.1** `34795a3b7868c29028075633061385f6403b9486`,
pending compatible **0.35.2** adopts committed Shared **0.3.2**
`c16444bf006f17c5bb4dda5ad070a0f345da9623` through the canonical exporter.
The 91-file selection adds the linked advisory README task; no schedule, prose
rewrite or release gate is activated. The root-only shared formatter remains
unselected; Memory owns its two-workspace compiler/formatting roster.

Shared tool recipes and Memory's own Cargo/formatter/compiled helper recipes
preserve Make jobserver descriptors. The existing execution guard still refuses
unsafe Make modes before substituted effects. Shared completion fixes remain
canonical exports. Three consumer-owned fixtures also require explicit completion
before success/cleanup, retaining unfinished evidence and original nonzero
failure status. No vendored source is edited in place.

Evidence: `target/qualification/0.35.2-shared032/`.

- `before.log` and `before/` reproduce both bugs against released source with
  substitutes: Bash 3.2 nounset falsely returns zero/removes evidence, and a
  parallel Cargo descriptor probe encounters a closed descriptor.
- `retention.log`, `adapters.log`, `portable.log` qualify actual fixture handlers
  and consumer Make routes with substituted effects on current Bash/Make and
  genuine Bash 3.2.57/GNU Make 3.81 on Linux. Early zero exits, nounset, command
  failures and completed success/failure have the expected status/retention.
  Actual formatting recipes still cover both maintained workspaces.
- `setup-reuse-offline.log` and `admission.log` pass real parallel common setup
  reuse and offline admission with the existing selected installations.
- `validate.log` passes full `make validate`, including strict Clippy, 241 library
  tests, public/compile-fail/tooling/doc tests, release/hook/evidence fixtures,
  rustdoc, Wasm checks/five budgets, development packaging and Rust 1.88 all targets.
- `lint.log` and `docs.log` pass workflow/ShellCheck and local-reference checks.
  `source-head.txt`, `qualified.diff`, the new consumer test copy and
  `inputs.sha256` bind executable qualification to its actual inputs. Final
  qualification prose and issue links are checked separately after the gate.

Both maintained manifests/locks are byte-identical to captured inputs. Delivered
Host filesystem/artifact 0.12.2 was already selected; no dependency upgrade is
performed. Compiler/formatter/stable-structures/Testkit/PocketIC selections,
canister API and durable format remain unchanged. No source staging, commit,
push, release, publication, fixture Git history or sibling mutation occurs.
Local Linux/substitute checks do not qualify native macOS, actual installed
composition or live delivery. [#52](https://github.com/dragginzgame/ic-memory/issues/52)
owns delivery/native acceptance; reusable fixes belong to
[Shared #99](https://github.com/dragginzgame/shared-tooling/issues/99) and
[Shared #103](https://github.com/dragginzgame/shared-tooling/issues/103).
