# Issue reconciliation — 2026-10-03

Reconciled requests #2–#8 against released ic-memory 0.14.0–0.15.3, current
source/tests, IcyDB's logical-memory qualification and its released-dependency
issue follow-ups. The maintainer authorized updating GitHub issue dispositions.
All seven original implementation requests are closed as completed; each issue's original
body is retained beneath a dated resolution with release evidence and limits.

| Issue | Released implementation and qualification | Disposition |
| --- | --- | --- |
| [#2](https://github.com/dragginzgame/ic-memory/issues/2) Key-only allocation | 0.14.0; [contract](key-only-recovery.md), standalone/composed example and runtime request tests. 0.15.0 adds committed authority verification without replaying host admission. | Closed; documented measurement limits retained |
| [#3](https://github.com/dragginzgame/ic-memory/issues/3) Recovery bounds | 0.14.0; numerical limits, hostile-input and history-boundary tests, writer/reader agreement and upgrade headroom. 0.14.3 tightens the outer codec bound. | Closed |
| [#4](https://github.com/dragginzgame/ic-memory/issues/4) Omitted allocation access | 0.14.0 explicit redeclaration, 0.14.1 precommit historical selection; production-runtime marker preservation and authorization rejection. IcyDB generated reconciliation covers debt and pending markers. | Closed; database retirement remains consumer-owned |
| [#5](https://github.com/dragginzgame/ic-memory/issues/5) Recovered admission | 0.14.1; [contract](recovered-admission.md), recovered-journal example, admission/default-runtime and compile-fail tests. IcyDB uses the shared role preparation hook. | Closed |
| [#6](https://github.com/dragginzgame/ic-memory/issues/6) Bootstrap cleanup | 0.14.2; [cleanup report](logical-bootstrap-cleanup.md), canonical-permutation/duplicate/fingerprint tests and matched Wasm results; IcyDB adoption recorded. | Closed; wider range-table sharing explicitly deferred |
| [#7](https://github.com/dragginzgame/ic-memory/issues/7) Opaque payload codec | 0.14.3; [codec qualification](opaque-ledger-payloads.md), pre-allocation bounds and maximum-size writer/reader tests; released IcyDB lifecycle acceptance under its unchanged ceiling. | Closed; candidate-only adoption wording is superseded |
| [#8](https://github.com/dragginzgame/ic-memory/issues/8) Typed diagnostic tests and lint expectations | 0.15.2; exact diagnostic-code assertion and justified expectations; IcyDB explicitly confirms released implementation/adoption. | Closed |

## Downstream acceptance

IcyDB's maintained 0.258 qualification records three generated PocketIC upgrades:
empty omitted-store retirement preserves surviving rows/IDs, journal debt blocks
removal, and a valid pending marker blocks registry changes. Rejected transitions
recover with the original actor. These establish consumer integration while
preserving the distinction between allocation ownership and database retirement.
The 0.258 status tracker also records published 0.14.3 adoption and resolution of
the former lifecycle cost gate at 5,131,230 instructions.

The [released 0.15.0 follow-up on #7](https://github.com/dragginzgame/ic-memory/issues/7#issuecomment-5947699680)
reports all three lifecycle-participant tests passing, with a worst phase of
5,169,089 instructions against the unchanged 12,750,000 ceiling. The
[recorded 0.15.3 follow-up on #8](https://github.com/dragginzgame/ic-memory/issues/8#issuecomment-5948796045)
reports four bootstrap tests and seven installed-canister tests passing. Its
worst participant phase is 5,169,206 instructions; empty/populated stable extents
remain 23,134,208 bytes. Maintainer Clippy and the public Rust 1.88 dependency
path pass, with the same five existing consumer MSRV warnings. A stale IcyDB
fixture was corrected to create a conflicting layout through explicit bootstrap,
preserving typed mismatch, rejection conservation and retry coverage.

These are reported downstream results, not tests rerun by this reconciliation,
not complete upstream CI and not matched performance comparisons against the
original compiler/dependency graph. Raw consumer Wasm and cycle deltas remain
unmeasured. Historical changelogs retain their original release status.

## New Canic qualification request

Canic's 2026-10-02 recheck confirms 0.15.3 fixes diagnostic-triggered default
runtime construction: export, commit-recovery and both doctor helpers return
typed `NotBootstrapped`, then allow configured 16-page bootstrap. Its current
review status also records managed-component lifecycle qualification. This
feedback is resolved by the released runtime, independently of #8's maintenance
request.

The 2026-10-03 request for a composed-host example and repeated cold reopens is
covered by [the new public-API example](../examples/composed_host.rs) and its two
tests, enabled in the ordinary suite. They check two cold reopens over the same
backing, fixed/logical IDs and retained data, current authority, unchanged
geometry, cold admission, effect-free warm adoption, typed consumer rejection
without candidate commitment, and host-first native bootstrap on each worker.
This additional example/test work shipped in **0.15.4**. It requires no runtime
API or persisted-format change. Closing #2–#8 relied on their previously released
implementations, independently of this additional qualification.

Canic still owns its PocketIC participant and store-restoration qualification;
the reported Toko failures do not establish an ic-memory defect. No sibling
repository source, release, commit, tag or push is part of this reconciliation.

Local validation passes the serialized unit/integration/compile-fail/doc suite
(243 library tests plus the two new example regressions), runnable example,
strict all-target Clippy, Wasm test compilation and Rust 1.88 all-target check.
All five existing raw Wasm budgets pass: core 258,931 bytes, diagnostics 312,802,
key-only 258,544, admission 261,772 and integration 267,035. These are current
probe sizes, not matched consumer or IC execution deltas.
