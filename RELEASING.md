<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-memory/ic-memory-readme-header.svg" alt="IC Memory — Stops upgrades from mixing up stored data" width="100%">
</p>

# Releasing ic-memory

This guide covers explicitly authorized release work. Ordinary fixes and release
preparation leave edits unstaged; an explicit commit or PR request authorizes its
scoped contribution workflow, and an explicit standard release request authorizes
its documented commit/tag/push effects. See [contribution rules](rules/contributions.md)
and [AGENTS.md](AGENTS.md). The Rust `repo-tool` example is development tooling,
not canister runtime code. Prerequisites and native host evidence are in
[host support](docs/host-support.md).

This repository selects `RELEASE_DELIVERY=direct`: the authorized release pushes
the selected branch and tag atomically. Other delivery selections are refused
before runner dispatch. The shared runner's PR path requires separately adopted
merged-source preparation and receipt adapters; ordinary contribution PRs do not
select that release policy.

## Source and dependency preparation

Before 1.0, breaking consumer contracts require a minor release; compatible work
uses a patch. Maintain one numbered, undated pending entry, `## [X.Y.Z]`, in both
`CHANGELOG.md` and `docs/changelog/<major>.<minor>.md`. Notes select the candidate
without changing package versions. Preparation refuses mismatched or duplicate
identities and preserves historical entries.

Implementation and pending notes must be committed through an authorized
contribution workflow before releasing. The selected source must be clean, on
the selected branch, with no active build.
Preserve the tracked dependency selections and prepare their cache:

```sh
make fetch-dependencies
```

Both maintained workspaces track their `Cargo.lock`. The root lock is a required
source-bound qualification input. Release preflight
checks the selected cache with `cargo fetch --locked --offline`; it never retries
online or regenerates the lockfile. Validation, version refresh and packaging
remain offline. A root-version refresh may change only the `ic-memory` entry,
never dependency selection. Cache preparation is a separate network operation.
Prepare the complete checksum-pinned jq/yq/ripgrep/cloc host set with
`make install-host-tools`, as described in [host support](docs/host-support.md).
`make check-pins` checks declarations and
tracked locks; `make validate` includes it and the canonical pinning fixtures.
Commit the initial tracked-lock adoption before running those gates. Complete
any earlier unfinished release using its qualified source/tooling first: old
source commits without the root lock cannot satisfy the new source binding.
Retain their plans, receipts and archives; do not reinterpret or discard them.

## Maintainer commands and recovery

```sh
make release-patch
make release-minor
make release-major
# Defaults: RELEASE_REMOTE=origin RELEASE_BRANCH=main
```

All three delegate to the unchanged vendored Shared Tooling runner. It owns
preflight, the full gate, exact preparation, explicit staging, the `Release X.Y.Z`
commit, annotated `vX.Y.Z` tag and atomic branch/tag push. All kinds use the same
pinned `make validate` gate. The branch must already exist remotely, with its
refreshed remote head an ancestor of local HEAD. Overrides select a remote name
and branch explicitly; the saved push URL identity must remain unchanged.

Rerun a **normal target** after interruption. Once preparation may start,
the runner selects unfinished intent before computing any new increment. Its
plan and directory lock live in Git's `release-state` directory; the plan fixes
kind, previous/candidate versions, source commit, UTC date, branch and destination.
Matching effects are reconciled without a second bump or commit. Before the
release is committed, preparation remains bound to its saved source and kind.
After commit, newer descendant fixes can remain at HEAD: late adapters verify
the exact older `RELEASE_COMMIT`, its saved source and annotated tag. An unchanged
same-kind retry finishes only that release. A newer descendant or a different
requested increment first completes it, then runs fresh preflight/full validation
for the next candidate from the actual local version. Explicit resume completes
only its selected release. Conflicting payload, history, index, destination, tag
or unfinished intent stops recovery.
Inspect a stale lock's recorded owner before manual removal; do not steal it.

Preflight and validation failures restart fresh gates through the normal target,
preserving prior attempts. Optional explicit selection uses the same checks:

```sh
make release-resume VERSION=X.Y.Z
```

Push uses `--no-follow-tags --atomic` and exactly the selected branch and candidate
tag refspecs. There is no force push or non-atomic fallback. A lost push reply is
reconciled with exact remote identities; a failed remote query stops recovery.
Success retains plans, logs, receipts and archives. Cleanup and package
publication are separate operations.

The Rust consumer adapters consume the runner's eight `RELEASE_*` selections.
They validate source/input identities, finalize the root and detail notes with
the saved UTC date, update Cargo/README metadata and qualify the packages. The
explicit staged file set is `Cargo.toml`, `Cargo.lock`, `crates/ic-memory/README.md`, `CHANGELOG.md`
and the candidate minor-line detail file. The root lock edit changes only the
`ic-memory` package version; the independent qualification lock is not a release
edit. Git mutations belong exclusively to the common runner.

Preflight checks both working files and the entire index. Restoring a working
file does not hide unrelated or arbitrary staged content. The same index guard
runs again before commit admission, including file modes. Committed checks read
metadata from the selected SHA and require its sole parent, release subject and
exact prepared metadata to match saved intent.

The root `[workspace.package]` owns the release version and MSRV;
`crates/ic-memory/Cargo.toml` inherits them and stays unchanged during a bump.
The canonical dependency example is in the package README.

Metadata writes use same-directory atomic replacement, with `Cargo.toml` last.
An interrupted earlier write can resume from exact original/prepared files. Once
the candidate manifest is present, all owned metadata including the lock is
complete; prepared checks can finish offline workspace verification and
packaging from saved successful validation, without another bump or full gate.
Returned preparation failures restore only owned edits; conflicting files and
independently changed dependency selections are preserved and refused.

