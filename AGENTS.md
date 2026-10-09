# Repository Instructions

## Active shared rules and local overlay

At the maintainer's explicit direction, read and follow the current local
[Shared Tooling baseline](../shared-tooling/DRAGGINZGAME.md) and its linked rules
and guides. This includes uncommitted working-tree changes in `../shared-tooling`.
Read the applicable rules again when they change; no upstream commit, clean tree
or snapshot refresh is required for them to apply. This is the maintainer-approved
exception to revision-bound policy adoption while the shared rules are being
developed locally. Other repositories, including Shared Tooling, are read-only
by default. Prefer reporting proposed repairs through issues in their owning
repository. An explicit maintainer authorization naming the repository and
intended change permits that scoped exception; carry it through the necessary
implementation and qualification without asking again. It does not authorize
unrelated sibling edits, commits, pushes, releases or publication.

The existing tooling snapshot at revision
`ee48bb37c98c771e77b92fd891f0757d8c1c8b99` is recorded in
[.shared-tooling.snapshot](.shared-tooling.snapshot). It identifies the vendored
files, hashes and executable modes; it does not freeze the active local policy.
Keep its provenance accurate and never edit vendored files in place. Do not
attribute uncommitted shared files to that recorded revision. The live local
[user-triggered maintenance rule](../shared-tooling/rules/agent-maintenance.md)
also applies within its activation scope and is included in this recorded snapshot.
These instructions are the repository's local overlay.

The single 86-file snapshot uses reviewed committed Shared Tooling 0.2.2,
including the archiver, tool-evidence selector, their fixtures and the shared
retention action. It replaces the supplemental archive manifest and its duplicate
verifier records. The consumer collector owns product log selection and calls
the shared selector; full tool retention remains the command default, while CI
explicitly selects compact retention for freshly verified complete active sets.
Failed, unknown and unselected bundles remain full. Never attribute subsequent
dirty sibling fixes to this committed export.
The shared IC bundle contains five tools. PocketIC selection, provisioning,
offline admission and managed lifecycle belong to the published Testkit CLI,
selected independently in the root Makefile. `install-runtime-server` is explicit
network setup; `runtime-server-check` is offline; `test-runtime` qualifies the
supplied Wasm and independent runner with the prepared server. An old six-tool
IC bundle fails the new offline check and requires explicit `install-ic-tools`;
prior bundles, receipts and failure evidence stay retained. The runner reads
`IC_TESTKIT_POCKET_IC_URL`; no retired server-path alias remains.
The snapshot also includes single-document exception admission, checkout-local
formatter lookup and the linked read-only release-source helper. Memory's Rust
adapter retains ownership of release source admission; including the helper
completes the linked governance selection without introducing another gate.
This revision includes the reviewed installer operand corrections, explicit
fixture companion declarations and literal host/IC active-link admission.
The consumer release-runner fixture is simulation-only; the real-Git tracking
fixture remains upstream-owned and is not selected. Fourteen explicit additions
complete the linked maintenance catalog, optional coordinator and scheduler
templates. Their adoption activates no task, agent or schedule. The current
MSRV and std/no_std guidance is included without changing compiler selection.
Fleet/context LOC regression programs and both fleet reporter implementations
are no longer selected in Memory; their ownership remains with Shared Tooling.
Run fleet reports there. The local workspace LOC fixture and `make cloc` remain
selected, including the explicit independent qualification-manifest option.

The snapshot includes the common
audit methods, local host/IC setup, dependency checker, release-command checker
and linked rules, including the standard Rust workspace layout. Cargo inheritance
checks cover both approved workspace roots. The unchanged `make/tools.mk`
owns setup, offline tool checks and LOC commands. Its complete host set includes
pinned cloc; existing installations must refresh explicitly. Rust LOC defaults
to the root workspace; `CLOC_MANIFEST` explicitly selects the independent
qualification manifest without combining graphs. The retained local LOC fixture
owns its target/manifest isolation; consumer dispatch needs no target workaround.
Fleet/context fixture qualification stays upstream. The shared Rust-tool
installer and fixtures are available through explicit `install-rust-tools` and
`rust-tools-check`; the existing required host/IC aggregate and separate exact
cargo-sort formatter setup remain unchanged.
The Rust installer refuses redirected install/build/receipt paths before tool
execution and after Cargo returns. Tool-command fixtures normalize their physical
scratch root; the IC installer uses the included canonical pin parser.
The release runner and its PR helper are exported unchanged. This consumer
selects direct delivery and rejects other delivery selections before dispatch;
PR release adapters and their real-Git qualification fixture are not adopted.
The canister audit addendum is linked guidance; product limits remain local.
The common metadata fixture also includes its workspace-version reader. Product
version parsing stays in the Rust release adapter. The linked tag-maintenance
guide is documentation only; no tag-maintenance executable is adopted. CI and
release checks use these immutable exports without a sibling checkout. The live
local policy exception above continues to govern subsequent uncommitted rule
development. The read-only `scripts/dev/gh-ci.sh` helper supports exact-commit
inspection across workflows; use authenticated GitHub CLI and the shared
maintenance rule's evidence checks. Relevant issue work in GitHub repositories
owned by `dragginzgame` follows that rule's standing authority; issue actions in
other GitHub repositories require explicit destination/action authorization.
Inspection still does not authorize source repair.

