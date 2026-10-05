<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-memory/ic-memory-readme-header.svg" alt="IC Memory — Internet Computer helper library" width="100%">
</p>

# Simplification follow-up — 0.17.0 baseline

Reviewed on 2026-10-03 against released commit `a72a4a4`, with a clean starting
tree. The changes below shipped in **0.17.1**, released at `197477b`. The
[previous audit](simplification-followup-0.16.1.md) is fully released in 0.17.0.
The validation below records this audit's pre-release evidence; subsequent work
is recorded in the [0.17.1 follow-up](simplification-followup-0.17.1.md).

This follow-up removes duplicate generation state from recovered authority,
repeated empty-store guards after physical selection, and repeated lifecycle
checks during adoption. Each invariant stays with the owner that establishes it.

| Confidence / finding | Owner and caller evidence | Simplification and obligations | Risk and verification |
| --- | --- | --- | --- |
| 1 — Confirmed; implemented — duplicate recovered generation state | `RecoveredLedger` in [record.rs](../../../src/ledger/record.rs); `LedgerCommitStore::recover_committed` and `commit` in [ledger/mod.rs](../../../src/ledger/mod.rs). Private diagnostic genesis and validation fixtures are the other constructors. | Recovery rejects physical/logical generation mismatch before constructing the immutable, non-deserializable proof. Commit integrity-checks the ledger and writes its logical generation as the physical generation. Synthetic diagnostic genesis uses zero for both. Remove the private physical-generation field and constructor argument; derive both public generation accessors from the ledger. Keep mismatch rejection at recovery and physical predecessor validation at commit. | Low; public signatures and const accessors remain, as do durable bytes and format checks. Public accessor assertions cover genesis, first commit, recovered generation 2 and imported generation 3. The exact mismatch regression remains; the compile-fail fixture still checks constructor privacy using its current private signature. Derived `Debug` output now omits the duplicate private field. |
| 2 — Confirmed; implemented — repeated initialization guards | `LedgerCommitStore::recover_or_initialize` in [ledger/mod.rs](../../../src/ledger/mod.rs) and `DualCommitStore::commit_payload_with_generation` in [physical.rs](../../../src/physical.rs). Runtime bootstrap and generic explicit genesis/import entrypoints use these paths. | `select_authoritative_slot` rejects every invalid present slot first, handles valid/ambiguous present slots, and returns `NoValidGeneration` only for two absent slots. Both mutation paths then called `is_uninitialized()` to re-establish the same fact through an exclusive mutable borrow. Remove those guards, keeping the selection error authoritative. | Low; no error or public API removal. The existing recovery matrix now also exercises initialization: empty stores accept genesis, valid stores retain their recovered authority, and corruption on either slot, ambiguity, unsupported format, invalid CBOR, generation mismatch and invalid committed integrity preserve the store and the exact rejection. Existing physical commit/rejection tests cover automatic and explicitly numbered commits. |
| 3 — Confirmed; implemented — repeated adoption lifecycle checks | `MemoryRuntime::verify_authority` and private `verify_requirement` in [adoption.rs](../../../src/runtime/adoption.rs). Owned callers, `verify_default_memory_manager_authority`, key-only/composed-host examples and runtime macros converge on this entrypoint. | Previously `committed_allocations()` checked readiness, then every private helper call matched the same lifecycle through `&self`. No mutation or callback occurs between those checks. Match the lifecycle once and pass the resolved sealed snapshot to the helper. Keep readiness before authority discovery, fixed requirements before logical requests, and the existing key/authority/ID/schema/label error order. | Low; no public signature, error variant, durable format or fingerprint change. Existing adoption regressions check typed rejection, additional host/foreign requirements, assigned IDs, unchanged backing/capability/configuration and no replay of host preparation. Default-runtime, macro and composed-host regressions cover adapters. |

No new trait, registry, cache or policy mode is introduced. The per-requirement
verifier now depends only on the sealed host declarations; it cannot reach
runtime memory or lifecycle state. No additional test duplicates this private
implementation: existing observable adoption tests cover its obligations.
`DualCommitStore::is_uninitialized` remains a supported public observation; only
redundant internal calls after successful classification are removed.

Final configuration cleanup (confirmed; implemented): remove three inactive
deserialization-only Serde attributes from the serialize-only derives on
`PolicyIdentity` in [policy.rs](../../../src/policy.rs) and
`MemoryManagerRangeAuthority` in [range_authority.rs](../../../src/slot/range_authority.rs).
Their custom readers use `PolicyIdentityRepresentation` and
`MemoryManagerRangeAuthorityDto`; strict unknown-field and explicit optional-field
checks remain on those decoding DTOs. No serialization shape or reader behavior
changes. Existing policy, range-authority and diagnostic-metadata tests cover the
active owners; no new test is added for inactive attributes.

## Complexity retained and coverage

- Source and resolved snapshots have different jobs: warm-bootstrap identity
  and current assigned declarations. They remain separate.
- Fixed requirements check ID and label; logical requirements retain the host's
  assignment. Their shared verifier preserves that distinction.
- Default configured bootstrap and doctor helpers release TLS borrows while
  registration hooks run; checks on both sides of those hooks remain necessary.
- Reservation and active-allocation claims differ in lifecycle acceptance and
  error precedence; their similar shapes do not justify merging them.
- Growth arithmetic, bucket capacity, reentry and backing-refusal checks protect
  actual failure modes. No growth change is included in this patch.
- Physical/logical mismatch rejection, checksums for every present slot and
  equal-generation ambiguity rejection remain necessary. The raw DTOs still
  contain both generations because they are independent evidence before recovery.
- Envelope length/platform checks were inspected but not changed. No additional
  public error removal is included merely to simplify bounded arithmetic.

This pass traced adoption, default adapters, policy dispatch, admission/resolution,
snapshot construction/fingerprints, declaration validation, recovered-proof
construction, logical commit/recovery, physical selection and pinned substrate
growth. No coordinated downstream change or further proven obsolete API was
identified. Registry concurrency and macro grammar were not comprehensively
reaudited. Remote issue feedback, full downstream builds, live IC execution,
Lean/whitepaper builds and Git-mutating release-flow tests were not run.

The three changes above are the priorities, in ranked order: remove duplicate
proof state, converge initialization on physical classification, and narrow the
adoption verifier's dependencies. They delete redundant facts and checks without
introducing a new concept.

An independently reviewable sequence is the recovered-proof change with its
accessor/privacy fixtures; the two initialization guards with the recovery
matrix extension; the adoption change; then audit/release-status documentation.
No API hard cut or downstream migration is required by these changes. No
compatibility removal remains blocked on adoption evidence in this patch.

## Validation

Passed: 262 library tests, integration and seven compile-fail cases, two
composed-host regressions and five doctests (five existing sketches ignored);
strict all-target Clippy and Rustdoc, Rust 1.88 all-target checking and Wasm test
compilation, plus offline package verification with the working-tree changes.
The extended recovery/initialization matrix also passes after its
final invalid-integrity case was added.
Formatting, whitespace checks and all five existing raw Wasm gates pass: core
249,892 bytes, diagnostics 298,069, key-only 249,510, admission 253,087 and runtime
integration 259,404. These are current gate measurements, not consumer or IC
execution cost measurements.

The final three-attribute cleanup was checked separately with existing policy,
slot/range-authority and diagnostic-metadata tests plus strict all-target Clippy,
formatting and whitespace checks. The broader gates and measurements above
precede that configuration-only deletion.
