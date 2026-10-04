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

## Fresh constructor growth refusal — issue #9

[#9](https://github.com/dragginzgame/ic-memory/issues/9) reports that the stable
structures manager panics when its first metadata growth is refused. Both public
constructors reproduced that panic on empty backing memory, with one failed
growth and no writes. Canic traced its configured default bootstrap to the same
construction path; that source trace does not establish an IC execution failure.

The fix released in **0.15.7** reserves the metadata page before initializing the
dependency and returns
`RuntimeConstructionError::Growth(RuntimeGrowError::BackingRefused { additional_pages: 1 })`
on refusal. The failed attempt makes no reads or writes and leaves zero pages.
Both constructors can retry the same backing successfully after refusal is
removed. Reopening preserves geometry without growth or writes. Caller-supplied
nonempty blank memory still fails layout validation without changing bytes.

Configured default bootstrap propagates the construction error through
`RuntimeStateError::Construction`. Its failed singleton initialization remains
cached; the retry result above concerns caller-owned `MemoryRuntime` backing.
No API compatibility path or durable-format change is introduced.

Before release, the constructor fix passed 256 library tests and the ordinary
integration, compile-fail and doc suites, strict all-target Clippy and rustdoc,
Rust 1.88 all-target checking,
and Wasm test compilation. An external public-API probe confirms both constructors
return the typed error without panicking, with one failed growth, zero writes and
zero pages, then succeed on retry. All five raw Wasm budgets pass: core 257,247
bytes, diagnostics 310,950, key-only 256,860, admission 259,995 and integration
266,487. These are historical local probes before the final 0.15.7 changes, not
downstream or IC cost measurements. The release changelog records final release
validation.

#9 is closed as completed, with **0.15.7** recorded as the released fix version
requested by Canic. Downstream adoption and qualification remain consumer-owned;
closure does not certify those activities. This section records the ic-memory
changes only.

## Released 0.20.0 consumer recheck

The [2026-10-03 qualification receipt](consumer-qualification-0.20.0.md) records
fresh checks against published ic-memory **0.20.0** after the consumer dependency
updates finished. IcyDB passes 16 focused native admission/default/participant
tests, four public bootstrap error tests, three installed lifecycle tests, and
three installed logical-memory retirement/rejection/recovery tests. Its worst
measured lifecycle phase is **4,610,556 instructions** against the unchanged
12,750,000 ceiling; empty and populated stable extents are 23,134,208 bytes.

Canic passes native and Wasm `canic-core` checks and 13 focused native memory
tests on its direct 0.20.0 dependency. Its optional published-IcyDB test
composition still resolves 0.18.0 and remains unqualified. These focused results
do not resolve the maintainer's separate report that Canic is not working.
They are local checks of dirty consumer worktrees, not complete downstream CI or
a matched performance comparison. Subsequent IcyDB query edits are outside
these passing receipts; both dependency lockfiles stayed unchanged. The existing
issue closures and historical release evidence remain unchanged; no GitHub
issue or comment was updated.

## Released 0.24.3 IcyDB recheck

The [2026-10-04 qualification receipt](consumer-qualification-0.24.3.md) records
27 passing tests against published ic-memory **0.24.3** on an unchanged clean
IcyDB checkout: 16 native admission/default/participant tests, four public
bootstrap/adoption error tests, one typed-cause projection test, and six installed
lifecycle/logical-memory tests. The worst lifecycle phase is **4,454,129
instructions** against the unchanged 12,750,000 ceiling; empty and populated
stable extents remain 23,134,208 bytes. These measurements do not establish a
release-to-release performance improvement.

Canic was neither inspected nor modified in this recheck. Its deployment
problems remain outside the passing IcyDB receipt, and its current selected
graph and lifecycle integration still require qualification once settled.
Historical issue dispositions and release evidence remain unchanged; no GitHub
issue or comment was updated.

## Released 0.24.4 isolated IcyDB recheck

The [2026-10-04 qualification receipt](consumer-qualification-0.24.4.md) records
27 passing focused tests against published ic-memory **0.24.4** in an isolated
snapshot of the same committed IcyDB Rust sources used for the 0.24.3 recheck.
Only ic-memory changed in the snapshot dependency graph. The worst lifecycle
phase is **4,454,357 instructions** against the unchanged 12,750,000 ceiling;
empty and populated stable extents remain 23,134,208 bytes.

The active IcyDB checkout advanced independently during qualification and was
not edited by these checks. Its newer commit and graph are outside this receipt.
Canic remains unqualified on its current graph. No GitHub issue disposition or
comment was updated.

## Released 0.24.6 Canic memory recheck

The [2026-10-04 Canic receipt](consumer-qualification-0.24.6-canic.md) records
14 passing native memory regressions and default-feature core Wasm compilation
against published ic-memory **0.24.6**. The consumer is an isolated snapshot of
the Canic **0.110.52** release commit; only ic-memory changed from its original
0.24.2 graph. Configured bootstrap, diagnostic projection, growth refusal/retry
and bucket-exhaustion conservation pass without a Canic source patch.

These results cover the specified release source and graph, not the active
Canic worktree, the open ic-memory 0.24.7 candidate, complete Canic CI, installed
lifecycle behavior or live deployment health. No GitHub issue or comment was
updated; historical dispositions and earlier qualification limits remain intact.
