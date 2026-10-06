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
- The common jq/yq pair under `.tools/host/bin` for `make check-pins`; ripgrep
  for `make test-pins`. The reviewed pins live in `ci/tool-versions.env`.
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
formatting targets and runs on every declared native CI host. Linux hook evidence
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

The repository helper uses the registry `ic-host-tools` development dependency
for streaming file hashes and digest parsing. It is selected only for native
targets and is absent from canister dependency graphs. `shasum` remains a setup
and CI prerequisite through common scripts, rather than a release adapter
subprocess. Product receipts, release identities and Wasm budgets stay local.
