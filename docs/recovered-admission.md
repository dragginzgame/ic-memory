<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-memory/ic-memory-readme-header.svg" alt="IC Memory — Stops upgrades from mixing up stored data" width="100%">
</p>

# Recovered-metadata admission

Current admission contract, introduced in 0.14.1 for completed issue #5 and
simplified in 0.14.2. The design review and measurements below retain their
original 0.14.0/0.14.1 scope; current downstream acceptance is recorded in the
[issue reconciliation](issue-reconciliation.md).

## Contract

`RuntimeBootstrapPolicy::prepare_bootstrap` adds one hook inside the existing
runtime flow: recover → prepare → resolve → validate → commit → persist → publish.
The existing policy identity covers preparation semantics/configuration too.
There is no additional bootstrap entrypoint or separately published capability.

The hook receives `BootstrapAdmission`: the original sealed declarations and
an iterator over validated recovered key/slot/state/latest-schema metadata.
The iterator exposes at most 255 current ownership records; the ledger stores
no generation or schema audit trail. It offers no memory handles, raw manager, ledger DTO or mutation API.
`include_historical(authority, key)` selects only a known nonretired allocation
under an explicit current host grant. It retains the recovered slot and schema.
Unknown/retired selections reject instead of entering ordinary new placement.
Selections cannot add grants, remove original declarations or change host policy.
Duplicate and oversized completion sets reject. Selection errors remain latched
for this attempt so ignoring a returned error cannot publish a partial set.

Consumers can reject identity/key-set transitions in the same hook. Their error
is returned before staging or persistence. Key naming and interpretation are
consumer contracts, not inferred by ic-memory. Hosts compose library preparation
functions in one policy hook, then run the existing final policy on the completed
set. Library warm adoption only checks committed keys and opens them.

A matching warm bootstrap returns its existing capability without preparing
again. Changing preparation requires a new policy identity; the existing bound
identity and sealed-source checks reject mismatched warm bootstrap. A failed
attempt remains unbootstrapped and may retry preparation against the same durable
mapping. Fresh backing may acquire the manager/root Cell before preparation;
existing mappings never advance on admission rejection. IC trap rollback still
owns interrupted-write atomicity. A callback is trusted host code: metadata and
selection work are structurally bounded, not arbitrary consumer computation.

## IcyDB-shaped example reviewed before implementation

Given current declarations for `db.main.control.v1` and a new store, inspect all
recovered keys in the host-granted pool. Consumer policy requires the existing
control identity to remain declared, permits additional stores, and rejects a
replacement control identity. Collect keys matching the consumer's exact journal
role grammar under `db.main`; explicitly select only omitted journal keys.
After one successful commit, open those journals and let IcyDB inspect debt and
pending-commit markers before deciding database lifecycle operations.

This requires allocation keys to encode the consumer's identity/role sufficiently
to enumerate candidate journals. At the original review, the supplied IcyDB
references established the ordering gap without proving its role grammar or
lifecycle integration. Subsequent acceptance is recorded in the reconciliation.
If incarnation or accepted-schema admission requires control-store contents,
allocation metadata alone cannot establish that fact: retain the check after
opening, or separately design durable identity metadata. This change does not
add persistent identity, interpret journals, retire databases or clear data.

Original review conclusion: allocation-role discovery can close the
declaration-ordering gap without pre-commit opens. A complete consumer
integration supplies and tests its exact role grammar and post-commit lifecycle
checks. No second ledger or persistent mode is needed for the allocation-level
contract.

## Usage and errors

[`examples/recovered_admission.rs`](../crates/ic-memory/examples/recovered_admission.rs) is the
runnable example. Run `cargo run --example recovered_admission`. A host calls
its generated consumers from its policy's `prepare_bootstrap` method. Existing
`bootstrap_default_memory_manager_with_policy`, configured default bootstrap and
`MemoryRuntime::bootstrap` all use that same hook. A library joining a warm host
uses `verify_default_memory_manager_authority(...)` or owned
`runtime.verify_authority(...)` to check its requirements, then resolves IDs and
opens by key. It cannot append selections after the host has committed. Missing
keys require coordination with the host's next cold bootstrap, not another
commit or replacement profile.

`BootstrapAdmissionError` distinguishes unknown, retired, duplicate, oversized,
invalid-registration and current-grant failures. These are wrapped in
`RuntimeBootstrapError::Admission`. Latched selection failures take precedence
over a consumer's generic return error. Consumer-only rejections use
`RuntimeBootstrapError::AdmissionPolicy`. After completion, the existing typed
resolution, validation, staging and persistence errors still apply.

Known reserved keys may activate through the existing reservation contract.
Selected keys preserve the latest diagnostic schema metadata and durable slot.
Selection does not establish application schema support. Governance keys cannot
be selected externally. Unselected records continue to own their slots and gains
no new capability. Current grants authorize selections; metadata does not claim
that a historical authority string was durably recorded.

Doctor reports do not call preparation: their validation result covers the
supplied declaration set and allocation policy, not consumer admission or a
predicted completed set. Preparation may run again after a failed cold attempt;
consumer code must tolerate retry and avoid external side effects. Ordinary warm
bootstrap does not replay it. Semantic policy identity changes are the host's
responsibility, including admission configuration.

## Qualification and bounded work

### Composed-host cold reopens and native threads

