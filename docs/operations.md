# Operations and diagnostics

*Documentation reviewed against ic-memory 0.24.12.*

This guide is for application owners who need to inspect storage allocation,
choose a `MemoryManager` bucket size, or diagnose a bootstrap failure. Start
with the [README](../README.md) if you first need the purpose and basic setup of
`ic-memory`.

## What the reports measure

`ic-memory` can report how stable memory is divided among `MemoryManager` IDs
without opening application stores. The report distinguishes:

- locations bound to stores in the current application;
- the internal `ic-memory` allocation ledger;
- allocated locations whose current owner is unknown; and
- bytes outside the manager's assigned region.

Use `runtime.memory_allocations()` or
`default_memory_manager_memory_allocations()` for the detailed, per-ID
`MemoryAllocations` report. It returns all 255 usable IDs in order, including
zero-size IDs and the ledger at ID 0.

Use `runtime.memory_allocation_summary()` or
`default_memory_manager_memory_allocation_summary()` for recurring numeric
metrics. `MemoryAllocationSummary` contains the same capacity and conservation
totals without constructing 255 rows or copying store names, owners, or range
claims.

Both paths read exactly 34,848 bytes of validated manager metadata. They do not
decode ledger history, initialize stores, write, grow memory, or advance the
ledger generation. Default-runtime helpers also refuse to construct a missing
runtime merely to answer a diagnostic request.

## Understanding the numbers

The reports measure allocated address space, not how many bytes of meaningful
application data are present. In particular:

- **Physical extent** is the size of the supplied backing memory.
- **Virtual extent** is the addressable size exposed by individual virtual
  memories.
- **Allocated bucket bytes** are physical bytes assigned to memory IDs.
- **Bucket slack** is assigned bucket capacity beyond virtual extent.
- **Unknown binding bytes** belong to IDs whose current store cannot be named
  safely.
- **Unmanaged bytes** lie outside the manager's assigned region.
- `payload_bytes` is unavailable because `ic-memory` does not inspect each
  collection's internal format.

The following identities should hold:

```text
physical bytes = manager metadata + assigned bucket bytes + unmanaged bytes
assigned bucket bytes = sum(per-ID bucket bytes)
                      = known binding bytes + unknown binding bytes
                      = virtual bytes + bucket slack
```

A current range declaration is policy metadata, not proof of historical
ownership. Retired and omitted stores remain unknown rather than being guessed
from their bytes.

## Choosing a bucket size

Fresh runtimes default to 128 Wasm pages, or 8 MiB, per bucket.
`MemoryRuntime::new` honors the setting already persisted in an existing
same-release memory.

To choose a different size for new memory, construct an explicit runtime with
`MemoryRuntime::new_with_config(memory, MemoryManagerConfig::new(pages)?)`, or
select the default runtime's setting on its first bootstrap with
`bootstrap_default_memory_manager_with_config(config, &policy)`.

All nonzero `u16` page counts are supported. The setting is immutable for that
runtime. Existing memory must match exactly: configuration never shrinks,
migrates, or reinterprets an existing layout.

Smaller buckets reduce the physical allocation caused by small stores, but they
also reduce total table capacity and can increase growth operations and
cross-bucket reads. Do not select a value from fixture measurements alone. Base
the choice on the application's expected store count, lifetime capacity, and
live measurements. The historical
[CANIC-162 report](canic162-memory-attribution.md) records the detailed tradeoff
study that introduced configurable buckets.

## Bootstrap state and early observations

`is_default_memory_manager_bootstrapped()` and `committed_allocations()` do not
construct a missing runtime. They return `false` and `NotBootstrapped`,
respectively, without choosing a bucket size.

Default memory opens, ID resolution, diagnostic export, commit-recovery
inspection, and doctor reports follow the same rule. This ensures that an early
observation cannot silently select the 128-page default before the real
bootstrap owner applies its intended configuration and policy.

An existing but unbootstrapped explicit runtime can report physical allocation
with unknown application bindings. For pre-bootstrap inspection with a known
configuration, construct `MemoryRuntime::new_with_config(memory, config)` and
then call its recovery or doctor methods.

## Doctor reports

Use `default_memory_manager_doctor_report()` for operator-facing validation
with the built-in policy. If the runtime uses a custom policy, call
`default_memory_manager_doctor_report_with_policy(&policy)`, or call
`runtime.doctor_report(&declarations, &policy)` on an explicit runtime.

A doctor report includes:

- stable-cell status;
- protected commit recovery;
- recovered ledger export;
- registered declarations and range authority;
- validation under the tested policy;
- live slot sizes when recovery permits measurement; and
- the tested and established runtime binding.

Failures include a stable `DiagnosticCode` in addition to a human-readable
message. Automation should use the code rather than parsing the prose.

Doctor validation does not run `RuntimeBootstrapPolicy::prepare_bootstrap`,
predict which historical stores a consumer will select, or certify consumer
admission. It is a read-only diagnostic, not permission to open memory.

Use `default_memory_manager_commit_recovery_diagnostic()` when only commit-slot
presence, validity, authoritative generation, and corruption or ambiguity are
needed. `default_memory_manager_diagnostic_export()` additionally requires a
completed bootstrap.

## Growth failures

`RuntimeMemory::grow` returns `Result<u64, RuntimeGrowError>` to direct callers.
Ordinary backing refusal, arithmetic overflow, reentry, and bucket exhaustion
are distinct typed errors. A normal refusal occurs before manager bucket
assignment, preserving virtual extents and manager metadata for retry.

The upstream `ic_stable_structures::Memory` trait requires an `i64` result, so
only that adapter maps a growth failure to `-1`. Applications calling the
runtime handle directly should use `?`, `match`, or an explicit error handler.

A zero-page growth request returns the current virtual extent without backing
reads, writes or growth. It still rejects reentrant growth while another handle
holds the runtime's capacity reservation.

## Security and access control

Allocation reports do not authorize endpoints or users. Keep controller checks
and response filtering in the integrating application. A report can describe a
range claim or current binding, but only committed runtime authority can open a
store.

For lower-level runtime ownership, policy, recovery, and manual persistence
details, continue with [ADVANCED.md](../ADVANCED.md). The non-negotiable safety
properties are in [SAFETY.md](../SAFETY.md).
