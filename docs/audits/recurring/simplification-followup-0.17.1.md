<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-memory/ic-memory-readme-header.svg" alt="IC Memory — Internet Computer helper library" width="100%">
</p>

# Simplification follow-up — 0.17.1 baseline

Reviewed on 2026-10-03 against released commit `197477b`, starting with a clean
tree. The changes below shipped in 0.18.0, release commit `8743499`. The previous
audit shipped in 0.17.1. The next follow-up is recorded in the
[0.18.0 baseline audit](simplification-followup-0.18.0.md).

Staging still treated immutable validated declarations as raw DTOs, with count,
schema and numeric-overflow errors that public callers could not reach. Record
helpers then validated the same schema again. Keep validation where raw input
enters and make the record update depend on those established facts.

| Finding / confidence | Owners and callers | Evidence and smallest simplification | Obligations, risk and verification |
| --- | --- | --- | --- |
| 1 — Confirmed; implemented — redundant validated-staging failures | `AllocationLedger::stage_validated_generation` in [stage.rs](../../../src/ledger/stage.rs), `AllocationStageError` in [error.rs](../../../src/ledger/error.rs), and `ValidatedAllocations` in [capability.rs](../../../src/capability.rs). Generic/runtime bootstrap and manual staging use this path. | The only production proof constructor is `validate_allocations`: it checks decoded snapshot count, slots, uniqueness and schema metadata before storing immutable declarations. Recovery/commit establish a bounded contiguous history, so the proof's base generation cannot approach `u64::MAX`. Staging first requires that receiver generation to match. Remove declaration/schema revalidation and the three unreachable stage-error variants; use those established bounds for advancement/count conversion. | Source/API hard cut; no durable format change. Keep stale-proof rejection, receiver bounds, claim conflicts, retirement and final output bounds. Existing history-limit coverage retains the actual finite-history failure. New decoded-snapshot and full-slot-domain tests exercise the real proof boundary; a real proof rejects a `u64::MAX` receiver as stale. |
| 2 — Confirmed; implemented — repeated schema checks in record updates | Private `AllocationRecord::{from_declaration, active, reserved, observe_declaration, observe_schema}` in [record.rs](../../../src/ledger/record.rs), called by `record_declaration` and `record_reservation` in [stage.rs](../../../src/ledger/stage.rs). Other uses are checked test fixtures. | Active inputs come from the proof above. Reservation staging validates each raw declaration before its record update. Make these private record helpers infallible, construct the same schema record directly, and remove the reservation wrapper that only forwarded schema observation. Keep one schema-history update owner. | Low internal risk within the same API-cut patch. Preserve reserved-to-active promotion, unchanged-schema suppression, last-seen updates, generation ordering and cloning before mutation. Public `SchemaMetadataRecord::new` still validates raw metadata. Reservation schema/count errors remain reachable and unchanged. Lifecycle, reservation, fixture and history tests cover both helper callers. |

## API cut and review sequence

Remove `AllocationStageError::TooManyDeclarations`,
`AllocationStageError::InvalidSchemaMetadata` and
`AllocationStageError::GenerationOverflow`. Consumers naming these variants
must remove those match arms or constructions. Use snapshot validation for
declaration/schema rejection and ledger integrity for the finite history limit.
Raw reservation and retirement overflow errors remain supported. Do not add
aliases or deprecated variants. This requires the next minor release,
**0.18.0**; this audit does not bump package versions.

A local Rust search under `/home/adam/projects`, excluding build/vendor output,
found references to those three variants only in ic-memory. This is source
search evidence, not complete downstream build or deployment qualification.

Review the staging/record/error changes with the behavioral tests together, then
the capability documentation and audit/release-status updates. All removals are
local; no coordinated consumer implementation change or migration requirement
was established. The tests that fabricated invalid schema or overflow proofs
were replaced with real input/proof boundary coverage. Capability compile-fail
tests remain because constructor privacy is a live authority invariant.

## Complexity retained and limits

- A proof can come from another valid ledger at the same generation. Claim
  validation at staging therefore remains necessary; the existing conflict test
  now constructs that proof through public recovery and validation and checks
  rejection without modifying the receiver.
- Staging clones before updates and checks receiver/output resource bounds.
  Ledger DTOs can be independently decoded or assembled; they are not proof
  objects. This patch does not make arbitrary receiver histories trustworthy.
- Reservations accept raw declarations. Their count, schema, slot, key and
  lifecycle checks stay at that boundary. Active promotion and reservation
  refresh retain distinct semantics and error precedence.
- Physical selection, current-format decoding, checksums and persistence
  confirmation still establish durable authority before opening storage.

This pass traces proof construction, active/reservation staging, record mutation,
committed history bounds, public metadata constructors and relevant fixtures.
It does not comprehensively repeat registry concurrency, macros, growth,
release tooling or decoder preflight audits. No remote issue check, full
downstream suite, live IC run, Lean/whitepaper build or Git-mutating release-flow
test is included. No further unresolved deletion is promoted to a finding.

## Validation

Passed: 263 library tests, integration tests, seven compile-fail cases, two
composed-host regressions and five doctests (five existing sketches ignored);
strict all-target Clippy and Rustdoc; Rust 1.88 all-target checking; Wasm test
compilation; offline package verification; formatting and whitespace checks.
The final public-proof/different-history conflict regression also passes after
replacing its private proof fixture.

All five unchanged raw Wasm gates pass: core 249,456 bytes, diagnostics 297,505,
key-only 249,076, admission 252,537 and runtime integration 258,852. These are
current gate measurements, not matched consumer or IC execution cost deltas.
Current durable fixtures, checksum expectations and declaration fingerprints
remain unchanged.
