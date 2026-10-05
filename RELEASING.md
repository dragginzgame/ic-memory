<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-memory/ic-memory-readme-header.svg" alt="IC Memory — Stops upgrades from mixing up stored data" width="100%">
</p>

# Releasing ic-memory

This guide is for repository maintainers. Application users do not need these
steps to depend on `ic-memory`.

The maintainer exclusively owns commits, tags and pushes. Agents prepare source,
documentation and focused validation in the working tree, leaving changes
unstaged and uncommitted. See [AGENTS.md](AGENTS.md). The release helper is the
Rust development example `repo-tool`; it is not canister runtime code.

Prerequisites, pinned formatter setup, hook activation and native qualification
are declared in
[docs/host-support.md](docs/host-support.md). Development and CI use Rust 1.99.0;
the crate's MSRV remains Rust 1.88.0. The workflow requires Git, GNU Make and
`shasum`. Python is no longer a tooling dependency.

## Release policy and source preparation

Before 1.0, breaking public API or semantic changes require a minor release.
Patch releases preserve the public contract. Hard cuts still remove superseded
APIs and formats directly; they do not justify an incompatible patch.
Previously published releases and tags are historical records and are not
rewritten. Inspect affected downstream callers before choosing a release kind.

Keep one numbered, undated pending entry at the top: `## [X.Y.Z]`. Development
selects that candidate from the latest actual release and the complete pending
batch, respecting compatible maintainer overrides. Notes alone do not authorize
package or lockfile version changes. Keep root summaries concise and link extended
notes in `docs/changelog/<major>.<minor>.md` and their existing evidence owners.
Preparation requires the pending heading to match its computed candidate;
it refuses a conflicting selection rather than renumbering the notes. Historical
entries remain unchanged. Changelog checks apply to release preparation, not
downstream deployments.

The current local [common release contract](../shared-tooling/docs/releases.md)
applies at the maintainer's direction, including uncommitted changes. Read it
before changing release tooling. Applying that policy and preparing its local
implementation do not require an upstream commit or snapshot refresh.

The workflow below still needs implementation alignment with that contract:
the current targets lack `release-major`, a shared release lock and an exact
saved-plan recovery path through the same normal target. The latest local
contract selects unfinished intent before any fresh increment, reconciles the
saved version, commit and destination automatically, and stops on conflicts.
Explicit `release-resume VERSION=X.Y.Z` is optional selection under that contract;
it must not be a required step after interruption. The current ic-memory helper
does not yet provide those common-runner guarantees. These are implementation
gaps, independent of the
shared rules' commit state. Snapshot integrity alone does not qualify release
adapters or their behavior. Agents must not execute the maintainer-owned release
commands to prove adoption.

The maintainer commits the implementation and changelog before preparation;
source must then be clean. Check for active builds before compiling or editing.
Select and cache dependency inputs separately, before release mutation:

```sh
# A fresh checkout needs explicit dependency selection first:
cargo generate-lockfile
make fetch-dependencies
```

Do not regenerate an existing validated lockfile. Cache preparation is a network
step; validation and packaging run offline with the selected lockfile. Dependency
upgrades are separate work. A missing cache or lockfile fails rather than retrying
online. The lockfile remains untracked, but its hash is release evidence.

Focused checks include `make test-tooling`, `make fmt-check`,
`make verify-shared-tooling`, and `make lint-tooling` with separately installed
lint tools. `make validate`, `make validate-toolchain`, `make wasm-size` and
package qualification are full gates: run only when explicitly requested or in
configured CI. Release preparation explicitly invokes `make validate`.

## Maintainer release commands

```sh
make release-patch   # Validate, prepare, stage, commit, qualify, tag, push
make release-minor   # Same workflow with a minor release
make publish-dry-run # Check the qualified release; no upload
make publish         # Publish that release to crates.io
```

