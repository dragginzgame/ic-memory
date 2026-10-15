# Repository Instructions

## Active shared rules and local overlay

Apply the reviewed local [Shared Tooling baseline](DRAGGINZGAME.md) and its
linked rules. [.shared-tooling.snapshot](.shared-tooling.snapshot) owns the exact
policy and executable revision. The maintainer-approved cleanup retires live
inheritance from the sibling checkout; uncommitted upstream rules do not apply
until reviewed adoption. Other repositories remain read-only unless explicitly
authorized under the shared contribution rules.

Before changing tooling, formatting, release adapters or retention, read
[the local adoption boundaries](docs/shared-tooling-adoption.md). Never patch
vendored files or attribute dirty shared source to a committed export.

The virtual repository-root workspace owns shared package metadata and the
dependency catalog. The library package is `crates/ic-memory/`; its source,
examples, tests, current wire fixtures and canonical package README live there.
Follow the [workspace layout rules](rules/rust-workspaces.md).

Product contracts and numeric limits remain owned by this crate. Host support
and prerequisites are declared in [docs/host-support.md](docs/host-support.md).

## Development and qualification commands

- Before editing or compiling, check for active Cargo, rustc and rustdoc builds.
- Focused checks: `make shared-tooling-check`, `make check-pins`, `make test-pins`, `make test-tools`, `make test-tooling`,
  `make test-release-adapters`, `make test-release-runner`, `make test-hooks`, `make test-fixture-cell`,
  `make test-failure-evidence`, `make fmt-check`, `make lint-tooling`, or an appropriately
  selected Rust test. `make runtime-server-check` and `make test-runtime` are
  focused installed qualification with explicitly prepared tools and both caches.
- Full release suite: `make validate`, under the validation cadence below.
  Development packaging qualifies working-tree edits; release
  admission and the separate `make package` target still require clean source.
  Release, publication and installed runtime qualification retain their separate
  authority. Never commit merely to satisfy a check.
- Dependency preparation: `make fetch-dependencies` is a separate network step.
  Both maintained workspaces track their existing selected lockfiles. The
  independent graph is prepared separately with
  `cargo fetch --locked --manifest-path testing/runtime-qualification/Cargo.toml`.
  Validation and release preparation require those lockfiles and populated
  cache, and run offline without changing dependency selection.
  The documented release entry points also authorize locked cache preparation
  before helper compilation. An explicitly offline caller stays offline; ordinary
  helper commands and validation never fetch online automatically.
- Release commands require an explicit request for the selected repository,
  release and destination. Never invoke them merely to qualify adoption.
  Tooling tests substitute Git effects; do not create commits/tags/pushes to
  satisfy fixture prerequisites, including in disposable repositories.

## Formatting and Git hooks

Follow the reviewed [Git hook rules](rules/git-hooks.md).
The reviewed hook and installer are vendored unchanged. `make fmt` and
`make fmt-check` cover both the library and `testing/runtime-qualification`,
using the [selected catalog](ci/tool-versions.env) before the pinned Rust formatter.
Setup is explicit in [docs/host-support.md](docs/host-support.md); formatting never installs tools,
builds, fetches dependencies or changes selected lockfiles. The shared formatter
prerequisite checker owns exact cargo-sort and rustfmt admission; this consumer
owns the selected pin, toolchain and two-workspace roster.

`make install-hooks` changes only local `core.hooksPath`. The pre-commit hook
refreshes selected index entries. Exercise it only in disposable fixtures during
qualification; explicitly authorized commits may invoke it on their selected
real index entries. Preserve partial-staging refusal and unrelated work.
Source adoption, local activation and native host qualification are distinct.
The recorded installer resolves physical checkout paths, so logical checkout
aliases work without a consumer setup adapter.

## Release recovery policy

Read the reviewed [release contract](docs/releases.md).
The common vendored runner owns release ordering, locks, saved intent and Git
effects; Rust consumer adapters own metadata and qualification receipts. See
[RELEASING.md](RELEASING.md) for actual commands and evidence. Qualify canonical
runner substitutes, consumer Make dispatch and Rust adapters separately; no stub
pass proves a native or live release. Preserve failed gate logs, earlier receipts
and retained archives across retries. Never invoke a maintainer release command
to prove adoption.

