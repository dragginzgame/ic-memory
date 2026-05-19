# Changelog

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