Publication is a separate network effect and requires configured crates.io
credentials. `PUBLISH_DRY_RUN=1 make publish` also performs a dry run.
The release targets push the current branch and its `vX.Y.Z` annotated tag
atomically to `origin`. The branch must already exist remotely, and its
refreshed remote head must be an ancestor of the local source commit.

To review the version edits before committing:

```sh
make patch           # Or make minor; stops after source validation/preparation
# Review git diff, then the maintainer runs:
make release-stage
make release-commit  # Commit, qualify the final archive, then annotate the tag
make release-push
```

Preparation changes only the root package version, README dependency example
and the matching pending changelog heading. It refreshes only the root package
version in `Cargo.lock`; any other dependency-selection change is rejected.
Failed preparation restores these files and preserves existing receipts and
build artifacts. A non-mutating `make fmt-check` after metadata preparation
must pass before packaging or staging; the hook must not repair a saved release
payload. Release commits must contain exactly the expected edits and
identify the validated source in their commit message.

A rejected push can be retried with `make release-push`; do not bump again.
A failed final package or tag step can be retried with `make release-commit`
without creating another commit. `make qualify-release` repeats final package
qualification without committing, tagging or pushing. It requires the original
prepared evidence and unchanged dependency/compiler identities.
Its prepared package HEAD must identify the validated source, and the retained
prepared archive must still match its recorded digest. Missing or corrupted
prepared artifacts stop qualification before repackaging; Cargo's working
archive may differ after a failed final package without preventing a retry.
These are the current helper's phase-specific retries. Rerunning `release-patch`
or `release-minor` after an interrupted preparation is not yet automatic saved-plan
recovery; the common workflow must replace that orchestration before those
normal targets can be used for the new recovery contract.

## Evidence and publication

Receipts live under Cargo's actual target directory, including configured paths
and `CARGO_TARGET_DIR`, in `release-validation/`:

- `<version>-prepared.json` binds the validated source, selected lockfile,
compiler identities, commands and package built during preparation.
- `<version>.json` additionally binds the final archive to the release commit.
  Cargo embeds Git metadata, so this archive is qualified after the commit.
- `artifacts/<sha256>.crate` retains each qualified archive independently of
  Cargo's working package path, which subsequent packaging may replace.

Compiler identities include the pinned Cargo/rustc and the MSRV rustc. The
receipts record SHA-256 digests for the lockfile, package and applicable Cargo
configuration files, plus build flag/profile environment values. Release
validation explicitly uses the repository pin even if a local Make override is
set. Release qualification rejects `RUSTC`, `RUSTC_WRAPPER`,
`RUSTC_WORKSPACE_WRAPPER`, `RUSTDOC` and their `CARGO_BUILD_*` aliases. Unset these
variables, including empty assignments, before preparing or using release
evidence. Remove `build.rustc`, `build.rustc-wrapper`,
`build.rustc-workspace-wrapper` and `build.rustdoc` from discovered Cargo
configuration files in the checkout, its ancestors and Cargo home. These checks
keep qualification tied to the declared toolchains; compiler and wrapper
replacements are unsupported in the release workflow. Recorded compiler flags
and profile overrides remain supported.

Staging, committing, pushing and publishing refuse stale or missing required
evidence. Publication
never generates a replacement lockfile. Keep the prepared and final evidence,
selected lockfile and package archive for the maintainer workflow; a new
checkout alone is not publication qualification.

Cargo owns registry upload behavior and immutable package-version identity.
If publication is interrupted, inspect crates.io for the exact version before
retrying; a lost reply is not proof of failure. A successful source check is not
proof of publication, downstream adoption or deployment.

`make test-tooling` exercises the release workflow using substituted commands
and temporary files owned by each fixture. It checks rollback, invalid evidence,
final qualification, retry ordering and atomic push arguments. It creates no Git
commits, tags or pushes, invokes no live Cargo publication, and needs no network.
Wasm budget tests exercise exact ceilings, missing/oversized artifacts and Cargo
metadata failure against the selected artifact directory.
