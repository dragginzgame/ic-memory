# Allocation-pool flow audit — 2026-10-10

## Scope, identities and conclusion

Trigger: maintainer requested an audit of the pending hard cut and removal of
redundancy. Method: Shared Tooling's unchanged
[flow convergence and duplication](../../../audits/flow-convergence-and-duplication.md),
with the [product overlay](code-hygiene.md). Repairs are explicitly authorized by
this request; no sibling edit, dependency change or Git delivery is authorized.

Source base: released Memory 0.33.4,
`9137192d4b3425a229fa21df5f343883f56d80e8`, plus the uncommitted 0.34.0 pool and
tooling batch. The incoming root syn lockfile edit remains unrelated and intact.
Shared's live clean revision during review is
`83efac446348dea024798a331d77933b24b429dc`; Memory's immutable 89-file snapshot
remains `43a0dc46cdc3c77e70a68e192561642ed50a3e0f` (0.2.10). The selected method's
bytes match both checkouts, SHA256
`35fd87d36a09ed0971841abf28535d43470c713a82581c2251b924da1ab8f484`.
The overlay's stale revision reference was aligned with the recorded snapshot;
its resulting SHA256 is
`2dc33a5e69dda595b93e09d98a31378e9a1d2e8c6f4715dc427f18026a46c8ec`.

Verdict after repair: **PASS for the selected allocation/admission/projection
flow**. No further actionable duplication was found in that scope. This is not
a whole-repository, performance, native macOS or installed IC acceptance verdict.
Earlier reports are not directly comparable to this changed method/scope.

Reviewed families: checked pool/request producers, macro/registry sealing,
bootstrap and doctor resolution, custom validation, committed key lookup,
consumer adoption and physical binding projections. Traced the generic
validation/commit boundary to confirm its independent obligations. Byte IO,
codec internals, external consumer generators, release tooling and fleet
adoption are excluded from a new audit verdict: their domain qualification is
separate and their implementations were not changed by this cleanup. Existing
host/native/IC gaps remain linked from [pool qualification](../../allocation-pool-qualification.md).

## Owner and entry-to-result trace

| Behavior | Owner and input | Carried result | Consumers |
| --- | --- | --- | --- |
| Static requirements | Registry canonicalizes checked key/owner/schema requests | Immutable source snapshot and request fingerprint | Host bootstrap, doctor, warm binding comparison |
| Pool policy | Host-owned checked namespace grants and exclusions | Immutable bound MemoryAllocationPool | Resolver, historical selection, owner projections |
| Placement | Resolver combines recovered occupancy and admitted requests | Existing checked DeclarationSnapshot | Generic validation and staging |
| Current metadata/access | Persistence confirmation after stable write | CommittedAllocations | Opens, adoption, physical binding reports |
| Application checks | Governance-exempt adapter delegates external rows | Exact custom policy error | Bootstrap validation and doctor result |

Default helpers converge into the same owned runtime; generated macros add only
request metadata. Doctor resolves through the same placement owner and calls
the same generic checks without preparation, writes or capability publication.
Adoption uses the committed declaration and the bound host grant; it does not
reconstruct placement or replay admission. Slot lookup and schema lookup share
one borrowed declaration search with no allocation or second index.

## Findings and implemented dispositions

1. **LOW — mixed source/resolved state.** `SealedDeclarationSnapshot` retained
   request fields alongside allocation/resolved-row fields, although its two
   construction paths populated mutually exclusive representations. Source
   fingerprints also encoded redundant allocation projections. Removed those
   fields and the combined builder branches. Source snapshots contain only
   canonical requests; resolution returns the existing DeclarationSnapshot.
   The public snapshot API and durable ledger format stay as selected for 0.34.0;
   diagnostic fingerprint values now derive solely from source metadata.
2. **LOW — duplicated current allocation metadata.** Runtime binding retained
   resolved rows and another allocation snapshot after commitment, duplicating
   capability metadata and host owner labels. Deleted the resolved-row type and
   binding field. The committed capability supplies key/ID/schema; current owner
   projections borrow the bound pool. No owner index, cache or replacement DTO
   is added. Failed persistence still publishes neither binding nor capability.
3. **LOW — repeated admission and error wrapping.** The runtime policy adapter
   repeated pool checks already completed by the resolver on the same immutable
   rows. Removed its declaration lookup and claim checker. It now only excludes
   private governance from custom callbacks. `RuntimePolicyError` is removed;
   custom errors pass directly through AllocationValidationError. Pool failures
   remain typed resolution/admission failures before callbacks. This public
   error-shape hard cut belongs to the existing pending 0.34.0, not a patch.

These are resolved structural findings, not claims of previously corrupted data.
The owning work remains [Memory #44](https://github.com/dragginzgame/ic-memory/issues/44).
The [retirement inventory](../../allocation-pool-removals.md) records exact symbols.
Before-edit registry/policy sources remain with the focused evidence.

## Intentional retention and state-space effect

Retain independent constructor/serde admission, known-only historical selection
and ignored-error poisoning, protected ledger recovery, generic historical claim
validation, persistence confirmation and no-open-before-commit. These enforce
separate trust/lifecycle boundaries; similar checks are not redundant flow.

Retain the governance adapter: application callbacks must not govern the private
ledger key. Retain source snapshots for exact warm-call binding and pool state
for current host policy. Retain the low-level numeric persistence DTO: it is the
resolved ledger boundary, not a second component allocation mode. Detailed and
numeric physical reports share measurement but keep their existing different
output/allocation contracts. No IO specialization or recovery fallback changed.

Removed the mixed request/resolved representation and redundant stored maps;
added no product mode, cache, index, durable field or compatibility path. The
ledger markers, key-to-ID mappings, tombstones and geometry remain unchanged.
No timing, instruction, memory-footprint or Wasm-size improvement is claimed.

## Focused verification

Evidence: `target/qualification/0.34.0-audit/`, Linux x86_64, pinned Rust 1.99.0,
MSRV compiler 1.88.0, selected existing lockfile, offline. The library has no
optional feature matrix. Intermediate compile failures remain separately logged.

- `lib-tests-final.log`: 241 library tests, including the new foreign-owner and
  excluded-ID ordering/unchanged-bytes/corrected-retry regression. Existing
  custom rejection now asserts the exact unwrapped error.
- `public-tests.log`: allocation reports, default configuration/custom policy,
  diagnostic DTOs, explicit runtime, macros and nine compile-fail boundaries.
- `composed-host.log`: two cold-reopen/native-worker composition tests.
- `clippy-final.log`: strict library/tests/examples Clippy.
- `doctests.log`: package README doctests.
- `msrv-wasm.log`: Rust 1.88 Wasm library, core probe and IO fixture compilation.
- `snapshot-format.log`: immutable snapshot verification and both-workspace format.

These are newly executed checks, not renamed earlier passes. Full validation,
Wasm-size gates, installed PocketIC execution and native macOS runs were not
invoked. Real Canic/IcyDB/Jobs IC lifecycle and final package-identity qualification
remain required under #44; this audit does not close that issue or qualify dirty
source through released 0.33.4 CI. No commit, push, release or publication occurred.
