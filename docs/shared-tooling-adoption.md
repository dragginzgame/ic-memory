# Memory tooling adoption and qualification boundaries

Read this when changing the shared snapshot, tool setup, release adapters,
formatting or evidence retention. It preserves the adoption history and local
selection boundaries previously embedded in AGENTS.md. The manifest owns exact
export and policy identity; [AGENTS.md](../AGENTS.md) owns the local overlay.
Historical shared-version descriptions do not override that selected policy
or establish new native qualification. Code paths are repository-relative.

The existing tooling snapshot at revision
`3f87ddc1a0c1616775b2114c42a4adee7376c9b1` (Shared 0.20.0) is recorded in
[.shared-tooling.snapshot](../.shared-tooling.snapshot). It identifies the vendored
files, hashes and executable modes, including the adopted policy.
Keep its provenance accurate and never edit vendored files in place. Do not
attribute uncommitted shared files to that recorded revision. The adopted
[user-triggered maintenance rule](../rules/agent-maintenance.md) applies within
its activation scope. The former live-policy exception is retired; future policy
changes require reviewed snapshot adoption.
These instructions are the repository's local overlay.

## Stable-memory governance policy

The maintainer approved requiring checked `ic-memory` handles for production
stable stores. The local [agent instructions](../AGENTS.md#stable-memory-governance)
record that boundary alongside the [allocation safety contract](../SAFETY.md).
The common rule from
[PR #251 — Require IC Memory governance for production stable stores](https://github.com/dragginzgame/shared-tooling/pull/251)
is now adopted in the recorded Shared 0.20.0 baseline. Policy adoption does not
establish consumer implementation or qualification.

IC Storage 0.3 exposes governed collection wrappers consuming `RuntimeMemory`;
its generic engine remains private. Memory's 0.46 cut added exclusive
live leases and removed the blanket substrate re-export. Published consumer
adoption and qualification remain under
[Storage #1 — Adopt Memory 0.46 exclusive leases and root backing exports](https://github.com/dragginzgame/ic-storage/issues/1).
The host still owns exclusive backing custody; generic aliases cannot be
detected. Memory does not depend on Storage. This boundary changes no store
format or dependency selection and does not establish consumer qualification.

## Recorded snapshot adoption

The current selection has 71 explicit roots and 105 payloads from committed
Shared 0.20.0 at `3f87ddc1a0c1616775b2114c42a4adee7376c9b1`.
Memory no longer dispatches or vendors the generic Shared runner, parser,
installer, common Make, formatter-prerequisite, LOC and checksum regression
matrices. Canonical selection retirement removes twelve fixture roots and two
unused derived helpers; the directly used Cargo cache-key and CI inspection
commands are now explicit roots. Payload bytes remain at the same revision.

| Retained proof | Owner/caller |
| --- | --- |
| Snapshot hashes, modes and selected roots | Memory `shared-tooling-check` |
| Selected catalogs and installed native tools | Memory `tools-check`, formatter admission and native CI reuse |
| Actual pin policy/catalog | Memory `check-pins` against its two workspace inputs and exception catalog |
| Release source, receipts, rollback and Make dispatch | Memory repo-tool and release-adapter tests, with substituted Git/Cargo effects |
| Actual index formatting, setup and failure isolation | Memory hook fixture |
| Archive selections, transport bindings and retained failures | Memory failure-evidence and retention fixtures |
| Generic implementation matrices | Shared Tooling `scripts/ci/` and producer CI |

`test-host-tools.sh` remains selected solely because Memory's failure-evidence
fixture forces an early real installer refusal and checks that its original
status/logs survive the actual collector/archive round trip. It does not run the
full host matrix. Consumer hook tests retain missing/wrong formatter admission.
No product test, installed tool check, full release gate or native host roster is
removed. No CI duration reduction is claimed without comparable timings.

The preceding adoption initially selected 81 roots and 119 payloads.
Canonical preview and refresh used an isolated clean checkout of that exact
revision; the sibling's uncommitted 0.20.1 work is not adopted. Seven new required
companions cover maintenance selection, version reporting, formatting evidence,
Make fixtures/jobserver admission, stable versions and release-delivery checking.

Shared's common setup and PATH now cover host/Cargo tools. Memory explicitly
selects `install-ic-tools` and `ic-tools-check` in the ordered local target lists
and exports its prepared IC path. CI already publishes that path after the same
setup/check pair; formatting needs only common Cargo tools. Consumer fixtures
copy the new delivery checker and stable-version module where their callers
require them. Host/Cargo/IC pins, Rust toolchains, both dependency lockfiles and
prepared tool build identities remain unchanged; no tool installation is needed.

The unused README renderer remains retired under
[#119](https://github.com/dragginzgame/ic-memory/issues/119). Memory does not
select the retired release-prose heuristic, fleet evidence reporter or standalone
YQ installation. Optional Cargo boundary and release-delivery data checks are
not enabled by this adoption; the existing local direct-delivery guard remains.

The earlier Shared 0.15.0 export retained the earlier payload closure and adds four
required companions over 0.12.4: Cargo publication, Rust toolchain and CI
change-selection guides, plus canonical release version/date admission. The runner and increment
helper use that admission; Memory retains independent parsed metadata, receipt,
source and recovery checks. The temporary 0.12.5 validation-policy reference is
retired in favor of the now-selected local baseline. Tool pins, installer/build
identities, Make and formatter entrypoints are unchanged, so this refresh requires
no tool reinstall. Fixture completion and retention use the updated shared
helpers, including status and evidence preservation. The upstream 0.15.0 changelog
also describes open PR work; only bytes in the exact committed export are adopted.

Memory's native release adapter directly invokes the canonical version/date
admission helper. The launcher also selects the workspace-version reader directly,
bringing the initial selection to 82 explicit roots with 113 payloads at that
source revision before the README renderer retirement. It shares manifest admission/rendering and stable-version admission;
Memory retains lock-package selection, scratch custody and network scope.
The adapter parses the helper's typed projection while
retaining independent release input, receipt and recovery binding. No executable
payload changes or prepared-tool refresh is required for that selection change.

The 0.12.2 refresh directly selects `fixture-retention-probes.sh`. Memory's
retention checker supplies its own fixture roster and retains checker-refusal
and archive round-trip coverage. Shared owns the seven-case injection/status/
evidence loop; callers keep physical anchoring and product-specific assertions.
The canonical evidence action delegates tool selection/archive orchestration to
the derived collector. The finalizer requires numbered candidates at the top;
Memory keeps its saved-source renderer and receipt/rollback boundaries. Tool
pins, compiler/MSRV, native Host selection and the independent runtime graph are
unchanged. The 0.12.4 selection adds the advisory README dependency renderer
as a directly used root. It preserves unsupported or ambiguous examples and
reminders; Memory retains independent release identity and receipt admission.
Producer full-suite failures remain separate from focused renderer and consumer
qualification, tracked by [Shared #179 — Correct dashboard width fixtures](https://github.com/dragginzgame/shared-tooling/issues/179).

The single 112-file snapshot uses reviewed committed Shared Tooling at that
revision, whose committed source `VERSION` is 0.15.0. The manifest records that
display version separately from its authoritative commit identity. The 81 explicit
roots select the reviewed governance catalog and Memory's maintained Make, hook, CI,
archive and fixture entrypoints. Their generated companion closure preserves
the directly invoked canonical local-package lock rewriter and the baseline-linked
change-reporting rule. The common host/Cargo and
IC catalogs are required installer companions, replacing their two redundant
explicit roots. All versions and platform digests match the prior selection;
Memory needs no exception catalog. The 0.11 refresh replaces the two CI installer
wrapper roots with `install-ci-tool.sh`, and the artifact-discovery root with
`download-evidence.sh`. Automatic retirement removes the three unchanged owned
files. The new downloader and 0.11.1's derived annotated-tag checker and fixture
lifecycle helper keep the closure at 103 payloads.
Both helpers are now explicit entrypoints because Memory's native tag adapter
and two local basic fixtures invoke them directly. Shared 0.11.2 corrects cleanup
refusal, diagnostic failure and physical-directory anchoring; Memory preserves
hook-specific diagnostics and its caller-retained round-trip mode. The new Cargo
publication-admission helper remains unselected; Memory keeps its receipt-bound
publication owner.
The directly invoked pure workspace-version renderer and the Cargo cache-key
helper's derived catalog-record reader extend the closure to 105 payloads.
Memory independently admits the rendered manifest's parsed meaning before
installation, retaining exact saved-source recovery and its local transaction.
The optional fleet/CI catalog-binding checker remains unselected.
Its exact upload ID/digest, repository/run and explicit token remain bound by
Memory's one-upload workflow fixture. Shared's two-upload downloader fixture
remains unselected and qualifies at its producer; no exporter/fleet fixture is
adopted. The Rust fixture's cache-key companion remains selected.
Make, standalone setup, formatting and failure evidence use the executing
snapshot's catalogs. The consumer collector selects those catalogs even when
collecting a different repository root. CI's native reuse checks exercise the
standalone defaults rather than overriding them. Compiler/MSRV, application
dependencies and Testkit/PocketIC selections remain consumer-owned.
The 0.10.1 Cargo cache-key fix ignores comments, record order and unrelated pins
only for proven literal catalogs, retaining build/compiler/native identities
and conservative byte-based keys for other syntax. The 0.10.1 refresh changed four
payloads and no tool selection. Shared's 0.10.2 sibling-dashboard fix is outside
Memory's selected closure. The isolated export uses only committed source;
subsequent dirty producer changes remain unadopted. The scoped renderer work
under Shared #159 is included only through the published 0.11.4 commit; no dirty
source is attributed to this export.
The 0.9.7 Cargo build-policy change requires explicit prepared-tool refresh;
pins remain unchanged. Subsequent refreshes reuse the manifest's recorded
roots. Shared 0.8.0 automatically removes unchanged obsolete owned files outside
their closure; `--retire-file` is removed. Review canonical preview and local
callers/other-bundle ownership before refreshing, and keep directly used helpers
as explicit entrypoints. Earlier refreshes retained all payloads; the current
0.11 retirements are recorded above. Follow the selected
[snapshot procedure](consuming-snapshots.md#entrypoint-selection-in-070), keeping
root intent distinct from derived payloads and independently reviewing local
callers and other bundle ownership. Memory has one shared payload manifest.
Shared 0.8.1 enforces exact snapshot fields and retains both build-identity
observations on tool-installation refusal. Memory's full/compact, Rust-only and
hosted archive checks preserve those four identity files once, without a failed
generation acquiring an activation receipt. Tool pins/build identity are unchanged.
The two mandatory integrity helpers and the dependency checker's private jq
program derive from their canonical owners instead of redundant root records.
Consumer-invoked release helpers remain explicit roots; all payloads are retained.
Shared 0.8.2 adds advisory Cargo-tool freshness to the task catalog. Run its
read-only checker from Shared Tooling; the consumer snapshot selects the linked
guidance without importing the central checker or adding a network gate/schedule.
Shared 0.8.3 admits both observed diagnostic leaves before and after Cargo,
reads formatting pins from the selected Rust catalog, and declares common Make
installer/LOC companions. Memory's roster remains unchanged. Canonical refresh
replaces individual payloads atomically after rechecking conflicts; the consumer
ignores its abandoned `.shared-tooling-payload.*` scratch files.
Shared 0.8.4 adds complete JSON output to the local workspace LOC reporter,
with independent manifest selection and failure status preserved. Memory keeps
the default `make cloc` table; `bash scripts/dev/cloc.sh --json` selects the new
projection directly. Fleet aggregation stays upstream. The snapshot guidance
now consistently describes derived companions instead of manual file selection.
Shared 0.9.0 makes this verifier the common manifest syntax reader for refresh
and fleet reports. `--read-manifest FILE` emits complete admitted records without
payload authentication; ordinary verification retains its independent integrity
checks. Both modes reject NUL bytes before parsing the same captured manifest.
The v1 format, consumer commands and payload roster are unchanged. Memory does
not import the central exporter or fleet reporters.
Shared 0.9.1 consolidates host/IC lock acquisition, verified-cache reuse and exit
cleanup in the shared store. Both lock releases are attempted on exit; the
original installation status wins over cleanup failure, while cleanup alone
still fails the command. Retained candidates and prior selections survive, and
prepared bundles need no reinstall. Memory retains the same roots and payloads.
Shared 0.9.2 captures checksum manifests once and rejects NUL before hashing
their references. Host/IC/Rust installer admission rejects supplied and resolved
CR/LF paths before configuration loading or setup, while ordinary logical aliases
remain supported. The reviewed exports retain existing pins and prepared tools.
Shared 0.9.3 centralises host/IC asset downloads with bounded curl transient
retries. Permanent download and checksum failures remain terminal; failed
candidates and prior selections remain retained. The upstream real-curl loopback
fixture stays Shared-owned; Memory's adopted installer fixtures exercise final
HTTP failure propagation and retention without adding another fixture root.
Shared 0.9.4 admits only complete 40- or 64-character lowercase Git identities
in snapshot parsing, refresh observations and release recovery. The selected
simulation fixture proves malformed saved source identities fail before effects
without replacing the plan. Memory retains direct delivery; the PR helper is
exported unchanged, while its real-Git producer fixture remains unselected.
Shared 0.9.5 adds exact-ID artifact discovery as a directly used CI entrypoint.
Memory's owned workflow supplies this attempt's upload ID, repository/run and
explicit token, and bounds the read-only gate to two minutes before the existing
download/digest/payload checks. The producer retry fixture assumes Shared's
two-download workflow, so it stays upstream; Memory's failure-evidence fixture
owns its one-download bindings without duplicating the retry policy.
Shared 0.9.6/0.9.7 admit complete JSON PR identities and separately retain raw
repository observations before shell extraction. Memory keeps direct delivery;
the PR producer fixture remains unselected. Shared 0.9.7 also validates user
Cargo offline-policy types before source/setup dispatch. `make install-rust-tools`
explicitly selects prepared builds under the new helper digest; previous builds
and cache evidence remain retained. Source adoption, prepared-tool activation
and native qualification remain separately recorded under Memory #62.
Shared 0.9.8 derives its read-only Cargo CI cache-key companion from the selected
Rust-tool fixture. Producer native cache transport and portable-suite timing
remain Shared-owned. Shared 0.9.10 qualifies the fixture from its neutral
consumer directory and borrows existing immutable Git history, refusing
commit/tag/push effects. This addresses
[Shared #143](https://github.com/dragginzgame/shared-tooling/issues/143) and
[#144](https://github.com/dragginzgame/shared-tooling/issues/144) without a local
workaround or gate skip. Previous failed qualification remains recorded in
[the release-workflow evidence](release-workflow-qualification.md#shared-098-source-adoption-and-qualification-limits).
The new baseline-linked change-reporting rule is the added explicit policy root.
Shared 0.9.11 preserves filesystem lock failures and Cargo source-acquisition
statuses. Its changed Cargo build-helper digest requires explicit
`make install-rust-tools` followed by `make rust-tools-check`, retaining prior
generations. Shared 0.9.12 includes separately preparing missing locked sources
within authorized development/validation; explicit offline settings remain
authoritative. Shared 0.9.13 refuses unobservable implicit CI branch selection;
use an explicit branch, commit or run instead. The current snapshot also includes
0.9.14's selected-attempt failed-job log recovery and 0.9.15/0.9.16's maintenance
and LOC path admission and release-lock diagnostics. Later dirty Shared changes
remain live guidance, not bytes attributed to this committed snapshot.
This includes
the corrected producer version and the package-impact guidance: contributor-only
tooling changes use compatible library patch notes. It includes
the archiver, tool-evidence selector, their fixtures and the shared
retention action. It replaces the supplemental archive manifest and its duplicate
verifier records. The consumer collector owns product log selection and calls
the shared selector; full tool retention remains the command default, while CI
explicitly selects compact retention for freshly verified complete active sets.
Failed, unknown and unselected bundles remain full. Never attribute subsequent
dirty sibling fixes to this committed export.
Shared 0.6.5 adds exact-worktree context for staged Rust receipt checks, with
compiler/catalog selection still owned by the export and read-only admission
against the original prepared tools. The hook resolves its temporary export
physically. Shared 0.6.6 selects cargo-sort-derives 0.14.1 for literal macro-path
preservation. Refresh the common Rust selection explicitly; prior slots and
failure evidence remain retained. Memory's formatter sequence is unchanged and
does not run derive sorting; its development compiler already exceeds the tool's
Rust 1.88 build floor. The library MSRV remains independently qualified.
The optional shared validation checkpoint runner is not selected; Memory keeps
its existing Make stages and Rust release-input/receipt admission. The common
fixture construction guidance is selected without introducing recipe overrides.
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
Memory selects `make/consumer.mk` as its common entrypoint and retains
direct-delivery admission, forced release-only cache preparation, complete
common-tool preparation/admission and all Rust metadata/validation/publication
adapters. The selected formatting module receives Memory's paired complete
commands for both workspaces, preserving its explicit compiler and coverage.
Common snapshot and formatter admission use `shared-tooling-check` and
`format-tools-check`; CI and current guides select those names.
The release runner and its PR helper are exported unchanged. This consumer
selects direct delivery and rejects other delivery selections before dispatch;
PR release adapters and their real-Git qualification fixture are not adopted.
The canister audit addendum is linked guidance; product limits remain local.
The common metadata fixture also includes its workspace-version reader. Product
version parsing stays in the Rust release adapter. The linked tag-maintenance
guide is documentation only; no tag-maintenance executable is adopted. Release-note
finalization uses the unchanged shared helper; Memory retains saved-source and
payload admission plus its bare historical target-identity check. CI and release
checks use these immutable exports without a sibling checkout. The read-only
`scripts/dev/gh-ci.sh` helper supports exact-commit inspection across workflows; use authenticated GitHub CLI and the shared
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

