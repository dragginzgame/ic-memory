# CANIC-162: bounded physical allocation attribution

Date: 2026-09-10. Workspace implementation only; package version remains 0.12.3.
Canic and Toko Miner were inspected read-only. Their pending integration and the
reported 232 MiB live observation remain separate from this fixture evidence.

## Supported API and accounting

```rust
impl<M: ic_stable_structures::Memory> MemoryRuntime<M> {
    pub fn memory_allocations(&self)
        -> Result<MemoryAllocations, RuntimeDiagnosticError>;
    pub fn new(memory: M) -> Result<Self, RuntimeConstructionError>;
    pub fn new_with_config(memory: M, config: MemoryManagerConfig)
        -> Result<Self, RuntimeConstructionError>;
    pub const fn memory_manager_config(&self) -> MemoryManagerConfig;
    pub fn open_memory(&self, stable_key: &str, expected_id: u8)
        -> Result<RuntimeMemory<M>, RuntimeOpenError>;
}

pub fn default_memory_manager_memory_allocations()
    -> Result<MemoryAllocations, RuntimeDiagnosticError>;
pub fn bootstrap_default_memory_manager_with_config<P: RuntimeBootstrapPolicy>(
    config: MemoryManagerConfig, policy: &P,
) -> Result<CommittedAllocations, RuntimeBootstrapError<P::Error>>;

impl MemoryManagerConfig {
    pub const fn new(bucket_size_pages: u16)
        -> Result<Self, RuntimeConstructionError>;
    pub const fn bucket_size_pages(self) -> u16;
}
```

`MemoryManagerConfig` validates nonzero `u16` page counts. Its default is 128.
`new` uses that default on fresh memory and honors the persisted setting on
reopen. `new_with_config` rejects any mismatch before manager initialization.
A runtime's configuration is immutable and bound to its sole manager authority;
repeated explicit default bootstrap checks that authority before bootstrap.
The application policy's identity and declaration binding remain independently
enforced. No policy grants access by itself, no reset or shrink is provided,
and no migration or alternate durable format is introduced.

The owned, serializable report contains:

- `current_generation: Option<u64>` from committed runtime authority; unknown
  before bootstrap, without decoding ledger state to recover it.
- Actual `bucket_size_pages`, `bucket_size_bytes`, `manager_layout_version`,
  `bucket_capacity`, `allocated_buckets`, `remaining_buckets`, and
  `maximum_bucket_bytes` (table capacity, before backing limits).
- `physical_extent` and total `virtual_extent`, each with `wasm_pages` and
  `bytes`. Physical extent describes the supplied backing. It represents IC
  stable memory for `DefaultMemoryImpl` on Wasm and host vector extent in these
  fixtures; it does not include canister heap or platform billing allocation.
- `manager_metadata_bytes = 65,536`, decomposed into a 2,080-byte header,
  32,768-byte bucket table, and 30,688 bytes of padding.
- `allocated_bucket_bytes`, `bucket_slack_bytes`, `known_binding_bytes`,
  `unknown_binding_bytes`, and `unmanaged_bytes` after the assigned region.
- Exactly 255 `memories`, ordered by ID 0–254, each with `binding`, optional
  `range_claim`, `virtual_extent`, `allocated_buckets`, `allocated_bytes`,
  `bucket_slack_bytes`, and `payload_bytes: None`.

`AllocationBinding::Current { stable_key, owner }` comes from successfully bound
current declarations. `Ledger { stable_key, owner }` labels ic-memory's reserved
ledger slot at ID 0; it does not validate that slot's payload. `Unknown` includes
retired, absent, and unbound declarations. A range claim is only current policy
metadata, not proof of historical ownership. No retired ID is omitted merely
because its key is unavailable. Before bootstrap all application bindings are
unknown; the reserved ledger identity is still labeled.

Every reported size is measured from validated metadata or exact arithmetic
on those measurements; none is an estimate. Payload occupancy is unavailable.
For example, an initialized cell with an eight-byte value can have a 65,536-byte
virtual extent and an 8,388,608-byte assigned bucket. Only 8,323,072 bytes are
bucket rounding slack; bytes inside the virtual extent may include structure
metadata and free space that this report does not measure.