Run `cargo run --example composed_host`. Its two public-API regressions are also
enabled in the ordinary `cargo test -- --test-threads=1` suite; they can be run
alone with `cargo test --example composed_host -- --test-threads=1`.

The host admits `host.` and `db.` namespaces into one common pool, requests
three keys and delegates consumer identity admission through one host policy.
After writing all three stores, it reconstructs the runtime over the same
backing twice with 16-page buckets. Cold bootstrap runs preparation and commits
one generation. Consumer adoption and matching warm bootstrap do not rerun
preparation or write bytes. Assigned IDs, current bindings, sizes and payloads
remain intact.

Before each accepted reopen, a replacement consumer control declaration
returns `RuntimeBootstrapError::AdmissionPolicy` with the exact consumer error.
No capability or store handle is published, and the complete existing backing
remains byte-for-byte unchanged. A new runtime then admits the original
declarations over that same evidence. The identity rule is illustrative;
ic-memory does not infer application intent or database readiness.

The native regression starts two workers. Each observes typed
`NotBootstrapped` before adoption, bootstraps the configured host on its own
thread, and only then adopts consumer authority and touches stores. The static
declaration registry is shared across the program; native default runtimes and
their backing are thread-local. Applications must follow this ordering before
touching any thread-local store initializer, including host stores that are
independent of database readiness.

These are substrate/public-API regressions using `VectorMemory`, not executed
Canic/IcyDB lifecycle participants. Canic still owns its PocketIC participant
and store-restoration qualification. Reported Toko failures have not established
an ic-memory defect. IC instructions/cycles and matched consumer Wasm deltas
for this additional regression are unmeasured; native timing is not a metric.

### Recovery and historical selection

Production `MemoryRuntime` tests preserve an omitted journal's debt/commit marker
and commit exactly one new generation. They reject unknown, duplicate, foreign,
revoked and retired selections, consumer key-set/owner replacement, excessive
completion, corruption, exhausted placement and failed backing growth. Rejection
leaves existing memory unchanged and publishes no capability. Retry succeeds
against the unchanged evidence; reserved selection preserves schema metadata.
Configured default-runtime tests verify the same preparation flow and warm
adoption without replay or profile replacement. A compile-fail test demonstrates
that admission supplies no open method. Existing interrupted/corrupt commit tests
remain applicable; arbitrary partially written native backing is not made
crash-atomic by this hook.

Recovered metadata has at most 255 records; iteration borrows key/slot/state and
latest-schema references without copying records. At most 254 external
requests, including the original set, can be completed. Membership uses the sealed
canonical source request vector; only earlier selections need a
scan. Selection also scans bounded recovered records and checks current grants.
Completion returns the selected requests directly; the resolver builds the final
canonical snapshot once, without constructing an intermediate completed snapshot.
With S selected keys, D current declarations, H historical records and G grants,
selection work is O(S × (D + H + G)); all four counts are at most 255. The
consumer's own computation is trusted code, not metered by ic-memory. No new
persisted fields, formats, mode flags or accounting machinery are added. For an
identical completed declaration set the durable metadata-byte delta is zero.

The original 0.14.1 matched Rust 1.97.1 raw, uncompressed Wasm builds use the
committed `wasm-size` profile and target `wasm32-unknown-unknown`, comparing the
unchanged three probes against release `33a28e1` (0.14.0):

| Probe | 0.14.0 bytes | 0.14.1 bytes | Delta |
| --- | ---: | ---: | ---: |
| Core | 255,114 | 255,762 | +648 |
| Diagnostics | 307,589 | 307,931 | +342 |
| Key-only | 254,762 | 255,341 | +579 |
| Admission enabled | — | 258,960 | +3,619 versus 0.14.1 key-only |

The admission probe retains the key-only export/workload shape and adds recovered
journal discovery and selection, ensuring that code remains reachable. At that
release, CI and `make wasm-size` covered four probes with a 260,000-byte
admission budget. The current Make/CI gates cover five probes and allow 264,000
admission bytes after the Rust 1.99 toolchain qualification. IC instructions/cycles
for fresh bootstrap and repeated opens are **unmeasured**: no IC execution harness is
configured here. No native timing is substituted. Opening itself is unchanged.

The original 0.14.1 implementation added one preparation context and no stored
lifecycle state. The [logical bootstrap cleanup report](logical-bootstrap-cleanup.md)
records the subsequent construction/snapshot simplification and matched sizes.

## Source boundary

At the issue's original baseline, the pinned [generated bootstrap](https://github.com/dragginzgame/icydb/blob/f3bd969be9dd7087a0c00e2655d703c397756616/crates/icydb-model/src/build/actor/db/store.rs#L766-L809)
executes before the [persisted registry reconciliation](https://github.com/dragginzgame/icydb/blob/f3bd969be9dd7087a0c00e2655d703c397756616/crates/icydb-core/src/db/database_format/convergence.rs#L43-L86).
The allocation-level hook shipped in 0.14.1. IcyDB subsequently adopted shared
role admission and qualified debt/pending-marker checks through generated
production bootstrap; [the reconciliation](issue-reconciliation.md#downstream-acceptance)
records that evidence and the later released-dependency acceptance. The pinned
sources describe the original ordering gap, not a current integration blocker.
Live application composition and consumer lifecycle checks remain downstream
responsibilities.
