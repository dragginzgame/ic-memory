# ic-memory code hygiene overlay

Use the unchanged [shared code hygiene method](../../../audits/code-hygiene.md)
and [audit contract](../../../audits/README.md), recorded at Shared Tooling
`a3430b34b32a60f3b245a2b4f7e2f5321556fe56` in
[the snapshot](../../../.shared-tooling.snapshot). The shared method owns generic
questions, severity, verdicts, evidence and repair authority. This overlay owns
the crate-specific obligations below. An audit does not authorize fixes.

## Product authorities and traces

- Current [safety contract](../../../SAFETY.md), [advanced API](../../../ADVANCED.md),
  [current ledger](../../current-ledger.md) and [operations](../../operations.md).
  Review current contracts; historical reports do not establish present behavior.
- Public API roots: `crates/ic-memory/src/lib.rs`, `crates/ic-memory/src/capability.rs`, `crates/ic-memory/src/declaration.rs`,
  `crates/ic-memory/src/policy.rs`, `crates/ic-memory/src/slot/`, `crates/ic-memory/src/runtime/`, and `crates/ic-memory/examples/composed_host.rs`.
  Inventory constructors separately from inert/diagnostic DTOs and manual owners.
- Trust transitions: declarations/decoded DTOs → validation → staged ledger →
  physical persistence → committed capability → runtime publication. Trace
  `crates/ic-memory/src/cbor.rs`, `crates/ic-memory/src/validation.rs`, `crates/ic-memory/src/ledger/`, `crates/ic-memory/src/physical.rs`,
  `crates/ic-memory/src/stable_cell.rs`, `crates/ic-memory/src/bootstrap.rs`, `crates/ic-memory/src/registry.rs` and `crates/ic-memory/src/runtime/`.
  Recovery must reject malformed current-format bytes and corrupt/ambiguous slots
  before issuing authority. Keep constructor/deserialization invariants aligned;
  decoded DTOs and diagnostics never grant memory access.
- Admission and warm adoption remain distinct. Verify explicit grants, logical
  placement, per-thread native bootstrap ordering, nonconstructing observations
  and the sole owned manager. Manual persistence confirmation remains caller-owned.
- Classify panic sites against stable-memory input reachability and the substrate's
  `Storable` contract. Preserve typed distinctions where operator action differs:
  corruption, invalid input, unsupported format, policy rejection and readiness.
- Trace bounds, checked arithmetic, growth failure, zero-length/page-edge IO,
  bounded decode/recovery, retained bytes and physical allocation accounting.
  Never describe allocated backing bytes as application payload occupancy.
- Consumer policy stays outside ic-memory. README/examples should show the shortest
  supported path; advanced authority/recovery caveats belong in ADVANCED/SAFETY.
  Archived API names and measurements retain their original source identities.

## Focused evidence

Inspect `crates/ic-memory/src`, `crates/ic-memory/tests`, `crates/ic-memory/examples`, `crates/ic-memory/fixtures/current`, `testing/runtime-qualification`,
Make/CI and maintained guides for the selected scope. Trace the macros in
`crates/ic-memory/src/lib.rs` through the registry, examples and compile-fail consumers. Runtime,
host release adapters and the independent PocketIC harness have different contracts;
name omitted families rather than implying they passed.

Use `rg` inventories for public/serde/panic/unsafe sites and their callers.
Prefer existing negative behavior and compile-fail capability tests over new
private-layout assertions. Declaration-registry tests mutate shared state: run
their selected cases with `--test-threads=1`.

- Formatting: `make fmt-check` checks both workspaces without builds or installs.
- Product changes: select the affected Rust test on the pinned toolchain using
  `cargo +1.99.0 test --locked --offline <selection> -- --test-threads=1`.
- Host tooling: select `make verify-shared-tooling`, `test-tools`, `test-pins`,
  `test-tooling`, `test-release-adapters`, `test-release-runner`, `test-hooks`
  or `lint-tooling` as relevant. Scripts use disposable substitutes where effects
  would otherwise create commits, tags or publication; never run the real hook.
- Check active builds before compilation or edits. Full validation, Wasm budgets,
  packaging, installed PocketIC and broad MSRV qualification require explicit
  request or configured CI. Audits do not automatically install tools or fetch.
- Performance evidence follows the owning qualification procedure, for example
  [runtime IO](../../runtime-io-qualification.md). Bind matched bytes/instructions/
  timings to compiler, lock, artifact and workload; counts are inspection aids.

## Reports and history

Keep reports in `docs/audits/recurring/`, using distinct dated filenames for new
runs. Record source/dirty scope, shared revision and overlay hash, chosen families,
checks and gaps under the common contract. This is not an automatic schedule.

The [frozen prior definition](frozen/code-hygiene-19efb34.md) reproduces historical
method evidence only and is ineligible for new runs. Old composite risk scores
are `N/A (method change)` against the shared finding-based method. Preserve
historical reports unchanged; any reused test evidence needs matching assertions
and identities. [Adoption review](shared-tooling-adoption-2026-10-06.md) maps the
prior obligations without claiming a fresh product audit.
