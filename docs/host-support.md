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
compilation. The 0.25.13 changes require their own subsequent native CI run.

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
- Git, GNU Make, Bash 3.2 or newer, `sed`, and `shasum` with SHA-256 support.
  On macOS, install Xcode Command Line Tools for native compilation and Git.
  The system `make` is sufficient when it is GNU Make; Homebrew `gmake` is an
  alternative. Do not substitute BSD Make.
- A selected `Cargo.lock` and cached dependencies before offline validation.
  This library keeps its development lockfile untracked. In a fresh checkout,
  explicitly select it with `cargo generate-lockfile`, then run
  `make fetch-dependencies`. Preserve that file throughout qualification and
  release; do not regenerate it after validation.
- Actionlint and ShellCheck for `make lint-tooling`. CI installs exact
  consumer-owned versions and hashes from `ci-tool-versions.env` using the
  reviewed shared installers. Local installation is a separate network step.
- Crates.io credentials are needed only for maintainer publication. Tests use
  deterministic command substitutes and never need credentials or network.

## Native checks

Focused tooling checks are `make verify-shared-tooling test-tooling fmt-check`
and `make lint-tooling`. Compilation must wait for any existing build to finish.

CI runs `make validate-toolchain` and the offline all-target MSRV check on each
declared host. The full gate covers library/integration/compile-fail tests,
doctests, strict Clippy, warning-denied documentation, Wasm compilation and raw
size budgets, and package verification. Full gates run only on explicit request
or in their configured CI pipeline.

Release-flow tests cover rollback, evidence refusal, maintainer command
arguments, and final archive qualification through substituted command effects.
They do not prove a live push or publication. The native release prerequisites
and evidence checks are shared by the maintainer workflow in
[RELEASING.md](../RELEASING.md). Live publication and downstream deployment need
their own authorization and observations.
