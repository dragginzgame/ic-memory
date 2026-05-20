# Code Hygiene Audit - 2026-05-20

## Executive Summary

Risk score: **3/10**.

The crate is in good hygiene shape for the current release. Formatting,
tests, clippy, docs, feature checks, and whitespace checks all pass. The main
trust boundaries are now named clearly, `ValidatedAllocations` is not
serializable or publicly constructible, decoded ledger and declaration DTOs are
revalidated before authority, and the normal bootstrap path is cohesive.

The remaining issues are mostly documentation/API-surface hygiene around
advanced low-level APIs. The code exposes physical commit and compatibility
helpers that are legitimate for custom integrations, but some rustdoc wording
does not make the trust boundary sharp enough. Future maintainers could read
those helpers as the normal path and bypass the higher-level
`LedgerCommitStore` / `AllocationBootstrap` sequencing.

## Commands Run

All required checks passed:

```text
cargo fmt --all --check
cargo test -p ic-memory
cargo clippy -p ic-memory --all-targets -- -D warnings
cargo doc -p ic-memory --no-deps
cargo check -p ic-memory --all-features
cargo check -p ic-memory --no-default-features
git diff --check
```

Targeted inventories were built from:

```text
rg "unwrap|expect|panic!|todo!|unimplemented!" src tests
rg "pub " src
rg "Deserialize|Serialize|Default|Clone|Copy" src
rg "compatibility|unsafe|advanced|deprecated|TODO|FIXME|HACK" src README.md ADVANCED.md
rg "pub\\(|pub(crate)|pub struct|pub enum|pub trait|pub fn" src
rg "from_slice|from_bytes|decode|deserialize|Deserialize" src
rg "Result<|thiserror|panic!" src
```

## Inventory Summary

Authority-bearing values:

- `ValidatedAllocations`: in-memory authority, not serde, constructor is
  `pub(crate)`.
- `AllocationSession`: public session wrapper, requires `ValidatedAllocations`.
- Default runtime validated state: published only after bootstrap.

Decoded or durable DTOs:

- Ledger DTOs: `AllocationLedger`, `AllocationHistory`, `AllocationRecord`,
  `GenerationRecord`, `LedgerCommitStore`, `DualCommitStore`,
  `CommittedGenerationBytes`.
- Declaration DTOs: `AllocationDeclaration`, `DeclarationSnapshot`.
- Slot DTOs: `AllocationSlotDescriptor`, `AllocationSlot`.
- Diagnostic DTOs: `DiagnosticExport`, `DiagnosticRecord`,
  `DiagnosticGeneration`, range-authority exports.

Panic paths:

- Most `expect` calls are test-only.
- Macro registration/opening uses `expect` because ctor registration and macro
  open expressions cannot return a crate-specific `Result`.
- `StableCellLedgerRecord::from_bytes` panics because
  `ic-stable-structures::Storable::from_bytes` is panic-shaped; the runtime
  preflights stable-cell bytes through a fallible helper before `Cell::init`.

## High Findings

No high findings.

## Medium Findings

### M1. Physical commit API rustdoc can be read as the normal ledger path

Reference: `src/physical.rs:305`

Issue: `DualCommitStore::commit_payload_at_generation` says it is "the
preferred API for logical ledger commits", but the function accepts arbitrary
bytes and does not decode, run compatibility checks, or validate committed
ledger integrity.

Why it matters: The implementation is correct for a low-level physical commit
primitive, but the wording is too inviting. A future custom integration could
call this directly for ledger commits and accidentally skip `LedgerCommitStore`
validation.

Recommended fix: Change the rustdoc to say this is the preferred physical-slot
primitive used by `LedgerCommitStore`, and that normal ledger commits should go
through `LedgerCommitStore::commit` or `AllocationBootstrap`.

Suggested regression test: None needed for a docs-only fix. Existing tests
already cover the higher-level validation path.

### M2. Explicit compatibility APIs need stronger "advanced only" warning

Reference: `src/ledger/mod.rs:110`, `src/ledger/mod.rs:155`,
`src/ledger/mod.rs:182`