If the candidate lock precedes the base manifest, the Make helper launcher uses
`--no-deps` Cargo metadata and the prepared yq to find the selected target
directory. It compiles current source against a coherent scratch manifest and
an unchanged copy of the lock, using `--locked --offline`. It runs the adapter
in the real checkout; compilation alone grants no release authority. Preflight
requires the saved source-bound successful validation before accepting this
partial state. Metadata and builds remain under `repo-tool-bootstrap/` in the
selected target directory. Normal helper calls use the original workspace.

`release-version`, `release-files` and the other named adapters are runner
interfaces, not alternative maintainer orchestration. They require the saved
selection and evidence where applicable. `make qualify-release` can finish final
package qualification without Git effects, using the original prepared evidence.
Creating a final receipt requires HEAD to remain the selected release commit
through packaging. A changed HEAD stops before recording final evidence;
historical recovery reuses intact existing qualification.

## Evidence and publication

Under Cargo's actual target directory (including configured overrides),
`release-validation/` contains:

- `<version>-validated.json`: successful full-gate source, saved selection,
  original lock bytes, toolchain identities, configuration and exact command.
- `<version>-prepared.json`: that validation plus refreshed lock digest and the
  package qualified before the release commit.
- `<version>.json`: final package bound to the exact release commit. Cargo embeds
  Git metadata, so final packaging follows commit creation.
- `artifacts/<sha256>.crate`: retained qualified archives independent of Cargo's
  replaceable working package path.
- `attempts/verify.*`: unique stdout/stderr logs, including failed full gates.
  Replaced successful validation receipts are also archived here before replacement.

Receipts are written atomically. Existing prepared/final receipts and archives
are checked, never silently repaired. Missing/corrupted prepared evidence stops
final qualification before packaging; a working archive replaced by failed final
packaging does not invalidate its intact retained prepared archive. Completed
prepared/final checks reuse valid evidence without repackaging.

Archive retention streams into the shared durable publisher's private staging
file and checks the copied digest before publication, then independently hashes
the retained file. Publication can report an error after complete bytes become
visible. Retry checks and reuses those bytes; an existing corrupt archive stops
qualification rather than being replaced. Staging allocation and synchronization
belong to `ic-host-fs`; release identities and receipts remain local.

Older-commit recovery requires intact prepared and final receipts and retained
archives. The replaced working package may belong to newer source; only the
retained archive for the selected older SHA supplies historical package evidence.
The selected lockfile, compiler identities, build configuration and qualification
commands must still match the saved evidence. Changed qualification inputs,
missing receipts or corrupted archives stop recovery. No receipt schema changes
or evidence migration are required by the 0.27.1 tooling update. If an older
release has no final receipt, qualify that exact release commit first using
`make qualify-release`; the adapter never packages newer source as that release.

Compiler identities include pinned Cargo/rustc and MSRV rustc. Discovered Cargo
configuration digests and build flags/profile overrides are recorded. Qualification
rejects compiler/wrapper replacements: `RUSTC`, `RUSTC_WRAPPER`,
`RUSTC_WORKSPACE_WRAPPER`, `RUSTDOC`, their `CARGO_BUILD_*` aliases and corresponding
`build` keys in checkout, ancestor and Cargo-home configuration. Unset even empty
assignments. Ordinary recorded flags/profile settings remain supported.

The current receipt schema is a hard cut from the earlier phase workflow. Old
receipts are not accepted or migrated. Finish outstanding earlier releases with
their original tooling before adopting this workflow. Preserve old evidence;
there is no automatic deletion or conversion of it or canister stable data.

```sh
make publish-dry-run
make publish # Separate network effect; requires crates.io credentials.
```

Helper compilation and workspace discovery use explicit `--offline` flags;
the launcher preserves the caller's `CARGO_NET_OFFLINE` setting for dispatched
commands. Publication needs network access, including registry inspection in a
dry run. An explicitly offline caller remains offline; setup/validation never
change that setting to retry online.

`PUBLISH_DRY_RUN=1 make publish` also selects a dry run. Publication requires the
exact commit, annotated tag, selected dependencies and qualified archive; it
never creates replacement evidence. A fresh checkout alone is not qualification.
If publication is interrupted, inspect crates.io's exact version before retrying;
a lost reply is not proof of failure. Release targets never publish implicitly.

## Focused workflow checks

The runner and Rust validation adapter use the shared Make execution check before
dispatching gates. It rejects inherited ignore-errors, dry-run, question, touch
and version-only modes; rerun without those modes. Normal release selections and
jobserver settings remain inherited. A refusal preserves earlier validation
receipts and does not create a successful validation attempt.

- `make test-tooling`: Rust adapters with substituted Git/Cargo/gate effects;
  recovery, rollback, input binding, artifact refusal and publication checks.
- `make test-release-adapters`: the shared checker exercises Make entry points,
  runner failures and conflicting selections with a substitute runner. Local
  fixtures cover selection forwarding, unique gate logs and locked bootstrap.
- `make test-release-runner`: unchanged canonical runner with command substitutes;
  ordering, all increments, Git effect scope, locks and interruption reconciliation.
- `make verify-shared-tooling`, `make test-hooks`, `make fmt-check` and
  `make lint-tooling`: snapshot, setup/formatting and portable tooling checks.

These tests create no commits/tags/pushes and perform no publication or full gate.
They do not prove a live release or native macOS behavior. The declared native CI
matrix includes them in `make validate-toolchain`. Full gates (`make validate`,
`make validate-toolchain`, `make wasm-size`, package qualification) run only on
explicit request or in configured CI. Current focused evidence is recorded in
[release workflow qualification](docs/release-workflow-qualification.md).
