# Repository Instructions

## Active shared rules and local overlay

At the maintainer's explicit direction, read and follow the current local
[Shared Tooling baseline](../shared-tooling/DRAGGINZGAME.md) and its linked rules
and guides. This includes uncommitted working-tree changes in `../shared-tooling`.
Read the applicable rules again when they change; no upstream commit, clean tree
or snapshot refresh is required for them to apply. This is the maintainer-approved
exception to revision-bound policy adoption while the shared rules are being
developed locally. The sibling remains read-only to agents.

The existing tooling snapshot at revision
`47cd2ccaf0e8b428f06e6db0262df76cfc1581de` is recorded in
[.shared-tooling.snapshot](.shared-tooling.snapshot). It identifies the vendored
files, hashes and executable modes; it does not freeze the active local policy.
Keep its provenance accurate and never edit vendored files in place. Do not
attribute uncommitted shared files to that recorded revision. The live local
[user-triggered maintenance rule](../shared-tooling/rules/agent-maintenance.md)
also applies within its activation scope and is included in this recorded snapshot.
These instructions are the repository's local overlay.

The 46-file snapshot uses the reviewed local 0.1.7 commit, including the common
audit methods, local host/IC setup, dependency checker, release-command checker
and linked rules. CI and
release checks use these immutable exports without a sibling checkout. The live
local policy exception above
continues to govern subsequent uncommitted rule development.

The user-provided ownership rule below is the approved local exception to shared
tag/push authority: commits, tags and pushes always belong to the maintainer,
including when an agent is asked to prepare a release. This preserves the
maintainer's review of source and release identity before external effects.

Product contracts and numeric limits remain owned by this crate. Host support
and prerequisites are declared in [docs/host-support.md](docs/host-support.md).

## Development and qualification commands

- Before editing or compiling, check for active Cargo, rustc and rustdoc builds.
- Focused checks: `make verify-shared-tooling`, `make check-pins`, `make test-pins`, `make test-tooling`,
  `make test-release-adapters`, `make test-release-runner`, `make test-hooks`,
  `make fmt-check`, `make lint-tooling`, or an appropriately
  selected Rust test.
- Full gates: `make validate`, `make validate-toolchain`, `make wasm-size` and
  package/release qualification. Run these only on explicit request or in CI.
- Dependency preparation: `make fetch-dependencies` is a separate network step.
  Both maintained workspaces track their existing selected lockfiles.
  Validation and release preparation require those lockfiles and populated
  cache, and run offline without changing dependency selection.
- Do not run maintainer release commands or tests that create commits/tags/pushes,
  including in disposable repositories. Tooling tests substitute command effects.

## Formatting and Git hooks

Follow the current local [Git hook rules](../shared-tooling/rules/git-hooks.md).
The reviewed hook and installer are vendored unchanged. `make fmt` and
`make fmt-check` cover both the library and `testing/runtime-qualification`,
using exactly cargo-sort 2.1.4 before the pinned Rust formatter. Setup is explicit
in [docs/host-support.md](docs/host-support.md); formatting never installs tools,
builds, fetches dependencies or changes selected lockfiles.

`make install-hooks` changes only local `core.hooksPath`. The pre-commit hook
refreshes selected index entries, so agents must not run it on this repository's
real index. Exercise it only in disposable fixtures without commits or tags.
Source adoption, local activation and native host qualification are distinct.
The recorded installer resolves physical checkout paths, so logical checkout
aliases work without a consumer setup adapter.

## Release recovery policy

Read the current local [release contract](../shared-tooling/docs/releases.md).
The latest local rule requires a normal release target to select saved unfinished
intent before computing any new increment, reconcile exact effects on descendant
history, then run fresh gates for a requested next increment. Late checks bind
`RELEASE_COMMIT` separately from the validation source and current HEAD. Stop on
identity, payload, destination, qualification-input or concurrency conflicts. Explicit
`release-resume` is an optional selection command, not a required recovery step.
Preflight/validation failures repeat fresh gates while preserving earlier evidence.

