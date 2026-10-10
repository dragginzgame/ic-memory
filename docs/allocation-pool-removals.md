# 0.34.0 symbol retirement inventory

Compared with released 0.33.4 `9137192d4b3425a229fa21df5f343883f56d80e8`.
Names include private/nested helpers and tests. `[Trait]` identifies a trait implementation.
The repeated numeric macro callbacks occurred in two retired arms; each name is listed once.

## Deletions

### crates/ic-memory/examples/key_only.rs

Remove the old per-owner range fixture builder; the host constructs MemoryAllocationPool directly.

- `grant` (fn).

### crates/ic-memory/src/diagnostics.rs

Replace fixed/range projections and constructors with source MemoryRequest values and the tested MemoryAllocationPool.

- `DiagnosticDeclaration` (struct).
- `DiagnosticDeclaration::new` (fn).
- `DiagnosticRangeAuthority` (struct).
- `DiagnosticRangeAuthority::new` (fn).

### crates/ic-memory/src/lib.rs

Retire numeric/range macro forms and their generated callbacks. The current macro registers key-only MemoryRequest metadata.

- `ic_memory_declaration::__ic_memory_register_static_declaration` (fn).
- `ic_memory_declaration::__ic_memory_defer_static_declaration` (fn).
- `ic_memory_range` (macro_rules!).
- `ic_memory_range::__ic_memory_register_static_range` (fn).
- `ic_memory_range::__ic_memory_defer_static_range` (fn).

### crates/ic-memory/src/registry.rs

Remove fixed component input and range registration/sealing paths. Source snapshots accept only requests; host pool policy is separate and runtime-owned.

- `StaticMemoryDeclaration::new` (fn).
- `StaticMemoryDeclaration::into_declaration` (fn).
- `StaticMemoryRangeDeclaration` (struct).
- `StaticMemoryRangeDeclaration::new` (fn).
- `StaticMemoryRangeDeclaration::authority` (fn).
- `StaticMemoryRangeDeclaration::record` (fn).
- `StaticMemoryRangeDeclaration::into_record` (fn).
- `SealedDeclarationSnapshot::registered_ranges` (fn).
- `SealedDeclarationSnapshot::range_authority` (fn).
- `SealedDeclarationSnapshot::user_ranges_registered` (fn).
- `register_static_memory_declaration` (fn).
- `register_static_memory_manager_range` (fn).
- `register_static_memory_range_declaration` (fn).
- `register_static_memory_manager_declaration` (fn).
- `register_static_memory_manager_declaration_with_schema` (fn).
- `internal_ledger_range` (fn).

### crates/ic-memory/src/registry/tests.rs

Retire fixed/range-only fixtures. Canonical request tests, lifecycle tests and checked host pool tests cover the maintained boundaries.

- `registers_static_memory_ranges` (fn).
- `static_range_declaration_uses_record_authority` (fn).
- `snapshot_rejects_duplicate_static_memory_declarations` (fn).
- `snapshot_order_is_independent_of_registration_order` (fn).
- `snapshot_overlap_errors_preserve_bound_order_despite_differing_metadata` (fn).
- `large_snapshot_permutations_preserve_all_metadata_and_fingerprints` (fn).
- `equal_fixed_sort_keys_reject_independently_of_label_and_schema_order` (fn).
- `resolved_history_permutations_match_fully_sealed_declarations_and_fingerprints` (fn).

### crates/ic-memory/src/runtime/allocations.rs

Remove per-range diagnostic ownership. MemoryAllocation.pool_eligible reports current host membership without claiming historic custody.

- `AllocationRangeClaim` (struct).

### crates/ic-memory/src/runtime/default.rs

Consolidate opens into open_default_memory_manager_memory(key); no alternate name remains.

- `open_default_memory_manager_memory_by_key` (fn).

### crates/ic-memory/src/runtime/mod.rs

Consolidate opens into MemoryRuntime::open_memory(key); no alternate name remains.

- `MemoryRuntime::open_memory_by_key` (fn).

### crates/ic-memory/src/runtime/policy.rs

Replace numeric-range validation with validate_runtime_claim, checking current namespace ownership and pool eligibility.

- `RuntimeMemoryManagerPolicy::validate_runtime_range` (fn, renamed to
  `validate_runtime_claim` with namespace and pool checks).

### crates/ic-memory/src/runtime/request_tests.rs

Replace partition/fixed-contract fixtures with shared-pool/cold-reopen tests. Existing admission regressions own persistence retry, reserved activation, retirement and custom rejection; duplicate Limited/Revoked implementations are removed.

