<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-memory/ic-memory-readme-header.svg" alt="IC Memory — Internet Computer helper library" width="100%">
</p>

# Simplification follow-up — 0.18.0 baseline

Reviewed on 2026-10-03 against release commit `8743499`, starting with a clean
tree. The previous staging audit shipped in 0.18.0. This follow-up implements
three confirmed simplifications; these changes remain unstaged and uncommitted.

## Findings and implementation

| Finding | Owner and callers | Evidence and change | Preserved obligations |
| --- | --- | --- | --- |
| 1 — Confirmed; implemented — unreachable envelope length overflow | `LedgerPayloadEnvelope::{try_encode, decode_payload}` in [payload.rs](../../../src/ledger/payload.rs), used by public envelope operations and `LedgerCommitStore::{commit, recover}`. | The 16 MiB check precedes arithmetic/conversion. `MAX_COMMITTED_PAYLOAD_BYTES` already represents the maximum header plus payload as `usize`; every admitted payload fits `u64`. Remove the three unreachable branches and `PayloadLengthOverflow`. Correct panic and byte-ceiling documentation. | Keep the untrusted decoded `u64` conversion, byte ceiling, format/error ordering, exact length and borrowed recovery payload. Encoded bytes and current fixtures remain unchanged. |
| 2 — Confirmed; implemented — unreachable stable-cell length overflow | `decode_stable_cell_payload` in [stable_cell.rs](../../../src/stable_cell.rs), reached by runtime initialization and diagnostic record readers. | Physical capacity and the `usize` recovery ceiling are checked before conversion. Every admitted length fits `usize`. Remove the failure branch and `StableCellPayloadError::LengthOverflow`; document the bounded cast with a local Clippy expectation. | Keep marker/layout checks, physical-capacity rejection before the recovery ceiling, empty-memory handling and rejection before allocation or payload reads. |
| 3 — Confirmed; implemented — impossible growth conversion failures | `RuntimeMemory::grow` and its `Memory::grow` adapter in [backing.rs](../../../src/runtime/backing.rs), used by direct callers and stable structures. | Admission bounds bucket totals by the `u16` capacity before narrowing. Successful previous-page counts originate from nonnegative `i64` values. Use the admitted bucket cast and `cast_signed` for the unchanged successful result, without a second failure path. | Keep raw arithmetic overflow, bucket exhaustion, reservation before manager growth, refusal/retry, reentrancy protection, accounting after success and the upstream `-1` sentinel on real errors. |

## API cut and review sequence

Remove public variants `LedgerPayloadEnvelopeError::PayloadLengthOverflow` and
`StableCellPayloadError::LengthOverflow`. Consumers naming these variants must
remove those match arms or constructions. No aliases or deprecated variants
remain. This source/API cut is prepared in the **0.19.0** changelog. Package
versions are updated by the maintainer's release workflow after the source and
numbered changelog entry are committed.

A local Rust source search under `/home/adam/projects`, excluding build/vendor
output, found no remaining references to either error name. This is not complete
downstream build or deployment qualification. No durable-format migration,
fallback reader or mixed-version operation is introduced.

Review the envelope implementation, enum/docs and length-boundary tests together;
then the stable-cell implementation, enum and header-only rejection regression;
then the growth implementation and existing growth/capacity test extensions.
Each production change has an independent owner and can be reviewed separately.

## Behavioral verification

New envelope tests check encoding above the ceiling, every truncated header
length, oversized and `u64::MAX` declared lengths, and mismatches in both
directions. Existing maximum-size decoding still checks borrowed storage.
The stable-cell header-only regression now checks both ceiling-plus-one and
`u32::MAX`, including physical-capacity error precedence, without payload reads,
growth or writes. Existing growth regressions additionally check successful
upstream previous-page returns, including all admitted buckets at each tested
bucket size. Refusal still returns the sentinel without changing state.

Passed: 265 library tests, integration tests, seven compile-fail cases, two
composed-host regressions and five doctests (five existing sketches ignored);
strict all-target Clippy and Rustdoc; Rust 1.88 all-target checking; Wasm test
compilation; formatting and whitespace checks. Current durable fixtures,
checksums and declaration fingerprint regressions pass unchanged.

## Complexity retained and limits

Dual-slot recovery, generation matching, persistence confirmation, registry
sealing/concurrency, strict optional-field presence and the allocation-free CBOR
preflight remain necessary. Bounded serde visitors enforce distinct domain
limits. Raw reservations still need validation both before policy callbacks and
at their independently callable staging boundary. Source and resolved snapshots
retain different identity and assignment semantics. Compile-fail tests continue
to enforce live authority boundaries rather than deleted architecture.

The audit traced these candidates and their tests, and spot-reviewed bootstrap,
admission/adoption, registry, recovery, codecs, growth, macros, dependencies and
CI wiring. It does not claim exhaustive review of every example, release-script
branch or whitepaper/Lean artifact. Release-flow tests were not run because they
create commits and tags reserved for the maintainer. No downstream build, remote
issue check, live IC qualification, package verification or raw Wasm size gate
was rerun. No performance or binary-size saving is claimed.