Fleet inventory and exporter/verifier integration fixtures remain upstream-owned.
The consumer does not vendor the fleet fixtures or unused exporter. Never create
a fixture commit or alter the real index to satisfy a test prerequisite.

The shared Make execution checker guards the runner, hook and consumer Rust
validation adapter before gate dispatch. It refuses inherited Make modes that
skip execution or ignore failures, preserving ordinary selections and jobserver
settings. Qualify rejection through real Make with substituted release effects.

Follow the shared [contribution rules](rules/contributions.md). Ordinary fixes
remain local; an explicit commit or PR request authorizes its scoped Git workflow.
Merge, integration-branch push, release, publication and deployment effects retain
their separate target/effect authorization. The maintainer-approved live-local
policy exception above remains; there is no blanket agent-commit prohibition.

The virtual repository-root workspace owns shared package metadata and the
dependency catalog. The library package is `crates/ic-memory/`; its source,
examples, tests, current wire fixtures and canonical package README live there.
Follow the [workspace layout rules](rules/rust-workspaces.md).

Product contracts and numeric limits remain owned by this crate. Host support
and prerequisites are declared in [docs/host-support.md](docs/host-support.md).

## Development and qualification commands

- Before editing or compiling, check for active Cargo, rustc and rustdoc builds.
- Focused checks: `make verify-shared-tooling`, `make check-pins`, `make test-pins`, `make test-tools`, `make test-tooling`,
  `make test-release-adapters`, `make test-release-runner`, `make test-hooks`,
  `make test-failure-evidence`, `make fmt-check`, `make lint-tooling`, or an appropriately
  selected Rust test. `make runtime-server-check` and `make test-runtime` are
  focused installed qualification with explicitly prepared tools and both caches.
- Full gates: `make validate`, `make validate-toolchain`, `make wasm-size` and
  package/release qualification. Run these only on explicit request or in CI.
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

Follow the current local [Git hook rules](../shared-tooling/rules/git-hooks.md).
The reviewed hook and installer are vendored unchanged. `make fmt` and
`make fmt-check` cover both the library and `testing/runtime-qualification`,
using exactly cargo-sort 2.1.4 before the pinned Rust formatter. Setup is explicit
in [docs/host-support.md](docs/host-support.md); formatting never installs tools,
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

Read the current local [release contract](../shared-tooling/docs/releases.md).
The latest local rule requires a normal release target to select saved unfinished
intent before computing any new increment, reconcile exact effects on descendant
history, then run fresh gates for a requested next increment. Late checks bind
`RELEASE_COMMIT` separately from the validation source and current HEAD. Stop on
identity, payload, destination, qualification-input or concurrency conflicts. Explicit
`release-resume` is an optional selection command, not a required recovery step.
Preflight/validation failures repeat fresh gates while preserving earlier evidence.
The runner rechecks the payload, index, working files and exact annotated tag
after the final push hook. Completed direct resume observes the local/remote tag
and branch ancestry without replaying effects; known descendants remain valid.
Missing or conflicting refs and unavailable observations stop completion.

The common vendored runner owns release ordering, locks, saved intent and Git
effects; Rust consumer adapters own metadata and qualification receipts. See
[RELEASING.md](RELEASING.md) for actual commands and evidence. Qualify canonical
runner substitutes, consumer Make dispatch and Rust adapters separately; no stub
pass proves a native or live release. Preserve failed gate logs, earlier receipts
and retained archives across retries. Never invoke a maintainer release command
to prove adoption.

## Contributions and Git authority

People and agents use topic branches and pull requests under the shared
[contribution rules](rules/contributions.md), with normal repository/fork
permissions, required reviews, checks and branch protections.

- Ordinary fixes, continuation and release preparation leave edits unstaged and
  uncommitted unless the user also requests the relevant Git effect.
- An explicit commit request authorizes staging the scoped work and creating its
  commit; it does not implicitly authorize a push.
- An explicit PR request includes the necessary topic branch, scoped commits,
  branch push and PR creation/update. Do not require the maintainer to commit
  first or ask for the same authorization again at each step.
- A PR request does not authorize merging, direct integration-branch pushes,
  rewriting shared history, unrelated work or sibling edits.
- A selected standard release request authorizes its documented commit/tag/push
  effects. Package publication, deployment and cleanup need separate requests.
  Preparing a release or changing these rules is not a request to execute one.
- The current release runner uses direct delivery. PR contributions do not
  select PR release delivery or bypass protected branches.
- Review the entire index before an authorized commit and stage only the scoped
  changes. If committed source is a prerequisite without Git authorization,
  report that boundary; never commit merely to satisfy a check.

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
