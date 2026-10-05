# Shared release workflow qualification

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