Conservation identities, valid also for incomplete key attribution:

```text
physical_extent.bytes = manager_metadata_bytes
                      + allocated_bucket_bytes + unmanaged_bytes
allocated_bucket_bytes = sum(memories[*].allocated_bytes)
                       = known_binding_bytes + unknown_binding_bytes
                       = virtual_extent.bytes + bucket_slack_bytes
```

The ledger is one of the rows and is already included in these totals. Never
add its bytes a second time. Unknown binding bytes are already assigned to IDs;
unmanaged bytes are outside the assigned region. Do not combine either with
payload usage or relabel physical-minus-virtual as application overhead.

## Read bounds, validation, and ownership

The adapter in `src/runtime/layout.rs` is coupled to the exact
`ic-stable-structures = "=0.7.2"` documented MGR V1 layout. It reads the header
and full fixed table: two reads totaling 34,848 bytes. It rejects unsupported
magic/version/byte order, reserved fields, zero bucket size, excess bucket
count, holes or entries beyond the allocated prefix, inconsistent per-ID
virtual extents, insufficient physical extent, and byte overflow. A report also
checks persisted bucket size and per-ID sizes against its live runtime.
Failures are typed through `RuntimeConstructionError` / `MemoryManagerLayoutError`
and `RuntimeDiagnosticError`; no fallback interpretation is attempted.

Current registration/range counts are checked against the ID domain before
metadata reads or copies. Current sealed keys are validated to at most 128
bytes, authorities to 256; the returned text is bounded above by 255 ×
(128 + 256 + 256) = 163,200 bytes, plus fixed row/container storage. Collection
never reads or clones the ledger cell, historical records, schema history, or
generations. A bounded table read is not pagination of an unbounded export.

One private `Rc` shares access to the owned backing with the sole manager.
`RuntimeMemory<M>` wraps its virtual handle, implements `Memory` and `Clone`,
and preserves support for non-Clone, non-Send, non-Sync backing types. No raw
backing or manager handle is exposed. The caller must still honor the existing
sole-owner contract and the `Memory` trait contract; arbitrary alias writes or
concurrent outside mutation are not a supported runtime operation.

The default report inspects existing TLS only. If no runtime has been
constructed it returns `NotBootstrapped`; it never constructs a manager as a
side effect. A previously constructed but unbootstrapped runtime can report.
Explicit fresh construction does initialize the manager, and bootstrap does
commit: these are separate lifecycle actions, not diagnostic work.

## Canic adoption example

Keep controller authorization on the existing protected observation and Root
relay. In the ops owner, replace the current committed-handle loop with this
substrate call (existing Canic error mapping shown):

```rust
fn allocation_snapshot() -> Result<ic_memory::MemoryAllocations, InternalError> {
    let report = ic_memory::default_memory_manager_memory_allocations()
        .map_err(MemoryRegistryOpsError::from)?;
    // Preserve Canic's current requirement for completed bootstrap.
    if report.current_generation.is_none() {
        return Err(InternalError::public(
            crate::diagnostics::codes::STATE_INVALID,
        ));
    }
    Ok(report)
}
```

This is the ops collection boundary. Map the owned report into Canic's own
Candid DTO at its existing DTO boundary; ic-memory does not add Candid or an
endpoint. The current `MemoryAllocationsResponse` cannot express this report
fully: change it directly to carry all 255 rows, binding variants, optional
range claims, per-ID bucket/slack values, manager metadata and separate
unknown-binding/unmanaged residuals. Preserve `payload_bytes = None` and the
measurement meaning. Do not filter to current keys or coerce unknown keys into
fabricated strings or `Active` state. If maintaining a derived current-only
view, label it as such and retain all excluded allocation bytes in explicit
residuals so that its conservation still works.