The common vendored runner owns release ordering, locks, saved intent and Git
effects; Rust consumer adapters own metadata and qualification receipts. See
[RELEASING.md](RELEASING.md) for actual commands and evidence. Qualify canonical
runner substitutes, consumer Make dispatch and Rust adapters separately; no stub
pass proves a native or live release. Preserve failed gate logs, earlier receipts
and retained archives across retries. Never invoke a maintainer release command
to prove adoption.

## User-Owned Commits and Pushes

The user exclusively owns committing and pushing. LLM agents must never create,
amend, or rewrite commits, push branches or tags, or invoke scripts that perform
those actions. Requests to prepare a release or push changes do not authorize an
agent to perform these user-owned actions.

- Leave edits unstaged and uncommitted for the user to review.
- Do not create, move, or delete release tags.
- Do not run `make release-minor`, `make release-patch`, `make release-major`
  or `make release-resume`; these
  commands commit, tag, or push.
- Prepare source, documentation, and validation within the working tree, then
  report the result and any commands the user needs to run.
- If a workflow requires committed source or a clean working tree, stop before
  that prerequisite and explain the required user action. Never commit or push
  merely to satisfy a workflow's preconditions.

## Pre-1.0 Hard-Cut Policy

Until `1.0.0`, every release uses the current API and durable format only. Do
not preserve backward compatibility with an earlier pre-1.0 release.

- Do not add deprecated forwarders, compatibility shims, renamed aliases,
  legacy modules, or old macro forms.
- Do not add serde field aliases, version-routed legacy decoders, fallback
  readers, compatibility fixtures, or defaults whose purpose is to accept an
  earlier wire shape.
- When an API or format changes, remove the superseded path in the same change
  and update current fixtures, documentation, and downstream callers directly.
- Historical changelogs and archived design documents may describe removed
  behavior, but executable code and current documentation must not retain it.
- Breaking public API or semantic changes require a minor release before 1.0.
  Patch releases preserve the public contract. Hard cuts remove superseded paths
  in the same change; they do not permit incompatible patch releases.
- Compile-fail tests enforce maintained capability and trust boundaries. Do not
  add tests whose only purpose is prohibiting a retired name.

## Changelog maintenance

Before editing release notes, read the current local
[changelog rules](../shared-tooling/rules/changelogs.md). They own automatic
candidate selection, content, issue links, detail structure and history
preservation, including while those rules are uncommitted.

- Automatically maintain one numbered, undated pending entry, `## [X.Y.Z]`,
  after each meaningful completed batch. Derive it from the latest actual release
  and the complete pending scope; a prepared manifest is not another released base.
- Before 1.0, choose patch for compatible work and minor for breaking consumer
  contracts. Honour a compatible maintainer-selected version. Reuse the same
  pending entry until release; do not assign a version to every cleanup slice.
- Selecting notes does not authorize changing manifests or lockfiles, committing,
  tagging, pushing or publishing. Final release dates belong to release preparation.
- Keep the root changelog concise and focused on practical effects. Put extended
  implementation notes in `docs/changelog/<major>.<minor>.md`; link existing
  qualification evidence rather than copying routine validation logs into root.
- Link actual resolved GitHub issues, mark breaking changes and required consumer
  actions, align root/detail identities and links, and preserve published history.

## Cargo dependency ownership

Keep direct dependency versions, sources and default-feature selections in
the root `[workspace.dependencies]`; package dependency tables inherit them.
Centralization must preserve selected dependency versions and effective features.
Apply the current local
[Cargo dependency rules](../shared-tooling/rules/cargo-dependencies.md).

The maintainer-accepted 0.25.14 qualification design is the local exception for
`testing/runtime-qualification`: it remains an unpublished independent workspace,
whose authoritative dependency root is its own `Cargo.toml`. Its package tables
inherit its own workspace catalog. This keeps PocketIC's host-only graph and
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
to the independent qualification workspace and its caller-owned server API;
changing that pair requires repeating installed qualification. These are existing
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