## Contributions, compatibility and changelogs

Follow the reviewed shared [contribution rules](rules/contributions.md),
[hard-cut baseline](DRAGGINZGAME.md#pre-10-contracts) and
[changelog rules](rules/changelogs.md). Ordinary work leaves
staging, commits, package versions and release effects to the user. An explicit
PR request includes its scoped contribution workflow; it does not select PR
release delivery. Memory uses direct release delivery. Never commit merely to
satisfy a validation prerequisite.

Keep root notes concise, with extended notes in `docs/changelog/<major>.<minor>.md`.
Historical documents may describe retired formats; maintained code and fixtures
use only the current contract. Do not add legacy decoders, Serde field aliases
or tests whose sole purpose is prohibiting a retired name.

## Cargo dependency ownership

Keep direct dependency versions, sources and default-feature selections in
the root `[workspace.dependencies]`; package dependency tables inherit them.
Centralization must preserve selected dependency versions and effective features.
Apply the reviewed
[Cargo dependency rules](rules/cargo-dependencies.md).

The maintainer-accepted 0.25.14 qualification design is the local exception for
`testing/runtime-qualification`: it remains an unpublished independent workspace,
whose virtual root and authoritative dependency catalog are its own
`Cargo.toml`. Its sole package is
`testing/runtime-qualification/crates/ic-memory-runtime-qualification/`, whose
package metadata and dependency tables inherit that root. The selected lockfile
is `testing/runtime-qualification/Cargo.lock`; normal library gates exclude this
graph. Locked metadata/formatting cover it during layout work, and installed
qualification retains the caller-owned PocketIC server and Wasm requirements. This keeps PocketIC's host-only graph and
separate selected lockfile outside the library's normal gates and dependencies.
This exception does not permit other nested workspaces or drifting duplicate
declarations. It preserves the accepted qualification scope rather than silently
merging or upgrading the two measured dependency graphs.

## Dependency exceptions

Registry requirements remain compatible ranges, except for two qualified
protocol boundaries recorded exactly in
[ci/dependency-pinning-exceptions.json](ci/dependency-pinning-exceptions.json).
`ic-stable-structures = "=0.7.2"` owns the manager/Cell layouts inspected by
`runtime/layout.rs` and `stable_cell.rs`; a change requires layout, corruption,
growth and installed IO/upgrade qualification. `pocket-ic = "=16.0.0"` is scoped
to the independent qualification workspace. The client pin remains unchanged;
Testkit 0.25.4 owns the reviewed 16.1.0 server selection. Changing that pair
requires repeating installed qualification; historical 16.0.0 server evidence
cannot qualify the new pairing. These are existing
maintainer-selected boundaries, not new dependency selections. Keep both
lockfiles, sources, features and toolchains unchanged during pinning adoption.

## Rust Item Documentation Style

For public structs, traits, and enums, prefer a wrapped rustdoc block with the
item name as a short heading, a blank rustdoc line before and after the heading,
and a blank source line before attributes:

```rust
///
/// SchemaMetadata
///
/// Optional diagnostic metadata for an in-place store schema.
///
/// This metadata helps humans and frameworks diagnose which schema version was
/// declared in each generation. It is bounded and validated for durable ledger
/// encoding, but it does not perform application schema migrations or validate
/// stable data semantics.
///

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaMetadata {
    /// Optional in-place schema version.
    pub schema_version: Option<u32>,
}
```

Keep this shape for new or edited item-level docs unless local context clearly
requires a different style.

## Validation policy

Apply Shared Tooling's [canonical validation cadence](https://github.com/dragginzgame/shared-tooling/blob/4256171b5b7ec4c4cc4078548c1c1617ee150699/DRAGGINZGAME.md#validation-and-evidence),
adopted by exact revision `4256171b5b7ec4c4cc4078548c1c1617ee150699` (0.12.5).
This revision-bound policy reference supersedes only the validation cadence in
our older snapshot; executable tooling and all other shared rules retain their
recorded snapshot revision. Local guidance supplies commands and product
qualification requirements. Configured CI and release gates remain unchanged.
On snapshot adoption of this policy, replace this separate reference with the
local baseline link; do not keep a duplicate cadence or temporary exception.
