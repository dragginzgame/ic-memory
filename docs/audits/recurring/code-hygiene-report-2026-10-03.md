<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/ic-memory/main/images/ic-memory-readme-header.svg" alt="IC Memory — Internet Computer helper library" width="100%">
</p>

# Code hygiene audit — 2026-10-03

## Summary

Baseline: released ic-memory 0.15.4, including the documentation review already
in the working tree. The fixes below are uncommitted working-tree changes.

The audit found one bootstrap lifecycle inconsistency, an excess-element decode
in the shared bounded visitor, and several redundant abstractions and copies.
They have been fixed without changing public API signatures, persisted encoding,
checksum bytes or the sealed-snapshot fingerprint algorithm. No high-severity
finding or retained backwards-compatibility reader was identified.

Residual hygiene risk: **2/10**, a subjective assessment after the fixes.

## Findings and fixes

The bootstrap and bounded-decoder fixes are behavioral. The remaining findings
are mechanical refactors that preserve existing behavior.

| Severity | Location | Finding and applied fix | Evidence |
| --- | --- | --- | --- |
| Medium | [src/runtime/default.rs:278](../../../src/runtime/default.rs#L278) | Configured bootstrap sealed declarations while holding the mutable TLS borrow. Registration hooks observing readiness received `ReentrantAccess`, unlike ordinary bootstrap. Share construction/error handling, check configuration before sealing, and release the borrow while hooks run. | New configured-hook regression failed against the original code and passes after the fix; configuration, cached-error, reentry and warm-adoption tests pass. |
| Low | [src/cbor.rs:124](../../../src/cbor.rs#L124) | A sequence without a size hint decoded one excess element before rejecting its count. Use a rejecting deserialize seed so the excess element is never constructed or decoded. | New unhinted-sequence regression failed against the original code and passes after the fix; existing hostile-input and writer/reader boundary tests pass. Definite-length CBOR already rejected oversized size hints. |
| Low | [src/physical.rs:309](../../../src/physical.rs#L309) | Each commit repeatedly selected authority and rescanned whole predecessor payloads. Automatic and explicitly numbered commits now share one checked mutation path and reuse the selected inactive slot. The permissive corruption-simulation helper is test-only. | New regression checks rotation, explicit initial generations, rejection conservation and rejection of a corrupt predecessor through both entrypoints; existing overflow, ambiguity and corruption tests pass. |
| Low | [src/capability.rs:132](../../../src/capability.rs#L132), [src/runtime/diagnostics.rs:294](../../../src/runtime/diagnostics.rs#L294) | Publishing external authority always deep-cloned its declarations, even when uniquely owned. Doctor validation also cloned the whole recovered history after recovery. Unwrap or clone the capability state according to ownership, and borrow recovered evidence during diagnostic validation. | Shared-capability regression verifies filtering leaves other capabilities unchanged; runtime publication, recovery and doctor tests pass. |
| Cleanup | [src/ledger/mod.rs:39](../../../src/ledger/mod.rs#L39) | A private codec trait, sealing trait and unit-type derives remained despite there being one private implementation. Remove the extension scaffold and use stateless associated codec functions. | Current-format fixture round trips, unsupported-shape rejection and ledger commit/recovery tests pass. |
| Cleanup | [src/hash.rs:5](../../../src/hash.rs#L5), [src/registry.rs:879](../../../src/registry.rs#L879) | Physical checksums and declaration fingerprints duplicated FNV constants and byte hashing. Share one scalar hashing function. | Standard hash vectors, chunked inputs, the pinned snapshot fingerprint and current wire fixtures remain unchanged. |
| Cleanup | [src/lib.rs:129](../../../src/lib.rs#L129), [src/stable_cell.rs:218](../../../src/stable_cell.rs#L218) | Ledger and stable-cell tests duplicated the same hex fixture decoder. Move it into the existing test-only support module. | Both fixture suites pass; no test utility is exported in production. |
| Cleanup | [src/capability.rs:132](../../../src/capability.rs#L132), [src/key.rs:129](../../../src/key.rs#L129) | Private unit markers duplicated the protection already supplied by private fields; stable-key segment validation checked emptiness twice. Remove both redundancies. | All seven compile-fail boundaries and canonical/noncanonical-key tests pass. |

## API, trust and compatibility inventory

| Category | Boundary retained |
| --- | --- |
| Recovered authority | `RecoveredLedger` has a private constructor and no serde implementation; maintained recovery checks physical slots, the current envelope, generation correspondence and ledger integrity. |
| Precommit authority | `ValidatedAllocations` has a private constructor and no serde implementation. Declaration and history validation precede staging. |
| Committed authority | `CommittedAllocations` has private fields and no serde implementation. Runtime publication follows persistence; manual owners retain the explicit persistence-confirmation obligation. |
| Runtime ownership | The owned/default runtime controls its sole manager, handles, geometry, admission and warm binding. Observations remain nonconstructing; verification does not replay admission. |
| Decoded DTOs | Ledger, declaration and physical-store serde values remain inert until their respective validation boundaries. Policy identities and range tables revalidate on deserialization. Diagnostics supply no open authority. |
| Durable format | Required optional fields remain explicitly present. Unknown fields, unsupported envelopes, old integer-array CBOR payloads and corrupt slots reject. |

No deprecated forwarders, renamed API aliases, legacy macro forms, serde field
aliases or version-routed/fallback readers were found. Human-readable payload
arrays are the current JSON DTO representation, not a legacy durable reader.
`deserialize_present_option` is intentional: it requires an explicit null/value
instead of silently accepting an omitted durable field. Empty DTO defaults are
for genesis construction; they do not replace corrupt persisted state.

Panic review distinguishes the substrate's panic-based `Storable` contract and
the documented manual `LedgerPayloadEnvelope::encode` convenience from fallible
maintained recovery. Runtime cell initialization remains preflighted. Schema,
slot and numeric `expect` calls rely on validated/bounded internal facts; fixture
and registry-reset assertions remain test-only. No new panic or unsafe path is
added by the fixes.

All normal dependencies have production uses. There are no package feature
flags, and the exact substrate pin remains necessary for the layout adapter.

## Validation

- Serialized suite: **248 library tests**, integration tests, all seven
  compile-fail cases, two composed-host regressions, and four doctests pass;
  five existing integration sketches remain ignored.
- Warning-denied all-target Clippy and Rustdoc, formatting and whitespace checks
  pass.
- All-feature and no-default-feature checks and 89 local documentation
  links/includes across 31 Markdown files pass.
- Wasm test compilation and Rust **1.88.0** all-target compilation pass.
- Current raw Wasm probes pass the existing Makefile gates on Rust **1.99.0**:

| Probe | Raw bytes | Existing ceiling |
| --- | ---: | ---: |
| Core | 259,965 | 260,000 |
| Diagnostics | 313,957 | 315,000 |
| Key-only | 259,578 | 260,000 |
| Admission | 262,821 | 264,000 |
| Runtime integration | 269,064 | 270,000 |

These are current gate results, not matched performance or IC instruction/cycle
deltas. Core has 35 bytes of headroom at this compiler pin. Fingerprint encoding
retains its bounded temporary CBOR buffer; a distinct streaming serializer
increased Wasm size beyond the gates and is not part of the final change.

## Review checklist

- [x] Fix configured-hook observation and add a failing-before/passing-after test.
- [x] Reject excess unhinted elements before invoking their deserializer.
- [x] Consolidate physical commits while preserving fail-closed recovery.
- [x] Remove redundant codec scaffolding, hashing, copies, markers and fixtures.
- [x] Preserve current wire fixtures, fingerprints and compile-time authority boundaries.
- [x] Validate native behavior, Wasm budgets and the declared MSRV.
- [x] Leave changes unstaged and uncommitted for maintainer review.

## Deferred design work

Finite generation/schema history, ledger compaction, bucket migration and a
larger slot domain remain explicit protocol decisions. This audit introduces
no migration or compatibility bridge. No additional mechanical fix remains
from the findings above.

## Audit quality

The review covered production runtime, registry, codec, recovery, validation,
staging, authority and diagnostics paths, their test boundaries, examples,
dependency metadata and release/CI wiring. The strongest evidence is the two
reproduced behavioral failures, current fixture/fingerprint preservation and
the full validation suite.

Release scripts were inspected read-only. Their Git-mutating end-to-end tests
were not invoked under the repository's user-owned commit/tag/push rule.
No downstream repository was changed, and no new PocketIC, live canister or
IC instruction/cycle qualification is claimed. A future pass would be stronger
with matched consumer lifecycle measurements of commit and diagnostic costs.
