<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/ic-memory/main/images/ic-memory-readme-header.svg" alt="IC Memory — Internet Computer helper library" width="100%">
</p>

# Simplification follow-up — 0.19.0 baseline

Reviewed on 2026-10-03 against release commit `3725f20`, starting with a clean
tree. This follow-up implements four confirmed audit findings. Changes remain
unstaged and uncommitted; package versions and historical changelogs are unchanged.

## Findings and implementation

| Finding | Owner and callers | Simplification | Preserved obligations |
| --- | --- | --- | --- |
| 1 — Redundant committed-generation membership checks | `AllocationLedger::validate_committed_integrity` in [integrity.rs](../../../src/ledger/integrity.rs), reached by ledger recovery, commit, and `new_committed`. | Structural validation bounds record references; strict chain validation establishes every generation from 1 through the current generation. Check only that each record's first generation is nonzero. Delete the membership helper and repeated last-seen, retirement, and schema-reference checks. | Keep structural validation, current-generation presence, exact parent chain, genesis rejection, and existing error precedence. Raw structural DTOs may still contain generation-zero references. |
| 2 — Unreachable runtime metadata error | Private `RuntimeMemoryManagerPolicy` in [policy.rs](../../../src/runtime/policy.rs), constructed by runtime bootstrap and diagnostic validation. | Both callers validate the allocation snapshot from the same immutable resolved declaration snapshot held by the adapter. Express the registered-key lookup as an internal invariant and remove its fallible plumbing and public `MissingDeclarationMetadata` variant. | Keep range authorization, custom policy rejection, internal governance handling, and raw-input validation. The assertion detects a future violation of the private shared-snapshot contract. |
| 3 — Impossible diagnostic genesis failure | `diagnostic_validation_ledger` in [runtime diagnostics](../../../src/runtime/diagnostics.rs), using genesis only for two absent commit slots. | Construct the known empty generation-zero ledger infallibly. Remove public `DiagnosticCode::GenesisLedger`, its serialized name, and its wire-test row. | Present corrupt storage still fails closed; unreadable storage and unsupported formats retain their existing codes. Doctor remains read-only. Other diagnostic wire names and durable ledger bytes are unchanged. |
| 4 — Impossible bootstrap genesis error and separate construction paths | Cold bootstrap in [runtime/mod.rs](../../../src/runtime/mod.rs) and diagnostic validation both need the same empty genesis. | Share crate-private `AllocationLedger::empty_genesis` in [record.rs](../../../src/ledger/record.rs). Remove the diagnostic-only wrapper and public `RuntimeBootstrapError::LedgerIntegrity` with its automatic conversion. | Public DTO constructors still validate arbitrary input. Protected recovery and commit still return real integrity failures through `LedgerCommit::Integrity`; empty-store initialization and persistence ordering remain unchanged. |

## API and diagnostic-wire cuts

`RuntimePolicyError::MissingDeclarationMetadata` is removed from the Rust API.
`RuntimeBootstrapError::LedgerIntegrity` and its automatic conversion from
`LedgerIntegrityError` are also removed; its sole production source was the
infallible empty-genesis construction. Real ledger integrity failures still
travel through `RuntimeBootstrapError::LedgerCommit`.
`DiagnosticCode::GenesisLedger` and its `genesis_ledger` serialized value are
removed from the diagnostic API and wire vocabulary. No aliases, legacy reader,
replacement version, or fallback category remain.

These cuts are prepared in the **0.20.0** changelog. Package versions remain
unchanged until the maintainer's release workflow runs after the source and
numbered changelog entry are committed.

A local source search under `/home/adam/projects` found these names only in
ic-memory itself. The maintainer confirmed that no known deployed readers or
saved diagnostic reports depend on the removed genesis code. This is local
consumer inspection plus maintainer deployment knowledge, not an independently
verified inventory of every external consumer or archive. Consumers naming either
removed variant must update directly. No durable-ledger migration or V1 format
change is involved.

## Review sequence and behavioral verification

Review the committed-integrity change with the genesis-reference regression,
which now includes later schema observations and retirement, and the history-gap
regression, which checks that chain failure still precedes genesis rejection.
Existing full-lifecycle tests cover valid schema and retirement references.

Review the runtime adapter and error enum together. Existing fixed/logical
bootstrap, range, revocation, and custom-policy tests exercise the remaining
policy behavior. A focused doctor regression checks combined fixed and logical
declarations before and after bootstrap, including unchanged backing bytes.

Review the infallible genesis construction, code enum, and wire-name fixture row
together. Existing doctor tests distinguish empty from corrupt storage without
writes and preserve unsupported-format reporting. Current durable fixtures,
checksums, and declaration-fingerprint tests remain authoritative.

Review the shared empty-genesis constructor, bootstrap call site, diagnostic
call site, and bootstrap error removal together. Existing bootstrap tests cover
empty-store initialization, cold reopen, refused persistence growth, and corrupt
persisted slots without publishing authority. The corrupt-slot regression
explicitly checks `RuntimeBootstrapError::LedgerCommit(LedgerCommitError::Integrity(...))`.
The private constructor neither reads nor accepts persisted data and adds no
public API or version discriminator.

Rechecked after the fourth change. Passed: 266 library tests, native integration
tests, seven compile-fail cases,
two composed-host regressions, and five doctests (five existing sketches
ignored); strict all-target Clippy; Rust 1.88 all-target checking; Wasm test
compilation; formatting and whitespace checks. All five raw Wasm size probes
pass the repository budgets on the pinned Rust 1.99 toolchain:

| Probe | Bytes | Budget |
| --- | ---: | ---: |
| Core | 248351 | 260000 |
| Diagnostics | 295498 | 315000 |
| Key-only | 247971 | 260000 |
| Admission | 251503 | 264000 |
| Runtime integration | 257748 | 270000 |

These measurements qualify the current budget gates; no same-toolchain baseline
comparison or size saving is claimed. Release-flow tests and package verification
were not rerun.

## Retained complexity and coverage limits

Structural and committed integrity remain distinct trust boundaries. Recovered,
validated, pending, and committed capabilities, persistence confirmation,
fail-closed dual-slot recovery, callback-safe registry sealing, bounded CBOR
decoding, and current diagnostic memory-size failures remain necessary.
Compile-fail tests enforce live authority boundaries; they were not replaced
with negative symbol searches.

No downstream repository was modified or fully rebuilt. Canic's optional
published-IcyDB composition remains separately unqualified under its own policy;
this change does not align dependencies to qualify that graph. No live IC
deployment or external diagnostic archive was inspected. No performance or
binary-size saving is claimed. Release-flow tests create user-owned commits and
tags and must remain a maintainer action.
