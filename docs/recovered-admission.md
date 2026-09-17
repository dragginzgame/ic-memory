# Recovered-metadata admission

Issue #5 contract review, against the released 0.14.0 runtime and the referenced
IcyDB bootstrap/convergence sources.

## Contract

`RuntimeBootstrapPolicy::prepare_bootstrap` adds one hook inside the existing
runtime flow: recover → prepare → resolve → validate → commit → persist → publish.
The existing policy identity covers preparation semantics/configuration too.
There is no additional bootstrap entrypoint or separately published capability.

The hook receives `BootstrapAdmission`: the original sealed declarations and
an iterator over validated recovered key/slot/state/latest-schema metadata.
The iterator exposes at most 255 records and does not copy generation or schema
history. It offers no memory handles, raw manager, ledger DTO or mutation API.
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
to enumerate candidate journals. The supplied IcyDB references establish the
ordering gap but do not prove its future key grammar or lifecycle integration.
If incarnation or accepted-schema admission requires control-store contents,
allocation metadata alone cannot establish that fact: retain the check after
opening, or separately design durable identity metadata. This change does not
add persistent identity, interpret journals, retire databases or clear data.

Review conclusion: allocation-role discovery can close the declaration-ordering
gap without pre-commit opens. Full IcyDB integration must still supply and test
its exact role grammar and post-commit lifecycle checks. No second ledger or
persistent mode is needed for the allocation-level contract.

## Usage and errors

[`examples/recovered_admission.rs`](../examples/recovered_admission.rs) is the
runnable example. Run `cargo run --example recovered_admission`. A host calls
its generated consumers from its policy's `prepare_bootstrap` method. Existing
`bootstrap_default_memory_manager_with_policy`, configured default bootstrap and
`MemoryRuntime::bootstrap` all use that same hook. A library joining a warm host
uses `committed_allocations().slot_for(...)` and key-based opens; it cannot append
selections after the host has committed. Missing keys require coordination with
the host's next cold bootstrap, not another commit or replacement profile.

`BootstrapAdmissionError` distinguishes unknown, retired, duplicate, oversized,
invalid-registration and current-grant failures. These are wrapped in
`RuntimeBootstrapError::Admission`. Latched selection failures take precedence
over a consumer's generic return error. Consumer-only rejections use
`RuntimeBootstrapError::AdmissionPolicy`. After completion, the existing typed
resolution, validation, staging and persistence errors still apply.

Known reserved keys may activate through the existing reservation contract.
Selected keys preserve the latest diagnostic schema metadata and durable slot.
Selection does not establish application schema support. Governance keys cannot
be selected externally. Unselected history continues to own its slots and gains
no new capability. Current grants authorize selections; metadata does not claim
that a historical authority string was durably recorded.

Doctor reports do not call preparation: their validation result covers the
supplied declaration set and allocation policy, not consumer admission or a
predicted completed set. Preparation may run again after a failed cold attempt;
consumer code must tolerate retry and avoid external side effects. Ordinary warm
bootstrap does not replay it. Semantic policy identity changes are the host's
responsibility, including admission configuration.

## Qualification and bounded work

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
latest-schema references without copying histories. At most 254 external
requests, including the original set, can be completed. Each selection performs
bounded scans of current declarations, recovered records and grants; completion
runs the existing canonical snapshot builder and resolver once. With S selected
keys, D current declarations, H historical records and G grants, selection work
is O(S × (D + H + G)); all four counts are at most 255. The consumer's own
computation is trusted code, not metered by ic-memory. No new persisted fields,
formats, mode flags or accounting machinery are added. For an identical completed
declaration set the durable metadata-byte delta is zero.

Matched Rust 1.97.1 raw, uncompressed Wasm builds use the committed `wasm-size`
profile and target `wasm32-unknown-unknown`, comparing the unchanged three probes
against release `33a28e1` (0.14.0):

| Probe | Baseline bytes | Current bytes | Delta |
| --- | ---: | ---: | ---: |
| Core | 255,114 | 255,762 | +648 |
| Diagnostics | 307,589 | 307,931 | +342 |
| Key-only | 254,762 | 255,341 | +579 |
| Admission enabled | — | 258,960 | +3,619 versus current key-only |

The admission probe retains the key-only export/workload shape and adds recovered
journal discovery and selection, ensuring that code remains reachable. CI and
`make wasm-size` cover all four probes; existing budgets are unchanged, with the
admission probe also limited to 260,000 bytes. IC instructions/cycles for fresh
bootstrap and repeated opens are **unmeasured**: no IC execution harness is
configured here. No native timing is substituted. Opening itself is unchanged.

Implementation size: the new admission module is 208 lines including metadata,
errors and rustdoc; the bootstrap policy adds a 13-line optional hook, and the
runtime adds four preparation statements before its existing resolution call.
No additional lifecycle state is stored. Qualification adds a 434-line runtime
test module, 63 lines of default-runtime coverage and one compile-fail case.
Examples, documentation, re-exports and probe configuration are separate.

## Source boundary

The issue's pinned [generated bootstrap](https://github.com/dragginzgame/icydb/blob/f3bd969be9dd7087a0c00e2655d703c397756616/crates/icydb-model/src/build/actor/db/store.rs#L766-L809)
executes before the [persisted registry reconciliation](https://github.com/dragginzgame/icydb/blob/f3bd969be9dd7087a0c00e2655d703c397756616/crates/icydb-core/src/db/database_format/convergence.rs#L43-L86).
This implementation supplies the missing allocation-level preparation mechanism;
it does not change that external repository. IcyDB must adopt the hook and verify
its real role grammar, debt checks and pending markers through generated
production bootstrap before claiming end-to-end integration complete.
