<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-memory/ic-memory-readme-header.svg" alt="IC Memory — Internet Computer helper library" width="100%">
</p>

# Simplification follow-up — 0.16.1 baseline

Reviewed on 2026-10-03 against released commit `96e4146`. All three changes
described here shipped in **0.17.0**, released at `a72a4a4`. The validation below
records the pre-release audit; subsequent work is recorded in the
[0.17.0 follow-up](simplification-followup-0.17.0.md).

The remaining avoidable complexity was concentrated in diagnostics and opening:
validated snapshot/capability facts were treated as fallible again, and doctor
validation received two dependent descriptions of the same recovery state.
These can be simplified at their existing owners. No new abstraction, policy
mode or compatibility path is needed.

## Ranked findings

| Rank / confidence | Exact owner and affected callers | Evidence and redundancy | Smallest change / obligations / verification |
| --- | --- | --- | --- |
| 1 — Confirmed; implemented | `MemoryRuntime::allocation_declarations` and `measure_allocations` in [allocations.rs](../../../src/runtime/allocations.rs); `RuntimeDiagnosticError` in [error.rs](../../../src/runtime/error.rs). Detailed and numeric allocation reports, including default-runtime adapters. | `registry::build_snapshot` rejects more than 254 external declarations/requests or ranges and adds one governance range. `StaticMemoryDeclaration::new` validates decoded slots. `SealedDeclarationSnapshot` has private immutable storage, no deserializer and one checked construction owner. Its diagnostics cannot reach either `AllocationBound` or the slot-error conversion. | Return the binding directly; rely on the sealed-slot invariant; remove both unreachable diagnostic error variants. Keep raw DTO validation, manager-layout reads, live-manager agreement and reentry checks. The new full external-ID-domain regression checks bindings, conservation, exact bounded reads, no writes/growth and agreement between detailed/numeric reports. Existing invalid-decoded-registration and corrupted-layout tests remain. |
| 2 — Confirmed; implemented | `diagnostic_validation` and `diagnostic_validation_ledger` in [diagnostics.rs](../../../src/runtime/diagnostics.rs), called by `MemoryRuntime::doctor_report`. | The doctor derives recovery exclusively from the decoded record, then passed both to validation. The record-present/recovery-absent genesis branch was unreachable. Physical `select_authoritative_slot` returns `NoValidGeneration` only for two absent slots; every invalid present slot rejects first. | Drop the redundant record argument and impossible branch. Use the recovery result to distinguish unreadable storage, empty commit storage, valid recovery and failed recovery. Retain borrowed successful recovery, empty-store genesis validation, typed unsupported-format classification and rejection without mutation. The new public doctor regression covers both empty representations and a readable record containing a corrupt physical slot. |
| 3 — Confirmed; implemented | `MemoryRuntime::memory_id` in [mod.rs](../../../src/runtime/mod.rs), `verify_requirement` in [adoption.rs](../../../src/runtime/adoption.rs), and `RuntimeOpenError` in [error.rs](../../../src/runtime/error.rs). Owned/default opens, ID lookup and host adoption. | ID lookup reads only immutable `CommittedAllocations` produced after declaration validation, staging and persistence. Adoption compares declarations from opaque sealed snapshots. Neither path can return `RuntimeOpenError::MemoryManagerSlot`; decoded DTOs cannot construct either authority type. | Remove the unreachable open-error variant and its error conversions; rely on committed/sealed slot invariants. Preserve malformed-key, not-bootstrapped, missing-key, governance, fixed-ID, authority and metadata rejection. Existing opening/adoption tests pass; the open mismatch regression now also checks caller-supplied sentinel ID 255 with the exact typed mismatch. |

## API and compatibility consequences

This is a source/API hard cut: remove
`RuntimeDiagnosticError::AllocationBound` and
`RuntimeDiagnosticError::MemoryManagerSlot`, and
`RuntimeOpenError::MemoryManagerSlot`. Consumers naming those variants
must remove the obsolete match arms or constructions. No deprecated aliases are
added. The corresponding `From<MemoryManagerSlotError>` conversions into the
runtime diagnostic/open error enums are removed as well. Slot validation and its
typed errors remain at raw declaration and
durable recovery boundaries. This cut shipped in the minor release **0.17.0**.

Canic and IcyDB local source searches found no references to the removed
variants. The additional open-variant search also covered local Rust sources
under `/home/adam/projects`. That is caller evidence, not complete downstream build or
deployment qualification. Diagnostic DTO shapes, public function signatures,
durable encoding, checksums and snapshot fingerprints are unchanged.

No compatibility removal is blocked on migration evidence in this patch. No
coordinated sibling implementation change is established. Application-owned
store framing, replay receipts and auth repair remain with their consumers.

## Reviewable sequence and priorities

1. Remove allocation-report revalidation and its dead error variants together.
   Keep the full-domain behavioral regression with that change.
2. Make the recovery result authoritative for doctor validation; include the
   empty/corrupt storage regression and update the existing typed-format test.
3. Remove the corresponding unreachable open-error variant and conversions;
   keep the raw-input rejection, committed-open and sealed-adoption coverage.

The API cut and focused validation are recorded in the 0.17.0 changelog.
Versioning, commits, tags and pushes remain maintainer-owned.

## Complexity retained

- Physical dual-slot validation, equal-generation ambiguity rejection and
  fail-closed handling of any invalid present slot protect durable authority.
- CBOR syntax preflight and bounded serde visitors enforce different limits
  before untrusted decoding/allocation. Explicit optional fields and current
  JSON byte arrays are current formats, not migration shims.
- Staging can receive an independently constructed ledger at the same
  generation; claim checks and raw reservation validation remain necessary.
- Detailed reports own per-ID strings; numeric summaries avoid those copies.
  Their shared measurement flow and different outputs remain useful.
- Compile-fail capability/admission/growth tests enforce live boundaries.
  Current-format fixtures and decoder-cause assertions protect format rejection
  and diagnostic evidence; no anti-resurrection test was identified for deletion.

## Validation and coverage limits

Passed: 262 library tests, integration tests, seven compile-fail cases, composed
host regressions and five doctests (five existing sketches ignored); strict
all-target Clippy and Rustdoc; Rust 1.88 all-target checking; Wasm test compilation;
formatting, whitespace checks and offline package verification.

All existing raw Wasm gates pass: core 249,906 bytes, diagnostics 298,166,
key-only 249,524, admission 253,036 and runtime integration 259,465. These are
current gate measurements, not matched consumer performance or IC cost deltas.

This pass traced allocation attribution, export/doctor preparation, snapshot
construction/resolution, range bounds, physical selection, ledger recovery,
committed capability construction and owned/default opening/adoption.
It revisited staging boundaries and searched current code/tests for obsolete
machinery and implementation-coupled assertions. It does not repeat every earlier
audit of macro grammar, registry/TLS concurrency, growth accounting or release
tooling. Full downstream suites, live IC behavior, Lean/whitepaper builds and
release-flow tests were not run. Release-flow tests create commits/tags and are
excluded by repository ownership instructions. No additional probable or
unresolved local deletion is promoted to a finding.
