# Rust workspace layout

These rules are part of the mandatory [engineering baseline](../DRAGGINZGAME.md).
They standardise maintained Rust package locations while preserving locally
owned APIs, canister identities, validation gates and deployment configuration.

## Standard layout

Every repository containing maintained Rust packages uses this layout, including
repositories with only one package:

```text
Cargo.toml                 # Virtual workspace; no [package]
Cargo.lock                 # Selected workspace dependency graph
crates/
  package-name/
    Cargo.toml             # [package].name = "package-name"
    src/
    tests/                 # When needed
```

- The repository root is a virtual workspace: its `Cargo.toml` declares
  `[workspace]`, without `[package]`, package targets or package dependency
  tables. Select the resolver explicitly using the supported toolchain; a layout
  change does not authorize a resolver, edition or MSRV upgrade.
- Put each maintained Rust package directly under `crates/<package-name>/`,
  using its existing Cargo package name. This includes libraries, binaries,
  canisters, schema packages, and separately packaged examples, tests and probes.
  Keep ordinary module tests, examples and fixtures with their owning package;
  they do not need separate packages merely to follow this layout.
- Declare the maintained members in the root workspace. Keep shared package
  versions and common package metadata in `[workspace.package]`, with inherited
  child values. Keep the complete direct dependency catalog in
  `[workspace.dependencies]` under the [Cargo dependency rules](cargo-dependencies.md).
  Preserve package identities, selected dependencies and effective features.
- Keep the selected `Cargo.lock`, workspace profiles and workspace-wide Cargo
  configuration at their owning root. Preserve intentional `default-members`,
  target distinctions and publication selections; a directory move must not
  silently widen normal builds or published packages.
- Application/deployment configuration, frontend code, scripts and documentation
  retain their locally owned locations. For example, an `apps/` directory may
  keep fleet configuration that points to Rust packages under `crates/`.
  Moving a package does not rename a deployed canister or authorize deployment.
- Repositories without maintained Rust packages do not need a Cargo workspace
  or an empty `crates/` directory. Generated scratch projects, vendored upstream
  sources and frozen historical evidence are not maintained package locations;
  do not move them or rewrite history to satisfy this layout.

## Independent workspaces and exceptions

The default is one workspace and one selected lockfile. A separate maintained
workspace requires a maintainer-approved exception in the local `AGENTS.md`,
naming its root, package scope, reason, lockfile and validation coverage. Preserve
existing approved dependency-graph boundaries; layout standardisation alone
does not authorize merging their graphs or upgrading their dependencies.

An approved independent workspace uses the same virtual-root and
`crates/<package-name>/` shape relative to its own root. For example, an approved
`testing/Cargo.toml` owns `testing/crates/<package-name>/` and `testing/Cargo.lock`.
Keeping a package outside this shape, or keeping a root package, requires an
explicit layout exception as well. A path's existing name or use as a test,
example or canister is not implicit approval.

## Adoption and verification

Adopt this rule through a reviewed [governance snapshot](../docs/consuming-snapshots.md).
Coordinate consumer work through issues and preserve unrelated working changes.
In the owning repository's authorized change:

1. Inventory maintained manifests, workspace roots, selected lockfiles and
   approved exceptions. Record old-to-new package paths before moving files.
2. Update workspace membership, dependency paths, package include/readme/license
   paths, build scripts, fixtures, source-relative paths, Make/CI commands,
   publication inputs and deployment package references together. Change any
   maintained scaffolding at its generator. Retire obsolete source locations
   rather than leaving duplicate packages or symlink aliases.
3. Preserve package names, versions, APIs, features, publication policy, canister
   identities and selected dependencies. Review packaging contents and
   `CARGO_MANIFEST_DIR`-relative inputs; a successful directory move alone does
   not establish equivalent behavior.
4. Run the owning focused locked metadata, formatting and path/packaging checks
   for every affected workspace with prepared tools and caches. Keep the existing
   Linux/macOS qualification obligations and local command authority. Adoption
   does not add a broad gate or authorize publication, cleanup or version changes.
5. Verify the adopted snapshot and update local instructions and documentation.
   Distinguish committed adoption from local preparation and native execution
   evidence from source inspection.

Source reports must independently exclude Cargo's selected build-output directory
using metadata, including custom target directories. The standard layout keeps
default build output outside package trees, but is not a substitute for correct
file inventory in the reporting helper.
