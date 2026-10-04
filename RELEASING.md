# Releasing ic-memory

This guide is for repository maintainers. Application users do not need these
steps to depend on `ic-memory`.

The release targets follow Canic's validate, bump, commit, tag, and push flow,
adapted for this single library crate. They require Python 3.11+, Git, Make,
Rust 1.99.0 with Clippy/rustfmt and `wasm32-unknown-unknown`, and the declared
MSRV toolchain. Publishing also requires crates.io credentials configured for
Cargo.

The maintainer exclusively owns commits, tags, and pushes. Automation and LLM
agents prepare source, documentation, and validation in the working tree but do
not perform those actions. See [AGENTS.md](AGENTS.md).

## Before preparing a version

Commit the implementation and add a nonempty, numbered entry at the top of
`CHANGELOG.md` for the next version. The release workflow requires committed
source and a clean working tree.

Run the complete validation when needed:

```sh
make validate
```

This runs release-flow regression tests, formatting, strict Clippy, serialized
Rust tests and doctests, Wasm budget regression tests, Wasm checks and size
budgets, the declared MSRV check, and package verification.

CI shares its installed-toolchain checks with `make validate-toolchain` and
reads the MSRV from `Cargo.toml` in a separate job. Development, CI, and the
default `VALIDATION_TOOLCHAIN` use the Rust 1.99.0 pin from
`rust-toolchain.toml`; the crate's declared MSRV is Rust 1.88.0.

## Prepare and release

The maintainer can run the complete patch or minor workflow:

```sh
make release-patch   # Validate, bump patch, commit, annotate tag, push
make release-minor   # Validate, bump minor/reset patch, commit, annotate tag, push
make publish-dry-run # Verify the tagged release without uploading
make publish         # Publish the tagged release to crates.io
```

The release targets push the current branch and its `vX.Y.Z` tag atomically to
`origin`. Publication is a separate command. `PUBLISH_DRY_RUN=1 make publish`
also performs a dry run. The branch must already exist on `origin`, and the
refreshed remote branch must be an ancestor of the local source commit.

For review between steps, `make patch` and `make minor` stop after validation
and version preparation. The maintainer then runs, in order:

```sh
make release-stage
make release-commit
make release-push
```

A rejected push can be retried with `make release-push`; do not bump the version
again. A failed tag step can be retried with `make release-commit` without
making another commit.

Preparation updates only `Cargo.toml` and the README dependency example and
refreshes the ignored local `Cargo.lock`. It verifies the final package and
restores those files if preparation fails. Release commits must contain only
the expected version edits and are bound to the validated source commit. Dirty
trees, stale prepared state, unrelated staged changes, and conflicting release
tags are rejected. The lockfile remains untracked.

## Focused tooling checks

`make test-release-flow` exercises release commands in disposable repositories
with a fake Cargo executable. It never publishes packages or contacts a hosted
Git remote.

`make test-wasm-size` checks artifact discovery and budget failures with a fake
Cargo executable and no Git operations. `make wasm-size` resolves the real
artifact directory through Cargo metadata, including `CARGO_TARGET_DIR` and
Cargo configuration, so stale files in another target directory cannot satisfy
the budgets.
