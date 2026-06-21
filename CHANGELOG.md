# Changelog

## 0.7.1

### Diagnostics

- Added optional live backing-memory size diagnostics for allocation records,
  including a default `MemoryManager` export helper that reports each
  `VirtualMemory::size()` in WebAssembly pages and bytes.
- Added a default `MemoryManager` commit-recovery diagnostic helper that can
  inspect protected ledger slots without requiring successful bootstrap.
- Revalidated decoded `MemoryManager` range-authority records so reversed
  ranges and the `255` sentinel cannot enter imported authority tables.
- Made raw stable-cell payload decoding classify empty memory as `NotStableCell`
  instead of relying on callers to preflight it.
- Removed duplicated internal constants for diagnostic metadata bounds and
  WebAssembly page size.

---

## 0.7.0

### Whitepaper and Lean model

- Added the `ic-memory` whitepaper, covering the stable-memory allocation
  governance problem, protocol model, allocation invariants, durable commit
  protocol, operational guidance, and current non-goals.
- Added a compact Lean model for the core allocation-safety argument, including
  checked lemmas for stable-key slot uniqueness, physical-slot key uniqueness,
  retired allocation tombstones, post-commit open authority, and generation
  monotonicity.
- Added mdBook/Nix/Lake scaffolding for building the Markdown whitepaper and
  Lean model without committing a PDF artifact.

### Protocol cleanup

- Removed the current-version compatibility range abstraction and the remaining
  ledger/envelope version-routing scaffold from recovery and commit.
- Simplified the durable ledger, envelope, diagnostics, slot descriptor, and
  schema metadata shapes by dropping unused physical-format, ledger schema,
  envelope version, descriptor substrate/version, and schema-fingerprint fields.
- Made committed generation parent links mandatory, using `0` for the first
  generation instead of accepting an absent parent value.
- Refreshed the current golden wire fixtures for the cut-down durable format.

---

## 0.6.2

### Protocol mutation coverage

- Added nested CBOR unknown-field regression tests for decoded
  `AllocationHistory`, `AllocationRecord`, `AllocationSlotDescriptor`, and
  `GenerationRecord` values inside the crate-owned ledger payload.
- Added a stable-cell wrapper regression test proving unknown top-level fields
  in `StableCellLedgerRecord` decode fail closed before the record can be used
  as a ledger anchor DTO.
- Clarified `LedgerPayloadEnvelope` rustdoc: decoding the envelope classifies
  protocol bytes only and does not establish allocation authority.

---

## 0.6.1

### Audit hardening

- Added `serde(deny_unknown_fields)` to `LedgerCommitStore`, closing the last
  authority-bearing durable DTO wrapper that could otherwise ignore future
  top-level CBOR fields during rollback.
- Added a regression test that mutates the `LedgerCommitStore` CBOR shape with
  an unknown top-level field and verifies that decode fails closed.
- Added compile-fail tests that lock the public API boundary around
  `RecoveredLedger` and `ValidatedAllocations`, proving downstream safe Rust
  cannot call their crate-private constructors.
- Made late `eager_init` registration fail closed after default runtime
  bootstrap instead of silently queueing a hook that will never run.
- Clarified that explicit genesis initialization APIs are privileged
  empty-store/import paths; normal users should prefer the default runtime or
  the golden bootstrap flow.
- Improved `ic_memory_key!` open failure text so it covers missing bootstrap,
  unvalidated keys, and key/id mismatches.

---

## 0.6.0

### Protocol authority boundary

- Added a logical ledger payload envelope inside each physically committed
  generation. Physical dual-slot recovery still selects the highest valid
  committed generation first; only then does `ic-memory` decode the logical
  envelope and route the ledger payload by schema/format metadata.
- Added `RecoveredLedger` as the crate-owned proof that a ledger crossed
  physical recovery, payload-envelope routing, compatibility checks, and
  committed-integrity validation.
- Changed `validate_allocations()` to require `RecoveredLedger` instead of raw
  `AllocationLedger`, so untrusted/manual ledger DTOs cannot mint
  `ValidatedAllocations`.
- Removed the public caller-supplied compatibility range surface from normal
  recovery and commit APIs. `LedgerCommitStore` now uses the crate-owned current
  protocol path.
- Added recovery tests for payload-envelope classification, unsupported
  envelope versions, and envelope/ledger metadata drift.
- Added reviewable v1 hex wire fixtures for payload envelopes, full commit
  stores, dual-slot recovery states, stable-cell records, and slot descriptors,
  with tests that decode, validate, recover, and re-encode them.
- Marked authority-bearing durable DTO structs with `serde(deny_unknown_fields)`
  so future fields fail closed instead of being silently ignored by older
  readers.
- Removed the custom payload encoding namespace. The logical ledger payload is
  the current `ic-memory` CBOR format inside the logical envelope.
- Removed custom allocation-slot substrates from the 0.6 authority model.
  Allocation slots are `ic-stable-structures::MemoryManager` `u8` IDs only,
  with ID 255 rejected as the sentinel.
- Removed public codec selection from the commit/bootstrap path. The durable
  ledger codec is crate-owned CBOR, so downstream crates cannot introduce a
  parallel ledger format.

---

## 0.5.1

### Audit hardening

- Made `ValidatedAllocations` an opaque non-serializable capability. It no
  longer derives serde traits and can only be produced by crate validation and
  bootstrap paths.
- Added deep validation for decoded DTOs before they can become allocation
  authority. Stable-key grammar and `MemoryManager` slot descriptor invariants
  are rechecked during snapshot validation and committed-ledger integrity
  validation.
- Raised the `validate_allocations()` authority boundary so the historical
  ledger must pass current compatibility and committed-integrity validation
  before it can produce `ValidatedAllocations`.
- Added stable-cell ledger preflight for the default runtime so corrupt
  `ic-stable-structures::Cell` storage is classified as a bootstrap error
  before `Cell::init` would otherwise panic while decoding the ledger record.
- Made the default runtime range-policy contract explicit: registered
  `ic_memory_range!` claims are enforced before caller-supplied policy, while
  framework adapters can omit user ranges and enforce application space in
  their own policy.
- Pinned the default developer and CI toolchain to Rust 1.95.0 while keeping
  the crate MSRV at Rust 1.85.0 through `package.rust-version` and an MSRV CI
  check.
- Updated crates.io metadata to describe `ic-memory` as a Memory ID registry
  wrapper for `ic-stable-structures`.

---

## 0.5.0

### Runtime registration

- Added a generic multi-crate runtime registration layer for downstream crates
  such as IcyDB, including range declarations, `ic_memory_key!`,
  `ic_memory_range!`, `eager_init!`, default `MemoryManager` bootstrap, and
  validated runtime opening without Canic.
- Moved the normal documentation path to the macro-based runtime API and moved
  lower-level ledger/bootstrap guidance to `ADVANCED.md`.
- Left TLS eager initialization out of `ic-memory`; framework helpers such as
  Canic's `eager_static!` should wrap ordinary `thread_local!` values and use
  `ic_memory_key!` / `ic_memory_range!` for allocation registration.

---

## 0.4.1

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