- `id` (fn).
- `fragmented_grants_place_in_order_and_exhaust_without_reusing_history` (fn).
- `exhaustion_and_fixed_conflicts_do_not_publish_or_change_committed_state` (fn).
- `reservation_activation_and_retirement_use_existing_claim_rules` (fn).
- `failed_persistence_publishes_no_mapping_and_retries_deterministically` (fn).
- `failed_persistence_publishes_no_mapping_and_retries_deterministically::Limited` (struct).
- `failed_persistence_publishes_no_mapping_and_retries_deterministically::Limited[Memory]::size` (fn).
- `failed_persistence_publishes_no_mapping_and_retries_deterministically::Limited[Memory]::grow` (fn).
- `failed_persistence_publishes_no_mapping_and_retries_deterministically::Limited[Memory]::read` (fn).
- `failed_persistence_publishes_no_mapping_and_retries_deterministically::Limited[Memory]::write` (fn).
- `current_custom_policy_can_reject_a_recovered_logical_key` (fn).
- `current_custom_policy_can_reject_a_recovered_logical_key::Revoked` (struct).
- `current_custom_policy_can_reject_a_recovered_logical_key::Revoked[AllocationPolicy]::Error` (type).
- `current_custom_policy_can_reject_a_recovered_logical_key::Revoked[AllocationPolicy]::validate_key` (fn).
- `current_custom_policy_can_reject_a_recovered_logical_key::Revoked[AllocationPolicy]::validate_slot` (fn).
- `current_custom_policy_can_reject_a_recovered_logical_key::Revoked[AllocationPolicy]::validate_reserved_slot` (fn).
- `current_custom_policy_can_reject_a_recovered_logical_key::Revoked[RuntimeBootstrapPolicy]::runtime_bootstrap_identity` (fn).

### crates/ic-memory/src/runtime/tests.rs

Remove the fixed-range ordering fixture; request/pool admission tests retain custom-policy rejection and before-commit ordering.

- `fixed_range_checks_preserve_custom_policy_admission_and_rejection_order` (fn).

### crates/ic-memory/src/slot/mod.rs

Remove owner-range/mode/coverage fixtures. Physical bounds tests remain; pool constructor/decode and runtime tests qualify namespace admission and exclusions.

- `tests::memory_manager_range_authority_accepts_non_overlapping_construction` (fn).
- `tests::memory_manager_range_authority_rejects_overlap` (fn).
- `tests::memory_manager_range_authority_rejects_sentinel_lookup` (fn).
- `tests::memory_manager_range_authority_finds_authority_for_id` (fn).
- `tests::memory_manager_range_authority_validates_slot_authority` (fn).
- `tests::memory_manager_range_authority_validates_slot_authority_mode` (fn).
- `tests::memory_manager_range_authority_validates_id_authority` (fn).
- `tests::memory_manager_range_authority_reports_authority_mismatch_before_mode_mismatch` (fn).
- `tests::memory_manager_range_authority_preserves_reserved_and_allowed_modes` (fn).
- `tests::memory_manager_range_authority_validates_complete_coverage` (fn).
- `tests::coverage_preserves_last_usable_id_and_outside_before_gap` (fn).
- `tests::memory_manager_range_authority_rejects_complete_coverage_gaps` (fn).
- `tests::memory_manager_range_authority_rejects_complete_coverage_outside_target` (fn).
- `tests::memory_manager_range_authority_from_records_sorts_checked_records` (fn).
- `tests::memory_manager_authority_record_constructor_validates_metadata` (fn).
- `tests::full_domain_range_permutations_preserve_order_and_metadata` (fn).
- `tests::memory_manager_range_authority_from_records_rejects_first_overlap_in_range_order` (fn).
- `tests::memory_manager_range_authority_preserves_inclusive_overlap_boundaries` (fn).
- `tests::memory_manager_range_authority_deserialization_rejects_overlap` (fn).
- `tests::memory_manager_range_authority_diagnostic_export_is_stable` (fn).

### crates/ic-memory/src/slot/range_authority.rs

Retire owner partitions, modes, their DTOs/decoders, coverage and overlap helpers. MemoryAuthority and MemoryAllocationPool check current namespace grants and physical exclusions.

- `MemoryManagerRangeMode` (enum).
- `MemoryManagerAuthorityRecord` (struct).
- `MemoryManagerAuthorityRecord[Deserialize]::deserialize` (fn).
- `MemoryManagerAuthorityRecord[Deserialize]::deserialize::Record` (struct).
- `MemoryManagerAuthorityRecord::new` (fn).
- `MemoryManagerAuthorityRecord::range` (fn).
- `MemoryManagerAuthorityRecord::authority` (fn).
- `MemoryManagerAuthorityRecord::mode` (fn).
- `MemoryManagerAuthorityRecord::purpose` (fn).
- `MemoryManagerRangeAuthority` (struct).
- `MemoryManagerRangeAuthorityDto` (struct).
- `MemoryManagerRangeAuthority[Deserialize]::deserialize` (fn).
- `MemoryManagerRangeAuthority::new` (fn).
- `MemoryManagerRangeAuthority::from_records` (fn).
- `MemoryManagerRangeAuthority::validate_slot_authority` (fn).
- `MemoryManagerRangeAuthority::validate_slot_authority_mode` (fn).
- `MemoryManagerRangeAuthority::validate_id_authority` (fn).
- `MemoryManagerRangeAuthority::validate_id_authority_mode` (fn).
- `MemoryManagerRangeAuthority::authority_for_id` (fn).
- `MemoryManagerRangeAuthority::authorities` (fn).
- `MemoryManagerRangeAuthority::validate_complete_coverage` (fn).
- `MemoryManagerRangeAuthorityError` (enum).
- `ranges_overlap` (fn).
- `validate_diagnostic_string` (fn).

