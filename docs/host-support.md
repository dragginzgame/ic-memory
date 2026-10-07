# Host support and qualification

ic-memory supports native development and maintainer workflows on these hosts.
Its canister runtime target remains `wasm32-unknown-unknown`.

| Host | Architecture | Native CI label | Qualification |
| --- | --- | --- | --- |
| Ubuntu 24.04 | x86-64 | `ubuntu-24.04` | 0.25.12 development gate and Rust 1.88.0 all-target check passed |
| macOS 15 | ARM64 | `macos-15` | 0.25.12 development gate and Rust 1.88.0 all-target check passed |
| macOS 15 | x86-64 | `macos-15-intel` | 0.25.12 development gate and Rust 1.88.0 all-target check passed |

The passing development gates and MSRV checks are recorded in
[CI run 37323134668](https://github.com/dragginzgame/ic-memory/actions/runs/37323134668)
for source `5481bc765b9c7bed4cfb928c9efa9d1aa6838a22`, with selected lockfile
SHA-256 `b67be11d2a175dc0ab35cb1429548a7f5dfb1adc030d3693d9348bfd1a51e2d0`.
The development jobs used Rust 1.99.0 for native tooling tests and the full gate
described below. Separate MSRV jobs logged and used Rust 1.88.0 for all-target
compilation. Subsequent changes, including the common release workflow, require their own
matching native CI evidence.

The earlier
[0.25.11 run](https://github.com/dragginzgame/ic-memory/actions/runs/37320179267)
does **not** qualify Rust 1.88.0 on any host. Its MSRV jobs installed 1.88.0, but
the repository toolchain pin selected 1.99.0 for the dependency and compilation
commands, as recorded by `rustc -Vv` in their logs. The corrected 0.25.12 workflow
sets `RUSTUP_TOOLCHAIN` explicitly for those commands.

The labels follow the [GitHub runner image matrix](https://github.com/actions/runner-images/blob/main/README.md).
Configured jobs alone are not passing evidence. Linux and cross-compilation
results do not qualify native macOS behavior. Link matching successful CI runs
when recording release qualification; retain failed runs and their limitations.

## Prerequisites

- Rustup with Rust 1.99.0, Clippy, rustfmt and `wasm32-unknown-unknown`; also
  install the declared MSRV, Rust 1.88.0. Toolchain changes need a demonstrated
  reason and separate qualification.
- Git, GNU Make, Bash 3.2 or newer, curl, tar with gzip/xz support, Perl,
  `sed`, `awk`, and either `sha256sum` or `shasum` with SHA-256 support.
  On macOS, install Xcode Command Line Tools for native compilation and Git.
  The system `make` is sufficient when it is GNU Make; Homebrew `gmake` is an
  alternative. Do not substitute BSD Make.
- The tracked root and independent runtime-qualification `Cargo.lock` files,
  with cached dependencies before offline validation. In a fresh checkout run
  `make fetch-dependencies` for the root selection. Prepare the independent graph
  separately with `cargo fetch --locked --manifest-path testing/runtime-qualification/Cargo.toml`
  when installed qualification is needed. Preserve both selections; never
  regenerate them to make a check pass.
- The common jq/yq/ripgrep set under `.tools/host/bin` for `make check-pins`
  and `make test-pins`. The reviewed pins live in `ci/tool-versions.env`.
  Interrupted manifest/lockfile recovery also uses yq to locate Cargo's selected
  target directory. Checks are offline and never install prerequisites.
- Actionlint and ShellCheck for `make lint-tooling`. CI installs exact
  shared versions and hashes from `ci/tool-versions.env` using the
  reviewed shared installers. Local installation is a separate network step.
- Crates.io credentials are needed only for maintainer publication. Tests use
  deterministic command substitutes and never need credentials or network.

## Developer setup and formatting

Install the exact manifest formatter separately from validation, then activate
the reviewed hook once per clone (also run the installer after setup updates):

```sh
source ci/tool-versions.env
cargo +1.99.0 install cargo-sort --version "$SHARED_TOOLING_CARGO_SORT_VERSION" --locked
make install-hooks
git config --get core.hooksPath # .githooks
make fmt-check
```

CI installs the same cargo-sort 2.1.4 explicitly. `make fmt` sorts manifests and
formats Rust in both the root workspace and `testing/runtime-qualification`.
`make fmt-check` checks the same inputs without changing them. Neither command
builds, installs tools, fetches dependencies or changes selected lockfiles.
Both first use the shared prerequisite checker to require the exact cargo-sort
pin and available rustfmt under the selected validation toolchain. Admission is
offline and probes versions only; unavailable tools fail before formatting.
Bare `make` prints available commands rather than preparing dependencies.

Prepare the common host and IC tool sets explicitly for the detected host:

```sh
make install-tools
make tools-check
export PATH="$PWD/.tools/host/bin:$PWD/.tools/ic/bin:$PATH"
make check-pins test-pins
```

Make selects those local paths automatically. Setup does not edit shell profiles.
See [common local setup](local-setup.md) for system bootstrap packages and
[IC tools](ic-tools.md) for the selected Quill, ICP CLI, didc, ic-wasm, PocketIC
and wasm-opt set. Host and IC setup activate independently; previous selections
and failed candidates stay under `.tools/`. `install-host-tools` and
`install-ic-tools` can prepare either set separately, with offline checks through
`host-tools-check` and `ic-tools-check`. Ordinary validation never installs tools.
CI explicitly installs and checks both sets on each declared native test host;
configured jobs alone do not qualify this adoption. The pin checker requires locks already tracked
by Git: agents leave new locks unstaged, so the maintainer must commit their
adoption before the real-checkout declaration/release gate can pass.

`make check-pins` also checks Cargo inheritance against each manifest's owning
workspace catalog. The independent runtime-qualification root keeps its approved
scope and lockfile; it is not merged into the library graph. `make test-pins`
reuses the shared metadata fixture for ordinary/inline dependency tables,
aliases, target/dev/build declarations, rejected child overrides and independent
workspace discovery. Its workspace-version reader is a fixture dependency;
the Rust release adapter still owns this crate's `package.version`.

The actionlint and ShellCheck setup commands delegate to the shared
`install-ci-tool.sh` implementation. Existing arguments and pins are unchanged.
Rejected checksums or versions preserve the selected executable and retain the
failed candidate. Snapshot integrity and focused Linux checks are separate from
native macOS qualification; see the
[adoption evidence](release-workflow-qualification.md#0283-shared-cargo-and-installer-adoption).
Both host installation and offline verification select the shared
`--with-ripgrep` option. It authenticates ripgrep 15.2.0 and its PCRE2 support
alongside jq/yq; CI uses this same set rather than a separate package-manager
installation. Prepare the expanded host set with `make install-host-tools`
before offline checks. Previous selections and rejected candidates are retained.

The pre-commit hook formats an export of the exact index and refreshes only the
selected files. It refuses partial staging and preserves unrelated working
edits; a formatter failure leaves the real index and files unchanged. The
installer refuses conflicting hook settings or executable private hooks rather
than silently replacing them. Reconcile those obligations before activating it.
The recorded shared installer resolves the physical checkout path, including
when entered through a logical alias such as macOS temporary paths.
CI and release preparation check formatting independently of hook activation.
`make test-hooks` exercises selected-file refresh, partial source/config staging,
formatter failure isolation and preservation of unrelated edits in disposable
repositories without creating commits or tags, including setup through a checkout
path alias. It uses the actual consumer
formatting targets and includes the shared prerequisite rejection fixture. It
runs on every declared native CI host. See the
[0.28.4 focused evidence](release-workflow-qualification.md#0284-portable-fixtures-and-formatter-admission).
Linux hook evidence
does not qualify native macOS; record matching native runs separately from the
historical runtime gates above.

## Native checks

Focused tooling checks are `make verify-shared-tooling tools-check test-tools check-pins test-pins test-tooling test-hooks fmt-check`,
`make test-release-adapters test-release-runner` and `make lint-tooling`. Compilation must wait for any existing build to finish.

CI runs `make validate-toolchain` and the offline all-target MSRV check on each
declared host. The full gate covers library/integration/compile-fail tests,
doctests, strict Clippy, warning-denied documentation, Wasm compilation and raw
size budgets, and package verification. Full gates run only on explicit request
or in their configured CI pipeline.

Release-flow tests separately cover Rust adapter recovery/input binding, actual
Make dispatch/log retention and canonical runner reconciliation through
substituted command effects. See the [focused evidence](release-workflow-qualification.md).
They do not prove a live push or publication. The native release prerequisites
and evidence checks are shared by the maintainer workflow in
[RELEASING.md](../RELEASING.md). Live publication and downstream deployment need
their own authorization and observations.

The repository helper uses registry `ic-host-fs` for streaming hashes and
durable file replacement, and `ic-host-artifacts` for digest parsing. Both
development dependencies disable unused default features and are selected only
for native targets. Durable publication synchronizes files and parent directories;
an error after rename may leave complete replacement bytes visible. Release
recovery reconciles those bytes against the saved intent before restoring or
retrying. These dependencies are absent from canister graphs. `shasum` remains a setup
and CI prerequisite through common scripts, rather than a release adapter
subprocess. Product receipts, release identities and Wasm budgets stay local.

Native CI routes disposable fixtures into a dedicated runner temporary directory.
Failed validation uploads those fixtures, setup/gate logs, retained tool candidates
and qualification receipts for 14 days, including hidden recovery evidence.
MSRV failures upload their compiler log separately. Logging preserves the failing
command's status through Bash pipefail; uploads run only after a failed job step.
Configured uploads and local substitute checks do not prove a live artifact upload.

For interactive CI inspection, use the unchanged shared helper:

```sh
GH_REPO=dragginzgame/ic-memory bash scripts/dev/gh-ci.sh --commit HEAD --all-workflows --limit 100
```

This requires an authenticated GitHub CLI session. The bounded listing is evidence
for the resolved commit; apply the [maintenance rule](../rules/agent-maintenance.md)
before claiming complete workflow coverage. Uncommitted edits have no CI result.

## Rust workspace locations

The repository root is a virtual Cargo workspace. Shared package metadata,
dependencies, profiles and the selected lockfile remain there; the library
package, examples, tests, current wire fixtures and full package guide live in
`crates/ic-memory/`. Existing root Make/Cargo commands select this sole default
member. A checkout path dependency must point to `crates/ic-memory`, for example
`ic-memory = { path = "../ic-memory/crates/ic-memory" }`.

The approved independent root remains `testing/runtime-qualification/`, with
its own lockfile and sole package at
`crates/ic-memory-runtime-qualification/`. Its locked fetch/run commands and
caller-owned PocketIC server requirements retain that root. Layout adoption
does not merge or update either dependency graph. Package archives contain the
library source, examples, tests, fixtures, package guide and root-owned MIT
license, copied unchanged from the repository-owned `LICENSE`; repository CI,
governance and operational evidence remain outside the
crate archive.
