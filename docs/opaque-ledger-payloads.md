# Opaque ledger payload encoding (#7)

The current codec stores `CommittedGenerationBytes::payload` as a definite-length
CBOR byte string. The payload is already a logical ledger envelope; the outer
codec no longer encodes or syntax-walks each opaque byte as a separate integer.
Recovery still follows recover → prepare → resolve → validate → persist → publish.
There is no additional decoder, cache, runtime mode or commit boundary.

This is an intentional pre-1.0 persisted-format hard cut. Recreate stable data
written by earlier releases. Logical format version remains **1**; no array
reader, format dispatch or migration bridge is supplied. The four affected
current fixtures have been replaced in place; logical envelope fixtures and
checksums are unchanged.

## Bounds and allocation

| Layer | Bound | Enforcement |
| --- | ---: | --- |
| Logical ledger | 16,777,216 bytes | Existing envelope and commit validation |
| Opaque payload including envelope | 16,777,240 bytes | Allocation-free CBOR preflight before serde; matching serializer/visitor limits |
| Outer stable-cell value | 33,558,528 bytes | Header check before raw buffer allocation, direct CBOR preflight, existing persistence admission |
| Nesting | 32 edges | Existing syntax walk before serde |

The envelope header length has one constant shared by its encoder and the
opaque payload bound. Two maximum payloads plus field metadata fit within the
outer bound. The preflight rejects oversized advertised byte strings even when
all advertised bytes are physically present. It rejects truncated, indefinite,
malformed and trailing data before constructing decoded objects. The byte-string
visitor is defense in depth; its length check alone is not the pre-allocation
proof. Ciborium's owned `deserialize_byte_buf` is required: `deserialize_bytes`
only accepts byte strings fitting its scratch buffer.

All maintained stable-cell, runtime and diagnostic readers use the existing
preflight. Fallible preflight before `Cell::init`, capacity admission before
`Cell::set`, checksum validation, dual-slot ambiguity/corruption rejection,
generation limits and no-publication-before-persistence remain in place. No
rejected record becomes an empty ledger.

The public serde DTO uses byte strings in binary formats and byte arrays in
human-readable formats. JSON round trips are tested. This is a serialization
format distinction, not a persisted compatibility path: binary decoding rejects
an array. Callers using serde directly bypass the maintained recovery preflight
and receive untrusted DTOs, not allocation authority.

## Qualification

Tests cover both maximum-size slots together, oversized writer rejection,
preflight rejection before serde is invoked, malformed stable-cell payloads
with unchanged memory after repeated rejection, and human-readable round trips.
Current fixtures cover valid newer and corrupt newer slots. Existing tests cover
checksums, ambiguous records, generation/history exhaustion, recovery bounds,
failed commitment, admission poisoning and publication restrictions.

Capacity-refusal tests now use more declarations with longer valid keys: compact
payloads otherwise fit the original backing capacity. Both still exercise
refusal without published capabilities and successful retry. Their backing
limits and production admission logic are unchanged.

Matched raw Wasm builds use baseline `31e3116` (0.14.2), Rust 1.97.1, the same
resolved dependencies, unchanged exports and the maintained `wasm-size` profile.
These are raw uncompressed bytes, not native timing measurements.

| Probe | Baseline bytes | Candidate bytes | Delta |
| --- | ---: | ---: | ---: |
| Core | 256,101 | 255,754 | −347 |
| Diagnostics | 308,657 | 308,136 | −521 |
| Key-only | 255,672 | 255,326 | −346 |
| Admission | 258,809 | 258,409 | −400 |

All four existing budgets pass without adjustment. JSON is a test-only dependency;
there is no added production dependency. Production edits touch four files:
`physical.rs`, `cbor.rs`, `constants.rs` and `ledger/payload.rs`. The flow is unchanged and opaque-byte processing is simpler; the serde
adapter adds local code to retain human-readable DTO behavior.

## Consumer execution

The maintained IcyDB `testing/integration/tests/lifecycle_participant.rs` was run
from a disposable copy of the sibling worktree with a local ic-memory override,
Rust 1.98.1, PocketIC 16, unchanged local/production profiles and canonical
post-link pipeline. All three tests passed, including deferred ordering, retained
rows, trap rollback/retry and hidden-participant ingress restrictions. No
instruction probes or ceiling changes were added.

| Phase | Candidate IC instructions |
| --- | ---: |
| Init | 4,172,879 |
| Empty post-upgrade | 4,993,957 |
| Populated post-upgrade | 5,091,615 |
| Converged post-upgrade | 5,133,140 |

The worst phase passes the unchanged **12,750,000** ceiling. Empty and populated
allocated stable extents both remain **23,134,208 bytes**. All 12 consumer
memory-admission tests and the default-manager test also passed. IC cycles and
matched consumer raw Wasm deltas are unmeasured; the matched upstream raw Wasm
measurements are above. The published baseline and prototype numbers in #7 are
separate evidence, not a newly measured matched baseline for this run.

The initial sandboxed attempt could not bind the PocketIC localhost server and
its build cache detected concurrent documentation changes. The successful retry
used fixed source inputs and a temporary local server outside the sandbox; the
runner shut it down afterward. The sibling repository was not modified. Native
lock resolution in the disposable checkout is not claimed byte-identical to the
consumer's retained release environment.

Upstream validation passed: formatting, strict all-target Clippy, 236 unit tests,
integration/compile-fail/doc tests, release-tool tests, Wasm test compilation,
unchanged size budgets and Rust 1.88 MSRV checks. `make validate` reached the
package step and stopped at Cargo's uncommitted-change guard; package creation
and verification subsequently passed with `--allow-dirty`. Five existing doc
examples remain ignored by the maintained test configuration.

A local dependency override is candidate evidence, not released-dependency
acceptance. Issue #7 remains open until release and IcyDB adoption are qualified
under the unchanged ceiling. No release, commit or GitHub status change was made.

The final change touches 17 files, including four production owners, three test
owners, four replaced wire fixtures, one test dependency declaration and five
documentation files. The production change adds roughly 60 lines; it keeps one
recovery/commit flow and replaces per-byte codec work with one bounded byte
string operation.
