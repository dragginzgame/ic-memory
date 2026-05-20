# Changelog

## 0.5.0

### Ledger hardening

- Split allocation staging behavior into `ledger::stage`, keeping the public
  staging API stable while reducing the size of `ledger::mod`.
- Replaced saturating generation diagnostic counts with explicit fail-closed
  errors when declaration or reservation counts exceed the durable `u32` limit.
- Documented that empty validated and reservation generations are intentional
  generation boundaries.
- Documented and tested reserved-record retirement semantics.
- Documented the expected allocation-ledger size bounds behind the current
  clone-on-stage implementation.
- Added a generic multi-crate runtime registration layer for downstream crates
  such as IcyDB, including range declarations, `ic_memory_key!`,
  `ic_memory_range!`, `eager_init!`, default `MemoryManager` bootstrap, and
  validated runtime opening without Canic.

---

## 0.4.0

### Breaking cleanup

- Bumped from the already-published `0.3.0` to `0.4.0` because this release
  removes public APIs that were redundant or unused.
- Removed the unused `NamespaceAuthority` and `RangeAuthority` policy traits.
  Direct `MemoryManagerRangeAuthority` methods are the supported range-policy
  API.
- Removed the redundant `AllocationSlotDescriptor::memory_manager_checked`
  constructor alias. Use `AllocationSlotDescriptor::memory_manager`.
- Removed the redundant `MemoryManagerRangeAuthority::to_records` export alias.
  Use `authorities()` for the stable read-only diagnostic/export surface.

### Documentation

- Clarified that `AllocationBootstrap` is the golden path for whichever layer
  owns an `ic-memory` ledger store, not specifically for Canic.
- Documented framework-owned, library-owned, and application-owned bootstrap
  modes, plus the rule that exactly one owner should bootstrap a given ledger
  store.
- Updated the README golden path to use
  `AllocationBootstrap::initialize_validate_and_commit`.

---

## 0.3.0

### Native IC substrate

- Made `ic-stable-structures = "0.7.2"` a normal dependency instead of an
  optional feature-gated dependency.
- Made `serde_cbor = "0.11"` a normal dependency.
- Removed the `ic-stable-structures` feature; `stable_cell` support now always
  compiles and its ledger-anchor exports are always available.
- Added `CborLedgerCodec` as the built-in CBOR codec for `AllocationLedger`
  commit payloads.
- Clarified that the native ledger stack is `MemoryManager` ID 0 ->
  `ic-stable-structures::Cell<StableCellLedgerRecord, _>` ->
  `LedgerCommitStore` -> dual protected committed `AllocationLedger` payloads.
- Kept collection construction out of scope: `ic-memory` governs allocation
  ownership and does not wrap every `ic-stable-structures` collection.

---

## 0.2.0

### Breaking / API hardening

- Bumped from the already-published `0.1.0` to `0.2.0` because this release
  tightens public DTO construction and hides fields that were public in
  `0.1.0`.
- Made invariant-bearing durable DTO fields private where feasible, including
  allocation declarations, ledger histories, ledger records, physical commit
  slots, and slot descriptors.
- Added checked constructors and accessors for public allocation DTOs so callers
  do not need struct literals for normal use.
- Added `AllocationLedger::new_committed` for strict committed-ledger
  construction.
- Removed the unused public generation DTO API from the crate surface.
- Gated corrupt-write simulation helpers behind `#[cfg(test)]`; production code
  can no longer call them.

### Safety and validation

- Added schema metadata validation to declaration staging, reservation staging,
  and committed-ledger integrity validation.
- Centralized historical claim-conflict detection for declaration validation,
  declaration staging, and reservation staging while preserving existing public
  error variants.
- Preserved the core invariant: a stable key cannot move physical slots, and an
  active physical slot cannot be reused by another stable key.

### Structure and maintenance

- Split `slot` internals into descriptor, `MemoryManager`, and range-authority
  modules while keeping crate-level re-exports stable.
- Split ledger records, errors, and integrity checks out of the main ledger
  module.
- Kept staging and commit behavior public-compatible; no Canic-specific policy
  was added.

### Documentation

- Updated README, crate docs, rustdoc, and SAFETY docs for the current checked
  constructor/accessor API.
- Added a concise golden-path sketch showing recovery, declaration,
  validation, commit, and only-then-open ordering.
- Clarified stable-key permanence, reservation behavior, tombstones, checksum
  limits, non-goals, and the boundary between generic `ic-memory`
  infrastructure and Canic/IcyDB examples.

---

## 0.0.7

### Documentation

- Added stable-key formatting guidance to the README, including grammar rules,
  valid examples, and namespace conventions.
- Documented representative `canic.core.*` and `icydb.*` stable-key patterns.
- Clarified that stable keys are permanent logical allocation identities and
  should not be changed when only schema metadata changes.
- Updated README examples to show the open-stack range-authority model,
  package-record composition, and optional closed-policy coverage checks.

---

## 0.0.6

### Added

- Added `MemoryManagerRangeAuthority`, `MemoryManagerAuthorityRecord`, and
  `MemoryManagerRangeMode` for generic `MemoryManager` range authority policy
  and diagnostics.
- Added range-authority builders and validators, including ID-bound helpers,
  mode-aware validation, complete coverage checks, and `from_records` for
  composing records from multiple packages.
- Added concise `MemoryManager` declaration helpers on
  `AllocationDeclaration` and `DeclarationCollector`, including labeled,
  unlabeled, schema-aware, and builder-style variants.
- Added `MemoryManagerIdRange::all_usable`.

### Changed

- Made `MemoryManagerIdRange` serializable for diagnostic authority records.
- Added explicit range-authority errors for overlaps, invalid ranges, missing
  coverage, records outside a coverage target, mode mismatch, and invalid
  diagnostic strings.
- Updated examples to use the concise `MemoryManager` range and declaration
  helpers.

### Policy model

- Clarified that range authority is policy/diagnostic metadata only; durable
  allocation remains the core `stable_key -> allocation_slot` ledger model.
- Clarified that `Reserved` and `Allowed` do not allocate IDs.
- Clarified the open-stack model: packages publish only the ranges they own, and
  a final composition layer uses `from_records` to catch cross-package overlaps.
  Final closed policies may add application `Allowed` ranges or complete
  coverage checks, but intermediate frameworks should not claim the remaining ID
  space by default.

---

## 0.0.3

- Repositioned documentation around stable-memory slot drift.
- Added safety model documentation.
- Hardened physical/logical generation recovery.
- Added strict committed-ledger lifecycle tests.
- Made `MemoryManager` slot construction checked by default.
