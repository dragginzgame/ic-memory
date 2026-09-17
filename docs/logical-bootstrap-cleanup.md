# Logical bootstrap cleanup (#6)

Baseline: release 0.14.1, commit `91288c45e450d98c96e447a6f64487223d004087`.
This is a construction/repeated-work cleanup with no public API or durable-format
change and no additional lifecycle state, cache or allocator.

## Changes and ordering proof

Only `MemoryRequest` sorting uses `sort_unstable_by`. Accepted stable keys are
unique. Equal keys reject before admission regardless of authority or schema, so
sort stability cannot affect an accepted snapshot. Fixed declaration and range
comparators remain unchanged. Tests exercise all six permutations of three
requests, equivalent fingerprints, duplicate logical keys with differing metadata,
and fixed/logical duplicate keys.

Historical selection still calls `MemoryRequest::new` first, preserving early
key/authority bounds and namespace checks. After duplicate, count, known-key,
retirement and grant checks, it consumes that request with the recovered schema.
Only the schema is revalidated; authority/key allocation and parsing are reused.
Selection errors remain sticky even when a callback discards their results.

Admission completion now returns its selected requests instead of constructing a
sealed snapshot that resolution would immediately rebuild. The original input
retains its canonical request order. Before placement, the resolver marks **all**
historical slots occupied, including omitted, reserved and retired records.
Admission selections are known-only and therefore cannot compete with fresh
requests for a slot; processing them after the original requests cannot change
new placement. The final snapshot builder sorts and validates all resolved
fixed declarations together, checks collisions, rebuilds range authority and
computes the resolved fingerprint. The ordinary final policy, staging, durable
write and capability publication still follow that step.

The original sealed input remains the warm-bootstrap identity. A distinct final
snapshot/fingerprint still describes resolved assignments. The intermediate
completed fingerprint was never exposed or used as a runtime identity and is no
longer computed. Tests compare resolved results against independently sealed
fixed declarations for both historical-selection permutations, while asserting
that the original fingerprint is unchanged and differs from the resolved one.
Existing poisoning, failed-persistence/retry, policy/profile binding, reservation,
retirement, exhaustion and bounded-recovery tests remain in place.

## Work by bootstrap path

The counts below describe snapshot builds inside runtime bootstrap, excluding
initial linked-input sealing. They are source-level work counts, not instruction
or cycle measurements.

| Path | 0.14.1 | Current |
| --- | --- | --- |
| Fresh key-only bootstrap, no historical selection | One resolved build | One resolved build |
| Ordinary cold reopen, no historical selection | One resolved build | One resolved build |
| Cold bootstrap with historical completion | Completed build, then resolved build | One resolved build |
| Fixed-only bootstrap without historical completion | Reuse input | Reuse input |
| Matching warm bootstrap/adoption | No preparation or snapshot build | Unchanged |

Historical completion removes one complete declaration/range copy and
canonicalization, one range-authority reconstruction and one fingerprint encoding.
Selected requests move directly into the resolver; original requests are borrowed
rather than copied into a merged request set. Final declaration validation and
all recovery preflights remain. No unbounded collection or computation is added.

A prototype also reused canonical range tables for the final snapshot by splitting
the builder. It still needed owned copies and increased the four probes by a
further 330–397 bytes relative to the retained change. That part is deferred;
sharing ranges through new snapshot storage would be a wider redesign. The final
builder retains its existing range reconstruction and validation.

## Matched raw Wasm qualification

Both baseline and current sources use Rust 1.97.1, the same lockfile, unchanged
probe sources/exports, `wasm32-unknown-unknown` and the committed `wasm-size`
profile (size optimization, fat LTO, one codegen unit, stripped symbols).
Artifacts are raw and uncompressed. The baseline was built from `git archive`
in a separate temporary checkout.

| Probe | Baseline bytes | Current bytes | Delta |
| --- | ---: | ---: | ---: |
| Core | 255,760 | 256,099 | +339 |
| Diagnostics | 307,939 | 308,657 | +718 |
| Key-only | 255,340 | 255,671 | +331 |
| Admission enabled | 258,968 | 258,809 | −159 |

All existing budgets pass without changes. These are mixed code-size results,
not a claim that every consumer binary shrinks. An isolated unstable-sort change
increased these probes by 216–224 bytes; it is retained for allocation-free
request sorting, not claimed as a Wasm-size saving. Reusing the request identity
alone in that variant reduced the admission probe by 134 bytes. Combined results
need not be the sum of isolated changes because optimization is whole-program.

The admission-enabled probe retains the historical-selection path in its binary;
raw size does not measure the cost of executing that path. IC instructions/cycles
for fresh bootstrap, ordinary cold reopen, historical completion and warm reuse
are **unmeasured**: this workspace has no configured IC execution harness. No
native or wall-clock timing substitutes for them. The issue's IcyDB integration
size delta and shallow sorting-symbol attribution are separate measurements and
are not treated as attributable savings here.

Run the maintained comparison workload with:

```sh
make wasm-size
```

Production changes are limited to request schema replacement/sorting and resolver
inputs in `registry.rs`, completion in `runtime/admission.rs`, and the existing
runtime/diagnostic callers. Added tests live in `registry/tests.rs`. There is no
consumer application change or extra commit boundary.