Issue: `recover_with_compatibility`,
`recover_or_initialize_with_compatibility`, and `commit_with_compatibility`
accept caller-supplied compatibility ranges. The names are clear, but the
rustdoc does not warn that broadening compatibility can make an older reader
accept bytes it does not actually understand.

Why it matters: This is not a bug in the default path. The default path uses
`LedgerCompatibility::current()`. It is a hygiene issue because advanced API
docs should make misuse hard.

Recommended fix: Add rustdoc stating that callers should use these only for
explicit migration/adaptor code, and should not broaden compatibility unless the
codec and integrity validation understand every accepted schema version.

Suggested regression test: Add a future-version fixture test if this becomes a
behavioral compatibility change. For the docs-only warning, no test is needed.

## Low Findings

### L1. Public decode helper should explicitly call returned record inert

Reference: `src/stable_cell.rs:161`

Issue: `decode_stable_cell_ledger_record` returns a decoded
`StableCellLedgerRecord`. The surrounding module explains that stable-cell
preflight is fallible, but this helper's own rustdoc could be more explicit that
the decoded record is not authoritative until the embedded commit store recovers
and validates a ledger.

Why it matters: The helper is useful for diagnostics and preflight. Clearer
rustdoc would align it with the crate-wide raw/decoded/validated/authority
language.

Recommended fix: Add one sentence: "The returned record is decoded DTO state,
not authority; recover through `LedgerCommitStore` before trusting its ledger."

Suggested regression test: None.

### L2. Stale module-split TODO remains in `LedgerCommitStore`

Reference: `src/ledger/mod.rs:80`

Issue: The TODO says commit/recovery should move to `ledger::commit` once the
staging split is mechanical. Staging has already been split, so this TODO is now
either actionable or stale.

Why it matters: TODOs in protocol code age poorly. They make it unclear whether
the current module shape is intentional or unfinished.

Recommended fix: Either file a follow-up and replace the TODO with a reference,
or remove it if the current layout is acceptable for 0.5.x.

Suggested regression test: None.

## Cleanup Findings

### C1. Panic inventory is acceptable but should stay classified

Reference: `src/lib.rs:153`, `src/lib.rs:167`, `src/lib.rs:193`,
`src/lib.rs:217`, `src/stable_cell.rs:68`, `src/stable_cell.rs:186`

Issue: Production panic paths exist in macro registration/opening and
`Storable` serialization/deserialization.

Why it matters: These are mostly structural limitations of ctor macros and
`ic-stable-structures::Storable`, not immediate defects. Still, they are exactly
the kind of paths that should stay visible in recurring audits.

Recommended fix: No immediate code change. Keep the fallible stable-cell
preflight test, and keep macro docs explicit that memory opening requires prior
bootstrap.

Suggested regression test: Existing tests cover corrupt stable-cell preflight
and pre-bootstrap runtime behavior. Add a macro pre-bootstrap panic test only if
the macro surface changes.

## No Findings

- No public constructor can fabricate `ValidatedAllocations`.
- No serde path currently deserializes authority.
- `validate_allocations()` validates ledger compatibility and committed
  integrity before producing authority.
- Declaration snapshots revalidate decoded declaration DTOs.
- Runtime late registration fails closed after bootstrap seals the registry.
- Default runtime bootstrap publishes validated allocations only after staging
  and commit.

## Quick Fix Checklist

- Tighten `DualCommitStore::commit_payload_at_generation` rustdoc.
- Add advanced-only warnings to explicit compatibility APIs.
- Mark `decode_stable_cell_ledger_record` output as inert decoded DTO state.
- Resolve or remove the stale `ledger::commit` TODO.

## Deferred Design Work

- Versioned golden-wire fixtures for durable CBOR formats.
- A formal migration policy for future ledger schema versions.
- A narrower advanced API shape for custom physical commit integrations.

## Audit Quality

Confidence is high for current crate hygiene because the command suite is green
and the inventories cover public API, serde DTOs, panic paths, errors, and
recent validation boundaries.

This pass did not re-audit Canic integration behavior or temporal compatibility
in depth. Those are separate audits and should remain separate so this recurring
pass stays small and repeatable.

The next pass would be stronger with generated public API docs or `cargo public-api`
style output committed as an audit artifact, plus golden wire fixtures for the
durable DTO inventory.
