# Troubleshooting ic-memory

This guide starts with the symptom an application owner sees. Preserve the
existing stable memory while investigating. Never delete or replace the
allocation ledger merely to make bootstrap succeed.

## A quick safety rule

Most `ic-memory` failures happen before application stores are opened and before
a new allocation capability is published. The error is therefore a safety
barrier, not an instruction to reset storage.

| Symptom | Was a new open capability published? | First action |
| --- | --- | --- |
| Layout, ID, authority, or policy conflict | No | Compare the new declarations with the last working release |
| No eligible slot | No | Inspect the authority's current `Allowed` ranges and historical occupancy |
| Runtime not bootstrapped | No | Run the one correct bootstrap owner before opening stores |
| Bucket-size mismatch | No | Use the persisted setting; do not attempt an in-place resize |
| Corrupt or unsupported metadata | No | Preserve the bytes and collect read-only diagnostics |
| Retired allocation requested | No | Do not revive or reuse the retired key or slot |

## “The allocation layout changed”

Typical causes include:

- an existing stable key was assigned a different memory ID;
- an existing ID was assigned to a different key;
- two declarations use the same key or ID;
- a declaration moved outside its authority's range;
- the bootstrap policy or sealed declaration set changed during the life of an
  already bootstrapped runtime.

Compare the new declarations with the last working application version and the
doctor report. Correct accidental renames or ID changes in the new source. If
the application genuinely needs a different store, give it a new stable key and
an eligible, previously unused location. `ic-memory` does not move the old data
to that location.

Do not swap IDs back and forth experimentally against production memory. Test
the corrected declarations against a copy or representative fixture first.

## “No eligible free slot” or `MemoryResolutionError::Exhausted`

Automatic allocation uses only free IDs in a matching current `Allowed` range.
The following do not count as free space:

- governance IDs;
- fixed allocations;
- `Reserved` ranges;
- omitted historical allocations;
- reservations; and
- retired allocations.

Confirm that the requesting authority has an `Allowed` range, that its spelling
matches the request, and that at least one ID has never been claimed. If more
capacity is required, grant a new non-overlapping eligible range after checking
the host's complete layout. Do not reuse an omitted or retired ID.

## “Runtime has not completed bootstrap validation”

`RuntimeOpenError::NotBootstrapped` and
`RuntimeDiagnosticError::NotBootstrapped` mean that the current runtime has not
published committed allocation authority. Common causes are:

- a store opened before the application lifecycle called bootstrap;
- a library tried to open before its host bootstrapped;
- a native test bootstrapped a different thread's default runtime; or
- diagnostic code expected bootstrap to construct a missing default runtime.

Call the intended bootstrap helper once per concrete runtime before any stable
store initialization. In native tests, bootstrap each worker thread that uses
the thread-local default runtime. A library inside a composed application should
verify and adopt the host's committed declarations rather than running a second
bootstrap with its own policy.

## “Persisted bucket size differs from requested”

`RuntimeConstructionError::BucketSizeMismatch` means the backing memory already
contains a `MemoryManager` layout with a different bucket size. Bucket geometry
is immutable for that runtime.

Reopen the memory with its persisted configuration. Do not change the requested
number until construction succeeds, and do not expect a smaller value to shrink
existing memory. Moving data to a differently configured runtime is an explicit
application migration outside `ic-memory`.

## “Memory is foreign, unsupported, or corrupt”

Construction and recovery reject nonempty bytes that are not the supported
`MemoryManager` layout, unsupported manager versions, malformed stable-cell
records, invalid commit slots, and inconsistent ledger history. These errors
fail closed so that unknown bytes are not overwritten as a fresh ledger.

Preserve the original backing memory. Use an explicit runtime and its read-only
doctor or commit-recovery diagnostics where construction succeeds far enough to
permit them. Record the exact typed error and the application, crate, and
substrate versions. Restore from a known backup or perform a deliberately
designed external recovery only after understanding the corruption; there is no
automatic fallback to an older slot or empty ledger.

## “The key is not committed”

`RuntimeOpenError::StableKeyNotCommitted` means the current committed
declaration set does not grant that key. Check for a typo, an omitted
declaration, or opening before the correct host adoption step.

An omitted historical store remains owned but is intentionally unavailable to
ordinary opens. If the application must inspect a known historical journal,
include it through the maintained declaration or recovered-admission path before
commit. Naming it only in an open call grants nothing.

## “The requested memory ID does not match”

`RuntimeOpenError::MemoryIdMismatch` means the stable key is committed, but for
a different ID than the caller supplied. Correct the caller or use a key-based
open after resolving the committed ID. Do not change the durable declaration to
match an accidental caller value.

## “The authority or range is rejected”

Use the same explicit authority value for the component's range and its key
declarations. Fixed declarations must be inside a matching permitted range when
the host registers user ranges. Fresh automatic requests and historical
selection require explicit current grants; a custom policy alone does not
supply their placement pool.

`Reserved` permits fixed or matching historical claims. Use `Allowed` when a
range must supply new automatic allocations.

## “A retired store cannot be opened again”

Retirement is permanent allocation history, not a free-list operation. Neither
the stable key nor its former slot can be revived or assigned to something
else. This protects upgrades and rollbacks from interpreting unrelated bytes as
the retired store.

If the application needs a genuinely new store, declare a new stable key and a
never-used eligible slot. Recovery of the retired store's application data, if
required, belongs to an external application-specific recovery plan.

## Growth and capacity failures

`RuntimeGrowError` distinguishes arithmetic overflow, exhausted manager bucket
capacity, backing-memory refusal, and reentrant growth. Ordinary refusal occurs
before new manager buckets are assigned and can be retried after the underlying
capacity problem is resolved. A bucket-table capacity limit requires a new
storage design; it cannot be repaired by reusing retired IDs.

## Information to collect

Before escalating a problem, retain:

- the exact error variant and message;
- the current and last working `ic-memory` versions;
- the current and last working declarations, authorities, and bucket settings;
- a doctor report, when it can be produced safely;
- commit-recovery diagnostics, when available; and
- a protected copy or backup of the affected stable memory.

Do not include private application data in public issue reports. Allocation
metadata can also reveal internal component names, so review diagnostic output
before sharing it.

For report semantics and pre-bootstrap inspection, see
[Operations and diagnostics](operations.md). For hard safety boundaries, see
[SAFETY.md](../SAFETY.md).