`RuntimeOpenError` and policy validation still govern opens. Update stable-store
annotations such as `Cell<T, VirtualMemory<DefaultMemoryImpl>>` directly to
`Cell<T, ic_memory::RuntimeMemory<DefaultMemoryImpl>>`. Update imports in Canic
core/control-plane stores and any downstream store aliases that require the
concrete handle type; do not add a renamed `VirtualMemory` compatibility alias.
The existing macros already obtain the current handle from the default runtime.

Retain Canic's existing bootstrap call for the default/adopt-persisted behavior:

```rust
ic_memory::bootstrap_default_memory_manager_with_policy(
    &policy::CanicMemoryManagerPolicy::new(),
)?;
```

For a separately approved fresh-state 16-page policy, the memory bootstrap owner
could instead use:

```rust
let config = ic_memory::MemoryManagerConfig::new(16)
    .map_err(ic_memory::RuntimeStateError::from)?;
ic_memory::bootstrap_default_memory_manager_with_config(
    config,
    &policy::CanicMemoryManagerPolicy::new(),
)?;
```

Supply it before eager store initialization or any helper that constructs the
default runtime, including a bootstrap-readiness helper. Existing 128-page
memory explicitly rejects 16; it is not reduced in place. Use the same setting
on later bootstrap calls. This example is an adoption option, not a change to
Canic or Toko in this workspace.

## Disposable measurements and policy assessment

Reproduce three independent fresh-state runs:

```sh
cargo run --release --offline --example allocation_measurements > /tmp/run-1.csv
for run in 2 3; do
  target/release/examples/allocation_measurements > "/tmp/run-${run}.csv"
done
```

Retained CSVs: [run 1](measurements/canic162/run-1.csv),
[run 2](measurements/canic162/run-2.csv), [run 3](measurements/canic162/run-3.csv).
Environment: Rust 1.98.1 release profile, x86_64 Linux, AMD Ryzen Threadripper
7970X. Backing uses instrumented native `VectorMemory`. Times are elapsed host
microseconds with no acceptance threshold; they are not IC instructions,
cycles, latency, or capacity guarantees. Runs are small and not a statistically
controlled performance study; operation counts and byte sizes are the stronger
evidence. Allocator zero-fill cost heavily affects small-cell initialization.

Each fixture declares 40 IDs, bootstraps the actual ic-memory ledger, and
initializes 28 `Cell<u64>` stores with tiny values. Twelve declared IDs remain
unopened at that point. A further store then appends 8,192 × 1,024-byte rows to a
`StableVec`, reaching 129 virtual pages and crossing every candidate's bucket
boundary. Eleven declared IDs remain unopened. Subsequent phases perform 20,000
pseudo-random reads, 20,000 explicit crossing reads, reopen without bootstrap
(incomplete current attribution), rebootstrap, 100 read-only replays, 64 more
bootstrap generations, and a history-independent report. Manager construction
and every mutation phase are separate from diagnostic measurement. Counters
exclude the report used to capture each workload's resulting extents.

| Bucket pages | Bucket bytes | 28 cells + ledger physical bytes | Virtual bytes | Ledger bucket bytes | Table capacity |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 128 (default) | 8,388,608 | 243,335,168 (232.0625 MiB) | 1,900,544 | 8,388,608 | 256 GiB |
| 16 | 1,048,576 | 30,474,240 (29.0625 MiB) | 1,900,544 | 1,048,576 | 32 GiB |
| 8 | 524,288 | 15,269,888 (14.5625 MiB) | 1,900,544 | 524,288 | 16 GiB |
| 1 | 65,536 | 1,966,080 (1.875 MiB) | 1,900,544 | 65,536 | 2 GiB |

All rows have 29 assigned buckets plus one manager page. Each of the 28 cells
and the ledger has one virtual page. This deliberately small-Hub-like fixture
reproduces the *arithmetic* behind the reported 232 MiB; it does not identify
Toko's stores or establish that its live ledger and store extents match.

The candidates represent 1 MiB and 512 KiB allocation granularities and a
one-page lower bound. Their finite-table consequences justify comparing them:
32,768 slots are shared across all IDs including the ledger, and buckets are
not reclaimed when application contents are deleted. Capacity figures exclude
the metadata page and any stricter backing/platform limits. Sparse exhaustion
tests reach the table limit without allocating GiBs on the host and verify that
a further bucket returns `-1` without changing the report. Upstream backing
growth failure itself can panic; this work does not change growth atomicity.