### crates/ic-memory/tests/diagnostic_metadata.rs

Replace range-record decode fixtures with checked MemoryAuthority decode fixtures for the current diagnostic contract.

- `range_record_decode_rejects_invalid_metadata_before_table_assembly` (fn).
- `range_record_decode_preserves_current_shape_and_explicit_null` (fn).

### crates/ic-memory/tests/runtime_macros.rs

Remove the placeholder numeric-macro type; current macros register owner/key requests without a storage label type.

- `MacroStore` (struct).

The unused range-purpose constant `IC_MEMORY_AUTHORITY_PURPOSE` is also removed
from `src/slot/memory_manager.rs` and its public reexports.

## Moves and renames

From `src/slot/range_authority.rs` to `src/slot/id_range.rs`, unchanged physical bounds:

- `MemoryManagerIdRange` (struct).
- `MemoryManagerIdRange[Deserialize]::deserialize` (fn).
- `MemoryManagerIdRange[Deserialize]::deserialize::Bounds` (struct).
- `MemoryManagerIdRange::new` (fn).
- `MemoryManagerIdRange::all_usable` (fn).
- `MemoryManagerIdRange::contains` (fn).
- `MemoryManagerIdRange::start` (fn).
- `MemoryManagerIdRange::end` (fn).
- `MemoryManagerRangeError` (enum).

The following were renamed and narrowed; they are not additional allocators:

- `adoption_checks_fixed_and_logical_requirements_without_replaying_host` → `adoption_checks_logical_requirements_without_replaying_host` in `crates/ic-memory/src/runtime/adoption_tests.rs`.
- `adoption_rejects_missing_keys_wrong_authorities_ids_and_metadata` → `adoption_rejects_missing_keys_wrong_authorities_and_metadata` in `crates/ic-memory/src/runtime/adoption_tests.rs`.
- `GenericRangePolicy` → `GenericAllocationPolicy` in `crates/ic-memory/src/runtime/policy.rs`.
- `GenericRangePolicy[AllocationPolicy]::Error` → `GenericAllocationPolicy[AllocationPolicy]::Error` in `crates/ic-memory/src/runtime/policy.rs`.
- `GenericRangePolicy[AllocationPolicy]::validate_key` → `GenericAllocationPolicy[AllocationPolicy]::validate_key` in `crates/ic-memory/src/runtime/policy.rs`.
- `GenericRangePolicy[AllocationPolicy]::validate_slot` → `GenericAllocationPolicy[AllocationPolicy]::validate_slot` in `crates/ic-memory/src/runtime/policy.rs`.
- `GenericRangePolicy[AllocationPolicy]::validate_reserved_slot` → `GenericAllocationPolicy[AllocationPolicy]::validate_reserved_slot` in `crates/ic-memory/src/runtime/policy.rs`.
- `GenericRangePolicy[RuntimeBootstrapPolicy]::runtime_bootstrap_identity` → `GenericAllocationPolicy[RuntimeBootstrapPolicy]::runtime_bootstrap_identity` in `crates/ic-memory/src/runtime/policy.rs`.

The follow-up audit also removes `StaticMemoryDeclaration`'s pending-only
replacement `ResolvedMemoryDeclaration` and its `authority`/`declaration` methods.
The original `StaticMemoryDeclaration`, `authority` and `declaration` are therefore
retirements, rather than retained renames. No resolved-row type remains.

In `src/registry.rs`, `SealedDeclarationSnapshot::allocation_snapshot`,
`registered_declarations` and `registered_declaration` are removed: source
snapshots now hold only requests. CommittedAllocations is the owner for current
placement/schema projections.

In `src/runtime/policy.rs`, `RuntimeMemoryManagerPolicy::declaration_authority`
and `validate_runtime_claim` are removed. Pool admission already occurs in
resolution; current owners are projected from the bound pool. The original
`validate_runtime_range` is therefore a retirement, not a retained rename.
`RuntimePolicyError` in `src/runtime/error.rs` is removed; custom validation
errors are carried directly. In `src/capability.rs`, `slot_for_key` is renamed
`declaration_for_key` and returns the complete borrowed declaration so slot,
schema and adoption projections share one lookup.
