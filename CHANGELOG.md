# Changelog

## [0.35.1]

- Adopt Shared Tooling 0.3.1: common setup refuses unsupported hosts or unavailable
  Rust/Cargo before installation starts; host-tool failures report the selected
  tool, expected version, path and repair command.
  [#51](https://github.com/dragginzgame/ic-memory/issues/51),
  [Shared #101](https://github.com/dragginzgame/shared-tooling/issues/101).

[Detailed notes](docs/changelog/0.35.md)

## [0.35.0] - 2026-10-10

- **Breaking tooling setup:** adopt Shared Tooling 0.3.0's complete host, IC and
  Cargo toolset. Run `make install-tools`, then `make tools-check`; remove direct
  `--with-ripgrep`/`--with-cloc` installer options. Validation checks the complete
  set before builds, and release preflight prepares it.
  [#50](https://github.com/dragginzgame/ic-memory/issues/50),
  [Shared #98](https://github.com/dragginzgame/shared-tooling/issues/98).
- Report unavailable CI failure logs and retain partial observations instead of
  treating empty output as completed inspection.
  [Shared #97](https://github.com/dragginzgame/shared-tooling/issues/97).

[Detailed notes](docs/changelog/0.35.md)

## [0.34.1] - 2026-10-10

- Adopt Shared Tooling 0.2.13, remove the duplicate Make flag parser, and check
  required tools before builds. Release preflight prepares the selected host tools.
  [#42](https://github.com/dragginzgame/ic-memory/issues/42),
  [#47](https://github.com/dragginzgame/ic-memory/issues/47),
  [#48](https://github.com/dragginzgame/ic-memory/issues/48).
- Refresh release-helper libraries to IC Host 0.11.0 and validate working-tree
  packages without requiring a development commit. Release source stays checked.

[Detailed notes](docs/changelog/0.34.md)

## [0.34.0] - 2026-10-10

- **Breaking:** components request permanent keys and owners from one host-owned
  allocation pool. Replace numeric declarations/ranges and alternate open helpers;
  bootstrap now requires explicit namespace grants and physical exclusions,
  and custom validation errors are carried directly.
  Existing ledger bytes and key-to-ID bindings remain unchanged.
  [#44](https://github.com/dragginzgame/ic-memory/issues/44).
- Adopt Shared Tooling 0.2.10 with concise two-workspace formatting output and
  retained failure logs. [#48](https://github.com/dragginzgame/ic-memory/issues/48).
- Reject hidden unsafe Make modes and authenticate exact-ID CI artifact readback.
  [#47](https://github.com/dragginzgame/ic-memory/issues/47),
  [Shared #93](https://github.com/dragginzgame/shared-tooling/issues/93).
- Update README dependency examples automatically during release preparation;
  missing or ambiguous examples produce reminders instead of blocking release.
  [#49](https://github.com/dragginzgame/ic-memory/issues/49).

[Detailed notes](docs/changelog/0.34.md)

## [0.33.4] - 2026-10-09

- Refresh Shared Tooling for checkout-local Make admission, recursive invocation
  and recorded snapshot version/revision, preserving release routing and cache policy.
  [#42](https://github.com/dragginzgame/ic-memory/issues/42),
  [Shared #30](https://github.com/dragginzgame/shared-tooling/issues/30).
- Restore GNU Make 3.81 parsing for developer and release commands while keeping
  forced release settings. [#45](https://github.com/dragginzgame/ic-memory/issues/45).
- Adopt IC Host 0.10's consolidated durable writer while retaining producer,
  cleanup and publication-state errors in release tooling.
  [#46](https://github.com/dragginzgame/ic-memory/issues/46).

[Detailed notes](docs/changelog/0.33.md)

## [0.33.3] - 2026-10-09

- Share standard release-command routing through Shared Tooling 0.2.6 while
  preserving checkout-local runner selection, release admission and locked cache
  preparation even with conflicting caller settings.
  [#42](https://github.com/dragginzgame/ic-memory/issues/42),
  [Shared #91](https://github.com/dragginzgame/shared-tooling/issues/91).
- Keep recovery bootstrap metadata in Cargo's exact selected target directory,
  including paths ending in newlines, while preserving parser failures.
  [#43](https://github.com/dragginzgame/ic-memory/issues/43).

[Detailed notes](docs/changelog/0.33.md)

## [0.33.2] - 2026-10-09

- Adopt Shared Tooling 0.2.5's required-helper declarations so incomplete reusable
  test selections are rejected before replacing consumer files.
  [Shared #73](https://github.com/dragginzgame/shared-tooling/issues/73).
- Preserve literal hook selections and support formatting-hook setup and execution
  in checkout paths ending in newlines.
  [Shared #89](https://github.com/dragginzgame/shared-tooling/issues/89).
- Refresh host-side release-file dependencies to IC Host 0.9.3.
- Reject changed release tags before package publication and when checking its
  completion, preserving evidence without automatically retrying publication.
  [#41](https://github.com/dragginzgame/ic-memory/issues/41).

[Detailed notes](docs/changelog/0.33.md)

## [0.33.1] - 2026-10-09

- Preserve untouched metadata files and their permissions when release
  preparation fails before publication.
  [#39](https://github.com/dragginzgame/ic-memory/issues/39).
- Allow empty pending note bodies during release preparation while retaining
  release identity and qualification checks.
  [#40](https://github.com/dragginzgame/ic-memory/issues/40).

[Detailed notes](docs/changelog/0.33.md)

## [0.33.0] - 2026-10-09

### Breaking

- Run fleet LOC/tooling reports in Shared Tooling; retire Memory's two copied
  reporters and local report offering. Local workspace `make cloc` stays available.
  [#34](https://github.com/dragginzgame/ic-memory/issues/34).

### Fixed

- Adopt Shared Tooling 0.2.2's exact-path CI tool publication, preserving files
  and failed candidates when the executable destination changes during setup.
  [Shared #88](https://github.com/dragginzgame/shared-tooling/issues/88).

[Detailed notes](docs/changelog/0.33.md)

## [0.32.0] - 2026-10-09

### Breaking

- Adopt Shared Tooling 0.2.1's five-tool IC bundle and transfer PocketIC setup
  to IC Testkit. Rerun `make install-ic-tools`, then prepare runtime qualification
  separately with `make install-runtime-server`. The runner now consumes
  `IC_TESTKIT_POCKET_IC_URL`. Prior bundles and evidence remain intact.
  [#38](https://github.com/dragginzgame/ic-memory/issues/38).

### Fixed

- Reject multi-document dependency-exception catalogs and find checkout-local
  formatters during isolated hooks without requiring a shell PATH export.
  [#38](https://github.com/dragginzgame/ic-memory/issues/38).

- Keep release-workflow tests runnable when temporary paths already exist,
  preserving retained failure evidence and unrelated files.
  [#37](https://github.com/dragginzgame/ic-memory/issues/37).
- Refresh host-side release-file dependencies to IC Host 0.9.0.
- Install and check the final IC tool row even when a selected pin matrix has
  no final newline. [Shared #87](https://github.com/dragginzgame/shared-tooling/issues/87).

[Detailed notes](docs/changelog/0.32.md)

## [0.31.11] - 2026-10-09

- Preserve the original release-preparation failure and all restoration errors,
  while continuing to restore other owned metadata and preserving concurrent edits.
  [#36](https://github.com/dragginzgame/ic-memory/issues/36).
- Refresh host-side release-file dependencies to IC Host 0.8.9.

[Detailed notes](docs/changelog/0.31.md)

## [0.31.10] - 2026-10-09

- Prepare locked dependency caches during releases before compiling the helper,
  while preserving explicit offline settings and actionable failure messages.
  [#35](https://github.com/dragginzgame/ic-memory/issues/35).
- Preserve queued and running native CI for each main commit, while newer PR
  revisions can replace earlier review runs.
  [#33](https://github.com/dragginzgame/ic-memory/issues/33).
- Refresh host-side release-file dependencies to IC Host 0.8.6.
- Keep consumer tooling checks focused on local workspace integration; leave
  fleet regression fixtures with Shared Tooling.
  [#34](https://github.com/dragginzgame/ic-memory/issues/34).

[Detailed notes](docs/changelog/0.31.md)

## [0.31.9] - 2026-10-09

- Preserve exact Git index bytes when checking clean source, while retaining
  changed-path diagnostics and original Git errors. Reject unrelated release
  changes even when filenames contain only whitespace.
  [#31](https://github.com/dragginzgame/ic-memory/issues/31).
- Refresh host-side release-file dependencies to IC Host 0.8.3, preserving
  receipt and archive behavior.

[Detailed notes](docs/changelog/0.31.md)

## [0.31.8] - 2026-10-08

- Refresh host-side release-file dependencies to IC Host 0.8.2, preserving
  receipt and archive behavior.
- Restore the composed-host example link after the workspace relocation.
  [#32](https://github.com/dragginzgame/ic-memory/issues/32).

[Detailed notes](docs/changelog/0.31.md)

## [0.31.7] - 2026-10-08

- Keep native hook qualification working under inherited `CDPATH`, and adopt
  committed installer path corrections with explicit fixture dependencies.
  [#30](https://github.com/dragginzgame/ic-memory/issues/30),
  [Shared Tooling #67](https://github.com/dragginzgame/shared-tooling/issues/67),
  [#73](https://github.com/dragginzgame/shared-tooling/issues/73).
- Reject malformed active host/IC tool links and keep consumer release-runner
  qualification simulation-only.
  [Shared Tooling #75](https://github.com/dragginzgame/shared-tooling/issues/75),
  [#70](https://github.com/dragginzgame/shared-tooling/issues/70).
- Show Git status entries when clean-source checks refuse package or release
  qualification, making staged, working and untracked paths visible.
  [#31](https://github.com/dragginzgame/ic-memory/issues/31).
- Refresh host-side release-file dependencies to IC Host 0.8.1, preserving
  receipt and archive behavior.

[Detailed notes](docs/changelog/0.31.md)

## [0.31.6] - 2026-10-08

- Refresh host-side release-file dependencies to IC Host 0.7.1, preserving
  durable receipt and archive behavior.
- Keep release-adapter qualification working under inherited `CDPATH`.
  [Shared Tooling #67](https://github.com/dragginzgame/shared-tooling/issues/67).
- Reduce failed CI archives by retaining diagnostics for freshly verified active
  tool sets, while preserving failed and unselected bundles. Consolidate shared
  tooling into one snapshot.
  [#29](https://github.com/dragginzgame/ic-memory/issues/29),
  [Shared Tooling #66](https://github.com/dragginzgame/shared-tooling/issues/66).
- Refresh shared release tracking and tooling LOC handling.
  [Shared Tooling #62](https://github.com/dragginzgame/shared-tooling/issues/62),
  [#69](https://github.com/dragginzgame/shared-tooling/issues/69).

[Detailed notes](docs/changelog/0.31.md)

## [0.31.5] - 2026-10-08

- Archive failed CI evidence before upload, preserving Unix filenames, modes
  and symlinks. Keep evidence scripts working under inherited `CDPATH` and
  unusual checkout paths. Add native upload/download coverage for retained failures.
  [#28](https://github.com/dragginzgame/ic-memory/issues/28),
  [Shared Tooling #59](https://github.com/dragginzgame/shared-tooling/issues/59).

[Detailed notes](docs/changelog/0.31.md)

## [0.31.4] - 2026-10-08

- Refresh release-file support to IC Host 0.5.1, preserving streamed archive
  hashes and streaming receipt JSON through its durable publisher.
  [#27](https://github.com/dragginzgame/ic-memory/issues/27).
- Recheck release integrity after the final push hook and verify tags and
  destination history on completed retries.
  [#25](https://github.com/dragginzgame/ic-memory/issues/25),
  [Shared Tooling #58](https://github.com/dragginzgame/shared-tooling/issues/58).

[Detailed notes](docs/changelog/0.31.md)

## [0.31.3] - 2026-10-08

- Fix sibling LOC checks under trailing-slash and aliased temporary roots.
  Keep direct release delivery explicit when refreshing the shared runner.
  [#23](https://github.com/dragginzgame/ic-memory/issues/23),
  [Shared Tooling #57](https://github.com/dragginzgame/shared-tooling/issues/57).

[Detailed notes](docs/changelog/0.31.md)

## [0.31.2] - 2026-10-07

- Preserve native archive-copy errors and verify owned staging and parent
  identities through IC Host 0.4.5
  ([Host #14](https://github.com/dragginzgame/ic-host-tooling/issues/14)).
- Fix macOS compilation of the durable release-file writer
  ([Host #18](https://github.com/dragginzgame/ic-host-tooling/issues/18)).
- Guard optional Rust-tool installation against redirected paths and fix
  tool-command fixtures under trailing-slash or aliased temporary roots.
  [#23](https://github.com/dragginzgame/ic-memory/issues/23),
  [Shared Tooling #54](https://github.com/dragginzgame/shared-tooling/issues/54),
  [#56](https://github.com/dragginzgame/shared-tooling/issues/56).
- Allow explicitly requested agent commits and pull-request contributions,
  preserving separate merge, integration-branch push and release authority.
  [#24](https://github.com/dragginzgame/ic-memory/issues/24).

[Detailed notes](docs/changelog/0.31.md)

## [0.31.1] - 2026-10-07

- Keep LOC fixtures independent of enclosing Cargo workspaces and inherited
  build output, and allow tooling tests before committing adoption. Remove
  the consumer's fixture workaround and unused distribution exporter.
  [#23](https://github.com/dragginzgame/ic-memory/issues/23),
  [Shared Tooling #47](https://github.com/dragginzgame/shared-tooling/issues/47),
  [#50](https://github.com/dragginzgame/shared-tooling/issues/50),
  [#53](https://github.com/dragginzgame/shared-tooling/issues/53).
- Exclude aliased Cargo build directories from Rust LOC/test reports and expose
  explicit optional Rust-tool setup/check commands through the shared include.
  [Shared Tooling #31](https://github.com/dragginzgame/shared-tooling/issues/31),
  [#51](https://github.com/dragginzgame/shared-tooling/issues/51).

[Detailed notes](docs/changelog/0.31.md)

## [0.31.0] - 2026-10-07

- **Breaking developer setup:** require the complete pinned jq/yq/ripgrep/cloc
  set. Run `make install-host-tools` to refresh existing installations before
  offline checks. Reuse common setup commands and add Rust workspace and sibling
  tooling LOC reports. [#21](https://github.com/dragginzgame/ic-memory/issues/21).
- Keep LOC test fixtures independent of inherited Cargo build directories, and
  select the root fixture's own workspace with checkout-local scratch.
  [Shared Tooling #47](https://github.com/dragginzgame/shared-tooling/issues/47).
  [#48](https://github.com/dragginzgame/shared-tooling/issues/48).
- Reject inherited Make modes that skip formatting or release validation, or
  hide failed gates, preventing false success and validation receipts.
  [#20](https://github.com/dragginzgame/ic-memory/issues/20).
- Retain qualified package archives through durable streaming publication,
  preserving recovery after publication errors and unrelated temporary files.
  [#22](https://github.com/dragginzgame/ic-memory/issues/22).

[Detailed notes](docs/changelog/0.31.md)

## [0.30.0] - 2026-10-07

- **Breaking checkout layout:** move Rust packages under `crates/` with virtual
  workspace roots. Point library path dependencies and package-specific manifest
  commands at `crates/ic-memory/`; existing root Make/Cargo commands retain their
  selection. The independent runtime qualification graph and lockfile stay separate.
  [#19](https://github.com/dragginzgame/ic-memory/issues/19).

[Detailed notes](docs/changelog/0.30.md)

## [0.29.0] - 2026-10-07

- **Breaking developer setup:** use the shared pinned jq/yq/ripgrep set for
  local checks and CI. Run `make install-host-tools` to refresh an existing
  jq/yq-only installation before offline validation.
- Guard release destinations and delegate release-file writes to durable host
  filesystem tooling, preserving recovery when publication reports an error
  after replacing a file.
  [#16](https://github.com/dragginzgame/ic-memory/issues/16),
  [shared-tooling #25](https://github.com/dragginzgame/shared-tooling/issues/25).
- Add exact-commit CI inspection through the shared GitHub helper.
  [#17](https://github.com/dragginzgame/ic-memory/issues/17).
- Preserve failed consumer release-test fixtures and report their locations,
  keeping receipts and command traces available for diagnosis. Failed native CI
  also uploads retained fixtures, tool candidates and validation logs.
  [#15](https://github.com/dragginzgame/ic-memory/issues/15).

[Detailed notes](docs/changelog/0.29.md)

## [0.28.4] - 2026-10-06

- Repair portable host-tool fixtures by restoring authenticated archive bytes,
  avoiding false checksum failures when archive headers change on macOS.
  [shared-tooling #17](https://github.com/dragginzgame/shared-tooling/issues/17).
- Reuse shared offline formatter prerequisites for the pinned cargo-sort and
  rustfmt, keeping formatting and hook checks across both workspaces.
  [#14](https://github.com/dragginzgame/ic-memory/issues/14).

[Detailed notes](docs/changelog/0.28.md)

## [0.28.3] - 2026-10-06

- Check Cargo inheritance in both workspaces and reuse the common CI installers,
  reducing duplicated setup code while preserving tool pins and release policy.
  [#13](https://github.com/dragginzgame/ic-memory/issues/13).

[Detailed notes](docs/changelog/0.28.md)

## [0.28.2] - 2026-10-06

- Keep release and hook fixtures independent of inherited Make includes and
  dry-run settings, preventing misleading failures during nested validation.

[Detailed notes](docs/changelog/0.28.md)

## [0.28.1] - 2026-10-06

- Reuse Shared Tooling’s release-command checker, reducing duplicated fixtures
  and expanding failure and conflicting-target checks.
- Refresh shared tooling with portable file hashes and safer IC tool receipts.

[Detailed notes](docs/changelog/0.28.md)

## [0.28.0] - 2026-10-06

- Use Shared Tooling’s pinned local host/IC setup and audit methods, replacing
  duplicate installers, tool selections and the generic audit checklist.
  [#11](https://github.com/dragginzgame/ic-memory/issues/11) ·
  [#12](https://github.com/dragginzgame/ic-memory/issues/12)
- Use ic-host-tools for release artifact hashes and digest parsing, removing
  the SHA-256 subprocess and output parser from the repository adapter.
- Provision the missing ripgrep prerequisite in native CI before running shared
  pin and tool fixtures.
- **Breaking developer setup:** replace the standalone yq installer with
  `make install-tools` (or `make install-host-tools`). Full validation now checks
  the local parser set. Library APIs, stable-memory formats and receipt schemas
  are unchanged.

[Detailed notes](docs/changelog/0.28.md) ·
[Adoption qualification](docs/audits/recurring/shared-tooling-adoption-2026-10-06.md)

## [0.27.3] - 2026-10-06

- Restore publication network access by keeping helper compilation offline without
  forcing its child commands offline. Explicit caller network settings remain
  respected; library APIs and durable formats are unchanged.

[Detailed notes](docs/changelog/0.27.md) ·
[Qualification](docs/release-workflow-qualification.md#0273-publication-network-scope)

## [0.27.2] - 2026-10-06

- Preserve tracked dependency selections in CI and release preparation; check
  dependency/action pins using the locally reviewed Shared Tooling checker.
- Keep hook checks working with tracked lockfiles and retain failed fixtures
  with diagnostics.
- Include the root lockfile in exact release metadata and interruption recovery,
  including a retry after its write precedes the manifest. Reject dependency
  drift and arbitrary staging. Library APIs and durable formats are unchanged.
  [#10](https://github.com/dragginzgame/ic-memory/issues/10)

[Detailed notes](docs/changelog/0.27.md) ·
[Qualification](docs/release-workflow-qualification.md#0272-tracked-dependencies-and-local-pinning)

## [0.27.1] - 2026-10-06

- Update shared release tooling so normal commands can finish an already-qualified
  interrupted release after newer fix commits, then validate the requested next
  increment. Preserve the original commit, tag, receipts and archives.
- Refuse final package qualification if HEAD changes during packaging, preserving
  prepared evidence instead of recording a package against the wrong commit.
- Reject unrelated or arbitrary staged changes during release preflight, even
  when working files have been restored. Library APIs, stable-memory formats and
  qualification receipts are unchanged.
  [#10](https://github.com/dragginzgame/ic-memory/issues/10) ·
  [Shared Tooling #5](https://github.com/dragginzgame/shared-tooling/issues/5)

[Detailed notes](docs/changelog/0.27.md) ·
[Release qualification](docs/release-workflow-qualification.md#0271-selected-commit-recovery)

## [0.27.0] - 2026-10-06

- **Breaking API and durable format:** retain current ownership and latest schema
  metadata, removing per-upgrade and schema history. Update history/timestamp
  callers and external DTO fixtures. Earlier ledgers are unsupported; retained
  installations need an explicit data disposition before deployment.
- Bound logical metadata to 64 KiB. Keep tombstones, stale-proof checks and
  protected commits; repeated upgrades no longer grow an audit trail.
- Remove the temporary heap buffer from manager validation and allocation
  diagnostics, using a bounded 32 KiB stack buffer.

[Detailed notes](docs/changelog/0.27.md) ·
[Ledger contract](docs/current-ledger.md) ·
[Focused qualification](docs/current-ledger-qualification.md)

## [0.26.2] - 2026-10-06

- Reject oversized ledger text during CBOR preflight, before allocating owned
  strings. Corrupt storage still fails closed; APIs and persisted bytes are
  unchanged.

[Detailed notes](docs/changelog/0.26.md) ·
[Codec qualification](docs/ledger-codec-qualification.md#0262-text-preflight-follow-up)

## [0.26.1] - 2026-10-05

- Bound ledger serialization while writing, so oversized commits return the
  existing limit error without first growing an oversized output buffer.
- Reduce recovery allocations by reserving admitted collection lengths, with
  room for next-generation staging. APIs and persisted bytes are unchanged.

[Detailed notes](docs/changelog/0.26.md) ·
[Codec qualification](docs/ledger-codec-qualification.md)

## [0.26.0] - 2026-10-05

- Use the shared maintainer release workflow for patch, minor and major releases.
  Rerunning the same target reconciles interrupted preparation and Git effects at
  the saved version and destination. Retain gate logs, receipts and archives.
- **Breaking maintainer workflow:** replace phase commands with the common release
  targets and use the current qualification receipt schema. Finish outstanding
  earlier releases with their original tooling before adopting it. Library APIs
  and stable-memory formats are unchanged.
- Finalize root and detailed changelog entries with the same saved UTC date.
- Fix hook-test setup that could incorrectly block release qualification after
  tooling adoption was committed.

[Detailed notes](docs/changelog/0.26.md) ·
[Focused qualification](docs/release-workflow-qualification.md)

## [0.25.14]

- Fix runtime memory bounds checks that could corrupt a neighboring allocation
  through cached or overflowing accesses.
- Reduce ledger encoding and generation-validation allocations, reject oversized
  declaration snapshots earlier, and warn when a pending bootstrap commit is
  discarded before persistence and confirmation.
- Add installed IO and upgrade rollback qualification. Durable formats and valid
  IO remain unchanged; bounds enforcement has a measured instruction cost.
- Require release preparation to match the numbered pending changelog entry.
- Add selected-file formatting hooks and matching manifest/Rust checks for both
  workspaces. Developer setup pins the formatter and activates the local hook,
  including through aliased checkout paths.

[Detailed notes](docs/changelog/0.25.md) ·
[Qualification and measurements](docs/runtime-io-qualification.md)

## 0.25.13

- Reject Cargo compiler and wrapper replacements during release qualification.
  Refuse `RUSTC`, `RUSTC_WRAPPER`, `RUSTC_WORKSPACE_WRAPPER`, `RUSTDOC`, their
  `CARGO_BUILD_*` aliases, and the corresponding `build` keys in discovered Cargo
  configuration files, including explicitly empty settings. Receipts continue
  to identify the declared toolchains; alternate compiler execution cannot pass
  as that qualification. Existing flags and profile settings remain recorded.
- Refuse preparation before validation or version mutation, and refuse final
  qualification, push and publication before dispatch, while preserving existing
  receipts and archives. Cover both Cargo configuration filenames, inherited
  configuration and empty/nonempty environment overrides. Ordinary job/profile
  configuration remains supported. Malformed Cargo configuration reports its
  path without exposing file contents.
- Public APIs, durable ledger bytes, dependency selection, toolchains and size
  budgets are unchanged. Consumers need no source adoption. No runtime performance
  or binary-size improvement is claimed.
- Focused validation: all 13 tooling tests pass on Rust 1.99.0 and 1.88.0;
  strict helper/test Clippy, formatting, workflow lint, ShellCheck, shared snapshot
  verification and whitespace checks pass. The three new override regressions
  fail against 0.25.12. Command effects are substituted; no live commits, tags,
  pushes or publication are exercised. Current-source full release gates and
  native CI qualification await the maintainer's committed source. The host
  support record separately documents the successful published 0.25.12 CI run.

## 0.25.12

- Select the declared Rust 1.88.0 toolchain explicitly for MSRV CI dependency
  preparation and all-target compilation. The 0.25.11 jobs installed 1.88.0,
  but the repository's `rust-toolchain.toml` selected 1.99.0 for the subsequent
  commands; their successful status does not qualify MSRV support.
- Refuse release preparation if the ignored lockfile changes during the second
  remote inspection after validation. Compare the mutation/rollback snapshot
  with the originally selected bytes before any version edit, dependency refresh
  or packaging. Preserve the separately changed lockfile and existing evidence.
  Add a regression that injects a valid changed dependency after validation.
- Refuse final package qualification when the prepared receipt identifies a
  different package HEAD or its retained archive is missing or corrupted. Check
  that evidence before invoking Cargo, preserving existing final receipts and
  artifacts on refusal. A failed final package remains retryable even after Cargo
  replaces its working archive, provided the retained prepared archive is intact.

## 0.25.11

- Adopt the reviewed Shared Tooling 0.1.0 snapshot at
  `41e1fd0ba41460bd2127cbf98ac8a4b2b2020d3e`, with offline integrity verification
  and the existing maintainer-owned commit/tag/push policy as the local overlay.
  Require minor releases for future breaking pre-1.0 APIs or semantics; preserve
  published history. Undecided release notes use one current `[Draft]` heading.
- Replace Python release and Wasm tooling with the Rust `repo-tool` development
  example. Replace `test-release-flow` and `test-wasm-size` with `test-tooling`;
  tests substitute command effects and create no commits, tags, pushes or live
  publications. Maintainer release commands retain their names and ownership.
- Require a selected lockfile and explicitly prepared cache before offline
  validation. Bind prepared/final release evidence to source and release Git
  identities, dependency selection, compiler/build configuration, qualification
  commands and SHA-256 package hashes. Qualify the final archive after the
  release commit, preserving Cargo's embedded Git metadata. Publication refuses
  missing or changed evidence and never regenerates a lockfile. Failed
  preparation restores version surfaces and retains artifacts and old receipts.
- Pin CI actions, restrict token permissions, disable checkout credentials, add
  timeouts/concurrency control and checksum-verified workflow/shell lint tools.
  Configure native macOS 15 ARM64/x86-64 jobs alongside Ubuntu 24.04 and document
  host prerequisites. Native macOS execution, live release effects and downstream
  adoption remain unqualified until separately observed.
- Tooling qualification: eight deterministic workflow/artifact tests,
  strict Clippy including test code, Rust 1.88.0 helper compilation, formatting,
  workflow lint, ShellCheck and shared snapshot verification pass offline.
  Full current-source release gates await the maintainer's committed clean source;
  earlier sorting-only measurements and checks below retain their original scope.
- Canonicalize fixed declarations and range registrations with in-place
  unstable sorting, matching the existing request ordering. Remove stable-sort
  scratch buffers while retaining the original comparators: accepted keys are
  unique, equal fixed comparator keys reject as duplicate slots, and equal
  range bounds reject as overlaps. Refusal order and canonical fingerprints
  remain unchanged.
- Exercise independently reordered 245-declaration/245-range snapshots with
  distinct labels, schema versions, modes and purposes. Verify equal fixed sort
  keys with different label/schema metadata still produce the same duplicate
  error. These behavior-preservation tests pass against 0.25.10 as well; existing
  overlap-order and pinned fingerprint tests remain unchanged.
- Public APIs, durable encodings, recovery bounds and allocation policy are
  unchanged. Consumers need no source adoption. IC instruction and cycle deltas
  were not measured.
- Before tooling adoption, matched Rust 1.99.0 builds of 0.25.10 source and the
  sorting candidate, using the same
  lockfile and size profile, reduce all five raw Wasm probes by 607–615 bytes.
  Core decreases from 241,889 to 241,274 bytes; runtime integration decreases
  from 250,627 to 250,012 bytes. All existing size ceilings remain unchanged.
- Sorting-change validation before tooling adoption: 276 library tests,
  public integration/composed-host tests, all
  eight compile-fail cases and six active doctests pass. Strict all-target Clippy,
  Rust 1.88.0 all-target compilation, warning-denied Rustdoc, formatting,
  whitespace checks, Wasm test compilation, package verification and all five
  raw Wasm size gates pass. Downstream builds and deployments were not qualified.

## 0.25.10

- Check ledger allocation-record and generation-history counts before visiting
  schema histories. Subtract each schema-history length from the remaining
  aggregate capacity and stop at the first excess instead of eagerly summing
  every history. Preserve the existing allocation/generation/schema limit
  refusal order and exact `LimitExceeded` errors.
- Verify the exact 65,536-entry aggregate schema-history boundary and rejection
  of one additional entry in a separately bounded history. Exercise simultaneous
  limit violations through structural and committed integrity, staging and
  protected commit; rejected commits preserve both slots and recovered authority.
  These behavior-preservation checks also pass against 0.25.9.
- Public APIs, durable encodings, declaration fingerprints, recovery ceilings
  and policy/history checks are unchanged. Consumers need no source adoption.
  No runtime performance improvement is claimed.
- Validation: 274 library tests, public integration/composed-host tests, all
  eight compile-fail cases and six active doctests pass. Strict all-target Clippy,
  Rust 1.88.0 all-target compilation, warning-denied Rustdoc, formatting,
  whitespace checks, Wasm test compilation, package verification and all five
  raw Wasm size gates pass. Downstream builds and deployments were not qualified.

## 0.25.9

- Check `GenerationRecord` runtime fingerprints during JSON/CBOR decoding with
  the existing constructor rule. Accept explicit null or printable ASCII through
  256 bytes; reject missing fields, empty text, overlong text, non-ASCII text and
  control characters before records reach ledger recovery. Valid encodings,
  durable formats and sealed declaration fingerprints remain unchanged.
- Hard-cut `LedgerIntegrityError::DiagnosticMetadata` and remove the repeated
  fingerprint scan from committed ledger integrity validation. Generation
  records own fingerprint text validity; ledger bounds, allocation claims,
  generation ordering, chain links, staging and protected commit checks remain.
  Malformed persisted fingerprints now report `LedgerCommitError::Codec`.
- Add public constructor/decode error checks, printable-ASCII/256-byte round
  trips and an independent current CBOR shape assertion. Verify recovery,
  diagnostic recovery and explicit initialization refuse malformed fingerprints
  in either protected slot order without modifying either slot. The malformed
  decode and recovery-classification regressions fail against 0.25.8; the valid
  shape and metadata-bound assertions pass against both implementations.
- Validation: 272 library tests, public integration/composed-host tests, all
  eight compile-fail cases and six active doctests pass. Strict all-target Clippy,
  Rust 1.88.0 all-target compilation, warning-denied Rustdoc, formatting,
  whitespace checks, Wasm test compilation, package verification and all five
  raw Wasm size gates pass. No removed error-variant usage was found in inspected
  Canic, IcyDB or blob-service source; downstream builds and deployments were not
  qualified.

## 0.25.8

- Decode `DeclarationSnapshot` through its checked constructor and fingerprint
  setter. JSON and CBOR now reject excess declarations, duplicate keys/slots and
  malformed runtime fingerprints before snapshots reach policy or allocation
  validation. Retain count-before-uniqueness and slot-before-key refusal order,
  collection checks before fingerprint checks, required explicit fingerprint
  nulls, unknown-field rejection and the original declaration order.
- Hard-cut `DeclarationSnapshot::validate` and
  `AllocationValidationError::Snapshot`. Remove the repeated snapshot scan from
  allocation validation; construction and decoding own these invariants. Policy,
  historical claims, retirement, generation bounds, staging and protected commit
  checks remain. Valid encodings, durable ledger formats and sealed declaration
  fingerprints are unchanged.
- Consolidate malformed snapshot coverage at the public decoding boundary. Both
  collection and fingerprint regressions fail against 0.25.7 for JSON and CBOR;
  current shape, explicit-null, printable-ASCII/256-byte and full 255-slot
  round-trip assertions pass against both implementations.
- Validation: 271 library tests, public integration/composed-host tests, all
  eight compile-fail cases and six active doctests pass. Strict all-target Clippy,
  Rust 1.88.0 all-target compilation, warning-denied Rustdoc, formatting,
  whitespace checks, Wasm test compilation and all five raw Wasm size gates pass.
  Inspected IcyDB source has one diagnostic match on the removed error variant;
  adoption requires deleting that arm in `crates/icydb/src/error/bootstrap.rs`.
  No relevant removed API usage was found in inspected Canic/blob-service source.
  Downstream builds and deployments were not qualified.

## 0.25.7

- Decode `MemoryManagerAuthorityRecord` through its checked constructor. JSON
  and CBOR now reject invalid authority/purpose text before records reach
  registration or range-table assembly. Preserve field-specific constructor
  errors, required explicit purpose nulls, unknown-field rejection and current
  encodings; no durable format or declaration fingerprint changes.
- Hard-cut `MemoryManagerAuthorityRecord::validate` and remove repeated metadata
  scans from static range registration and range-table construction. The record
  owns text validity; registration retains its independent governance namespace
  restriction, and the table retains input-order overlap refusal and canonical
  range ordering. Checks on raw caller-supplied expected authorities remain.
- Consolidate malformed-record tests at the public decoding boundary. Extend
  printable-ASCII/256-byte round trips and pin the current CBOR shape. Verify
  decoded governance records cannot bypass external registration. The malformed
  metadata regression fails against 0.25.6 for both JSON and CBOR; valid metadata
  and shape assertions pass against both implementations.
- Validation: 272 library tests, public integration/composed-host tests, all
  eight compile-fail cases and six active doctests pass. Strict all-target
  Clippy, Rust 1.88.0 all-target compilation, warning-denied Rustdoc, formatting,
  whitespace checks and all five raw Wasm size gates pass. Inspected Canic,
  IcyDB and blob-service callers use constructors/accessors and need no source
  changes for the removed method; downstream builds and deployments were not
  qualified.

## 0.25.6

- Check optional declaration labels during JSON/CBOR decoding with the existing
  constructor rule. Accept explicit null or printable ASCII through 256 bytes;
  reject missing fields, empty labels, overlong text, non-ASCII text and control
  characters before declarations reach registration or reservation policy.
  Valid encodings, declaration fingerprints and durable ledger formats remain
  unchanged.
- Hard-cut `AllocationDeclaration::validate` and
  `AllocationReservationError::InvalidDeclaration`. Remove the forwarding
  reservation validator and repeated label scans from static registration,
  snapshot validation, reservation bootstrap and reservation staging. Keep
  snapshot count/uniqueness checks, fingerprints, policy order, historical claim
  checks, generation bounds, local staging and protected commit sequencing.
- Consolidate decoded-label regressions at the public decoding boundary. Keep
  constructor error classification, explicit-null and current CBOR encoding
  assertions, reservation count-before-policy checks and genesis conservation.
  The decode regression fails against 0.25.5, while the valid-label and current
  encoding assertions pass against both implementations. Correct the safety
  guide's stale reference to `SchemaMetadata::validate`.
- Validation: 273 library tests, public integration/composed-host tests, all
  eight compile-fail cases and six active doctests pass. Strict all-target
  Clippy, Rust 1.88.0 all-target compilation, warning-denied Rustdoc, formatting,
  whitespace checks and all five raw Wasm size gates pass. No relevant removed
  API usage was found in inspected Canic/IcyDB source; downstream builds and
  deployments were not qualified.

## 0.25.5

- Store optional schema versions as checked nonzero values. Construction and
  JSON/CBOR decoding reject version zero; valid encodings, required explicit
  nulls, current wire fixtures and declaration fingerprints remain unchanged.
- Hard-cut `SchemaMetadata::validate`,
  `DeclarationSnapshotError::SchemaMetadata`,
  `AllocationReservationError::InvalidSchemaMetadata` and
  `LedgerIntegrityError::InvalidSchemaMetadata`. Remove repeated schema-version
  checks from declarations, logical requests, reservations and ledger history.
  `SchemaMetadataRecord::new` now returns the record directly. Schema constructor
  errors, label checks, policy, history ordering and commit validation remain.
- Malformed schema versions in persisted ledger history now reject as
  `LedgerCommitError::Codec`. Verify protected recovery preserves both slots;
  retain decoded-label refusal tests at registry, allocation-validation and
  reservation boundaries. New schema-decode regressions fail against 0.25.4,
  while valid wire-encoding assertions pass against both implementations.
- Select Cargo's self-contained manifest schema to avoid Even Better TOML's
  false rejection of valid lint settings. Cargo configuration is unchanged.
- Hard-cut `validate_stable_cell_ledger_memory` and expose the existing
  `decode_stable_cell_ledger_record_from_memory` reader. Manual owners can retain
  its decoded record for recovery instead of validating, discarding it and
  decoding again. Empty memory remains unwritten; envelope bounds, typed decode
  failures and protected recovery requirements remain unchanged. Update manual
  integration guidance and consolidate wrapper assertions into the reader tests.
- Read ledger records for exports, commit diagnostics and doctor reports through
  the owned manager's virtual memory directly. Remove unused growth-state handles
  from these read-only paths; bootstrap persistence and application handles keep
  guarded growth. Extend borrowed, nonclone backing coverage to exports and doctor
  reports, preserving backing bytes.
- Validation: 276 library tests, public integration/composed-host tests, all
  eight compile-fail cases and six active doctests pass. Strict all-target
  Clippy, Rust 1.88.0 all-target compilation, warning-denied Rustdoc, formatting,
  whitespace checks and all five raw Wasm size gates pass. No removed API usage
  was found in inspected Canic/IcyDB source; downstream builds and deployments
  were not qualified.

## 0.25.4

- Establish stable-key grammar and usable range bounds during deserialization,
  sharing their existing constructor rules. `StableKey` retains its decoded
  string directly; `MemoryManagerIdRange` keeps private ordered bounds excluding
  sentinel ID 255. Valid JSON/CBOR shapes and declaration fingerprints remain
  unchanged.
- Hard-cut `StableKey::validate`, `MemoryManagerIdRange::validate`,
  `AllocationRetirement::validate` and `LedgerIntegrityError::InvalidStableKey`.
  Remove repeated key/range checks from declaration, record and retirement
  validation. Constructor key errors, diagnostic metadata, duplicate claims,
  authorization, history, recovery and persistence checks remain.
- Malformed keys and ranges now reject during decode. Invalid logical ledger
  keys report `LedgerCommitError::Codec` instead of a later integrity error;
  retirement and range records cannot carry invalid decoded identities into
  staging or registry admission. No compatibility forwarders or fallback readers
  are retained.
- Consolidate malformed-range tests at the decoding boundary and retain registry
  refusal tests for unchecked schema/purpose metadata. Pin current key/range
  encodings and verify malformed-key recovery preserves protected slots. Both
  new identity-decoding regressions fail against 0.25.3.
- Validation: 274 library tests, public integration/composed-host tests, all eight
  compile-fail cases and six doctests pass. Strict library/test Clippy, Rust
  1.88.0 library/test compilation, Wasm library compilation, warning-denied
  Rustdoc, formatting, whitespace checks and all five raw Wasm size gates pass.

## 0.25.3

- Branch bootstrap directly on the authoritative runtime lifecycle. Remove
  the derived `already_bootstrapped` flag and its separate conditional while
  preserving policy-identity validation, effect-free warm binding checks and
  publication only after persistence succeeds.
- Read virtual page counts directly from the owned manager for allocation
  accounting and ledger diagnostics. Size-only observations no longer construct
  `RuntimeMemory` handles or clone their shared growth owner. Keep persisted
  layout validation, live manager comparisons and guarded growth on actual
  memory handles.
- Public APIs, durable formats, diagnostic output and declaration fingerprints
  are unchanged. Consumers need no source adoption for these internal cleanups.
- Validation: 52 focused runtime, allocation accounting, default-runtime,
  admission, growth, public-runtime and composed-host tests, and strict
  library/test Clippy pass. Rust 1.88.0 library compilation, Wasm library
  compilation, warning-denied Rustdoc, formatting, whitespace checks and all
  five raw Wasm size gates pass.

## 0.25.2

- Reuse the owned range-authority input vector when validating and ordering
  records. Remove the second growing vector while preserving input-order
  metadata validation, inclusive overlap boundaries and the first conflicting
  range reported. Process only the accepted prefix so later input keeps its
  original refusal order.
- Exercise ascending, reversed and interleaved inputs across all 255 usable
  IDs, retaining authority names, modes, purposes and canonical CBOR output.
  The new behavioral regression also passes against 0.25.1.
- Public APIs, durable formats, declaration fingerprints and range policy
  behavior are unchanged; consumers need no source adoption for this cleanup.
- Validation: 67 focused slot/range, registry, logical placement, admission and
  adoption tests, strict library/test Clippy, Rust 1.88.0 library compilation,
  Wasm library compilation, warning-denied Rustdoc, formatting, whitespace checks
  and all five raw Wasm size gates pass. No runtime speedup is claimed.

## 0.25.1

- Hash sealed declarations' canonical CBOR directly into the existing FNV-1a
  state. Remove the temporary encoded fingerprint buffer and subsequent scan;
  retain the same fingerprint material, algorithm version and values.
- Resolve committed memory IDs from validated borrowed key text, removing the
  temporary owned `StableKey` from ID resolution and memory opens. Typed
  capability callers share the same lookup. Preserve grammar and reserved-key
  refusal before bootstrap checks, committed authority and missing-key errors.
  The refusal-order and memory-conservation regression also passes against
  the preceding implementation.
- Remove per-guide release review stamps and their manual release-checklist
  maintenance. Current guides describe the maintained implementation; package
  metadata owns the release version. Historical qualification and measurement
  records retain their exact versions and scope.
- Correct the safety guide's runtime owner to `MemoryManager<Rc<M>>`.
- Record published 0.25.0 Canic qualification: 32 focused Core tests, strict
  library/test Clippy and default-feature Wasm compilation. Separately record
  the published blob dependency alignment, strict adapter/consumer library
  Clippy, managed Fast-profile consumer build and embedded fixture regeneration
  and verification. Installed behavior and deployments remain unqualified.
- Public APIs, durable formats and diagnostic fingerprint values are unchanged.
  Validation: 59 focused tests across fingerprinting, key validation, capability,
  adoption, placement, admission, default/public runtime and doctor behavior;
  strict library/test Clippy, Rust 1.88.0 library compilation, Wasm library
  compilation, warning-denied Rustdoc, formatting, local documentation links
  and heading anchors, and whitespace checks pass. Core raw Wasm is 242,402
  bytes against 260,000; runtime integration is 251,440 against 270,000.
  Both focused size gates pass.

## 0.25.0

- Replace `AllocationSlot` and `AllocationSlotDescriptor` with one checked
  `MemoryManagerSlot`. Construct it with `MemoryManagerSlot::new(id)`; `.id()`
  returns a usable MemoryManager ID directly. Construction and deserialization
  reject sentinel ID 255 before the value reaches allocation execution.
- Remove the descriptor module, unchecked slot constructor, repeated slot
  validation and extraction assertions, and the unreachable invalid-slot
  integrity/retirement errors. Keep raw numeric-ID validation, ownership,
  authorization, duplicate claims, history validation and persistence ordering.
- Preserve the current nested slot encoding through private serde-only fields.
  Valid durable and diagnostic slot encodings, current format identifiers,
  checksums and declaration fingerprints are unchanged. Invalid slots now fail
  decoding rather than subsequent ledger integrity or retirement validation.
- Exercise construction and JSON/CBOR decoding across all 255 usable IDs and
  reject sentinel, out-of-range and malformed encoded slots. Update declaration,
  retirement and persisted-corruption regressions for the owning decode boundary;
  retain rejection conservation, doctor classification and retry checks.
- Update public callers, examples, current fixture naming and safety guidance.
  This is a source API hard cut: consumers must update their policy signatures,
  slot constructors and accessors. No aliases or superseded execution path remain.
- Validation: the complete Rust suite passes, including 272 library tests,
  public integrations, two composed-host regressions and six active doctests;
  all eight compile-fail cases pass, including the new checked-slot privacy
  boundary. Strict all-target Clippy, Rust 1.88.0 all-target compilation,
  warning-denied Rustdoc, formatting, whitespace checks, Wasm test compilation
  and all five raw Wasm size gates pass. Canic's isolated source candidate passes
  32 focused memory/ABI/metrics tests and strict Core Clippy. Published dependency
  adoption, the independent blob adapter and live deployments remain unqualified.

## 0.24.15

- Reuse the sealed snapshot's canonical fixed-key lookup and binary-search its
  original logical requests during admission membership checks. Remove the
  separate scan of copied fixed-declaration DTOs without adding another index,
  cache or source of truth.
- Keep governance membership explicit and scan earlier historical selections in
  callback order. Preserve accepted selections after a later latched failure,
  rejection precedence and the final resolve/validate/commit boundary.
- Add a behavioral regression covering unsorted fixed and logical inputs,
  descending selection order, unknown and reserved-namespace keys, and membership
  after failure. The regression also passes against the previous implementation.
- Update the recovered-admission guide. Public signatures, error payloads,
  durable and diagnostic formats, and declaration fingerprints are unchanged.
  No Canic source patch is required.
- Validation: the complete Rust test suite passes, including 273 library tests,
  compile-fail boundaries, public integrations, two composed-host regressions
  and six active doctests. Strict all-target Clippy, Rust 1.88.0 all-target
  compilation, warning-denied Rustdoc, formatting, whitespace checks, Wasm test
  compilation and all five raw Wasm size gates pass. Package/release workflows,
  consumer builds and live deployments were not rerun. Runtime performance was
  not measured.

## 0.24.14

- Make zero-page runtime growth return the current virtual extent after the
  shared reentrancy check. Skip capacity planning, backing IO and the upstream
  manager's redundant header write when no growth was requested. Positive growth
  retains its existing capacity reservation, refusal and persistence behavior.
- Extend existing growth regressions to cover zero-page calls through both the
  typed method and substrate `Memory` adapter, empty and grown memories, detached
  handles, backing refusal and reentry. The no-write assertion fails against the
  previous implementation and passes after the cleanup.
- Release decoded physical commit-slot records immediately after diagnostic
  recovery, before projecting normal exports or running doctor policy callbacks.
  Keep the independently recovered ledger and diagnostic evidence, read-only
  behavior, recovery errors and memory-measurement ordering unchanged.
- Remove four crate-private allocation-history mutation forwarders. Staging and
  corruption fixtures use the already crate-visible vectors directly; public
  read-only accessors, validation, lifecycle transitions and commit ordering
  remain unchanged.
- Document zero-page growth in the runtime API, operations guide and safety
  invariants. Public signatures, error payloads, durable and diagnostic formats,
  and declaration fingerprints are unchanged. No Canic source patch is required.
- Validation: all 272 library tests and two composed-host regressions pass.
  Strict all-target Clippy, Rust 1.88.0 all-target compilation, warning-denied
  Rustdoc, formatting, whitespace checks and all five raw Wasm size gates pass.
  Integration tests, doctests, package/release workflows, consumer builds and
  live deployments were not rerun for this candidate. Runtime performance was
  not measured.

## 0.24.13

- Decode the ledger stable-cell record once per cold bootstrap attempt and reuse
  it for recovery and staging. Remove the private cell-opening wrapper, the
  second panic-based decode and the complete record clone. Use capacity-checked
  `Cell` writes and publish committed authority only after persistence succeeds.
- Preserve fresh-cell acquisition before admission without committing genesis
  on rejection. Extend the existing regression to check that a second rejected
  attempt leaves the initialized root unchanged and a successful retry commits
  generation one. Existing-cell retries still decode persisted memory afresh.
- Move recovered allocation records and generation history into normal
  diagnostic exports instead of cloning them. Share the projection with the
  public borrowed constructor and doctor reports; retain doctor recovery evidence
  for validation and preserve memory-measurement and policy-callback ordering.
- Update runtime ownership and safety documentation. Public APIs, error payloads,
  durable and diagnostic formats, and declaration fingerprints are unchanged.
  No Canic source patch is required.
- Validation: the complete Rust test suite passes, including 272 library tests,
  compile-fail boundaries, public integrations, composed-host regressions and
  doctests. Strict all-target Clippy, Rust 1.88.0 all-target compilation,
  warning-denied Rustdoc, formatting, whitespace checks, Wasm test compilation
  and all five raw Wasm size gates pass. Package/release workflows, consumer builds
  and live deployments were not rerun. Runtime performance was not measured.

## 0.24.12

- Make the ledger cell local to each bootstrap attempt. Remove the runtime's
  cached cell, initialization guard and private persistence wrapper. Every retry
  now preflights persisted memory before recovery and admission; retain capacity
  checks and publish committed authority only after persistence succeeds.
- Extend the existing corruption regression to cover retrying the same runtime
  as well as reopening it. The same-runtime case fails with the old cache and
  passes with attempt-local cells, without rerunning admission or writing memory.
- Carry borrowed historical records in internal claim failures. Remove record
  index lookups and redundant ledger arguments from validation and staging error
  conversion. Preserve distinct declaration/reservation error precedence and
  the indexes needed for successful staging mutations.
- Keep declaration activation in staging after historical claim validation.
  Remove the single-caller record observation wrapper and redundant lifecycle
  branch. Reservations still activate only through declarations; retired claims
  remain rejected before mutation, and schema/last-seen observations are unchanged.
- Update runtime ownership and safety documentation. Public APIs, error payloads,
  durable and diagnostic formats, and declaration fingerprints are unchanged.
  No Canic source patch is required.
- Validation: all 272 library tests pass, including runtime retry, lifecycle,
  claim-conflict, current-format fixture and fingerprint regressions. Strict
  all-target Clippy, Rust 1.88.0 all-target compilation, formatting, whitespace
  checks and all five raw Wasm size gates pass. Integration tests, consumer builds
  and live deployments were not rerun.

## 0.24.11

- Keep doctor recovery results and physical diagnostics together until report
  construction. Remove independent optional recovery state and require concrete
  recovery evidence in the private ledger-export helper.
- Preserve empty, readable and corrupt storage classifications, read-only
  diagnostics, public report fields and durable formats. Extend existing tests
  to check evidence availability and agreement with the normal ledger export.
- Remove schema history's redundant lower-generation comparison. Its matching
  first entry and strict ordering already establish that bound. Preserve future
  generation, last-observation and malformed-history error precedence.
- Validation: focused doctor, ledger-integrity and registry regressions, strict
  all-target Clippy, formatting and whitespace checks pass. All five raw Wasm
  probes build within their unchanged size budgets. The complete Rust suite,
  consumer builds against this candidate and live deployments were not rerun.

## 0.24.10

- Disable unused constructor-priority support. Constructors still register hooks;
  the registry retains ownership of declaration and eager-initialization phases.
  Keep the constructor attribute used by ic-memory and Canic, while removing
  `link-section` from ic-memory's standalone dependency graph. No dependencies
  were upgraded and no Canic source patch is required.
- Public memory APIs, durable and diagnostic formats, and declaration
  fingerprints are unchanged.
- Validation: all 288 executed Rust tests pass, including current macro
  registration, compile-fail boundaries and doctests. Strict all-target Clippy,
  formatting, whitespace checks, Rust 1.88.0 all-target compilation, Wasm test
  compilation and all five raw Wasm size gates pass.
  Apple startup behavior was inspected in upstream source but not executed.
  Consumer builds and live deployments were not rerun for this candidate.

## 0.24.9

- Move historical selections' key, authority and schema into resolved
  declarations instead of copying fields from requests that are then discarded.
  Original sealed requests retain borrowed copy semantics through the same
  resolution loop. Placement order, current grant checks, schema preservation
  and final snapshot validation remain unchanged.
- Public APIs, durable and diagnostic formats, and declaration fingerprints are
  unchanged. No Canic source patch is required for these local changes.
- Validation: 272 library tests, six selected public runtime/configuration/macro
  integration tests and two composed-host regressions pass. Strict all-target
  Clippy, formatting, whitespace checks and all five raw Wasm size gates pass.
  Consumer builds, complete package verification and live deployments were not
  rerun for this candidate. Runtime performance improvements were not measured.

## 0.24.8

- Reuse the recovered ledger during declaration, reservation and retirement
  bootstrap staging instead of cloning its complete history. Share each staging
  implementation with the public borrowed API, which retains copy semantics.
- Move registry declarations, requests and ranges into snapshot sealing instead
  of cloning inputs that are immediately discarded. Logical resolution also
  transfers its completed declaration vector into the same builder. Retain
  borrowed public inputs, canonical ordering, error precedence and fingerprints.
- Extend reservation failure coverage to a conflict after an earlier item has
  been staged. Both borrowed staging and bootstrap leave the source ledger and
  protected store unchanged on failure.
- Public APIs, durable and diagnostic formats, and declaration fingerprints are
  unchanged. No Canic source patch is required for these local changes.
- Validation: 272 library tests, six selected public runtime/configuration/macro
  integration tests and two composed-host regressions pass. Strict all-target
  Clippy, formatting, whitespace checks and all five raw Wasm size gates pass.
  Consumer builds, complete package verification and live deployments were not
  rerun for this candidate. Runtime performance improvements were not measured.

## 0.24.7

- Remove the second grant lookup during fresh logical allocation. Placement
  already selects the lowest free ID from the declaring authority's current
  `Allowed` grants. Recovered assignments still require current authorization;
  final policy checks, historical occupancy and exhaustion behavior remain.
- Remove retirement staging's final bounds rescan. Its initial check already
  bounds records and schema history and reserves room for one generation;
  retirement preserves record and schema counts. Input checks, generation
  overflow handling and protected commit validation remain enforced.
- Use `Arc::make_mut` for application-only capability publication. Remove the
  manual unwrap-or-clone and unconditional replacement allocation. Retain
  isolation from shared validated and committed capabilities and preserve
  publication timing.
- Inline retirement bootstrap's single-caller staging/commit helper. Keep the
  public operation as the sequencing owner, with unchanged recovery, retirement
  error projection and protected commit checks.
- Remove snapshot sealing's temporary stable-key tree. Use canonical request
  adjacency and fixed-declaration binary search for duplicate detection;
  preserve mixed-conflict precedence, accepted ordering and fingerprints.
- Record focused qualification of the Canic 0.110.52 release source against
  published ic-memory 0.24.6: 14 native memory regressions and default-feature
  core Wasm compilation pass. Only ic-memory changed in the isolated dependency
  graph; the active Canic checkout and live deployment remain outside this proof.
- Public APIs, durable and diagnostic formats, and declaration fingerprints are
  unchanged. No Canic source patch is required for these local changes.
- Validation: 272 library tests, six selected public runtime/configuration/macro
  integration tests and two composed-host regressions pass. Shared-capability
  isolation, mixed duplicate-error precedence and retirement history bounds pass.
  Strict all-target Clippy, formatting, whitespace checks and all five raw Wasm
  size gates pass.
  Consumer builds, complete package verification and live deployments were not
  rerun for this candidate.

## 0.24.6

- Remove staged-ledger clones and recovery-proof round trips from declaration,
  reservation and retirement bootstrap commits. Share the checked commit
  operation while retaining caller-owned staged ledgers, predecessor validation,
  and capability publication only after persistence confirmation.
- Create new active and reserved records from borrowed declarations, copying
  only persisted key, slot and schema fields instead of cloning discarded labels.
- Remove the doctor's duplicate empty-cell success path. Use the maintained
  decoder's empty-memory behavior while preserving empty, readable and corrupt
  classifications and read-only diagnostics.
- Remove redundant authority, mode and purpose tie breakers from range
  canonicalization and delete the private range-mode ordering helper. Retain
  bound ordering, overlap-error precedence and declaration fingerprints.
- Remove the redundant range-start sentinel check. Ordered bounds and the
  existing end check still reject ID 255, including the singleton sentinel;
  reversed-bound errors retain precedence.
- Share one private allocation-count limit across declaration validation,
  record decoding, ledger integrity and reservation staging. Derive it from
  the usable ID domain; keep the 255-item ceiling and existing error ordering.
- Validate committed generation chains with one parent cursor starting at
  genesis. Remove the optional predecessor state and repeated defaulting;
  retain contiguous history, strict parent links and error precedence.
- Public APIs, durable and diagnostic formats, and declaration fingerprints are
  unchanged. No Canic source patch is required for these local changes.
- Validation: 272 library tests, six selected public runtime/configuration/macro
  integration tests and two composed-host regressions pass. Current byte
  fixtures, the pinned fingerprint, overlap diagnostics and sentinel boundary
  checks pass. Strict all-target Clippy, formatting, whitespace checks and all
  five raw Wasm size gates pass. Consumer builds, complete package verification
  and live deployments were not rerun for this candidate.

## 0.24.5

- Delete the four archived 0.12 runtime, construction and policy design
  documents, removing 818 lines of superseded guidance. Current architecture
  and safety documentation remain in `README.md`, `ADVANCED.md` and `SAFETY.md`.
- Record qualification of published 0.24.4 against an isolated IcyDB source
  snapshot: all 27 focused admission, error-projection, lifecycle and
  logical-memory tests pass. Installed lifecycle phases remain below the
  unchanged 12,750,000 instruction ceiling, with stable extents preserved.
  The receipt identifies the tested inputs; IcyDB's subsequent commit and
  dependency graph remain outside its scope. Canic qualification remains pending.
- This release changes documentation only. Public APIs, durable and diagnostic
  formats, and declaration fingerprints are unchanged.
- Validation: removed-document references, qualification evidence, receipt
  links and whitespace checks pass. The downstream results cover published
  0.24.4, not this candidate; producer test suites, package verification and
  live deployments were not rerun for this documentation-only change.

## 0.24.4

- Consolidate runtime governance filtering into one committed-capability
  operation. Remove the arbitrary prefix parameter and runtime forwarding
  helper; preserve shared-capability isolation and publication only after
  persistence confirmation.
- Remove the test-only CBOR map insertion wrapper. Build fixture maps directly
  with tuples and vector literals; retain unknown-field, missing-field and
  retirement-state rejection coverage.
- Remove the whitepaper, Lean model, mdBook/Nix/Lake scaffolding, all four
  associated maintainer targets, obsolete ignore rules and documentation links.
  Delete the superseded 0.6 protocol proposal. Current architecture and safety
  guidance remain in `ADVANCED.md` and `SAFETY.md`.
- Correct runtime ownership documentation for shared backing and lazy TLS
  construction with cached failures. Update the direct ledger writer's bound
  enforcement description. Record the published 0.24.3 IcyDB qualification:
  27 focused tests passed, including installed lifecycle and logical-memory
  recovery. That receipt covers 0.24.3, not this candidate; Canic qualification
  remains pending.
- Public APIs, durable and diagnostic formats, and declaration fingerprints are
  unchanged. No Canic source patch is required for these local changes.
- Validation: 270 library tests, three selected public runtime/configuration
  integration tests and two composed-host regressions pass. Strict all-target
  Clippy, formatting and whitespace checks, six Wasm budget-tooling regression
  tests, and all five raw Wasm size gates pass. Package contents were checked
  for removal of the whitepaper and Lean files; complete package verification,
  consumer builds and live deployments were not rerun for this candidate.

## 0.24.3

- Encode ledger CBOR directly into the final payload-envelope buffer. Remove
  the intermediate raw payload buffer and private codec unit type; share header
  construction with the public envelope writer. Durable bytes, checksums,
  recovery validation and ledger-byte limit errors remain unchanged.
- Reject unknown keys, mismatched slots and already-retired allocations before
  cloning ledger history during retirement staging. Retain input validation,
  staging bounds, generation checks and existing error precedence.
- Public APIs, durable and diagnostic formats, and declaration fingerprints are
  unchanged. No Canic source patch is required for these local changes.
- Validation: 270 library tests, eight public integration tests, seven
  compile-fail cases, two composed-host regressions and five doctests pass;
  five existing sketches remain ignored. Current byte fixtures, exact envelope
  comparison, oversized-commit conservation and retirement rejection checks
  pass. Strict all-target Clippy, Rust 1.88 all-target checking, warning-denied
  Rustdoc, Wasm test compilation and all five raw Wasm size gates pass.
  IcyDB's lockfile now selects released 0.24.2; consumer builds and live
  deployments were not requalified for this candidate. Runtime performance
  improvements were not measured.

## 0.24.2

- Track duplicate allocation slots with fixed occupancy arrays instead of
  general-purpose trees in declaration and ledger validation. Retain key and
  generation sets, decoded sentinel rejection and existing error precedence.
- Let `StaticMemoryRangeDeclaration::new` own external authority validation.
  Remove the registration-time recheck of its immutable checked input;
  reserved-authority, decoded-record and registry lifecycle checks remain.
- Validate retirement constructor keys once through `StableKey::parse`, then
  validate the supplied slot directly. Retain full retirement validation at
  decoded-input and staging boundaries, including key-before-slot errors.
- Public APIs, durable and diagnostic formats, and declaration fingerprints are
  unchanged. No Canic source patch is required for these local changes.
- Validation: 269 library tests, eight public integration tests, seven
  compile-fail cases, two composed-host regressions and five doctests pass;
  five existing sketches remain ignored. Extended existing tests cover
  duplicate-error precedence, malformed inputs and late range registration.
  Strict all-target Clippy, Rust 1.88 all-target checking, warning-denied
  Rustdoc, Wasm test compilation and all five raw Wasm size gates pass.
  Downstream builds and live deployments were not requalified.

## 0.24.1

- Resolve fresh logical requests by walking the declaring authority's ordered
  `Allowed` grants directly. Remove repeated per-ID grant searches; preserve
  lowest-free-ID placement, fixed claims, historical occupancy and exhaustion.
- Use one range-authority validator in runtime policy. Remove the preliminary
  lookup and second validation path; retain the exception for unclaimed fixed
  external slots when no user ranges exist. Governance ownership, strict grants
  and custom-policy callback ordering remain enforced.
- Simplify complete-coverage checking to one `u8` cursor. Remove widened state,
  impossible conversion errors and the redundant advancement condition;
  preserve precise gaps, ID 254 coverage and out-of-target error precedence.
- Public APIs, durable and diagnostic formats, and declaration fingerprints are
  unchanged. No Canic source patch is required for these local changes.
- Validation: 269 library tests, eight public integration tests, seven
  compile-fail cases, two composed-host regressions and five doctests pass;
  five existing sketches remain ignored. Strict all-target Clippy, Rust 1.88
  all-target checking, warning-denied Rustdoc and Wasm test compilation pass.
  All five raw Wasm probes pass their existing budgets. Downstream builds and
  live deployments were not requalified.

## 0.24.0

- Fix `StableKey::parse` to validate and store the same borrowed string. A
  stateful `AsRef<str>` implementation can no longer substitute a different
  value between validation and construction.
- Serialize static declaration registrations directly when computing sealed
  fingerprints. Remove the duplicate projection type and temporary projection
  vector; preserve the fingerprint material, bytes and algorithm version.
- Hard cut: remove `RuntimeGrowError::ManagerRefused`. Successful capacity
  admission and backing reservation establish the pinned manager's growth
  contract; an unexpected result is an internal invariant panic. Backing
  refusal, bucket exhaustion, arithmetic overflow and reentry remain typed
  errors, including retry and refusal conservation guarantees.
- Reject reservation batches exceeding 255 items before declaration validation
  or policy callbacks. Bootstrap and raw staging share one count rule; recovery,
  explicit genesis initialization and existing-store conservation remain.
- Hard cut: allocation validation, ledger integrity, staging, reservation and
  retirement errors carry `AllocationSlotDescriptor` directly. Remove all 17
  boxed slot fields and their allocations; update callers constructing or
  inspecting those fields to the current Rust API. Error variants and messages
  are unchanged.
- Construct range authority through ordered insertion and neighbouring overlap
  checks. Remove repeated full-table scans and sorts while preserving input
  validation order, first-overlap errors, canonical output and fingerprints.
- Durable ledger and diagnostic formats are unchanged.
- Downstream qualification remains outstanding. Canic was left untouched;
  consumer builds and live deployments were not requalified for this release.
- Validation: 266 library tests, eight public integration tests, seven
  compile-fail cases, two composed-host regressions and five doctests pass;
  five existing sketches remain ignored. Strict all-target Clippy, Rust 1.88
  all-target checking, warning-denied Rustdoc and Wasm test compilation pass.
  All five raw Wasm probes pass their existing budgets. The key and
  oversized-reservation regressions fail against their original implementations
  and pass after the fixes.

## 0.23.0

- Hard cut: retain `DiagnosticExport::from_ledger` as the sole ledger-export
  constructor. Remove `from_ledger_with_commit_recovery`,
  `from_ledger_with_memory_sizes` and
  `from_ledger_with_commit_recovery_and_memory_sizes`. Exporters fill the public
  observation fields directly. Runtime export and doctor share one recovered
  export path, measuring records without a temporary slot map or join. Protected
  recovery still precedes measurement; diagnostics remain read-only.
- Hard cut: remove `DiagnosticGeneration`. `DiagnosticExport::generations`
  contains `GenerationRecord` values directly. Diagnostic JSON/CBOR generation
  entries expose the record's fields directly instead of a nested `generation`
  object. Update readers and regenerate reports using the current shape; no
  alias or old decoder remains.
- Hard cut: remove `MemoryManagerRangeAuthority`'s `reserve`, `allow`,
  `reserve_ids`, `allow_ids` and their four `with_purpose` variants. Construct
  records with `MemoryManagerAuthorityRecord::new` using explicit modes, then
  compose them with `MemoryManagerRangeAuthority::from_records`. Remove private
  insertion forwarding and update maintained examples and behavior tests.
  Decoded-record validation, overlap rejection, ascending range order, purposes,
  complete coverage and error precedence remain. Remove builder-only coverage
  and consolidate duplicated constructor assertions.
- Remove the duplicate ledger-envelope prefix-length check. The checked prefix
  read preserves the same truncation error, minimum length and format-error
  precedence.
- Durable ledger encoding, checksums and declaration fingerprints are unchanged.
  No compatibility aliases, fallback readers or replacement framework are added.
- **Canic adoption is required after 0.23.0 publication:** update its ic-memory
  dependency and apply the prepared generation-reader patch together. Its public
  Candid response shape is unchanged. Requalify the published dependency graph
  and refresh/verify the embedded allocation peer before calling adoption
  complete. Canic remains on published 0.22 until this release is live.
- Validation: 261 library tests, eight public integration tests, the three
  maintained range examples, strict all-target Clippy, Rust 1.88 all-target
  checking, warning-denied Rustdoc and Wasm test compilation pass. All five raw
  Wasm probes pass their budgets for the main cleanup; the prefix-only follow-up
  passes all three focused envelope tests and strict all-target Clippy.
  Fourteen Canic memory tests and strict Core all-target/all-feature Clippy pass
  against the prepared reader in an isolated copy using the local candidate.
  Published-0.23 consumer adoption, installed-canister qualification and
  release-flow tests have not been run.

## 0.22.0

- Hard cut: remove the public `Validate` trait. `StableKey::validate` and
  `AllocationSlotDescriptor::validate` are now inherent methods; remove trait
  imports and use the concrete methods directly. Decoded DTO validation remains.
- Reuse checked request fields during logical placement instead of replaying
  public declaration constructors. Remove `MemoryResolutionError::Declaration`
  and update IcyDB's affected error match. Final snapshot validation, range
  authorization, deterministic placement, recovery and publication ordering remain.
- Populate detailed allocation-report bindings and range claims directly into
  ordered rows, removing per-ID metadata searches. Preserve the separate numeric
  summary, ledger attribution, all 255 rows and bounded read-only accounting.
- Share one borrowed declaration/history/policy check between bootstrap and
  doctor. Doctor no longer clones its resolved declaration snapshot or constructs
  a discarded pre-commit capability. Preserve validation ordering and read-only
  diagnostics; admission callbacks and application schema validation remain
  outside doctor's scope.
- Hard cut: remove `DiagnosticMemorySizeOutcome` and `DiagnosticCode::MemorySize`.
  `DiagnosticRecord::memory_size` now contains an optional `DiagnosticMemorySize`
  directly, without the `Measured` wrapper. Update diagnostic producers/readers
  and regenerate reports using the current shape; no old reader is retained.
  Invalid persisted slots still fail recovery before measurement. Qualify Canic's
  adapter and measured/unmeasured fixtures in an isolated copy and prepare an
  adoption patch; its public response shape is unchanged. Concurrent Canic work
  retains its published 0.21 reader until upstream adoption.
- Hard cut: remove `DeclarationCollector` and its mutable/consuming builder
  methods. Construct declarations with `AllocationDeclaration`, collect them in
  a `Vec` and pass it to `DeclarationSnapshot::new`. Remove builder-only tests
  and update the manual example; constructor and snapshot invariants remain.
- Durable formats and declaration fingerprints remain unchanged. No
  compatibility aliases or replacement framework are added.
- All 263 library tests, eight public integration tests, two composed host tests,
  seven compile-fail cases and five active doctests pass. Strict all-target
  Clippy, Rust 1.88 all-target checking, warning-denied Rustdoc and Wasm test
  compilation pass. Twenty focused IcyDB native tests and 13 Canic memory tests
  pass in isolated source copies against this candidate. No consumer dependency
  or lockfile changes are made by this cleanup. Canic's prepared reader requires
  adoption of the new ic-memory contract when it is published. Focused IcyDB
  library Clippy and Canic Core all-target/all-feature Clippy pass on those copies.
- Matched raw Wasm probes with Rust 1.99 and the unchanged lockfile all remain
  within budget: core 247,279 bytes (−674), diagnostics 292,604 (−2,353), key-only
  246,756 (−817), admission 250,034 (−894), runtime integration 256,530 (−834).
  Deltas compare this combined cleanup against the 0.21.0 release with the same
  probe sources and build profile; IC instruction/cycle changes are unmeasured.
  Installed-canister qualification, package verification and release-flow tests
  were not run; release-flow tests create user-owned commits and tags.

## 0.21.0

- Hard cut: remove `RuntimeStateError::InconsistentLifecycle`,
  `StaticMemoryDeclarationError::InconsistentLifecycle` and
  `StaticMemoryDeclarationError::SnapshotFingerprintEncoding`. Remove obsolete
  constructions or match arms directly; no aliases or compatibility paths remain.
  The known IcyDB error-classification fixture is updated without removing its
  coverage of live internal failures.
- Express private runtime and registry lifecycle guarantees as invariant
  assertions. Preserve typed construction, corruption, reentry, poisoning,
  deferred-hook, growth-refusal and policy errors, persistence-before-publication,
  and failed-bootstrap retry behavior. Document the invariant panic boundary.
- Build private ledger declarations and governance metadata infallibly from
  checked constants. Reuse the authoritative governance-range helper instead of
  reconstructing its bounds. Public declaration and range constructors retain
  their fallible validation boundaries.
- Attach historical schema metadata directly from the immutable recovered ledger,
  removing repeated validation and fallible propagation. Public metadata
  constructors and untrusted recovery still validate schemas; historical
  selection retains authorization, retirement, bounds and sticky failure checks.
- Make concrete ledger and declaration-fingerprint encoding into byte vectors
  infallible. Retain typed decoding errors, writer byte limits, current fixtures,
  checksums, canonical ordering and fingerprint bytes. Durable formats and
  diagnostic wire shapes remain unchanged.
- Validate 205 focused library tests across both cleanup passes, six public
  integration tests, two composed host tests, all seven compile-fail cases and
  five doctests. Strict all-target
  Clippy, Rust 1.88 all-target checking and Wasm test compilation pass. All five
  raw Wasm budgets pass; core is 247,953 bytes under its 260,000-byte ceiling.
  Twenty focused IcyDB native tests pass in an isolated copy with the initial
  local API hard cut patched in; consumer manifests and lockfiles remain unchanged.
  Package verification, installed-canister qualification and release-flow tests
  were not repeated; release-flow tests create maintainer-owned commits and tags.

## 0.20.0

- Hard cut: remove `RuntimePolicyError::MissingDeclarationMetadata`,
  `RuntimeBootstrapError::LedgerIntegrity` and `DiagnosticCode::GenesisLedger`.
  Remove obsolete match arms or constructions, the automatic conversion from
  `LedgerIntegrityError` into `RuntimeBootstrapError`, and use of the diagnostic
  wire value `genesis_ledger`. These failure paths were unreachable through
  maintained runtime operations. Real ledger integrity failures still return
  `RuntimeBootstrapError::LedgerCommit(LedgerCommitError::Integrity(...))`.
- Simplify committed-ledger generation membership validation after structural
  bounds and the contiguous parent chain have passed. Check only that each
  allocation's first generation is nonzero; remove repeated last-seen,
  retirement and schema-generation membership checks. Preserve structural
  validation, genesis rejection and error precedence.
- Make private runtime declaration-authority lookup infallible when validating
  the allocation snapshot from the same immutable resolved declaration snapshot.
  Remove metadata-error plumbing while preserving range authorization, custom
  policy rejection and internal governance handling.
- Share one private, infallible empty-genesis constructor between cold bootstrap
  and doctor validation. Remove the diagnostic-only wrapper and impossible
  genesis failure branches. Preserve empty-store initialization, fail-closed
  recovery, persistence ordering and read-only diagnostics.
- Extend genesis-reference coverage with later schema observations and
  retirement; verify history-gap errors still precede genesis rejection. Add
  doctor coverage for combined fixed and logical declarations before and after
  bootstrap, with unchanged backing bytes.
- Record the [follow-up audit](docs/audits/recurring/simplification-followup-0.19.0.md).
  Validate 266 library tests, integration and compile-fail tests, doctests,
  strict Clippy, Rust 1.88 all-target checking and Wasm test compilation. All
  five raw Wasm budgets pass; core is 248,351 bytes under its 260,000-byte ceiling.
  Durable ledger formats, current fixtures, checksums and declaration
  fingerprints remain unchanged; the diagnostic wire vocabulary loses only the
  removed genesis code. Package verification was not repeated. Release-flow
  tests were not run because they create commits and tags reserved for the
  maintainer.

## 0.19.0

- Hard cut: remove `LedgerPayloadEnvelopeError::PayloadLengthOverflow` and
  `StableCellPayloadError::LengthOverflow`. Remove obsolete match arms or
  constructions. These failures were unreachable after the existing byte
  ceilings passed; oversized input still returns the current typed size errors.
- Simplify ledger-envelope length arithmetic and stable-cell length conversion
  using the established byte bounds. Preserve the untrusted decoded `u64`
  conversion, physical-capacity and format checks, exact-length rejection and
  rejection before payload allocation or reads. Correct envelope panic and
  byte-ceiling documentation.
- Remove impossible growth conversion failures after bucket-capacity admission
  and when returning a successful previous-page count through `Memory::grow`.
  Preserve raw arithmetic overflow, bucket exhaustion, refusal/retry, reentry
  protection, accounting after successful growth and the upstream `-1` sentinel
  on actual errors.
- Add envelope coverage for oversized encoding, every truncated header length,
  `u64::MAX` declared lengths and mismatches in both directions. Extend the
  header-only stable-cell regression to cover `u32::MAX` and physical-capacity
  error precedence without payload reads, growth or writes. Extend growth tests
  to verify successful upstream previous-page returns, including full bucket
  capacity at each tested bucket size.
- Record the [follow-up audit](docs/audits/recurring/simplification-followup-0.18.0.md)
  and mark the previous findings as released in 0.18.0. Validate 265 library
  tests, integration and compile-fail tests, doctests, strict Clippy and Rustdoc,
  Rust 1.88 all-target checking and Wasm test compilation. Persisted formats,
  current fixtures, checksums and declaration fingerprints remain unchanged.
  Package verification and raw Wasm size gates were not repeated in this
  follow-up. Release-flow tests were not run because they create commits and
  tags reserved for the maintainer.

## 0.18.0

- Hard cut: remove `AllocationStageError::TooManyDeclarations`,
  `AllocationStageError::InvalidSchemaMetadata` and
  `AllocationStageError::GenerationOverflow`. Remove obsolete match arms or
  constructions. Declaration count and schema failures belong to snapshot
  validation; the finite committed-history limit remains an integrity error.
  These variants were unreachable with publicly obtained validated authority.
  Raw reservation and retirement overflow errors remain supported.
- Rely on immutable `ValidatedAllocations` facts during active staging. Remove
  repeated declaration count, schema and numeric-overflow checks. Preserve
  stale-proof rejection, receiver/output resource bounds, historical claim
  conflicts, retirement rejection and cloning before mutation. Document the
  proof's declaration and committed-history guarantees.
- Make private allocation-record construction and schema observation infallible
  after input validation. Remove the forwarding reservation observer and keep
  one schema-history update owner. Preserve reserved-to-active promotion,
  unchanged-schema suppression and last-seen updates. Public metadata
  constructors and raw reservation staging retain their validation boundaries.
- Replace tests that fabricate invalid schema or overflow proofs with decoded
  snapshot rejection and a real proof rejecting a `u64::MAX` receiver as stale.
  Add public-boundary coverage validating, staging, committing and recovering
  all 255 allocation slots. Strengthen the historical-conflict regression with
  a proof from a different valid ledger at the same generation and unchanged
  rejected receiver state.
- Record the [staging audit](docs/audits/recurring/simplification-followup-0.17.1.md)
  and mark the previous findings as released in 0.17.1. Validate 263 library
  tests, integration and compile-fail tests, doctests, strict Clippy and Rustdoc,
  Rust 1.88 all-target checking, Wasm test compilation and offline package
  verification. All five existing raw Wasm budgets pass; core is 249,456 bytes
  under its 260,000-byte ceiling. Persisted formats, diagnostic DTO shapes,
  checksums and declaration fingerprints are unchanged. Release-flow tests were
  not rerun because they create commits and tags reserved for the maintainer.

## 0.17.1

- Remove duplicate physical-generation state from `RecoveredLedger`. Recovery
  establishes physical/logical equality before constructing the proof; both
  public generation accessors now derive that value from the ledger. Preserve
  mismatch rejection, const accessors and constructor privacy. Derived `Debug`
  output no longer includes the redundant private field.
- Remove repeated empty-store checks after physical slot selection in explicit
  ledger initialization and physical commits. Only two absent slots permit
  genesis; corrupt or ambiguous slots still fail closed. Extend the recovery
  matrix to check initialization, exact rejection and unchanged stores for
  corrupt, unsupported, undecodable, mismatched and invalid-history records.
- Check runtime readiness once during authority adoption and verify requirements
  against the established host snapshot. Preserve fixed/logical distinctions,
  authority and metadata checks, error ordering and effect-free adoption without
  replaying host preparation.
- Remove inactive deserialization-only Serde attributes from the serialize-only
  derives on `PolicyIdentity` and `MemoryManagerRangeAuthority`. Their custom
  readers retain strict unknown-field, explicit optional-field and domain checks
  on the decoding DTOs.
- Record the [follow-up audit](docs/audits/recurring/simplification-followup-0.17.0.md)
  and mark the previous findings as released in 0.17.0. Qualify recovery/adoption
  changes with 262 library tests, integration and compile-fail tests, doctests,
  strict Clippy and Rustdoc, Rust 1.88 all-target checking, Wasm test compilation
  and offline package verification. All five raw Wasm budgets pass; core is
  249,892 bytes under its 260,000-byte ceiling. Recheck 34 focused tests and
  strict Clippy after the final Serde-attribute cleanup. Public API signatures,
  persisted formats, diagnostic DTO shapes, checksums and declaration
  fingerprints are unchanged. Release-flow tests were not rerun because they
  create commits and tags reserved for the maintainer.

## 0.17.0

- Hard cut: remove `RuntimeDiagnosticError::AllocationBound`,
  `RuntimeDiagnosticError::MemoryManagerSlot` and
  `RuntimeOpenError::MemoryManagerSlot`, plus the corresponding
  `From<MemoryManagerSlotError>` conversions into both runtime error types.
  Remove obsolete match arms or constructions; handle raw slot errors at the
  declaration or recovery boundary. These variants were unreachable through
  maintained runtime operations.
- Rely on sealed declaration bounds and validated slots during allocation
  reporting. Remove redundant count checks and their fallible binding lookup;
  preserve persisted manager-layout validation, live-manager agreement, reentry
  checks and bounded reads without writes or growth. Add coverage declaring
  every usable external ID and comparing detailed and numeric reports.
- Make the recovery result authoritative for doctor validation. Remove the
  redundant decoded-record argument and unreachable genesis branch. Preserve
  validation against genesis only for empty commit storage, borrowed successful
  recovery and typed rejection of unreadable, corrupt or unsupported records.
  Add coverage for both empty storage representations and a readable record
  containing a corrupt physical slot, with unchanged backing bytes.
- Remove unreachable slot-error conversions from committed ID lookup and host
  adoption. Preserve key, readiness, governance, authority, metadata and fixed-ID
  rejection. Extend the opening regression to check the exact
  `MemoryIdMismatch` for caller-supplied sentinel ID 255.
- Record the [simplification audit](docs/audits/recurring/simplification-followup-0.16.1.md).
  Validate 262 library tests, integration and compile-fail tests, doctests,
  strict Clippy and Rustdoc, Rust 1.88 all-target checking, Wasm test compilation
  and offline package verification. All five existing raw Wasm budgets pass;
  core is 249,906 bytes under its 260,000-byte ceiling. Persisted formats,
  diagnostic DTO shapes, checksums and declaration fingerprints are unchanged.
  Release-flow tests were not rerun because they create commits and tags reserved
  for the maintainer.

## 0.16.1

- Remove the sealed snapshot's duplicate authority map and private authority
  enum. Policy validation and host adoption share one lookup over canonical
  registered declarations. Preserve governance restrictions, external authority
  checks, custom-policy ordering and sealed declaration fingerprints.
- Stream recovered-ledger memory measurements into diagnostic export and doctor
  reports without collecting an intermediate vector. Keep public diagnostic
  DTOs and measurement behavior unchanged.
- Simplify ledger capacity reservation after the existing encoded-size limit
  establishes safe arithmetic bounds. Remove unreachable conversion/overflow
  branches and redundant saturating subtraction; preserve size rejection, typed
  growth failures, persistence ordering and retry behavior.
- Validate 260 library tests, integration and compile-fail tests, doctests,
  strict Clippy and Rustdoc, Rust 1.88 all-target checking, Wasm test compilation
  and offline package verification. All five existing raw Wasm budgets pass;
  core is 249,863 bytes under its 260,000-byte ceiling. Public API signatures,
  durable formats, checksums and declaration fingerprints are unchanged.
  Release-flow tests were not rerun because they create commits and tags reserved
  for the maintainer.

## 0.16.0

- Hard cut: `MemoryRuntime::memory_manager_config()` is no longer a `const fn`.
  Update enclosing const functions that call it to ordinary functions. Runtime
  configuration lookup reads the sole shared bucket geometry; returned values
  and persisted formats are unchanged.
- Keep backing memory, immutable bucket geometry and live bucket accounting in
  one shared growth state. Remove duplicate runtime fields while preserving
  cloned and detached handles, configuration checks and physical attribution.
- Confirm runtime persistence through `PendingBootstrapCommit::confirm_persisted`
  after the stable-cell write succeeds. Remove the private unpacking helper and
  duplicate generation selection before publishing allocation-open authority.
- Replace the derived generation-membership set with the range established by
  strict contiguous-history validation. Preserve structural checks and error
  ordering; add coverage rejecting allocation references to genesis generation.
- Share one runtime size-measurement flow over `RecoveredLedger`. Remove the
  unreachable per-slot failure branch and replace its fabricated-input test with
  persisted-corruption coverage for export, doctor and cold bootstrap. Public
  diagnostic DTO failure values remain supported; invalid persisted slots reject
  recovery before measurement and leave backing bytes unchanged.
- Share installed-toolchain validation between CI and Make through
  `make validate-toolchain`; read CI's MSRV from `Cargo.toml`. Correct runtime
  diagnostic guidance and record #9 as released in 0.15.7 and completed, with
  downstream adoption and qualification remaining consumer-owned.
- Validate 260 library tests, integration and compile-fail tests, doctests, six
  Wasm tooling tests, strict Clippy and Rustdoc, Rust 1.88 all-target checking,
  Wasm test compilation and offline package verification. All five existing raw
  Wasm budgets pass; core is 254,304 bytes under its 260,000-byte ceiling.
  Release-flow tests were not rerun because they create commits and tags reserved
  for the maintainer. Current wire fixtures, checksums and declaration
  fingerprints are unchanged.

## 0.15.7

- Return typed growth refusal from fresh runtime construction instead of allowing
  manager initialization to panic. Reserve the metadata page before writing;
  failed construction leaves zero pages and performs no reads or writes. Both
  constructors support retry with the same caller-owned backing. Configured
  default bootstrap propagates the construction error. Addresses Canic's feedback
  in [#9](https://github.com/dragginzgame/ic-memory/issues/9).
- Reject unknown fields inside durable retirement states. Enforce the serialized
  payload ceiling in both low-level physical commit entrypoints with
  `CommitRecoveryError::PayloadTooLarge` before slot mutation. Cover malformed
  nested records, exact byte limits, unchanged rejected stores and valid retries.
- Preserve decoder causes in ledger errors and doctor messages while retaining
  diagnostic codes. Correct declaration and reservation errors to report the
  actual 255-allocation limit.
- Return successful logical commit evidence from the checked ledger and physical
  commit, removing redundant checksum scans, decoding and integrity validation.
  Existing persisted bytes still pass the full fail-closed recovery boundary.
- Release declaration, request, range and hook inputs after registry sealing or
  terminal failure. Preserve immutable shared snapshots, cached errors and hook
  ordering through one sealing completion path.
- Count encoded ledger-record bytes during capacity admission without allocating
  a temporary serialization buffer. Share encoding with stable-cell persistence
  and verify measured lengths against current fixtures and the history boundary.
- Resolve Wasm artifacts through Cargo metadata so `CARGO_TARGET_DIR` and Cargo
  configuration cannot make budget checks read stale files. Add six tooling
  regressions for target discovery, oversized or missing artifacts and metadata
  failure; run them in CI and `make validate` through `make test-wasm-size`.
- Validate 259 library tests, integration and compile-fail tests, doctests, six
  Wasm tooling tests, strict Clippy and Rustdoc, Rust 1.88 all-target checking,
  Wasm test compilation and offline package verification. All five unchanged raw
  Wasm budgets pass; core is 254,648 bytes under its 260,000-byte ceiling.
  Current wire fixtures, checksums and declaration fingerprints are unchanged.

## 0.15.6

- Reuse protected slot-selection evidence in commit diagnostics, validating each
  present slot once instead of repeating payload checksum scans. Preserve
  fail-closed recovery and ambiguity classification; add coverage for valid ties,
  conflicting ties and corruption on either slot.
- Remove the unused public `AllocationValidationError::LedgerIntegrity` variant.
  Allocation validation requires `RecoveredLedger`; integrity failures are
  reported at the ledger recovery boundary.
- Share printable diagnostic-text validation across labels, runtime fingerprints,
  policy names, authorities and range purposes. Preserve the 256-byte ceiling,
  optional-field semantics, field-specific errors and rejection order. Add two
  public-API regressions for accepted boundaries and overlapping invalid inputs.
- Validate 249 library tests, integration and compile-fail tests, doctests,
  strict Clippy and Rustdoc, Rust 1.88 all-target compilation and Wasm test
  compilation. All five unchanged raw Wasm budgets pass; core is 259,455 bytes
  under its 260,000-byte ceiling. Persisted encoding, checksums and declaration
  fingerprints are unchanged.

## 0.15.5

- Fix configured default bootstrap holding a mutable TLS borrow while sealing
  declarations. Registration hooks can now observe the configured, unbootstrapped
  runtime without `ReentrantAccess`; construction and geometry failures still
  reject before sealing. Add a regression for hook readiness and summary reads.
- Reject excess elements in bounded sequences without a size hint before
  invoking their deserializer. Add a regression proving the rejected element is
  never decoded; existing definite-length CBOR bounds remain unchanged.
- Share automatic and explicitly numbered physical commits through one checked
  mutation path, avoiding repeated predecessor checksum scans. Preserve slot
  rotation, generation checks and rejection without mutation; add coverage for
  both entrypoints against a corrupt predecessor.
- Avoid unnecessary copies when filtering uniquely owned allocation authority
  and validating recovered history in doctor diagnostics. Remove unused private
  codec scaffolding, redundant capability markers and duplicate empty-key checks;
  share FNV hashing and test fixture decoding. Current wire fixtures, checksum
  bytes and declaration fingerprints remain unchanged.
- Refresh current guides, rustdoc and whitepaper coverage for admission, logical
  placement, configured bootstrap, typed growth, adoption and memory accounting.
  Mark archived designs and measurements as historical, and record the
  [code hygiene audit](docs/audits/recurring/code-hygiene-report-2026-10-03.md).
- Validate 248 library tests, integration and compile-fail tests, doctests,
  strict Clippy and Rustdoc, Rust 1.88 all-target compilation, Wasm test
  compilation and all five unchanged raw Wasm budgets. Public API signatures
  and persisted encoding are unchanged.

## 0.15.4

- Add the runnable [composed-host example](crates/ic-memory/examples/composed_host.rs) requested
  by Canic. Two cold reopens over the same backing retain fixed/logical IDs,
  authority and stored data with unchanged declarations and 16-page buckets.
  Cold attempts run host and consumer admission; warm adoption preserves the
  commitment without replaying admission. Typed consumer rejection leaves the
  existing backing unchanged and publishes no capability.
- Enable two public-API regressions in the ordinary test suite, covering cold
  reopens and configured host bootstrap on every native worker before consumer
  adoption or thread-local store initialization. Document that initialization
  order in the README and admission contract.
- Close implemented GitHub requests #2–#8 with released implementation and
  downstream adoption evidence. Update the [issue reconciliation](docs/issue-reconciliation.md)
  and codec qualification to record IcyDB's published 0.15.3 acceptance under
  its unchanged lifecycle instruction ceiling.
- Validate the full test suite, strict all-target Clippy, Wasm test compilation,
  all five existing raw Wasm size budgets and Rust 1.88 all-target checks.
  Runtime APIs and persisted encoding are unchanged. IC instruction/cycle and
  matched consumer Wasm deltas for the new regressions are unmeasured; Canic's
  participant/store-restoration qualification remains consumer-owned.

## 0.15.3

- Make default export, commit-recovery and both doctor diagnostic helpers
  nonconstructing. An absent runtime returns `RuntimeDiagnosticError::NotBootstrapped`
  without initializing memory or choosing 128-page buckets, preserving later
  configured bootstrap. Doctor inspection also leaves declarations unsealed
  when no runtime exists. Existing runtimes retain prebootstrap recovery/doctor
  inspection and typed construction/TLS errors. Addresses Canic's 0.15.2 feedback.

## 0.15.2

- Keep unsupported-format diagnostic coverage on `DiagnosticCode` rather than
  human-readable message wording.
- Replace narrow registration-hook and raw-read lint allowances with justified
  expectations so stale exceptions are reported. Preserve the existing unsafe
  read contract and test coverage. Addresses IcyDB feedback in
  [#8](https://github.com/dragginzgame/ic-memory/issues/8).
- Use the target's maximum `usize` in the declaration-count rejection test so
  its input does not overflow on 32-bit Wasm.
- Give the recovered-allocation iterator an explicit `must_use` reason to
  satisfy both Rust 1.88 and Rust 1.99 Clippy without lint allowances.
- Extract pre-bootstrap observation assertions into one test helper, reducing
  the configured-runtime test's cognitive complexity for Rust 1.88 Clippy
  while preserving fresh-thread isolation and assertion order.

## 0.15.1

- Pin development and primary CI validation to Rust 1.99.0. Read the compiler
  pin from `rust-toolchain.toml` in CI and Make to prevent validation drift;
  retain the declared Rust 1.88.0 MSRV.
- Address Rust 1.99 Clippy diagnostics by removing a redundant iterator
  `must_use` attribute and showing unexpected ledger contents in empty-value
  assertions.
- Share raw Wasm size enforcement between CI and Make and remove duplicate
  macro-test and doctest runs already covered by the serialized full suite.
- Matched-source, matched-lockfile Rust 1.99.0 builds add 2,924–3,383 raw Wasm
  bytes over Rust 1.97.1. Rebaseline only the admission budget from 260,000 to
  264,000 bytes; all other budgets and the `wasm-size` profile remain unchanged.
  See the [compiler comparison](docs/measurements/rust-1.99-toolchain.csv).

## 0.15.0

- Make default-runtime opens nonconstructing. An early open returns
  `NotBootstrapped` without initializing a manager or selecting its bucket size;
  cached construction and TLS access failures remain typed errors.
- Protect application and ledger growth through `RuntimeMemory`. Reserve
  physical backing capacity before assigning buckets. Hard-cut direct `grow`
  calls to `Result<u64, RuntimeGrowError>`; distinguish ordinary backing refusal,
  arithmetic overflow, reentrant growth and bucket exhaustion. Only the required
  upstream `Memory` trait adapter translates failures to `-1`. Propagate ledger
  capacity failures through `RuntimeBootstrapError::LedgerGrowth`; retain
  `StableCellLedgerWriteTooLarge` for the encoded record ceiling.
  Refusal preserves virtual extents and manager metadata and supports retry.
  All handles share one transient assigned-bucket count seeded on construction;
  remove the ledger's separate capacity preflight and metadata scan. The current
  durable format, policy and default bucket size are unchanged.
- Add `MemoryAllocationSummary`, `MemoryBindingSummary`,
  `MemoryRuntime::memory_allocation_summary` and the nonconstructing default
  helper. Numeric totals and current/ledger/unknown binding partitions share
  accounting with detailed attribution, retain the 34,848-byte metadata-read
  bound, and avoid per-ID rows and copied binding names. Payload occupancy
  remains unavailable.
- Add committed ID resolution and authority-scoped adoption verification for
  owned/default runtimes, plus `RuntimeAdoptionError`. Check fixed and logical
  requirements with typed missing-key, wrong-ID, current-authority and metadata
  errors without replaying admission or changing host configuration. Document
  the immutable declaration invariants of `CommittedAllocations` and update the
  composed-host example.
- Qualify refusal/retry, cloned and detached handles, interleaved allocations,
  reopen, reentry, bounded summary parity, and effect-free host adoption. Reject
  the superseded integer-returning growth API with a compile-fail test.
  Matched Rust 1.97.1 raw Wasm deltas against 0.14.3 are +283 bytes core,
  +934 diagnostics, +315 key-only and +215 admission; all existing budgets remain
  unchanged and pass. Add an integration probe exercising growth, summary
  serialization and adoption: 263,847 bytes under a new 270,000-byte budget.
  IC instruction/cycle costs and downstream lifecycle qualification are
  unmeasured. See the [raw size measurements](docs/measurements/0.15-runtime-integration.csv).

## 0.14.3

- Encode opaque committed payloads as bounded CBOR byte strings, removing the
  outer codec's per-byte integer encoding and syntax walk. The existing CBOR
  preflight checks the 16 MiB + 24-byte payload ceiling before deserialization;
  the stable-cell record ceiling is now 32 MiB + 4 KiB.
- This is a pre-1.0 persisted-format hard cut. Recreate earlier data; format
  version remains 1, with no compatibility reader or migration bridge.
  Human-readable DTO serialization continues to round-trip byte arrays.
- Replace current wire fixtures and qualify maximum-size writer/reader parity,
  malformed payload rejection, and capacity-refusal/retry behavior. Recovery,
  checksums, generation history and the persistence/publication boundary remain
  unchanged. All four raw Wasm probes shrink by 346–521 bytes; IcyDB's
  maintained lifecycle fixture passes with a worst phase of 5,133,140 instructions
  under its unchanged 12,750,000 ceiling using a local dependency override.
  Released-dependency adoption remains pending. See
  [codec qualification](docs/opaque-ledger-payloads.md).

## 0.14.2

- Removed the intermediate sealed snapshot from historical admission completion.
  The resolver consumes known-only selections alongside the original canonical
  requests, then builds and validates one final resolved snapshot. Original
  warm-bootstrap identity and resolved diagnostic fingerprints remain distinct.
- Reused validated request authority/key values when applying recovered schema
  metadata, preserving early bounds, current-grant checks and sticky rejection.
  Switched only unique-key request ordering to unstable sorting; fixed declaration
  and range comparators retain their existing ordering behavior.
- Added canonical-permutation, duplicate-rejection and resolved-fingerprint
  equivalence coverage. No API, durable format, lifecycle state, persistence
  boundary or recovery preflight changes were introduced.
- Recorded mixed matched Rust 1.97.1 raw Wasm results: admission decreases by
  159 bytes; core increases by 339, diagnostics by 718 and key-only by 331 bytes.
  All existing size budgets pass. Historical completion eliminates one snapshot
  build/fingerprint; IC instructions/cycles remain unmeasured. Further range-table
  sharing was deferred after its measured code-size increase. See the
  [cleanup qualification report](docs/logical-bootstrap-cleanup.md).

## 0.14.1

- Added `RuntimeBootstrapPolicy::prepare_bootstrap` inside the existing recovered
  ledger bootstrap flow. Hosts can compose consumer identity admission and
  explicitly complete historical declarations before resolution, final policy
  validation and the single persistence/publication boundary.
- Added bounded `BootstrapAdmission` metadata and known-only historical
  selection. Unknown, retired, duplicate, unauthorized or oversized selections
  reject the whole attempt, including when a callback ignores a selection error.
  Selected allocations retain their durable ID and latest schema metadata.
- Kept warm bootstrap and host adoption free of admission replay. Failed cold
  attempts can retry against unchanged committed mappings; default and owned
  runtimes use the same hook and preserve host policy and bucket configuration.
  No durable format or allocation-open authority changes were introduced.
- Added an IcyDB-shaped recovered-journal example, production runtime and
  capability-boundary tests, and the [admission contract](docs/recovered-admission.md).
  This addresses #5's allocation-level ordering gap. Actual generated IcyDB
  adoption, identity semantics, journal-debt and pending-commit qualification
  remain downstream work.
- Matched Rust 1.97.1 raw Wasm probes increase core by 648 bytes, diagnostics by
  342 bytes and key-only by 579 bytes. The new admission-enabled probe is
  258,960 bytes and is enforced under a 260,000-byte budget; existing probe
  budgets are unchanged. IC instruction/cycle costs remain unmeasured.

## 0.14.0

This release adds key-only allocation and bounded ledger recovery. It is an
intentional pre-1.0 admission hard cut: recovery rejects inputs outside the new
byte, collection and nesting limits, including indefinite-length CBOR. The
durable version-1 ledger shape remains unchanged; no legacy reader, automatic
history compaction or migration path is provided.

- Added `MemoryRequest`, static registration and key-only declaration/open
  macro forms, plus explicitly owned `SealedDeclarationSnapshot::new` inputs.
  After recovery, known keys retain their committed IDs; new requests resolve
  in stable-key order to the lowest free ID in an explicit host-owned `Allowed`
  grant. Fixed claims, governance slots, reservations, omitted allocations and
  retired slots remain unavailable to new keys.
- Added `MemoryRuntime::open_memory_by_key` and
  `open_default_memory_manager_memory_by_key`. Libraries can inspect committed
  assignments and adopt a bootstrapped host without replacing its policy or
  bucket configuration. Both runtime forms persist the complete resolved set
  before publishing allocation-open authority.
- Bounded logical ledger payloads to 16 MiB, stable-cell ledger values to
  64 MiB + 4 KiB, CBOR nesting to 32, allocation records to 255, and generation
  history and total schema history to 65,536 entries each. Added pre-allocation
  length checks, an allocation-free CBOR preflight, bounded collection decoding,
  and writer/staging checks with typed, fail-closed errors. Recreated runtimes
  still append a generation for unchanged declarations; history exhaustion is
  explicit and does not discard ownership or tombstones.
- Hardened ledger persistence against backing-memory growth refusal by reserving
  physical capacity before upstream manager bucket assignment. Failed bootstrap
  publishes no mapping; IC trap rollback remains the interrupted-write boundary.
- Clarified and tested omitted-store access through explicit reconciliation
  declarations supplied before sealing. Omitted keys cannot open through current
  authority, and revoked grants or explicit retirement reject redeclaration.
  **IcyDB integration for #4 remains unresolved:** its generated bootstrap must
  establish whether removed journal keys are available before sealing, then
  qualify journal-debt and pending-commit checks end to end. No unrestricted
  historical-open capability was added.
- Added a runnable standalone/composed-host example and the
  [recovery and integration qualification report](docs/key-only-recovery.md),
  covering deterministic placement, reservation activation, failed persistence,
  omitted/retired ownership, hostile decoding and history/record boundaries.
- Matched Rust 1.97.1 raw Wasm probes increased core from 240,300 to 255,112 bytes
  and diagnostics from 289,090 to 307,589 bytes. The equivalent key-only probe is
  254,762 bytes. Updated enforced budgets to 260,000 bytes for core/key-only and
  315,000 for diagnostics. Identical resolved declarations add no durable
  metadata fields; IC instruction/cycle measurements remain unavailable.

## 0.13.3

- Made default-runtime bootstrap status and committed-capability lookups
  nonconstructing. Missing runtimes return `false` / `NotBootstrapped` without
  initializing memory or selecting 128-page buckets; cached construction and
  TLS access failures remain typed errors.
- Exposed the existing built-in policy as `GenericRangePolicy` for use with
  configured bootstrap. Range enforcement, policy identity, host-owned runtime
  adoption, the durable format and the 128-page upstream default are unchanged.
- Added focused coverage for observation before configured bootstrap, repeated
  initialization, exact bucket matching and custom host-policy preservation.
- Matched Rust 1.98.1 raw Wasm probes using the same dependency lockfile leave
  core size unchanged at 240,387 bytes and reduce diagnostics from 289,422 to
  289,389 bytes. IC instruction/cycle deltas remain unmeasured. No database
  recreation is required by this change.

## 0.13.2

- Removed the redundant private backing adapter in favor of upstream's
  `Memory` implementation for `Rc<M>`, and forwarded `RuntimeMemory::read_unsafe`
  to the existing virtual memory. This removes wrapper-level destination
  zeroing while preserving default implementations for custom backings.
  Unsafe code remains denied by default, with scoped exceptions for the
  forwarding method and its raw-read tests.
- Added focused coverage for uninitialized destinations, specialized and
  default backing reads, discontiguous buckets, cloned handles, partial read
  failures, zero-length reads, upstream bounds behavior, and read-only effects.
- Updated the README to document upstream read delegation. IC instruction
  and cycle savings remain unmeasured.

## 0.13.1

- Re-exported the exact upstream substrate dependency as
  `ic_memory::ic_stable_structures`, making collections, backing memories, and
  traits available through the same dependency as `RuntimeMemory<M>`.
- Updated the README and advanced guide to use the re-export and remove the
  requirement for a separate direct `ic-stable-structures` dependency.
- Explicitly selected the CI validation and MSRV toolchains, and declared the
  development Wasm target, fixing target-installation mismatches caused by the
  repository toolchain override.
- Kept runtime ownership, allocation policy, bucket defaults, and the durable
  format unchanged.

## 0.13.0

This release adds bounded physical allocation attribution and explicit bucket
configuration to the owned memory runtime. It is an intentional pre-1.0 API
hard cut: runtime memory handles change type, while the durable allocation-ledger
format and the default 128-page bucket size remain unchanged.

### Runtime memory handles

- Changed runtime opens and memory macros to return `RuntimeMemory<M>`, which
  implements `Memory` and `Clone` without requiring a cloneable backing memory.
  Update stable-store annotations from `VirtualMemory<DefaultMemoryImpl>` to
  `ic_memory::RuntimeMemory<DefaultMemoryImpl>` directly.
- Retained one manager per runtime, with private shared backing access for
  attribution. Diagnostics do not grant allocation-open authority.

### Bounded allocation diagnostics

- Added `MemoryRuntime::memory_allocations()` and
  `default_memory_manager_memory_allocations()`, returning owned reports for
  all 255 usable IDs, including zero-size memories and the ic-memory ledger.
- Reported the actual persisted bucket size, physical and virtual extents,
  per-ID bucket allocation, current stable-key/owner bindings, manager metadata,
  and separate unknown-binding and unmanaged residuals with checkable totals.
- Distinguished bucket rounding slack from virtual extent and left payload
  occupancy explicitly unavailable. Retired or absent current keys remain
  unknown without omitting their physical allocations.
- Bounded successful collection to 34,848 metadata bytes without reading or
  decoding ledger history, initializing stores, writing, growing memory, or
  advancing a generation. The default helper does not construct a missing
  runtime.
- Added a validated read-only manager-layout adapter and pinned
  `ic-stable-structures` to exactly 0.7.2. Unsupported or corrupt metadata
  returns typed errors before manager initialization can write.

### Bucket configuration

- Added immutable `MemoryManagerConfig`, `MemoryRuntime::new_with_config`, and
  `bootstrap_default_memory_manager_with_config` for nonzero bucket sizes.
- Kept the fresh-state default at 128 pages (8 MiB). Ordinary construction
  honors the persisted setting; explicit configuration rejects mismatches
  before effects, including repeated default-runtime bootstrap.
- Bound configuration to the runtime's sole manager. Changing a requested
  setting does not shrink existing memory or introduce a migration path.

### Measurements and validation

- Added disposable small-store and growing-store measurements comparing 128-,
  16-, 8-, and 1-page buckets, with finite-table capacity and growth/access cost
  accounting. Evidence supports opt-in sizing while retaining the default.
- Added conservation, bucket-boundary, corrupt-metadata, access-separation,
  no-write/no-growth, capacity-exhaustion, and same-release recovery/replay
  coverage, including borrowed non-Clone backing memory.
- Added the [CANIC-162 integration handoff](docs/canic162-memory-attribution.md)
  with reproducible measurements and downstream adoption examples. Live Toko
  attribution and Canic adoption remain separate outstanding work.
- Included allocation-report serialization in the diagnostics Wasm probe.
  Core and diagnostics remain within their existing raw Wasm budgets at
  240,226 and 289,044 bytes respectively on Rust 1.97.1.
- Updated trybuild to 1.0.121 and raised the declared MSRV and its CI check to
  Rust 1.88.0.

### Release tooling

- Added Canic-style `make release-patch` and `make release-minor` flows that
  validate committed source, synchronize manifest/README versions, commit,
  create an annotated tag, and atomically push the branch and release tag.
- Added separate `make publish` and `make publish-dry-run` commands with clean
  release-commit and tag checks. Local preparation and stage/commit/push steps
  remain available individually for review and retry.
- Added release-flow tests using disposable Git remotes and a fake Cargo,
  including failed-validation/package rollback, staged-change rejection,
  remote conflicts, atomic push rejection, and publication dry runs.

## 0.12.3

This release hardens runtime policy identity and makes doctor diagnostics
policy-aware. It is an intentional pre-1.0 API and diagnostic-shape hard cut
and does not change the durable allocation-ledger format.

### Policy and bootstrap binding

- Replaced the unbounded `&'static str` policy identity with validated
  `PolicyIdentity`, containing a bounded policy-family name, nonzero semantic
  version, and optional caller-computed 32-byte configuration digest.
- Made identity construction fallible and revalidated diagnostic
  deserialization so malformed input cannot bypass the newtype invariants.
- Bound repeated bootstrap to the complete identity, including configured
  policy digest, while keeping the binding explicitly in-memory rather than
  durable upgrade history.
- Added a deterministic, versioned, non-cryptographic
  `SealedDeclarationFingerprint` for diagnostic comparison without replacing
  sealed-snapshot identity as bootstrap authority.

### Policy-aware diagnostics

- Changed `MemoryRuntime::doctor_report` to accept the policy it evaluates and
  added `default_memory_manager_doctor_report_with_policy` for custom-policy
  default runtimes.
- Included the tested policy identity and declaration fingerprint, the binding
  established by successful bootstrap, and a typed binding comparison in
  `MemoryRuntimeDoctorReport`.
- Added distinct diagnostic codes for invalid policy identity, runtime-binding
  mismatch, and per-slot memory-size failure.
- Changed doctor ledger size projection to preserve successful measurements
  when another allocation's slot is invalid, using
  `DiagnosticMemorySizeOutcome` per record.

### Size budgets

- Split the representative raw Wasm gate into a 245,000-byte bootstrap/open
  core tier and a 290,000-byte doctor/export diagnostics tier.
- Measured 239,150 and 281,858 bytes respectively on Rust 1.97.1.

## 0.12.2

This release makes runtime construction fail closed before
`ic-stable-structures` can initialize over unrecognized nonempty backing
memory. It is an intentional pre-1.0 API hard cut and does not change the
durable allocation-ledger format.

### Backing-memory construction safety

- Changed `MemoryRuntime::new(memory)` to return
  `Result<MemoryRuntime<M>, RuntimeConstructionError>`.
- Added raw backing-memory preflight for the pinned `MemoryManager` magic and
  layout version. Empty memory remains initializable; nonempty foreign or
  unsupported memory is rejected without mutation.
- Propagated default TLS construction failures through the existing typed
  runtime-state path without adding panic, fallback, reset, or compatibility
  behavior.
- Added byte-for-byte negative tests for foreign memory and unsupported
  `MemoryManager` versions, plus positive empty-memory and current-layout
  recovery coverage.

## 0.12.1

This release cleans up and hardens the explicit runtime architecture introduced
in 0.12.0. It makes one intentional pre-1.0 policy-trait hard cut and does not
change the durable allocation-ledger format.

### Runtime implementation structure

- Split the runtime implementation into focused core, policy, diagnostics,
  error, default TLS, and test modules.
- Kept `MemoryRuntime<M>` as the single owner of each backing memory's runtime
  state and kept the default API as thin entry points into one TLS runtime.
- Moved declaration-registry and runtime unit tests out of production
  implementation files so ownership and sealing paths are easier to review.

### Bootstrap and registration hardening

- Added `RuntimeBootstrapPolicy` with an explicit semantic identity. Repeated
  bootstrap is idempotent only when both that identity and the sealed
  declaration snapshot match the successful bootstrap; mismatches return typed
  errors without touching the ledger.
- Moved deferred constructor-registration failures into the declaration
  registry lifecycle, so open, sealing, sealed, and failed state have one
  canonical owner. The first sealing failure remains deterministic, and an
  impossible internal transition has a distinct typed error.
- Removed a redundant final registry lock during snapshot construction and
  lifecycle publication.
- Made the committed-capability compile-fail boundary independent of changing
  rustc missing-item wording.

### Regression coverage

- Added a downstream-style integration test for the default runtime with a
  custom `RuntimeBootstrapPolicy`, including repeated-bootstrap identity and
  generation checks.
- Pinned primary development and CI validation to Rust 1.97.1 while retaining
  the Rust 1.85.0 MSRV check.
- Made CI run the exact two-libtest-thread regression with
  `--test-threads=1`, and run the full test suite in serialized libtest mode.
- Added a representative stripped, uncompressed Wasm bootstrap/open probe with
  a 240,000-byte release budget.

## 0.12.0

This release is an intentional pre-1.0 runtime API hard cut. It removes the
split process-global/thread-local default runtime architecture without adding
aliases, reset hooks, compatibility forwarders, or fallback state.

### Explicit runtime ownership

- Added `MemoryRuntime<M>` as the canonical owner of one backing memory's
  `MemoryManager`, allocation-ledger cell, bootstrap lifecycle, committed
  allocation capability, opens, diagnostics, and live memory sizes.
- Added an explicit `Unbootstrapped` / `Bootstrapped { committed_allocations }`
  lifecycle. Capability publication occurs only after that runtime's
  stable-cell write succeeds, and failed bootstrap leaves the runtime
  unbootstrapped.
- Made repeated bootstrap on the same runtime object idempotent without
  advancing its ledger generation.
- Added `SealedDeclarationSnapshot`, an immutable canonical process-wide view
  of linked declarations, range authority, and policy metadata. Declaration
  sealing is independent from every concrete memory bootstrap.

### Default TLS runtime hard cut

- Replaced the separate TLS memory manager and ledger cell plus process-global
  bootstrap flag and committed capability with one thread-local
  `MemoryRuntime<DefaultMemoryImpl>`.
- Removed all process-global memory-runtime lifecycle/capability state and
  removed runtime reset support. Native libtest threads now bootstrap their own
  default memory; single-threaded IC Wasm retains canister-instance behavior.
- Changed default TLS entry to use fallible `RefCell` access and added typed
  runtime reentrancy/unavailability errors.
- Generalized `DefaultMemoryManagerDoctorReport` to
  `MemoryRuntimeDoctorReport`, and made the default doctor wrapper return a
  typed diagnostic result.
- Changed `is_default_memory_manager_bootstrapped()` to return a typed result
  and changed `ic_memory_key!` to return the typed memory-open result instead of
  panicking internally.

### Atomic declaration sealing

- Moved generated declaration/range registration work into the fallible seal
  lifecycle, before eager declaration hooks and final validation.
- Canonically sorted declarations and ranges before duplicate detection and
  snapshot publication, making snapshot meaning and declaration bytes
  independent of constructor order.
- Serialized concurrent snapshot requests and made them share one immutable
  snapshot. Late registration, duplicate declarations, recursive sealing, hook
  panic, and mutex poisoning remain distinct typed failures.
- Removed public collection and separately assembled snapshot functions that
  could expose unsealed registry views. Integrations now inspect
  `sealed_declaration_snapshot()`.

### Validation and format

- Added the exact two-libtest-thread regression, independent
  `MemoryRuntime<VectorMemory>` isolation/recovery tests, concurrent snapshot
  and runtime bootstrap tests, typed negative opens, failure publication tests,
  idempotence checks, and runtime-local diagnostics.
- Kept the stable-cell, protected commit-store, payload-envelope, allocation
  ledger, fixture, format marker, and format version bytes unchanged.
- Updated README, advanced and safety guidance, whitepaper operations, and
  rustdoc to distinguish linked declaration authority from concrete runtime
  ownership and to document bootstrap once per memory runtime.

## 0.11.1

This release tightens repository hygiene around the pre-1.0 hard-cut policy. It
does not change the current runtime API or durable format.

### Compatibility hygiene

- Audited the active API and decoder surface for deprecated forwarders,
  compatibility aliases, legacy modules, serde aliases/defaults, fallback
  readers, and migration shims. None remain.
- Removed test-only encodings of superseded envelope, record-field, and macro
  forms. The active suite now exercises only the current format and current
  authority boundaries.
- Removed current fixture and safety documentation that described superseded
  wire shapes; historical release notes remain the only record of them.

## 0.11.0

This is an intentional current-format and diagnostic-API hard cut. No legacy
decoder, compatibility shim, serde alias, or migration path was added.

### Durable format classification

- Added an explicit current ledger-format marker and version inside the
  protected payload envelope. Recognized pre-0.11 `ic-memory` payloads without
  the current discriminator now return typed
  `LedgerPayloadEnvelopeError::UnsupportedFormat` instead of falling through
  to a generic ledger decode or length error.
- Replaced the current golden fixtures in place and recomputed protected-slot
  checksums for the new envelope. Superseded fixtures and decoders are not
  retained.

### Machine-readable diagnostics

- Added stable machine-readable `DiagnosticCode` values alongside human
  messages in stable-cell, range-authority, and validation diagnostics. The
  doctor report no longer requires tooling to classify these failures by
  parsing prose.
- Added `DiagnosticFailure` and changed string-valued diagnostic errors to
  carry both a stable code and an operator-facing message.

### Authority ergonomics

- Allowed explicit macro authority arguments to use a shared compile-time
  string constant, reducing repeated-literal drift without restoring implicit
  ownership.

### WebAssembly size

- Measured equivalent `serde_cbor` and `ciborium` encode/decode Wasm probes;
  the raw optimized `ciborium` artifact was 55,288 bytes (41.19%) smaller. It
  remained 48,099 bytes (40.88%) smaller after `wasm-opt -Oz`.
- Confirmed that `crunchy` is present only in the all-target lockfile
  resolution and is not linked into the normal `wasm32-unknown-unknown`
  dependency graph. The complete method is recorded under `docs/audits/`.

## 0.10.0

This is an intentional current-format hard cut. No fallback decoder, missing-
field default, compatibility alias, or migration shim was added.

### Recovery hardening

- Fail closed when either present physical commit slot has an invalid marker or
  checksum. Recovery no longer falls back to an older generation, which could
  otherwise forget a newer allocation, retirement, or schema-history fact.
- Keep deterministic recovery for identical duplicate slots and select the
  highest generation only after every present slot validates.
- Add regressions proving a corrupt latest generation cannot roll allocation
  history back and a corrupt inactive slot cannot be silently overwritten.

### Decode and policy hardening

- Require every non-elided optional field in current durable CBOR records to be
  present. Explicit CBOR `null` remains the encoding of `None`; omission now
  fails closed instead of being interpreted as empty state.
- Apply caller-supplied default-runtime policy only to external declarations.
  The private `ic_memory.ledger.v1` allocation remains governed exclusively by
  ic-memory's internal namespace and range policy.

### State-model hard cut

- Changed `AllocationState::Retired` to
  `AllocationState::Retired { generation }` and removed the separate nullable
  `AllocationRecord::retired_generation` field and accessor. A retired record
  without a generation, or a live record with retirement metadata, is now
  unrepresentable.
- Removed the obsolete `MissingRetiredGeneration` and
  `UnexpectedRetiredGeneration` integrity errors.
- Replaced nullable diagnostic field pairs with explicit states:
  `CommitSlotDiagnostic` is now `Empty`, `Valid`, or `Invalid`;
  `CommitStoreDiagnostic::recovery` and
  `DiagnosticRangeAuthority::effective_authority` are `Result` values;
  corrupt stable-cell errors live in `DiagnosticStableCellStatus::Corrupt`;
  and `DiagnosticCheck` is now an enum carrying failure/not-run messages.
- Removed `DiagnosticCheckStatus`; `DiagnosticCheck` itself is the status.
- Replaced the current fixture set in place. Earlier pre-1.0 allocation-record
  and diagnostic shapes are intentionally rejected; there is no legacy decoder
  or in-crate migration path.

### Release checks and documentation

- Add WebAssembly test-target compilation to CI.
- Fix packaged README image and documentation links, and align recovery and
  capability wording across the safety guide and whitepaper.
- Keep stable keys and memory IDs unchanged while intentionally replacing the
  pre-0.10.0 allocation-ledger encoding.

## 0.9.0

This is an intentional hard-cut release. Removed APIs have no deprecated
forwarders, compatibility aliases, or legacy modules.

### Reduced public surface

- Removed the unused generic `AllocationSession`, `AllocationSessionError`,
  `StorageSubstrate`, and `LedgerAnchor` APIs. Persistence owners now use the
  opaque `CommittedAllocations` capability directly when authorizing their own
  storage-open path.
- Stopped exporting implementation-only physical recovery machinery:
  `ProtectedGenerationSlot`, `DualProtectedCommitStore`, `CommitSlotIndex`,
  `AuthoritativeSlot`, and `select_authoritative_slot`.
- Kept the concrete `DualCommitStore`, committed-generation DTOs, recovery
  errors, and diagnostics public for current stable-cell integrations.
- Narrowed `CommitStoreDiagnostic::from_store` to the concrete
  `DualCommitStore` instead of an extension trait.
- Added a repository-wide pre-1.0 hard-cut rule and removed redundant serde
  defaults from optional diagnostic fields so no annotation resembles an
  earlier-shape compatibility path.

### Durability clarification

- Clarified that both redundant commit slots are serialized inside one
  `ic-stable-structures::Cell` in the default runtime. ICP message execution
  provides atomic stable-memory commit and rollback; the embedded slot
  checksums provide fallback only from localized corruption when the enclosing
  record remains decodable.
- Preserved the 0.8 durable ledger, stable-cell, payload-envelope, and CBOR
  formats. Upgrading from 0.8 requires no stable-memory migration.

## 0.8.1

### Decode and ingestion hardening

- Reject trailing bytes after persisted CBOR ledger payloads and stable-cell
  ledger records instead of accepting a valid value prefix.
- Revalidate decoded `AllocationDeclaration` values when they enter the static
  declaration registry.
- Revalidate decoded `MemoryManagerAuthorityRecord` values when they enter the
  static range registry, and expose an explicit `validate()` method for other
  ingestion boundaries.
- Revalidate `AllocationRetirement` values before staging a retirement
  generation, and expose an explicit `validate()` method for decoded requests.

## 0.8.0

This is an intentional hard-cut release. It does not retain deprecated shims,
legacy macro forms, compatibility aliases, or renamed API forwarding methods.

### Breaking authority model

- Split allocation state into two distinct opaque types:
  - `ValidatedAllocations` is pre-commit validation state and cannot open
    storage. Its generation accessor is now `base_generation()`.
  - `CommittedAllocations` is the post-persistence capability accepted by
    `AllocationSession` and the default runtime's open path.
- Replaced `BootstrapCommit` with `PendingBootstrapCommit`. Generic persistence
  owners must durably write the mutated ledger record before calling
  `confirm_persisted()` to obtain `CommittedAllocations`.
- Removed `validated_allocations()`. The default runtime now exposes only
  `committed_allocations()`, published after its stable-cell write succeeds.
- Changed `AllocationSession::new` to require `CommittedAllocations` and renamed
  its state accessor from `validated()` to `committed()`.
- Renamed `RuntimeOpenError::StableKeyNotValidated` to
  `StableKeyNotCommitted`. `MemoryIdMismatch::validated_id` is now
  `committed_id`.

### Explicit authority registration

- Removed every implicit-authority form of `ic_memory_range!`,
  `ic_memory_declaration!`, and `ic_memory_key!`. Range and key declarations now
  require the same explicit stable `authority = "..."` value.
- Rejected all external `ic_memory.*` declarations and external attempts to use
  the reserved `ic-memory` authority identity.
- Replaced caller-controlled internal authority strings with private runtime
  provenance.
- Renamed `declaring_crate` fields and accessors to `authority` on static and
  diagnostic declaration APIs. No serde field alias is retained.
- Changed `StaticMemoryDeclaration::new` and
  `StaticMemoryRangeDeclaration::new` to return validation errors.

Before:

```rust,ignore
ic_memory::ic_memory_range!(start = 120, end = 129);
ic_memory::ic_memory_key!("app.users.v1", UsersStore, 120);
```

After:

```rust,ignore
ic_memory::ic_memory_range!(
    authority = "app",
    start = 120,
    end = 129,
);

ic_memory::ic_memory_key!(
    authority = "app",
    key = "app.users.v1",
    ty = UsersStore,
    id = 120,
);
```

### Manual bootstrap migration

Manual persistence owners must now make persistence confirmation explicit:

```rust,ignore
let pending = AllocationBootstrap::new(record.store_mut())
    .initialize_validate_and_commit(&genesis, declarations, &policy, committed_at)?;

persist_record(&record)?;
let committed = pending.confirm_persisted();
let session = AllocationSession::new(storage, committed);
```

Calling `confirm_persisted()` before the owning record is durably written
violates the protocol.

### Validation and storage hardening

- Validated reservation DTO invariants before invoking caller policy.
- Made `MemoryManagerRangeAuthority` deserialization validate every imported
  record and reject overlaps.
- Tightened committed ledger chronology: retirement must follow the final
  observation, schema history must begin at allocation creation, and schema
  changes cannot postdate the final observation.
- Validated retirement slot descriptors in `AllocationRetirement::new`.
- Removed all `ic_memory.*` governance allocations from published application
  capabilities while preserving them in durable ledger recovery state.

### Dependencies and packaging

- Replaced the unmaintained `serde_cbor` dependency with maintained `ciborium`.
  Current CBOR fixtures remain byte-for-byte stable without a compatibility
  decoder or legacy format path.
- Removed the `stable_structures` dependency re-export. Downstream code now
  imports `ic-stable-structures` directly.
- Reduced the published package from roughly 2.1 MiB compressed to about
  203 KiB by excluding an unused large decorative image.

---

## 0.7.5

### Runtime hardening

- Made default-runtime bootstrap return `RuntimeLockPoisoned` if the deferred
  eager-init hook queue lock is poisoned, instead of panicking.
- Made the default doctor report surface eager-init hook lock failures as a
  failed diagnostic check instead of panicking.
- Removed reachable production `expect(...)` paths from default ledger-cell
  initialization, stable-cell capacity sizing, and ledger schema-history
  staging.
- Added fallible `LedgerPayloadEnvelope::try_encode()` and used it from ledger
  commit paths so envelope length failures return `LedgerCommitError`.
- Replaced internal declaration-claim `unreachable!` arms with explicit
  validation and staging errors.

---

## 0.7.4

### Public API hygiene

- Hid constructor-bypassing fields on `AllocationRetirement`,
  `MemoryManagerAuthorityRecord`, and `SchemaMetadata`, replacing them with
  read accessors.
- Simplified `StaticMemoryRangeDeclaration::new` so the range authority comes
  only from the validated `MemoryManagerAuthorityRecord`.
- Made implementation modules private, keeping the intended public API on the
  crate root and updating exported macros to use root-level helpers.

### Code hygiene

- Centralized stable-cell ledger-record decoding for runtime reads and
  diagnostics.
- Consolidated static registry lock and sealed-state handling.
- Shared claim-conflict record lookup between validation and staging.
- Added strict unknown-field rejection to diagnostic export DTOs.
- Added a deterministic transition matrix test that checks committed ledger
  invariants across many declaration, reservation, and retirement sequences.

---

## 0.7.3

### Runtime hardening

- Kept the default runtime's internal ledger allocation in the durable ledger
  while removing it from the published/openable validated allocation set.
- Made public default-runtime opens reject `ic_memory.*` governance stable keys.
- Preflighted default ledger-cell writes so oversized records or failed stable
  memory growth return a bootstrap error before calling `Cell::set`.
- Revalidated full reservation declarations before staging reservation
  generations.
- Made static range declaration authority mismatches fail in release builds.
- Marked public error enums as non-exhaustive so future patch releases can add
  variants without breaking downstream wildcard matches.

---

## 0.7.2

### Diagnostics

- Added a default `MemoryManager` doctor report that combines stable-cell
  status, commit recovery, recovered ledger export, registered declarations,
  range authority, validation preflight, and live memory sizes.
- Documented default-runtime diagnostic behavior, including the fact that the
  doctor runs deferred `eager_init!` hooks before bootstrap and that custom
  policy diagnostics remain the framework adapter's responsibility.

---

## 0.7.1

### Diagnostics

- Added optional live backing-memory size diagnostics for allocation records,
  including a default `MemoryManager` export helper that reports each
  `VirtualMemory::size()` in WebAssembly pages and bytes.
- Added a default `MemoryManager` commit-recovery diagnostic helper that can
  inspect protected ledger slots without requiring successful bootstrap.
- Revalidated decoded `MemoryManager` range-authority records so reversed
  ranges and the `255` sentinel cannot enter imported authority tables.
- Made raw stable-cell payload decoding classify empty memory as `NotStableCell`
  instead of relying on callers to preflight it.
- Removed duplicated internal constants for diagnostic metadata bounds and
  WebAssembly page size.

---

## 0.7.0

### Whitepaper and Lean model

- Added the `ic-memory` whitepaper, covering the stable-memory allocation
  governance problem, protocol model, allocation invariants, durable commit
  protocol, operational guidance, and current non-goals.
- Added a compact Lean model for the core allocation-safety argument, including
  checked lemmas for stable-key slot uniqueness, physical-slot key uniqueness,
  retired allocation tombstones, post-commit open authority, and generation
  monotonicity.
- Added mdBook/Nix/Lake scaffolding for building the Markdown whitepaper and
  Lean model without committing a PDF artifact.

### Protocol cleanup

- Removed the current-version compatibility range abstraction and the remaining
  ledger/envelope version-routing scaffold from recovery and commit.
- Simplified the durable ledger, envelope, diagnostics, slot descriptor, and
  schema metadata shapes by dropping unused physical-format, ledger schema,
  envelope version, descriptor substrate/version, and schema-fingerprint fields.
- Made committed generation parent links mandatory, using `0` for the first
  generation instead of accepting an absent parent value.
- Refreshed the current golden wire fixtures for the cut-down durable format.

---

## 0.6.2

### Protocol mutation coverage

- Added nested CBOR unknown-field regression tests for decoded
  `AllocationHistory`, `AllocationRecord`, `AllocationSlotDescriptor`, and
  `GenerationRecord` values inside the crate-owned ledger payload.
- Added a stable-cell wrapper regression test proving unknown top-level fields
  in `StableCellLedgerRecord` decode fail closed before the record can be used
  as a ledger anchor DTO.
- Clarified `LedgerPayloadEnvelope` rustdoc: decoding the envelope classifies
  protocol bytes only and does not establish allocation authority.

---

## 0.6.1

### Audit hardening

- Added `serde(deny_unknown_fields)` to `LedgerCommitStore`, closing the last
  authority-bearing durable DTO wrapper that could otherwise ignore future
  top-level CBOR fields during rollback.
- Added a regression test that mutates the `LedgerCommitStore` CBOR shape with
  an unknown top-level field and verifies that decode fails closed.
- Added compile-fail tests that lock the public API boundary around
  `RecoveredLedger` and `ValidatedAllocations`, proving downstream safe Rust
  cannot call their crate-private constructors.
- Made late `eager_init` registration fail closed after default runtime
  bootstrap instead of silently queueing a hook that will never run.
- Clarified that explicit genesis initialization APIs are privileged
  empty-store/import paths; normal users should prefer the default runtime or
  the golden bootstrap flow.
- Improved `ic_memory_key!` open failure text so it covers missing bootstrap,
  unvalidated keys, and key/id mismatches.

---

## 0.6.0

### Protocol authority boundary

- Added a logical ledger payload envelope inside each physically committed
  generation. Physical dual-slot recovery still selects the highest valid
  committed generation first; only then does `ic-memory` decode the logical
  envelope and route the ledger payload by schema/format metadata.
- Added `RecoveredLedger` as the crate-owned proof that a ledger crossed
  physical recovery, payload-envelope routing, compatibility checks, and
  committed-integrity validation.
- Changed `validate_allocations()` to require `RecoveredLedger` instead of raw
  `AllocationLedger`, so untrusted/manual ledger DTOs cannot mint
  `ValidatedAllocations`.
- Removed the public caller-supplied compatibility range surface from normal
  recovery and commit APIs. `LedgerCommitStore` now uses the crate-owned current
  protocol path.
- Added recovery tests for payload-envelope classification, unsupported
  envelope versions, and envelope/ledger metadata drift.
- Added reviewable v1 hex wire fixtures for payload envelopes, full commit
  stores, dual-slot recovery states, stable-cell records, and slot descriptors,
  with tests that decode, validate, recover, and re-encode them.
- Marked authority-bearing durable DTO structs with `serde(deny_unknown_fields)`
  so future fields fail closed instead of being silently ignored by older
  readers.
- Removed the custom payload encoding namespace. The logical ledger payload is
  the current `ic-memory` CBOR format inside the logical envelope.
- Removed custom allocation-slot substrates from the 0.6 authority model.
  Allocation slots are `ic-stable-structures::MemoryManager` `u8` IDs only,
  with ID 255 rejected as the sentinel.
- Removed public codec selection from the commit/bootstrap path. The durable
  ledger codec is crate-owned CBOR, so downstream crates cannot introduce a
  parallel ledger format.

---

## 0.5.1

### Audit hardening

- Made `ValidatedAllocations` an opaque non-serializable capability. It no
  longer derives serde traits and can only be produced by crate validation and
  bootstrap paths.
- Added deep validation for decoded DTOs before they can become allocation
  authority. Stable-key grammar and `MemoryManager` slot descriptor invariants
  are rechecked during snapshot validation and committed-ledger integrity
  validation.
- Raised the `validate_allocations()` authority boundary so the historical
  ledger must pass current compatibility and committed-integrity validation
  before it can produce `ValidatedAllocations`.
- Added stable-cell ledger preflight for the default runtime so corrupt
  `ic-stable-structures::Cell` storage is classified as a bootstrap error
  before `Cell::init` would otherwise panic while decoding the ledger record.
- Made the default runtime range-policy contract explicit: registered
  `ic_memory_range!` claims are enforced before caller-supplied policy, while
  framework adapters can omit user ranges and enforce application space in
  their own policy.
- Pinned the default developer and CI toolchain to Rust 1.95.0 while keeping
  the crate MSRV at Rust 1.85.0 through `package.rust-version` and an MSRV CI
  check.
- Updated crates.io metadata to describe `ic-memory` as a Memory ID registry
  wrapper for `ic-stable-structures`.

---

## 0.5.0

### Runtime registration

- Added a generic multi-crate runtime registration layer for downstream crates
  such as IcyDB, including range declarations, `ic_memory_key!`,
  `ic_memory_range!`, `eager_init!`, default `MemoryManager` bootstrap, and
  validated runtime opening without Canic.
- Moved the normal documentation path to the macro-based runtime API and moved
  lower-level ledger/bootstrap guidance to `ADVANCED.md`.
- Left TLS eager initialization out of `ic-memory`; framework helpers such as
  Canic's `eager_static!` should wrap ordinary `thread_local!` values and use
  `ic_memory_key!` / `ic_memory_range!` for allocation registration.

---

## 0.4.1

### Ledger hardening

- Split allocation staging behavior into `ledger::stage`, keeping the public
  staging API stable while reducing the size of `ledger::mod`.
- Replaced saturating generation diagnostic counts with explicit fail-closed
  errors when declaration or reservation counts exceed the durable `u32` limit.
- Documented that empty validated and reservation generations are intentional
  generation boundaries.
- Documented and tested reserved-record retirement semantics.
- Documented the expected allocation-ledger size bounds behind the current
  clone-on-stage implementation.

---

## 0.4.0

### Breaking cleanup

- Bumped from the already-published `0.3.0` to `0.4.0` because this release
  removes public APIs that were redundant or unused.
- Removed the unused `NamespaceAuthority` and `RangeAuthority` policy traits.
  Direct `MemoryManagerRangeAuthority` methods are the supported range-policy
  API.
- Removed the redundant `AllocationSlotDescriptor::memory_manager_checked`
  constructor alias. Use `AllocationSlotDescriptor::memory_manager`.
- Removed the redundant `MemoryManagerRangeAuthority::to_records` export alias.
  Use `authorities()` for the stable read-only diagnostic/export surface.

### Documentation

- Clarified that `AllocationBootstrap` is the golden path for whichever layer
  owns an `ic-memory` ledger store, not specifically for Canic.
- Documented framework-owned, library-owned, and application-owned bootstrap
  modes, plus the rule that exactly one owner should bootstrap a given ledger
  store.
- Updated the README golden path to use
  `AllocationBootstrap::initialize_validate_and_commit`.

---

## 0.3.0

### Native IC substrate

- Made `ic-stable-structures = "0.7.2"` a normal dependency instead of an
  optional feature-gated dependency.
- Made `serde_cbor = "0.11"` a normal dependency.
- Removed the `ic-stable-structures` feature; `stable_cell` support now always
  compiles and its ledger-anchor exports are always available.
- Added `CborLedgerCodec` as the built-in CBOR codec for `AllocationLedger`
  commit payloads.
- Clarified that the native ledger stack is `MemoryManager` ID 0 ->
  `ic-stable-structures::Cell<StableCellLedgerRecord, _>` ->
  `LedgerCommitStore` -> dual protected committed `AllocationLedger` payloads.
- Kept collection construction out of scope: `ic-memory` governs allocation
  ownership and does not wrap every `ic-stable-structures` collection.

---

## 0.2.0

### Breaking / API hardening

- Bumped from the already-published `0.1.0` to `0.2.0` because this release
  tightens public DTO construction and hides fields that were public in
  `0.1.0`.
- Made invariant-bearing durable DTO fields private where feasible, including
  allocation declarations, ledger histories, ledger records, physical commit
  slots, and slot descriptors.
- Added checked constructors and accessors for public allocation DTOs so callers
  do not need struct literals for normal use.
- Added `AllocationLedger::new_committed` for strict committed-ledger
  construction.
- Removed the unused public generation DTO API from the crate surface.
- Gated corrupt-write simulation helpers behind `#[cfg(test)]`; production code
  can no longer call them.

### Safety and validation

- Added schema metadata validation to declaration staging, reservation staging,
  and committed-ledger integrity validation.
- Centralized historical claim-conflict detection for declaration validation,
  declaration staging, and reservation staging while preserving existing public
  error variants.
- Preserved the core invariant: a stable key cannot move physical slots, and an
  active physical slot cannot be reused by another stable key.

### Structure and maintenance

- Split `slot` internals into descriptor, `MemoryManager`, and range-authority
  modules while keeping crate-level re-exports stable.
- Split ledger records, errors, and integrity checks out of the main ledger
  module.
- Kept staging and commit behavior public-compatible; no Canic-specific policy
  was added.

### Documentation

- Updated README, crate docs, rustdoc, and SAFETY docs for the current checked
  constructor/accessor API.
- Added a concise golden-path sketch showing recovery, declaration,
  validation, commit, and only-then-open ordering.
- Clarified stable-key permanence, reservation behavior, tombstones, checksum
  limits, non-goals, and the boundary between generic `ic-memory`
  infrastructure and Canic/IcyDB examples.

---

## 0.0.7

### Documentation

- Added stable-key formatting guidance to the README, including grammar rules,
  valid examples, and namespace conventions.
- Documented representative `canic.core.*` and `icydb.*` stable-key patterns.
- Clarified that stable keys are permanent logical allocation identities and
  should not be changed when only schema metadata changes.
- Updated README examples to show the open-stack range-authority model,
  package-record composition, and optional closed-policy coverage checks.

---

## 0.0.6

### Added

- Added `MemoryManagerRangeAuthority`, `MemoryManagerAuthorityRecord`, and
  `MemoryManagerRangeMode` for generic `MemoryManager` range authority policy
  and diagnostics.
- Added range-authority builders and validators, including ID-bound helpers,
  mode-aware validation, complete coverage checks, and `from_records` for
  composing records from multiple packages.
- Added concise `MemoryManager` declaration helpers on
  `AllocationDeclaration` and `DeclarationCollector`, including labeled,
  unlabeled, schema-aware, and builder-style variants.
- Added `MemoryManagerIdRange::all_usable`.

### Changed

- Made `MemoryManagerIdRange` serializable for diagnostic authority records.
- Added explicit range-authority errors for overlaps, invalid ranges, missing
  coverage, records outside a coverage target, mode mismatch, and invalid
  diagnostic strings.
- Updated examples to use the concise `MemoryManager` range and declaration
  helpers.

### Policy model

- Clarified that range authority is policy/diagnostic metadata only; durable
  allocation remains the core `stable_key -> allocation_slot` ledger model.
- Clarified that `Reserved` and `Allowed` do not allocate IDs.
- Clarified the open-stack model: packages publish only the ranges they own, and
  a final composition layer uses `from_records` to catch cross-package overlaps.
  Final closed policies may add application `Allowed` ranges or complete
  coverage checks, but intermediate frameworks should not claim the remaining ID
  space by default.

---

## 0.0.3

- Repositioned documentation around stable-memory slot drift.
- Added safety model documentation.
- Hardened physical/logical generation recovery.
- Added strict committed-ledger lifecycle tests.
- Made `MemoryManager` slot construction checked by default.