| Bucket pages | After growing-store physical bytes | Total virtual bytes | Total buckets | Append backing grow calls | Random-get backing reads |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 128 | 260,112,384 | 10,354,688 | 31 | 1 | 40,002 |
| 16 | 39,911,424 | 10,354,688 | 38 | 8 | 40,019 |
| 8 | 24,182,784 | 10,354,688 | 46 | 16 | 40,039 |
| 1 | 10,420,224 | 10,354,688 | 158 | 128 | 40,312 |

Appending grows by 128 physical pages in each candidate, after the vector's
initial allocation. Smaller buckets increase growth and bucket-table writes.
The random-get phase reads the same 20,640,000 backing bytes in each case; more
rows cross a boundary with smaller buckets. Every explicit 32-byte crossing
read splits into two backing reads (40,000 total), as expected.

Median elapsed microseconds across the three runs:

| Bucket pages | Initialize 28 cells | Append rows | 20,000 random gets | Reopen + unbound report | 100 reports |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 128 | 132,234 | 5,458 | 3,762 | 84 | 2,991 |
| 16 | 17,183 | 5,243 | 3,884 | 61 | 2,991 |
| 8 | 8,492 | 5,351 | 4,470 | 60 | 2,957 |
| 1 | 1,198 | 5,296 | 3,926 | 61 | 3,057 |

Reopen without bootstrap performs zero writes/growth and reports application
binding bytes as unknown while preserving physical/per-ID bucket accounting.
Rebootstrap is deliberately separate and advances the generation. All 100
read-only replays are identical and use 200 reads totaling 3,484,800 bytes, with
zero writes/growth. After 64 additional bootstrap generations, the ledger still
fits one virtual page in this fixture; reporting still reads exactly 34,848
bytes. A separate corrupt-ledger test advertises an enormous cell length and
proves that attribution never reads or decodes it. Unmanaged-tail tests add
physical pages outside the manager and verify the separate residual identity.

**Decision:** retain the 128-page default. The size savings justify supported
opt-in configuration. Sixteen pages is a useful fresh-state evaluation candidate
with 32 GiB table capacity; eight pages trades that down to 16 GiB, and one page
limits the entire manager to 2 GiB. None is selected for Toko by this work:
actual live attribution, intended lifetime capacity, IC growth/access costs,
and downstream protected-observation adoption remain outstanding. There is no
live query, deployment, memory reduction, or inference of a leak in this evidence.

## Validation

- Full suite: 204 library tests, integration tests, all five trybuild cases,
  and doctests pass on Rust 1.98.1 (five existing doctests remain ignored).
- Focused runtime coverage: 22 tests, including nine new attribution/configuration
  cases. Tests cover conservation, zero IDs, crossings, reserved/unauthorized
  opens, corrupt/foreign/unsupported metadata, byte overflow, no writes/growth,
  history-independent reads, default TLS behavior, capacity exhaustion, and
  same-release recovery/replay. A public integration test also verifies owned
  reports and cloned handles with borrowed, non-Clone backing memory.
- `cargo +1.88.0 check --offline --all-targets`: passes at the declared MSRV.
- Warning-denied all-target Clippy passes on Rust 1.98.1 and CI Rust 1.97.1.
  The pre-existing MSRV increase also required three const accessor updates
  and two fixed-chunk fixture decoder lint adjustments.
- `cargo +1.97.1 check --offline --target wasm32-unknown-unknown --tests`: passes.
- Existing raw Wasm budgets remain unchanged: core 240,226 / 245,000 bytes;
  diagnostics, now including allocation-report serialization, 289,044 / 290,000
  bytes, built with Rust 1.97.1 and the existing `wasm-size` profile.
- Warning-denied rustdoc and `cargo package --allow-dirty --offline` (including
  package verification) pass. Formatting and diff whitespace checks pass.
  No versions, publication,
  deployment, or sibling files were changed.
