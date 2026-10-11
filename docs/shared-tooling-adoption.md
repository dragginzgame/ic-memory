# Memory tooling adoption and qualification boundaries

Read this when changing the shared snapshot, tool setup, release adapters,
formatting or evidence retention. It preserves the adoption history and local
selection boundaries previously embedded in AGENTS.md. The manifest owns exact
export identity; the live-policy exception in [AGENTS.md](../AGENTS.md) owns
active engineering policy. Historical shared-version descriptions do not freeze
that policy or establish new native qualification. Code paths are repository-relative.

The existing tooling snapshot at revision
`4fd979d13c32adb3cb2f51f213ebfc4f98339b3d` is recorded in
[.shared-tooling.snapshot](../.shared-tooling.snapshot). It identifies the vendored
files, hashes and executable modes; it does not freeze the active local policy.
Keep its provenance accurate and never edit vendored files in place. Do not
attribute uncommitted shared files to that recorded revision. The live local
[user-triggered maintenance rule](../rules/agent-maintenance.md)
also applies within its activation scope and is included in this recorded snapshot.
These instructions are the repository's local overlay.

The single 94-file snapshot uses reviewed committed Shared Tooling at that
revision, whose committed source `VERSION` is 0.6.3. The manifest records that
display version separately from its authoritative commit identity. This includes
the corrected producer version and the package-impact guidance: contributor-only
tooling changes use compatible library patch notes. It includes
the archiver, tool-evidence selector, their fixtures and the shared
retention action. It replaces the supplemental archive manifest and its duplicate
verifier records. The consumer collector owns product log selection and calls
the shared selector; full tool retention remains the command default, while CI
explicitly selects compact retention for freshly verified complete active sets.
Failed, unknown and unselected bundles remain full. Never attribute subsequent
dirty sibling fixes to this committed export.
Shared 0.3.6 makes all three hook index-tree observations preserve Git failure
status. Memory's consumer fixture qualifies matching-output failures separately
from normal two-workspace formatting. Its selected Cargo-tool installer also
supports explicit lockfile selection; Memory retains its existing independent
Testkit CLI version selection and both workspace lockfiles. The common hook
adoption checker is not selected; Memory owns its actual formatter integration.
Shared 0.3.7 explicitly fails mandatory portable assertions and rejects missing
Cargo host identity or non-executable candidates before activation. Memory's
routine CI adopts workflow/ref cancellation while preserving every required
host and gate; interrupted revisions remain unqualified on unfinished hosts.
Shared 0.3.9 runs Rust preflight, installed-tool probes and Cargo installation
from the selected consumer, preserving caller-relative catalogs, consumer-relative
lockfiles and explicit toolchain overrides. Its validation-output classifier is
not selected; Memory retains its own Rust validation and receipt adapters.
Shared 0.3.10 fixes host-fixture aliases and relative release-command fixture
roots, including nested Make probes. The 0.3.11 source corrects its earlier
version annotation; the snapshot records the current committed version and
exact revision. The derive-macro guidance is selected without adding derive
sorting to Memory's two-workspace formatting contract. The 0.3.12 common hook
adoption checker and its integration fixture remain unselected; Memory's owned
hook fixture continues to qualify its actual formatter integration.
The shared IC bundle contains five tools. PocketIC selection, provisioning,
offline admission and managed lifecycle belong to the published Testkit CLI,
selected independently in the root Makefile. `install-runtime-server` is explicit
network setup; `runtime-server-check` is offline; `test-runtime` qualifies the
supplied Wasm and independent runner with the prepared server. An old six-tool
IC bundle fails the new offline check and requires explicit `install-ic-tools`;
prior bundles, receipts and failure evidence stay retained. The runner reads
`IC_TESTKIT_POCKET_IC_URL`; no retired server-path alias remains.
Shared 0.3.4 selects Binaryen 133. Normal library Wasm builds and their budgets
remain unoptimized. Installed `test-runtime` separately qualifies the supplied
fixture and its `-O3`, `-Os` and `-Oz` outputs with the same startup, IO rejection
and upgrade rollback checks. All four bundles, optimizer/server logs and input
hashes remain retained; native CI owns Linux and both macOS host acceptance.
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
checks cover both approved workspace roots. The reviewed `make/tools.mk`
owns setup, offline tool checks and LOC commands. Its complete host set includes
pinned cloc; existing installations must refresh explicitly. Rust LOC defaults
to the root workspace; `CLOC_MANIFEST` explicitly selects the independent
qualification manifest without combining graphs. The retained local LOC fixture
owns its target/manifest isolation; consumer dispatch needs no target workaround.
Fleet/context fixture qualification stays upstream. The shared Rust-tool
installer and fixtures are available through explicit `install-rust-tools` and
`rust-tools-check`; the required aggregate now owns all twelve common
host/IC/Cargo executables in sequence. Product runtime-server setup remains explicit and separate; the
two-workspace formatter uses the selected local cargo-sort and pinned rustfmt.
Common Cargo executables use shared generations bound to the exact prepared
compiler, native environment, published source and packaged lockfile. The
standard tool-build environment excludes application flags, wrappers and Cargo
configuration; its managed source cache is required by offline checks. The
complete projection at `.tools/rust/common/bin` activates
only after all three pass; old installations, slots and failed builds remain.
CI and hook callers use that projection. Explicit setup follows host preparation
with `make install-rust-tools` and offline `make rust-tools-check`. Memory's
retention fixture covers successful slot builds/receipts and projection links;
the selected Testkit CLI version and PocketIC ownership remain unchanged.
Shared 0.6.1 propagates Cargo acquisition, admission and publication failures
explicitly, preserving the primary status, foreign locks and previous selections.
Shared Tooling owns the per-user host/IC/Cargo executable store, tracked in
[Shared #120](https://github.com/dragginzgame/shared-tooling/issues/120).
The canonical installers select verified shared generations through local
`.tools/host`, `.tools/ic` and `.tools/rust` references. Cargo tools retain
`.tools/rust/common/bin`; selected product CLIs use the installer-returned path.
Use the same `SHARED_TOOLING_STORE` setting for setup, offline checks and evidence
collection; never create a Memory-owned store or select a sibling PATH. The Cargo
build companion `scripts/dev/cargo-tool-build.sh` is explicitly selected alongside
`scripts/dev/tool-store.sh` and the shared installer fixture. Previous physical
local bundles remain retained but are no longer admitted as active tools. The
shared evidence selector owns the local Rust tree, reference resolution and physical shared
generation selection; Memory does not duplicate that selection. Memory's
round-trip fixture covers deduplication and exclusion of unrelated store entries.
The aggregate first performs read-only platform/catalog and Rust/Cargo
preflight through the existing installers before any installation. Host-tool
failures identify the selected tool, expected version, path and repair command.
The tool include also admits Make execution before dispatch and preserves Cargo
jobserver descriptors. Selected shared fixtures require explicit completion
before successful cleanup. The linked README freshness task is advisory only;
adoption enables no schedule, prose rewrite or release gate.
The Rust installer refuses redirected install/build/receipt paths before tool
execution and after Cargo returns. Tool-command fixtures normalize their physical
scratch root; the IC installer uses the included canonical pin parser.
The reviewed `make/release.mk` owns standard release entrypoints and conflicting
selection admission. Its `make/execution.mk` companion resolves the behavioral
probe beside the selected include, independently of runtime tooling-root values.
Memory retains direct-delivery admission, forced release-only
cache preparation, complete common-tool preparation/admission and all Rust metadata/validation/publication adapters. The
root-only formatting include is not selected: both maintained workspaces keep
Memory's explicit compiler and formatting coverage.
The release runner and its PR helper are exported unchanged. This consumer
selects direct delivery and rejects other delivery selections before dispatch;
PR release adapters and their real-Git qualification fixture are not adopted.
The canister audit addendum is linked guidance; product limits remain local.
The common metadata fixture also includes its workspace-version reader. Product
version parsing stays in the Rust release adapter. The linked tag-maintenance
guide is documentation only; no tag-maintenance executable is adopted. Release-note
finalization uses the unchanged shared helper; Memory retains saved-source and
payload admission plus its bare historical target-identity check. CI and release
checks use these immutable exports without a sibling checkout. The live
local policy exception above continues to govern subsequent uncommitted rule
development. The read-only `scripts/dev/gh-ci.sh` helper supports exact-commit
inspection across workflows; use authenticated GitHub CLI and the shared
maintenance rule's evidence checks. Relevant issue work in GitHub repositories
owned by `dragginzgame` follows that rule's standing authority; issue actions in
other GitHub repositories require explicit destination/action authorization.
The selected CI helper and its failure-evidence fixture report unavailable
failed-step logs and retain unsuccessful observations; successful runs may
legitimately have none. Inspection still does not authorize source repair.

Fleet inventory and exporter/verifier integration fixtures remain upstream-owned.
The consumer does not vendor the fleet fixtures or unused exporter. Never create
a fixture commit or alter the real index to satisfy a test prerequisite.

The shared Make execution checker guards the runner, hook and consumer Rust
validation adapter before gate dispatch. It refuses inherited Make modes that
skip execution or ignore failures, preserving ordinary selections and jobserver
settings. Qualify rejection through real Make with substituted release effects.
The canonical Shared 0.2.11 include and behavioral probe admit `MAKEFLAGS` and
GNU Make's generated `MFLAGS` independently before any recipe can dispatch.
They reject erased Makefile/command-line `MFLAGS` evidence and hidden unsafe
modes even when `MAKEFLAGS` is replaced. Memory's duplicate flag parser is removed;
actual consumer fixtures qualify rejection and ordinary recursive/parallel calls.
[Shared #30](https://github.com/dragginzgame/shared-tooling/issues/30) owns the
delivered common repair; Memory's native adoption remains separately qualified.

Follow the shared [contribution rules](../rules/contributions.md). Ordinary fixes
remain local; an explicit commit or PR request authorizes its scoped Git workflow.
Merge, integration-branch push, release, publication and deployment effects retain
their separate target/effect authorization. The maintainer-approved live-local
policy exception above remains; there is no blanket agent-commit prohibition.

