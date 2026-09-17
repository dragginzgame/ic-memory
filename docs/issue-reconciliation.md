# Issue reconciliation — 2026-09-17

Reviewed open issues #2–#7 against released ic-memory 0.14.0–0.14.2, current
source/tests and the sibling IcyDB worktree's 0.258 qualification notes. This is
an evidence review; it does not change GitHub issue states or claim that the
consumer's unreleased worktree is a published release.

| Issue | Upstream evidence | Disposition |
| --- | --- | --- |
| [#2](https://github.com/dragginzgame/ic-memory/issues/2) Key-only allocation | 0.14.0; `docs/key-only-recovery.md`, `examples/key_only.rs`, runtime request tests: canonical placement, retained IDs, reservations, revoked grants, exhaustion, persistence refusal/retry, host adoption | Implemented; ready for closure with the documented measurement limits and consumer evidence attached |
| [#3](https://github.com/dragginzgame/ic-memory/issues/3) Recovery bounds | 0.14.0; recovery-limit table, ledger/stable-cell boundary and hostile-input tests, no-op generation headroom and typed exhaustion | Implemented; ready for closure; #7 separately qualifies the replacement encoding and tighter outer bound |
| [#4](https://github.com/dragginzgame/ic-memory/issues/4) Omitted allocation access | 0.14.0 documents explicit redeclaration; 0.14.1 adds precommit historical selection; runtime tests preserve journal markers and reject unauthorized opens | Upstream contract complete; downstream logical-memory qualification now provides the formerly missing generated reconciliation evidence |
| [#5](https://github.com/dragginzgame/ic-memory/issues/5) Recovered admission | 0.14.1; `docs/recovered-admission.md`, `examples/recovered_admission.rs`, admission/default-runtime/compile-fail tests | Implemented; consumer now uses the shared preparation function and journal-only selection; ready for closure review |
| [#6](https://github.com/dragginzgame/ic-memory/issues/6) Bootstrap cleanup | 0.14.2; `docs/logical-bootstrap-cleanup.md`, permutation/duplicate/fingerprint tests and matched Wasm table; consumer notes explicitly record 0.14.2 adoption | Ready for closure; deferred range sharing is explicitly justified, IC execution costs were unmeasured upstream |
| [#7](https://github.com/dragginzgame/ic-memory/issues/7) Opaque payload codec | Current candidate and `docs/opaque-ledger-payloads.md` | Candidate lifecycle cost gate passes at 5,133,140 instructions; keep open through released implementation and adoption |

The IcyDB worktree's `docs/changelog/0.258.md` reports three executed generated
PocketIC upgrade cases in `testing/integration/tests/logical_memory.rs`: empty
omitted-store retirement preserves surviving rows/IDs; journal debt blocks
removal; a valid pending marker blocks registry changes. Rejected transitions
recover with the original actor. These distinguish generic allocation ownership
from database retirement and fill the integration gap recorded in older upstream
release notes. They are existing downstream evidence, not tests rerun by this
reconciliation itself.

The same consumer notes report matched integration costs and explicitly retain
the failing lifecycle ceiling tracked by #7. Closing implementation issues must
not imply that this separate release gate has passed. Historical release notes
remain historical; current codec limits are in `docs/key-only-recovery.md`.

The #7 candidate additionally passes the maintained consumer lifecycle test binary
(three tests), 12 memory-admission tests and the default-manager test in a
disposable checkout. This rerun does not include the separate logical-memory
upgrade binary cited above. See [candidate qualification](opaque-ledger-payloads.md)
for exact instruction counts, validation and release limitations.
