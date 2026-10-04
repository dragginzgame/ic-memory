# Changelog

## 0.24.8

- Reuse the recovered ledger during declaration, reservation and retirement
  bootstrap staging instead of cloning its complete history. Share each staging
  implementation with the public borrowed API, which retains copy semantics.
- Move registry declarations, requests and ranges into snapshot sealing instead
  of cloning inputs that are immediately discarded. Logical resolution also
  transfers its completed declaration vector into the same builder. Retain
  borrowed public inputs, canonical ordering, error precedence and fingerprints.
- Extend reservation failure coverage to a conflict after an earlier item has
  been staged. Both borrowed staging and bootstrap leave the source ledger and
  protected store unchanged on failure.
- Public APIs, durable and diagnostic formats, and declaration fingerprints are
  unchanged. No Canic source patch is required for these local changes.
- Validation: 272 library tests, six selected public runtime/configuration/macro
  integration tests and two composed-host regressions pass. Strict all-target
  Clippy, formatting, whitespace checks and all five raw Wasm size gates pass.
  Consumer builds, complete package verification and live deployments were not
  rerun for this candidate. Runtime performance improvements were not measured.

## 0.24.7

- Remove the second grant lookup during fresh logical allocation. Placement
  already selects the lowest free ID from the declaring authority's current
  `Allowed` grants. Recovered assignments still require current authorization;
  final policy checks, historical occupancy and exhaustion behavior remain.
- Remove retirement staging's final bounds rescan. Its initial check already
  bounds records and schema history and reserves room for one generation;
  retirement preserves record and schema counts. Input checks, generation
  overflow handling and protected commit validation remain enforced.
- Use `Arc::make_mut` for application-only capability publication. Remove the
  manual unwrap-or-clone and unconditional replacement allocation. Retain
  isolation from shared validated and committed capabilities and preserve
  publication timing.
- Inline retirement bootstrap's single-caller staging/commit helper. Keep the
  public operation as the sequencing owner, with unchanged recovery, retirement
  error projection and protected commit checks.
- Remove snapshot sealing's temporary stable-key tree. Use canonical request
  adjacency and fixed-declaration binary search for duplicate detection;
  preserve mixed-conflict precedence, accepted ordering and fingerprints.
- Record focused qualification of the Canic 0.110.52 release source against
  published ic-memory 0.24.6: 14 native memory regressions and default-feature
  core Wasm compilation pass. Only ic-memory changed in the isolated dependency
  graph; the active Canic checkout and live deployment remain outside this proof.
- Public APIs, durable and diagnostic formats, and declaration fingerprints are
  unchanged. No Canic source patch is required for these local changes.
- Validation: 272 library tests, six selected public runtime/configuration/macro
  integration tests and two composed-host regressions pass. Shared-capability
  isolation, mixed duplicate-error precedence and retirement history bounds pass.
  Strict all-target Clippy, formatting, whitespace checks and all five raw Wasm
  size gates pass.
  Consumer builds, complete package verification and live deployments were not
  rerun for this candidate.

## 0.24.6

- Remove staged-ledger clones and recovery-proof round trips from declaration,
  reservation and retirement bootstrap commits. Share the checked commit
  operation while retaining caller-owned staged ledgers, predecessor validation,
  and capability publication only after persistence confirmation.
- Create new active and reserved records from borrowed declarations, copying
  only persisted key, slot and schema fields instead of cloning discarded labels.
- Remove the doctor's duplicate empty-cell success path. Use the maintained
  decoder's empty-memory behavior while preserving empty, readable and corrupt
  classifications and read-only diagnostics.
- Remove redundant authority, mode and purpose tie breakers from range
  canonicalization and delete the private range-mode ordering helper. Retain
  bound ordering, overlap-error precedence and declaration fingerprints.
- Remove the redundant range-start sentinel check. Ordered bounds and the
  existing end check still reject ID 255, including the singleton sentinel;
  reversed-bound errors retain precedence.
- Share one private allocation-count limit across declaration validation,
  record decoding, ledger integrity and reservation staging. Derive it from
  the usable ID domain; keep the 255-item ceiling and existing error ordering.
- Validate committed generation chains with one parent cursor starting at
  genesis. Remove the optional predecessor state and repeated defaulting;
  retain contiguous history, strict parent links and error precedence.
- Public APIs, durable and diagnostic formats, and declaration fingerprints are
  unchanged. No Canic source patch is required for these local changes.
- Validation: 272 library tests, six selected public runtime/configuration/macro
  integration tests and two composed-host regressions pass. Current byte
  fixtures, the pinned fingerprint, overlap diagnostics and sentinel boundary
  checks pass. Strict all-target Clippy, formatting, whitespace checks and all
  five raw Wasm size gates pass. Consumer builds, complete package verification
  and live deployments were not rerun for this candidate.

## 0.24.5

- Delete the four archived 0.12 runtime, construction and policy design
  documents, removing 818 lines of superseded guidance. Current architecture
  and safety documentation remain in `README.md`, `ADVANCED.md` and `SAFETY.md`.
- Record qualification of published 0.24.4 against an isolated IcyDB source
  snapshot: all 27 focused admission, error-projection, lifecycle and
  logical-memory tests pass. Installed lifecycle phases remain below the
  unchanged 12,750,000 instruction ceiling, with stable extents preserved.
  The receipt identifies the tested inputs; IcyDB's subsequent commit and
  dependency graph remain outside its scope. Canic qualification remains pending.
- This release changes documentation only. Public APIs, durable and diagnostic
  formats, and declaration fingerprints are unchanged.
- Validation: removed-document references, qualification evidence, receipt
  links and whitespace checks pass. The downstream results cover published
  0.24.4, not this candidate; producer test suites, package verification and
  live deployments were not rerun for this documentation-only change.

## 0.24.4

- Consolidate runtime governance filtering into one committed-capability
  operation. Remove the arbitrary prefix parameter and runtime forwarding
  helper; preserve shared-capability isolation and publication only after
  persistence confirmation.
- Remove the test-only CBOR map insertion wrapper. Build fixture maps directly
  with tuples and vector literals; retain unknown-field, missing-field and
  retirement-state rejection coverage.
- Remove the whitepaper, Lean model, mdBook/Nix/Lake scaffolding, all four
  associated maintainer targets, obsolete ignore rules and documentation links.
  Delete the superseded 0.6 protocol proposal. Current architecture and safety
  guidance remain in `ADVANCED.md` and `SAFETY.md`.
- Correct runtime ownership documentation for shared backing and lazy TLS
  construction with cached failures. Update the direct ledger writer's bound
  enforcement description. Record the published 0.24.3 IcyDB qualification:
  27 focused tests passed, including installed lifecycle and logical-memory
  recovery. That receipt covers 0.24.3, not this candidate; Canic qualification
  remains pending.
- Public APIs, durable and diagnostic formats, and declaration fingerprints are
  unchanged. No Canic source patch is required for these local changes.
- Validation: 270 library tests, three selected public runtime/configuration
  integration tests and two composed-host regressions pass. Strict all-target
  Clippy, formatting and whitespace checks, six Wasm budget-tooling regression
  tests, and all five raw Wasm size gates pass. Package contents were checked
  for removal of the whitepaper and Lean files; complete package verification,
  consumer builds and live deployments were not rerun for this candidate.

## 0.24.3

- Encode ledger CBOR directly into the final payload-envelope buffer. Remove
  the intermediate raw payload buffer and private codec unit type; share header
  construction with the public envelope writer. Durable bytes, checksums,
  recovery validation and ledger-byte limit errors remain unchanged.
- Reject unknown keys, mismatched slots and already-retired allocations before
  cloning ledger history during retirement staging. Retain input validation,
  staging bounds, generation checks and existing error precedence.
- Public APIs, durable and diagnostic formats, and declaration fingerprints are
  unchanged. No Canic source patch is required for these local changes.
- Validation: 270 library tests, eight public integration tests, seven
  compile-fail cases, two composed-host regressions and five doctests pass;
  five existing sketches remain ignored. Current byte fixtures, exact envelope
  comparison, oversized-commit conservation and retirement rejection checks
  pass. Strict all-target Clippy, Rust 1.88 all-target checking, warning-denied
  Rustdoc, Wasm test compilation and all five raw Wasm size gates pass.
  IcyDB's lockfile now selects released 0.24.2; consumer builds and live
  deployments were not requalified for this candidate. Runtime performance
  improvements were not measured.

## 0.24.2

- Track duplicate allocation slots with fixed occupancy arrays instead of
  general-purpose trees in declaration and ledger validation. Retain key and
  generation sets, decoded sentinel rejection and existing error precedence.
- Let `StaticMemoryRangeDeclaration::new` own external authority validation.
  Remove the registration-time recheck of its immutable checked input;
  reserved-authority, decoded-record and registry lifecycle checks remain.
- Validate retirement constructor keys once through `StableKey::parse`, then
  validate the supplied slot directly. Retain full retirement validation at
  decoded-input and staging boundaries, including key-before-slot errors.
- Public APIs, durable and diagnostic formats, and declaration fingerprints are
  unchanged. No Canic source patch is required for these local changes.
- Validation: 269 library tests, eight public integration tests, seven
  compile-fail cases, two composed-host regressions and five doctests pass;
  five existing sketches remain ignored. Extended existing tests cover
  duplicate-error precedence, malformed inputs and late range registration.
  Strict all-target Clippy, Rust 1.88 all-target checking, warning-denied
  Rustdoc, Wasm test compilation and all five raw Wasm size gates pass.
  Downstream builds and live deployments were not requalified.

## 0.24.1

- Resolve fresh logical requests by walking the declaring authority's ordered
  `Allowed` grants directly. Remove repeated per-ID grant searches; preserve
  lowest-free-ID placement, fixed claims, historical occupancy and exhaustion.
- Use one range-authority validator in runtime policy. Remove the preliminary
  lookup and second validation path; retain the exception for unclaimed fixed
  external slots when no user ranges exist. Governance ownership, strict grants
  and custom-policy callback ordering remain enforced.
- Simplify complete-coverage checking to one `u8` cursor. Remove widened state,
  impossible conversion errors and the redundant advancement condition;
  preserve precise gaps, ID 254 coverage and out-of-target error precedence.
- Public APIs, durable and diagnostic formats, and declaration fingerprints are
  unchanged. No Canic source patch is required for these local changes.
- Validation: 269 library tests, eight public integration tests, seven
  compile-fail cases, two composed-host regressions and five doctests pass;
  five existing sketches remain ignored. Strict all-target Clippy, Rust 1.88
  all-target checking, warning-denied Rustdoc and Wasm test compilation pass.
  All five raw Wasm probes pass their existing budgets. Downstream builds and
  live deployments were not requalified.

## 0.24.0

- Fix `StableKey::parse` to validate and store the same borrowed string. A
  stateful `AsRef<str>` implementation can no longer substitute a different
  value between validation and construction.
- Serialize static declaration registrations directly when computing sealed
  fingerprints. Remove the duplicate projection type and temporary projection
  vector; preserve the fingerprint material, bytes and algorithm version.
- Hard cut: remove `RuntimeGrowError::ManagerRefused`. Successful capacity
  admission and backing reservation establish the pinned manager's growth
  contract; an unexpected result is an internal invariant panic. Backing
  refusal, bucket exhaustion, arithmetic overflow and reentry remain typed
  errors, including retry and refusal conservation guarantees.
- Reject reservation batches exceeding 255 items before declaration validation
  or policy callbacks. Bootstrap and raw staging share one count rule; recovery,
  explicit genesis initialization and existing-store conservation remain.
- Hard cut: allocation validation, ledger integrity, staging, reservation and
  retirement errors carry `AllocationSlotDescriptor` directly. Remove all 17
  boxed slot fields and their allocations; update callers constructing or
  inspecting those fields to the current Rust API. Error variants and messages
  are unchanged.
- Construct range authority through ordered insertion and neighbouring overlap
  checks. Remove repeated full-table scans and sorts while preserving input
  validation order, first-overlap errors, canonical output and fingerprints.
- Durable ledger and diagnostic formats are unchanged.
- Downstream qualification remains outstanding. Canic was left untouched;
  consumer builds and live deployments were not requalified for this release.
- Validation: 266 library tests, eight public integration tests, seven
  compile-fail cases, two composed-host regressions and five doctests pass;
  five existing sketches remain ignored. Strict all-target Clippy, Rust 1.88
  all-target checking, warning-denied Rustdoc and Wasm test compilation pass.
  All five raw Wasm probes pass their existing budgets. The key and
  oversized-reservation regressions fail against their original implementations
  and pass after the fixes.

## 0.23.0

- Hard cut: retain `DiagnosticExport::from_ledger` as the sole ledger-export
  constructor. Remove `from_ledger_with_commit_recovery`,
  `from_ledger_with_memory_sizes` and
  `from_ledger_with_commit_recovery_and_memory_sizes`. Exporters fill the public
  observation fields directly. Runtime export and doctor share one recovered
  export path, measuring records without a temporary slot map or join. Protected
  recovery still precedes measurement; diagnostics remain read-only.
- Hard cut: remove `DiagnosticGeneration`. `DiagnosticExport::generations`
  contains `GenerationRecord` values directly. Diagnostic JSON/CBOR generation
  entries expose the record's fields directly instead of a nested `generation`
  object. Update readers and regenerate reports using the current shape; no
  alias or old decoder remains.
- Hard cut: remove `MemoryManagerRangeAuthority`'s `reserve`, `allow`,
  `reserve_ids`, `allow_ids` and their four `with_purpose` variants. Construct
  records with `MemoryManagerAuthorityRecord::new` using explicit modes, then
  compose them with `MemoryManagerRangeAuthority::from_records`. Remove private
  insertion forwarding and update maintained examples and behavior tests.
  Decoded-record validation, overlap rejection, ascending range order, purposes,
  complete coverage and error precedence remain. Remove builder-only coverage
  and consolidate duplicated constructor assertions.
- Remove the duplicate ledger-envelope prefix-length check. The checked prefix
  read preserves the same truncation error, minimum length and format-error
  precedence.
- Durable ledger encoding, checksums and declaration fingerprints are unchanged.
  No compatibility aliases, fallback readers or replacement framework are added.
- **Canic adoption is required after 0.23.0 publication:** update its ic-memory
  dependency and apply the prepared generation-reader patch together. Its public
  Candid response shape is unchanged. Requalify the published dependency graph
  and refresh/verify the embedded allocation peer before calling adoption
  complete. Canic remains on published 0.22 until this release is live.
- Validation: 261 library tests, eight public integration tests, the three
  maintained range examples, strict all-target Clippy, Rust 1.88 all-target
  checking, warning-denied Rustdoc and Wasm test compilation pass. All five raw
  Wasm probes pass their budgets for the main cleanup; the prefix-only follow-up
  passes all three focused envelope tests and strict all-target Clippy.
  Fourteen Canic memory tests and strict Core all-target/all-feature Clippy pass
  against the prepared reader in an isolated copy using the local candidate.
  Published-0.23 consumer adoption, installed-canister qualification and
  release-flow tests have not been run.

## 0.22.0

- Hard cut: remove the public `Validate` trait. `StableKey::validate` and
  `AllocationSlotDescriptor::validate` are now inherent methods; remove trait
  imports and use the concrete methods directly. Decoded DTO validation remains.
- Reuse checked request fields during logical placement instead of replaying
  public declaration constructors. Remove `MemoryResolutionError::Declaration`
  and update IcyDB's affected error match. Final snapshot validation, range
  authorization, deterministic placement, recovery and publication ordering remain.
- Populate detailed allocation-report bindings and range claims directly into
  ordered rows, removing per-ID metadata searches. Preserve the separate numeric
  summary, ledger attribution, all 255 rows and bounded read-only accounting.
- Share one borrowed declaration/history/policy check between bootstrap and
  doctor. Doctor no longer clones its resolved declaration snapshot or constructs
  a discarded pre-commit capability. Preserve validation ordering and read-only
  diagnostics; admission callbacks and application schema validation remain
  outside doctor's scope.
- Hard cut: remove `DiagnosticMemorySizeOutcome` and `DiagnosticCode::MemorySize`.
  `DiagnosticRecord::memory_size` now contains an optional `DiagnosticMemorySize`
  directly, without the `Measured` wrapper. Update diagnostic producers/readers
  and regenerate reports using the current shape; no old reader is retained.
  Invalid persisted slots still fail recovery before measurement. Qualify Canic's
  adapter and measured/unmeasured fixtures in an isolated copy and prepare an
  adoption patch; its public response shape is unchanged. Concurrent Canic work
  retains its published 0.21 reader until upstream adoption.
- Hard cut: remove `DeclarationCollector` and its mutable/consuming builder
  methods. Construct declarations with `AllocationDeclaration`, collect them in
  a `Vec` and pass it to `DeclarationSnapshot::new`. Remove builder-only tests
  and update the manual example; constructor and snapshot invariants remain.
- Durable formats and declaration fingerprints remain unchanged. No
  compatibility aliases or replacement framework are added.
- All 263 library tests, eight public integration tests, two composed host tests,
  seven compile-fail cases and five active doctests pass. Strict all-target
  Clippy, Rust 1.88 all-target checking, warning-denied Rustdoc and Wasm test
  compilation pass. Twenty focused IcyDB native tests and 13 Canic memory tests
  pass in isolated source copies against this candidate. No consumer dependency
  or lockfile changes are made by this cleanup. Canic's prepared reader requires
  adoption of the new ic-memory contract when it is published. Focused IcyDB
  library Clippy and Canic Core all-target/all-feature Clippy pass on those copies.
- Matched raw Wasm probes with Rust 1.99 and the unchanged lockfile all remain
  within budget: core 247,279 bytes (−674), diagnostics 292,604 (−2,353), key-only
  246,756 (−817), admission 250,034 (−894), runtime integration 256,530 (−834).
  Deltas compare this combined cleanup against the 0.21.0 release with the same
  probe sources and build profile; IC instruction/cycle changes are unmeasured.
  Installed-canister qualification, package verification and release-flow tests
  were not run; release-flow tests create user-owned commits and tags.

## 0.21.0

- Hard cut: remove `RuntimeStateError::InconsistentLifecycle`,
  `StaticMemoryDeclarationError::InconsistentLifecycle` and
  `StaticMemoryDeclarationError::SnapshotFingerprintEncoding`. Remove obsolete
  constructions or match arms directly; no aliases or compatibility paths remain.
  The known IcyDB error-classification fixture is updated without removing its
  coverage of live internal failures.
- Express private runtime and registry lifecycle guarantees as invariant
  assertions. Preserve typed construction, corruption, reentry, poisoning,
  deferred-hook, growth-refusal and policy errors, persistence-before-publication,
  and failed-bootstrap retry behavior. Document the invariant panic boundary.
- Build private ledger declarations and governance metadata infallibly from
  checked constants. Reuse the authoritative governance-range helper instead of
  reconstructing its bounds. Public declaration and range constructors retain
  their fallible validation boundaries.
- Attach historical schema metadata directly from the immutable recovered ledger,
  removing repeated validation and fallible propagation. Public metadata
  constructors and untrusted recovery still validate schemas; historical
  selection retains authorization, retirement, bounds and sticky failure checks.
- Make concrete ledger and declaration-fingerprint encoding into byte vectors
  infallible. Retain typed decoding errors, writer byte limits, current fixtures,
  checksums, canonical ordering and fingerprint bytes. Durable formats and
  diagnostic wire shapes remain unchanged.
- Validate 205 focused library tests across both cleanup passes, six public
  integration tests, two composed host tests, all seven compile-fail cases and
  five doctests. Strict all-target
  Clippy, Rust 1.88 all-target checking and Wasm test compilation pass. All five
  raw Wasm budgets pass; core is 247,953 bytes under its 260,000-byte ceiling.
  Twenty focused IcyDB native tests pass in an isolated copy with the initial
  local API hard cut patched in; consumer manifests and lockfiles remain unchanged.
  Package verification, installed-canister qualification and release-flow tests
  were not repeated; release-flow tests create maintainer-owned commits and tags.

## 0.20.0

- Hard cut: remove `RuntimePolicyError::MissingDeclarationMetadata`,
  `RuntimeBootstrapError::LedgerIntegrity` and `DiagnosticCode::GenesisLedger`.
  Remove obsolete match arms or constructions, the automatic conversion from
  `LedgerIntegrityError` into `RuntimeBootstrapError`, and use of the diagnostic
  wire value `genesis_ledger`. These failure paths were unreachable through
  maintained runtime operations. Real ledger integrity failures still return
  `RuntimeBootstrapError::LedgerCommit(LedgerCommitError::Integrity(...))`.
- Simplify committed-ledger generation membership validation after structural
  bounds and the contiguous parent chain have passed. Check only that each
  allocation's first generation is nonzero; remove repeated last-seen,
  retirement and schema-generation membership checks. Preserve structural
  validation, genesis rejection and error precedence.
- Make private runtime declaration-authority lookup infallible when validating
  the allocation snapshot from the same immutable resolved declaration snapshot.
  Remove metadata-error plumbing while preserving range authorization, custom
  policy rejection and internal governance handling.
- Share one private, infallible empty-genesis constructor between cold bootstrap
  and doctor validation. Remove the diagnostic-only wrapper and impossible
  genesis failure branches. Preserve empty-store initialization, fail-closed
  recovery, persistence ordering and read-only diagnostics.
- Extend genesis-reference coverage with later schema observations and
  retirement; verify history-gap errors still precede genesis rejection. Add
  doctor coverage for combined fixed and logical declarations before and after
  bootstrap, with unchanged backing bytes.
- Record the [follow-up audit](docs/audits/recurring/simplification-followup-0.19.0.md).
  Validate 266 library tests, integration and compile-fail tests, doctests,
  strict Clippy, Rust 1.88 all-target checking and Wasm test compilation. All
  five raw Wasm budgets pass; core is 248,351 bytes under its 260,000-byte ceiling.
  Durable ledger formats, current fixtures, checksums and declaration
  fingerprints remain unchanged; the diagnostic wire vocabulary loses only the
  removed genesis code. Package verification was not repeated. Release-flow
  tests were not run because they create commits and tags reserved for the
  maintainer.

## 0.19.0

- Hard cut: remove `LedgerPayloadEnvelopeError::PayloadLengthOverflow` and
  `StableCellPayloadError::LengthOverflow`. Remove obsolete match arms or
  constructions. These failures were unreachable after the existing byte
  ceilings passed; oversized input still returns the current typed size errors.
- Simplify ledger-envelope length arithmetic and stable-cell length conversion
  using the established byte bounds. Preserve the untrusted decoded `u64`
  conversion, physical-capacity and format checks, exact-length rejection and
  rejection before payload allocation or reads. Correct envelope panic and
  byte-ceiling documentation.
- Remove impossible growth conversion failures after bucket-capacity admission
  and when returning a successful previous-page count through `Memory::grow`.
  Preserve raw arithmetic overflow, bucket exhaustion, refusal/retry, reentry
  protection, accounting after successful growth and the upstream `-1` sentinel
  on actual errors.
- Add envelope coverage for oversized encoding, every truncated header length,
  `u64::MAX` declared lengths and mismatches in both directions. Extend the
  header-only stable-cell regression to cover `u32::MAX` and physical-capacity
  error precedence without payload reads, growth or writes. Extend growth tests
  to verify successful upstream previous-page returns, including full bucket
  capacity at each tested bucket size.
- Record the [follow-up audit](docs/audits/recurring/simplification-followup-0.18.0.md)
  and mark the previous findings as released in 0.18.0. Validate 265 library
  tests, integration and compile-fail tests, doctests, strict Clippy and Rustdoc,
  Rust 1.88 all-target checking and Wasm test compilation. Persisted formats,
  current fixtures, checksums and declaration fingerprints remain unchanged.
  Package verification and raw Wasm size gates were not repeated in this
  follow-up. Release-flow tests were not run because they create commits and
  tags reserved for the maintainer.

## 0.18.0

- Hard cut: remove `AllocationStageError::TooManyDeclarations`,
  `AllocationStageError::InvalidSchemaMetadata` and
  `AllocationStageError::GenerationOverflow`. Remove obsolete match arms or
  constructions. Declaration count and schema failures belong to snapshot
  validation; the finite committed-history limit remains an integrity error.
  These variants were unreachable with publicly obtained validated authority.
  Raw reservation and retirement overflow errors remain supported.
- Rely on immutable `ValidatedAllocations` facts during active staging. Remove
  repeated declaration count, schema and numeric-overflow checks. Preserve
  stale-proof rejection, receiver/output resource bounds, historical claim
  conflicts, retirement rejection and cloning before mutation. Document the
  proof's declaration and committed-history guarantees.
- Make private allocation-record construction and schema observation infallible
  after input validation. Remove the forwarding reservation observer and keep
  one schema-history update owner. Preserve reserved-to-active promotion,
  unchanged-schema suppression and last-seen updates. Public metadata
  constructors and raw reservation staging retain their validation boundaries.
- Replace tests that fabricate invalid schema or overflow proofs with decoded
  snapshot rejection and a real proof rejecting a `u64::MAX` receiver as stale.
  Add public-boundary coverage validating, staging, committing and recovering
  all 255 allocation slots. Strengthen the historical-conflict regression with
  a proof from a different valid ledger at the same generation and unchanged
  rejected receiver state.
- Record the [staging audit](docs/audits/recurring/simplification-followup-0.17.1.md)
  and mark the previous findings as released in 0.17.1. Validate 263 library
  tests, integration and compile-fail tests, doctests, strict Clippy and Rustdoc,
  Rust 1.88 all-target checking, Wasm test compilation and offline package
  verification. All five existing raw Wasm budgets pass; core is 249,456 bytes
  under its 260,000-byte ceiling. Persisted formats, diagnostic DTO shapes,
  checksums and declaration fingerprints are unchanged. Release-flow tests were
  not rerun because they create commits and tags reserved for the maintainer.

## 0.17.1

- Remove duplicate physical-generation state from `RecoveredLedger`. Recovery
  establishes physical/logical equality before constructing the proof; both
  public generation accessors now derive that value from the ledger. Preserve
  mismatch rejection, const accessors and constructor privacy. Derived `Debug`
  output no longer includes the redundant private field.
- Remove repeated empty-store checks after physical slot selection in explicit
  ledger initialization and physical commits. Only two absent slots permit
  genesis; corrupt or ambiguous slots still fail closed. Extend the recovery
  matrix to check initialization, exact rejection and unchanged stores for
  corrupt, unsupported, undecodable, mismatched and invalid-history records.
- Check runtime readiness once during authority adoption and verify requirements
  against the established host snapshot. Preserve fixed/logical distinctions,
  authority and metadata checks, error ordering and effect-free adoption without
  replaying host preparation.
- Remove inactive deserialization-only Serde attributes from the serialize-only
  derives on `PolicyIdentity` and `MemoryManagerRangeAuthority`. Their custom
  readers retain strict unknown-field, explicit optional-field and domain checks
  on the decoding DTOs.
- Record the [follow-up audit](docs/audits/recurring/simplification-followup-0.17.0.md)
  and mark the previous findings as released in 0.17.0. Qualify recovery/adoption
  changes with 262 library tests, integration and compile-fail tests, doctests,
  strict Clippy and Rustdoc, Rust 1.88 all-target checking, Wasm test compilation
  and offline package verification. All five raw Wasm budgets pass; core is
  249,892 bytes under its 260,000-byte ceiling. Recheck 34 focused tests and
  strict Clippy after the final Serde-attribute cleanup. Public API signatures,
  persisted formats, diagnostic DTO shapes, checksums and declaration
  fingerprints are unchanged. Release-flow tests were not rerun because they
  create commits and tags reserved for the maintainer.

## 0.17.0

- Hard cut: remove `RuntimeDiagnosticError::AllocationBound`,
  `RuntimeDiagnosticError::MemoryManagerSlot` and
  `RuntimeOpenError::MemoryManagerSlot`, plus the corresponding
  `From<MemoryManagerSlotError>` conversions into both runtime error types.
  Remove obsolete match arms or constructions; handle raw slot errors at the
  declaration or recovery boundary. These variants were unreachable through
  maintained runtime operations.
- Rely on sealed declaration bounds and validated slots during allocation
  reporting. Remove redundant count checks and their fallible binding lookup;
  preserve persisted manager-layout validation, live-manager agreement, reentry
  checks and bounded reads without writes or growth. Add coverage declaring
  every usable external ID and comparing detailed and numeric reports.
- Make the recovery result authoritative for doctor validation. Remove the
  redundant decoded-record argument and unreachable genesis branch. Preserve
  validation against genesis only for empty commit storage, borrowed successful
  recovery and typed rejection of unreadable, corrupt or unsupported records.
  Add coverage for both empty storage representations and a readable record
  containing a corrupt physical slot, with unchanged backing bytes.
- Remove unreachable slot-error conversions from committed ID lookup and host
  adoption. Preserve key, readiness, governance, authority, metadata and fixed-ID
  rejection. Extend the opening regression to check the exact
  `MemoryIdMismatch` for caller-supplied sentinel ID 255.
- Record the [simplification audit](docs/audits/recurring/simplification-followup-0.16.1.md).
  Validate 262 library tests, integration and compile-fail tests, doctests,
  strict Clippy and Rustdoc, Rust 1.88 all-target checking, Wasm test compilation
  and offline package verification. All five existing raw Wasm budgets pass;
  core is 249,906 bytes under its 260,000-byte ceiling. Persisted formats,
  diagnostic DTO shapes, checksums and declaration fingerprints are unchanged.
  Release-flow tests were not rerun because they create commits and tags reserved
  for the maintainer.

## 0.16.1

- Remove the sealed snapshot's duplicate authority map and private authority
  enum. Policy validation and host adoption share one lookup over canonical
  registered declarations. Preserve governance restrictions, external authority
  checks, custom-policy ordering and sealed declaration fingerprints.
- Stream recovered-ledger memory measurements into diagnostic export and doctor
  reports without collecting an intermediate vector. Keep public diagnostic
  DTOs and measurement behavior unchanged.
- Simplify ledger capacity reservation after the existing encoded-size limit
  establishes safe arithmetic bounds. Remove unreachable conversion/overflow
  branches and redundant saturating subtraction; preserve size rejection, typed
  growth failures, persistence ordering and retry behavior.
- Validate 260 library tests, integration and compile-fail tests, doctests,
  strict Clippy and Rustdoc, Rust 1.88 all-target checking, Wasm test compilation
  and offline package verification. All five existing raw Wasm budgets pass;
  core is 249,863 bytes under its 260,000-byte ceiling. Public API signatures,
  durable formats, checksums and declaration fingerprints are unchanged.
  Release-flow tests were not rerun because they create commits and tags reserved
  for the maintainer.

## 0.16.0

- Hard cut: `MemoryRuntime::memory_manager_config()` is no longer a `const fn`.
  Update enclosing const functions that call it to ordinary functions. Runtime
  configuration lookup reads the sole shared bucket geometry; returned values
  and persisted formats are unchanged.
- Keep backing memory, immutable bucket geometry and live bucket accounting in
  one shared growth state. Remove duplicate runtime fields while preserving
  cloned and detached handles, configuration checks and physical attribution.
- Confirm runtime persistence through `PendingBootstrapCommit::confirm_persisted`
  after the stable-cell write succeeds. Remove the private unpacking helper and
  duplicate generation selection before publishing allocation-open authority.
- Replace the derived generation-membership set with the range established by
  strict contiguous-history validation. Preserve structural checks and error
  ordering; add coverage rejecting allocation references to genesis generation.
- Share one runtime size-measurement flow over `RecoveredLedger`. Remove the
  unreachable per-slot failure branch and replace its fabricated-input test with
  persisted-corruption coverage for export, doctor and cold bootstrap. Public
  diagnostic DTO failure values remain supported; invalid persisted slots reject
  recovery before measurement and leave backing bytes unchanged.
- Share installed-toolchain validation between CI and Make through
  `make validate-toolchain`; read CI's MSRV from `Cargo.toml`. Correct runtime
  diagnostic guidance and record #9 as released in 0.15.7 and completed, with
  downstream adoption and qualification remaining consumer-owned.
- Validate 260 library tests, integration and compile-fail tests, doctests, six
  Wasm tooling tests, strict Clippy and Rustdoc, Rust 1.88 all-target checking,
  Wasm test compilation and offline package verification. All five existing raw
  Wasm budgets pass; core is 254,304 bytes under its 260,000-byte ceiling.
  Release-flow tests were not rerun because they create commits and tags reserved
  for the maintainer. Current wire fixtures, checksums and declaration
  fingerprints are unchanged.

## 0.15.7

- Return typed growth refusal from fresh runtime construction instead of allowing
  manager initialization to panic. Reserve the metadata page before writing;
  failed construction leaves zero pages and performs no reads or writes. Both
  constructors support retry with the same caller-owned backing. Configured
  default bootstrap propagates the construction error. Addresses Canic's feedback
  in [#9](https://github.com/dragginzgame/ic-memory/issues/9).
- Reject unknown fields inside durable retirement states. Enforce the serialized
  payload ceiling in both low-level physical commit entrypoints with
  `CommitRecoveryError::PayloadTooLarge` before slot mutation. Cover malformed
  nested records, exact byte limits, unchanged rejected stores and valid retries.
- Preserve decoder causes in ledger errors and doctor messages while retaining
  diagnostic codes. Correct declaration and reservation errors to report the
  actual 255-allocation limit.
- Return successful logical commit evidence from the checked ledger and physical
  commit, removing redundant checksum scans, decoding and integrity validation.
  Existing persisted bytes still pass the full fail-closed recovery boundary.
- Release declaration, request, range and hook inputs after registry sealing or
  terminal failure. Preserve immutable shared snapshots, cached errors and hook
  ordering through one sealing completion path.
- Count encoded ledger-record bytes during capacity admission without allocating
  a temporary serialization buffer. Share encoding with stable-cell persistence
  and verify measured lengths against current fixtures and the history boundary.
- Resolve Wasm artifacts through Cargo metadata so `CARGO_TARGET_DIR` and Cargo
  configuration cannot make budget checks read stale files. Add six tooling
  regressions for target discovery, oversized or missing artifacts and metadata
  failure; run them in CI and `make validate` through `make test-wasm-size`.
- Validate 259 library tests, integration and compile-fail tests, doctests, six
  Wasm tooling tests, strict Clippy and Rustdoc, Rust 1.88 all-target checking,
  Wasm test compilation and offline package verification. All five unchanged raw
  Wasm budgets pass; core is 254,648 bytes under its 260,000-byte ceiling.
  Current wire fixtures, checksums and declaration fingerprints are unchanged.

## 0.15.6

- Reuse protected slot-selection evidence in commit diagnostics, validating each
  present slot once instead of repeating payload checksum scans. Preserve
  fail-closed recovery and ambiguity classification; add coverage for valid ties,
  conflicting ties and corruption on either slot.
- Remove the unused public `AllocationValidationError::LedgerIntegrity` variant.
  Allocation validation requires `RecoveredLedger`; integrity failures are
  reported at the ledger recovery boundary.
- Share printable diagnostic-text validation across labels, runtime fingerprints,
  policy names, authorities and range purposes. Preserve the 256-byte ceiling,
  optional-field semantics, field-specific errors and rejection order. Add two
  public-API regressions for accepted boundaries and overlapping invalid inputs.
- Validate 249 library tests, integration and compile-fail tests, doctests,
  strict Clippy and Rustdoc, Rust 1.88 all-target compilation and Wasm test
  compilation. All five unchanged raw Wasm budgets pass; core is 259,455 bytes
  under its 260,000-byte ceiling. Persisted encoding, checksums and declaration
  fingerprints are unchanged.

## 0.15.5

- Fix configured default bootstrap holding a mutable TLS borrow while sealing
  declarations. Registration hooks can now observe the configured, unbootstrapped
  runtime without `ReentrantAccess`; construction and geometry failures still
  reject before sealing. Add a regression for hook readiness and summary reads.
- Reject excess elements in bounded sequences without a size hint before
  invoking their deserializer. Add a regression proving the rejected element is
  never decoded; existing definite-length CBOR bounds remain unchanged.
- Share automatic and explicitly numbered physical commits through one checked
  mutation path, avoiding repeated predecessor checksum scans. Preserve slot
  rotation, generation checks and rejection without mutation; add coverage for
  both entrypoints against a corrupt predecessor.
- Avoid unnecessary copies when filtering uniquely owned allocation authority
  and validating recovered history in doctor diagnostics. Remove unused private
  codec scaffolding, redundant capability markers and duplicate empty-key checks;
  share FNV hashing and test fixture decoding. Current wire fixtures, checksum
  bytes and declaration fingerprints remain unchanged.
- Refresh current guides, rustdoc and whitepaper coverage for admission, logical
  placement, configured bootstrap, typed growth, adoption and memory accounting.
  Mark archived designs and measurements as historical, and record the
  [code hygiene audit](docs/audits/recurring/code-hygiene-report-2026-10-03.md).
- Validate 248 library tests, integration and compile-fail tests, doctests,
  strict Clippy and Rustdoc, Rust 1.88 all-target compilation, Wasm test
  compilation and all five unchanged raw Wasm budgets. Public API signatures
  and persisted encoding are unchanged.

## 0.15.4

- Add the runnable [composed-host example](examples/composed_host.rs) requested
  by Canic. Two cold reopens over the same backing retain fixed/logical IDs,
  authority and stored data with unchanged declarations and 16-page buckets.
  Cold attempts run host and consumer admission; warm adoption preserves the
  commitment without replaying admission. Typed consumer rejection leaves the
  existing backing unchanged and publishes no capability.
- Enable two public-API regressions in the ordinary test suite, covering cold
  reopens and configured host bootstrap on every native worker before consumer
  adoption or thread-local store initialization. Document that initialization
  order in the README and admission contract.
- Close implemented GitHub requests #2–#8 with released implementation and
  downstream adoption evidence. Update the [issue reconciliation](docs/issue-reconciliation.md)
  and codec qualification to record IcyDB's published 0.15.3 acceptance under
  its unchanged lifecycle instruction ceiling.
- Validate the full test suite, strict all-target Clippy, Wasm test compilation,
  all five existing raw Wasm size budgets and Rust 1.88 all-target checks.
  Runtime APIs and persisted encoding are unchanged. IC instruction/cycle and
  matched consumer Wasm deltas for the new regressions are unmeasured; Canic's
  participant/store-restoration qualification remains consumer-owned.

## 0.15.3

- Make default export, commit-recovery and both doctor diagnostic helpers
  nonconstructing. An absent runtime returns `RuntimeDiagnosticError::NotBootstrapped`
  without initializing memory or choosing 128-page buckets, preserving later
  configured bootstrap. Doctor inspection also leaves declarations unsealed
  when no runtime exists. Existing runtimes retain prebootstrap recovery/doctor
  inspection and typed construction/TLS errors. Addresses Canic's 0.15.2 feedback.

## 0.15.2

- Keep unsupported-format diagnostic coverage on `DiagnosticCode` rather than
  human-readable message wording.
- Replace narrow registration-hook and raw-read lint allowances with justified
  expectations so stale exceptions are reported. Preserve the existing unsafe
  read contract and test coverage. Addresses IcyDB feedback in
  [#8](https://github.com/dragginzgame/ic-memory/issues/8).
- Use the target's maximum `usize` in the declaration-count rejection test so
  its input does not overflow on 32-bit Wasm.
- Give the recovered-allocation iterator an explicit `must_use` reason to
  satisfy both Rust 1.88 and Rust 1.99 Clippy without lint allowances.
- Extract pre-bootstrap observation assertions into one test helper, reducing
  the configured-runtime test's cognitive complexity for Rust 1.88 Clippy
  while preserving fresh-thread isolation and assertion order.

## 0.15.1

- Pin development and primary CI validation to Rust 1.99.0. Read the compiler
  pin from `rust-toolchain.toml` in CI and Make to prevent validation drift;
  retain the declared Rust 1.88.0 MSRV.
- Address Rust 1.99 Clippy diagnostics by removing a redundant iterator
  `must_use` attribute and showing unexpected ledger contents in empty-value
  assertions.
- Share raw Wasm size enforcement between CI and Make and remove duplicate
  macro-test and doctest runs already covered by the serialized full suite.
- Matched-source, matched-lockfile Rust 1.99.0 builds add 2,924–3,383 raw Wasm
  bytes over Rust 1.97.1. Rebaseline only the admission budget from 260,000 to
  264,000 bytes; all other budgets and the `wasm-size` profile remain unchanged.
  See the [compiler comparison](docs/measurements/rust-1.99-toolchain.csv).

## 0.15.0

- Make default-runtime opens nonconstructing. An early open returns
  `NotBootstrapped` without initializing a manager or selecting its bucket size;
  cached construction and TLS access failures remain typed errors.
- Protect application and ledger growth through `RuntimeMemory`. Reserve
  physical backing capacity before assigning buckets. Hard-cut direct `grow`
  calls to `Result<u64, RuntimeGrowError>`; distinguish ordinary backing refusal,
  arithmetic overflow, reentrant growth and bucket exhaustion. Only the required
  upstream `Memory` trait adapter translates failures to `-1`. Propagate ledger
  capacity failures through `RuntimeBootstrapError::LedgerGrowth`; retain
  `StableCellLedgerWriteTooLarge` for the encoded record ceiling.
  Refusal preserves virtual extents and manager metadata and supports retry.
  All handles share one transient assigned-bucket count seeded on construction;
  remove the ledger's separate capacity preflight and metadata scan. The current
  durable format, policy and default bucket size are unchanged.
- Add `MemoryAllocationSummary`, `MemoryBindingSummary`,
  `MemoryRuntime::memory_allocation_summary` and the nonconstructing default
  helper. Numeric totals and current/ledger/unknown binding partitions share
  accounting with detailed attribution, retain the 34,848-byte metadata-read
  bound, and avoid per-ID rows and copied binding names. Payload occupancy
  remains unavailable.
- Add committed ID resolution and authority-scoped adoption verification for
  owned/default runtimes, plus `RuntimeAdoptionError`. Check fixed and logical
  requirements with typed missing-key, wrong-ID, current-authority and metadata
  errors without replaying admission or changing host configuration. Document
  the immutable declaration invariants of `CommittedAllocations` and update the
  composed-host example.
- Qualify refusal/retry, cloned and detached handles, interleaved allocations,
  reopen, reentry, bounded summary parity, and effect-free host adoption. Reject
  the superseded integer-returning growth API with a compile-fail test.
  Matched Rust 1.97.1 raw Wasm deltas against 0.14.3 are +283 bytes core,
  +934 diagnostics, +315 key-only and +215 admission; all existing budgets remain
  unchanged and pass. Add an integration probe exercising growth, summary
  serialization and adoption: 263,847 bytes under a new 270,000-byte budget.
  IC instruction/cycle costs and downstream lifecycle qualification are
  unmeasured. See the [raw size measurements](docs/measurements/0.15-runtime-integration.csv).

## 0.14.3

- Encode opaque committed payloads as bounded CBOR byte strings, removing the
  outer codec's per-byte integer encoding and syntax walk. The existing CBOR
  preflight checks the 16 MiB + 24-byte payload ceiling before deserialization;
  the stable-cell record ceiling is now 32 MiB + 4 KiB.
- This is a pre-1.0 persisted-format hard cut. Recreate earlier data; format
  version remains 1, with no compatibility reader or migration bridge.
  Human-readable DTO serialization continues to round-trip byte arrays.
- Replace current wire fixtures and qualify maximum-size writer/reader parity,
  malformed payload rejection, and capacity-refusal/retry behavior. Recovery,
  checksums, generation history and the persistence/publication boundary remain
  unchanged. All four raw Wasm probes shrink by 346–521 bytes; IcyDB's
  maintained lifecycle fixture passes with a worst phase of 5,133,140 instructions
  under its unchanged 12,750,000 ceiling using a local dependency override.
  Released-dependency adoption remains pending. See
  [codec qualification](docs/opaque-ledger-payloads.md).

## 0.14.2

- Removed the intermediate sealed snapshot from historical admission completion.
  The resolver consumes known-only selections alongside the original canonical
  requests, then builds and validates one final resolved snapshot. Original
  warm-bootstrap identity and resolved diagnostic fingerprints remain distinct.
- Reused validated request authority/key values when applying recovered schema
  metadata, preserving early bounds, current-grant checks and sticky rejection.
  Switched only unique-key request ordering to unstable sorting; fixed declaration
  and range comparators retain their existing ordering behavior.
- Added canonical-permutation, duplicate-rejection and resolved-fingerprint
  equivalence coverage. No API, durable format, lifecycle state, persistence
  boundary or recovery preflight changes were introduced.
- Recorded mixed matched Rust 1.97.1 raw Wasm results: admission decreases by
  159 bytes; core increases by 339, diagnostics by 718 and key-only by 331 bytes.
  All existing size budgets pass. Historical completion eliminates one snapshot
  build/fingerprint; IC instructions/cycles remain unmeasured. Further range-table
  sharing was deferred after its measured code-size increase. See the
  [cleanup qualification report](docs/logical-bootstrap-cleanup.md).

## 0.14.1

- Added `RuntimeBootstrapPolicy::prepare_bootstrap` inside the existing recovered
  ledger bootstrap flow. Hosts can compose consumer identity admission and
  explicitly complete historical declarations before resolution, final policy
  validation and the single persistence/publication boundary.
- Added bounded `BootstrapAdmission` metadata and known-only historical
  selection. Unknown, retired, duplicate, unauthorized or oversized selections
  reject the whole attempt, including when a callback ignores a selection error.
  Selected allocations retain their durable ID and latest schema metadata.
- Kept warm bootstrap and host adoption free of admission replay. Failed cold
  attempts can retry against unchanged committed mappings; default and owned
  runtimes use the same hook and preserve host policy and bucket configuration.
  No durable format or allocation-open authority changes were introduced.
- Added an IcyDB-shaped recovered-journal example, production runtime and
  capability-boundary tests, and the [admission contract](docs/recovered-admission.md).
  This addresses #5's allocation-level ordering gap. Actual generated IcyDB
  adoption, identity semantics, journal-debt and pending-commit qualification
  remain downstream work.
- Matched Rust 1.97.1 raw Wasm probes increase core by 648 bytes, diagnostics by
  342 bytes and key-only by 579 bytes. The new admission-enabled probe is
  258,960 bytes and is enforced under a 260,000-byte budget; existing probe
  budgets are unchanged. IC instruction/cycle costs remain unmeasured.

## 0.14.0

This release adds key-only allocation and bounded ledger recovery. It is an
intentional pre-1.0 admission hard cut: recovery rejects inputs outside the new
byte, collection and nesting limits, including indefinite-length CBOR. The
durable version-1 ledger shape remains unchanged; no legacy reader, automatic
history compaction or migration path is provided.

- Added `MemoryRequest`, static registration and key-only declaration/open
  macro forms, plus explicitly owned `SealedDeclarationSnapshot::new` inputs.
  After recovery, known keys retain their committed IDs; new requests resolve
  in stable-key order to the lowest free ID in an explicit host-owned `Allowed`
  grant. Fixed claims, governance slots, reservations, omitted allocations and
  retired slots remain unavailable to new keys.
- Added `MemoryRuntime::open_memory_by_key` and
  `open_default_memory_manager_memory_by_key`. Libraries can inspect committed
  assignments and adopt a bootstrapped host without replacing its policy or
  bucket configuration. Both runtime forms persist the complete resolved set
  before publishing allocation-open authority.
- Bounded logical ledger payloads to 16 MiB, stable-cell ledger values to
  64 MiB + 4 KiB, CBOR nesting to 32, allocation records to 255, and generation
  history and total schema history to 65,536 entries each. Added pre-allocation
  length checks, an allocation-free CBOR preflight, bounded collection decoding,
  and writer/staging checks with typed, fail-closed errors. Recreated runtimes
  still append a generation for unchanged declarations; history exhaustion is
  explicit and does not discard ownership or tombstones.
- Hardened ledger persistence against backing-memory growth refusal by reserving
  physical capacity before upstream manager bucket assignment. Failed bootstrap
  publishes no mapping; IC trap rollback remains the interrupted-write boundary.
- Clarified and tested omitted-store access through explicit reconciliation
  declarations supplied before sealing. Omitted keys cannot open through current
  authority, and revoked grants or explicit retirement reject redeclaration.
  **IcyDB integration for #4 remains unresolved:** its generated bootstrap must
  establish whether removed journal keys are available before sealing, then
  qualify journal-debt and pending-commit checks end to end. No unrestricted
  historical-open capability was added.
- Added a runnable standalone/composed-host example and the
  [recovery and integration qualification report](docs/key-only-recovery.md),
  covering deterministic placement, reservation activation, failed persistence,
  omitted/retired ownership, hostile decoding and history/record boundaries.
- Matched Rust 1.97.1 raw Wasm probes increased core from 240,300 to 255,112 bytes
  and diagnostics from 289,090 to 307,589 bytes. The equivalent key-only probe is
  254,762 bytes. Updated enforced budgets to 260,000 bytes for core/key-only and
  315,000 for diagnostics. Identical resolved declarations add no durable
  metadata fields; IC instruction/cycle measurements remain unavailable.

## 0.13.3

- Made default-runtime bootstrap status and committed-capability lookups
  nonconstructing. Missing runtimes return `false` / `NotBootstrapped` without
  initializing memory or selecting 128-page buckets; cached construction and
  TLS access failures remain typed errors.
- Exposed the existing built-in policy as `GenericRangePolicy` for use with
  configured bootstrap. Range enforcement, policy identity, host-owned runtime
  adoption, the durable format and the 128-page upstream default are unchanged.
- Added focused coverage for observation before configured bootstrap, repeated
  initialization, exact bucket matching and custom host-policy preservation.
- Matched Rust 1.98.1 raw Wasm probes using the same dependency lockfile leave
  core size unchanged at 240,387 bytes and reduce diagnostics from 289,422 to
  289,389 bytes. IC instruction/cycle deltas remain unmeasured. No database
  recreation is required by this change.

## 0.13.2

- Removed the redundant private backing adapter in favor of upstream's
  `Memory` implementation for `Rc<M>`, and forwarded `RuntimeMemory::read_unsafe`
  to the existing virtual memory. This removes wrapper-level destination
  zeroing while preserving default implementations for custom backings.
  Unsafe code remains denied by default, with scoped exceptions for the
  forwarding method and its raw-read tests.
- Added focused coverage for uninitialized destinations, specialized and
  default backing reads, discontiguous buckets, cloned handles, partial read
  failures, zero-length reads, upstream bounds behavior, and read-only effects.
- Updated the README to document upstream read delegation. IC instruction
  and cycle savings remain unmeasured.

## 0.13.1

- Re-exported the exact upstream substrate dependency as
  `ic_memory::ic_stable_structures`, making collections, backing memories, and
  traits available through the same dependency as `RuntimeMemory<M>`.
- Updated the README and advanced guide to use the re-export and remove the
  requirement for a separate direct `ic-stable-structures` dependency.
- Explicitly selected the CI validation and MSRV toolchains, and declared the
  development Wasm target, fixing target-installation mismatches caused by the
  repository toolchain override.
- Kept runtime ownership, allocation policy, bucket defaults, and the durable
  format unchanged.

## 0.13.0

This release adds bounded physical allocation attribution and explicit bucket
configuration to the owned memory runtime. It is an intentional pre-1.0 API
hard cut: runtime memory handles change type, while the durable allocation-ledger
format and the default 128-page bucket size remain unchanged.

### Runtime memory handles

- Changed runtime opens and memory macros to return `RuntimeMemory<M>`, which
  implements `Memory` and `Clone` without requiring a cloneable backing memory.
  Update stable-store annotations from `VirtualMemory<DefaultMemoryImpl>` to
  `ic_memory::RuntimeMemory<DefaultMemoryImpl>` directly.
- Retained one manager per runtime, with private shared backing access for
  attribution. Diagnostics do not grant allocation-open authority.

### Bounded allocation diagnostics

- Added `MemoryRuntime::memory_allocations()` and
  `default_memory_manager_memory_allocations()`, returning owned reports for
  all 255 usable IDs, including zero-size memories and the ic-memory ledger.
- Reported the actual persisted bucket size, physical and virtual extents,
  per-ID bucket allocation, current stable-key/owner bindings, manager metadata,
  and separate unknown-binding and unmanaged residuals with checkable totals.
- Distinguished bucket rounding slack from virtual extent and left payload
  occupancy explicitly unavailable. Retired or absent current keys remain
  unknown without omitting their physical allocations.
- Bounded successful collection to 34,848 metadata bytes without reading or
  decoding ledger history, initializing stores, writing, growing memory, or
  advancing a generation. The default helper does not construct a missing
  runtime.
- Added a validated read-only manager-layout adapter and pinned
  `ic-stable-structures` to exactly 0.7.2. Unsupported or corrupt metadata
  returns typed errors before manager initialization can write.

### Bucket configuration

- Added immutable `MemoryManagerConfig`, `MemoryRuntime::new_with_config`, and
  `bootstrap_default_memory_manager_with_config` for nonzero bucket sizes.
- Kept the fresh-state default at 128 pages (8 MiB). Ordinary construction
  honors the persisted setting; explicit configuration rejects mismatches
  before effects, including repeated default-runtime bootstrap.
- Bound configuration to the runtime's sole manager. Changing a requested
  setting does not shrink existing memory or introduce a migration path.

### Measurements and validation

- Added disposable small-store and growing-store measurements comparing 128-,
  16-, 8-, and 1-page buckets, with finite-table capacity and growth/access cost
  accounting. Evidence supports opt-in sizing while retaining the default.
- Added conservation, bucket-boundary, corrupt-metadata, access-separation,
  no-write/no-growth, capacity-exhaustion, and same-release recovery/replay
  coverage, including borrowed non-Clone backing memory.
- Added the [CANIC-162 integration handoff](docs/canic162-memory-attribution.md)
  with reproducible measurements and downstream adoption examples. Live Toko
  attribution and Canic adoption remain separate outstanding work.
- Included allocation-report serialization in the diagnostics Wasm probe.
  Core and diagnostics remain within their existing raw Wasm budgets at
  240,226 and 289,044 bytes respectively on Rust 1.97.1.
- Updated trybuild to 1.0.121 and raised the declared MSRV and its CI check to
  Rust 1.88.0.

### Release tooling

- Added Canic-style `make release-patch` and `make release-minor` flows that
  validate committed source, synchronize manifest/README versions, commit,
  create an annotated tag, and atomically push the branch and release tag.
- Added separate `make publish` and `make publish-dry-run` commands with clean
  release-commit and tag checks. Local preparation and stage/commit/push steps
  remain available individually for review and retry.
- Added release-flow tests using disposable Git remotes and a fake Cargo,
  including failed-validation/package rollback, staged-change rejection,
  remote conflicts, atomic push rejection, and publication dry runs.

## 0.12.3

This release hardens runtime policy identity and makes doctor diagnostics
policy-aware. It is an intentional pre-1.0 API and diagnostic-shape hard cut
and does not change the durable allocation-ledger format.

### Policy and bootstrap binding

- Replaced the unbounded `&'static str` policy identity with validated
  `PolicyIdentity`, containing a bounded policy-family name, nonzero semantic
  version, and optional caller-computed 32-byte configuration digest.
- Made identity construction fallible and revalidated diagnostic
  deserialization so malformed input cannot bypass the newtype invariants.
- Bound repeated bootstrap to the complete identity, including configured
  policy digest, while keeping the binding explicitly in-memory rather than
  durable upgrade history.
- Added a deterministic, versioned, non-cryptographic
  `SealedDeclarationFingerprint` for diagnostic comparison without replacing
  sealed-snapshot identity as bootstrap authority.

### Policy-aware diagnostics

- Changed `MemoryRuntime::doctor_report` to accept the policy it evaluates and
  added `default_memory_manager_doctor_report_with_policy` for custom-policy
  default runtimes.
- Included the tested policy identity and declaration fingerprint, the binding
  established by successful bootstrap, and a typed binding comparison in
  `MemoryRuntimeDoctorReport`.
- Added distinct diagnostic codes for invalid policy identity, runtime-binding
  mismatch, and per-slot memory-size failure.
- Changed doctor ledger size projection to preserve successful measurements
  when another allocation's slot is invalid, using
  `DiagnosticMemorySizeOutcome` per record.

### Size budgets

- Split the representative raw Wasm gate into a 245,000-byte bootstrap/open
  core tier and a 290,000-byte doctor/export diagnostics tier.
- Measured 239,150 and 281,858 bytes respectively on Rust 1.97.1.

## 0.12.2

This release makes runtime construction fail closed before
`ic-stable-structures` can initialize over unrecognized nonempty backing
memory. It is an intentional pre-1.0 API hard cut and does not change the
durable allocation-ledger format.

### Backing-memory construction safety

- Changed `MemoryRuntime::new(memory)` to return
  `Result<MemoryRuntime<M>, RuntimeConstructionError>`.
- Added raw backing-memory preflight for the pinned `MemoryManager` magic and
  layout version. Empty memory remains initializable; nonempty foreign or
  unsupported memory is rejected without mutation.
- Propagated default TLS construction failures through the existing typed
  runtime-state path without adding panic, fallback, reset, or compatibility
  behavior.
- Added byte-for-byte negative tests for foreign memory and unsupported
  `MemoryManager` versions, plus positive empty-memory and current-layout
  recovery coverage.

## 0.12.1

This release cleans up and hardens the explicit runtime architecture introduced
in 0.12.0. It makes one intentional pre-1.0 policy-trait hard cut and does not
change the durable allocation-ledger format.

### Runtime implementation structure

- Split the runtime implementation into focused core, policy, diagnostics,
  error, default TLS, and test modules.
- Kept `MemoryRuntime<M>` as the single owner of each backing memory's runtime
  state and kept the default API as thin entry points into one TLS runtime.
- Moved declaration-registry and runtime unit tests out of production
  implementation files so ownership and sealing paths are easier to review.

### Bootstrap and registration hardening

- Added `RuntimeBootstrapPolicy` with an explicit semantic identity. Repeated
  bootstrap is idempotent only when both that identity and the sealed
  declaration snapshot match the successful bootstrap; mismatches return typed
  errors without touching the ledger.
- Moved deferred constructor-registration failures into the declaration
  registry lifecycle, so open, sealing, sealed, and failed state have one
  canonical owner. The first sealing failure remains deterministic, and an
  impossible internal transition has a distinct typed error.
- Removed a redundant final registry lock during snapshot construction and
  lifecycle publication.
- Made the committed-capability compile-fail boundary independent of changing
  rustc missing-item wording.

### Regression coverage

- Added a downstream-style integration test for the default runtime with a
  custom `RuntimeBootstrapPolicy`, including repeated-bootstrap identity and
  generation checks.
- Pinned primary development and CI validation to Rust 1.97.1 while retaining
  the Rust 1.85.0 MSRV check.
- Made CI run the exact two-libtest-thread regression with
  `--test-threads=1`, and run the full test suite in serialized libtest mode.
- Added a representative stripped, uncompressed Wasm bootstrap/open probe with
  a 240,000-byte release budget.

## 0.12.0

This release is an intentional pre-1.0 runtime API hard cut. It removes the
split process-global/thread-local default runtime architecture without adding
aliases, reset hooks, compatibility forwarders, or fallback state.

### Explicit runtime ownership

- Added `MemoryRuntime<M>` as the canonical owner of one backing memory's
  `MemoryManager`, allocation-ledger cell, bootstrap lifecycle, committed
  allocation capability, opens, diagnostics, and live memory sizes.
- Added an explicit `Unbootstrapped` / `Bootstrapped { committed_allocations }`
  lifecycle. Capability publication occurs only after that runtime's
  stable-cell write succeeds, and failed bootstrap leaves the runtime
  unbootstrapped.
- Made repeated bootstrap on the same runtime object idempotent without
  advancing its ledger generation.
- Added `SealedDeclarationSnapshot`, an immutable canonical process-wide view
  of linked declarations, range authority, and policy metadata. Declaration
  sealing is independent from every concrete memory bootstrap.

### Default TLS runtime hard cut

- Replaced the separate TLS memory manager and ledger cell plus process-global
  bootstrap flag and committed capability with one thread-local
  `MemoryRuntime<DefaultMemoryImpl>`.
- Removed all process-global memory-runtime lifecycle/capability state and
  removed runtime reset support. Native libtest threads now bootstrap their own
  default memory; single-threaded IC Wasm retains canister-instance behavior.
- Changed default TLS entry to use fallible `RefCell` access and added typed
  runtime reentrancy/unavailability errors.
- Generalized `DefaultMemoryManagerDoctorReport` to
  `MemoryRuntimeDoctorReport`, and made the default doctor wrapper return a
  typed diagnostic result.
- Changed `is_default_memory_manager_bootstrapped()` to return a typed result
  and changed `ic_memory_key!` to return the typed memory-open result instead of
  panicking internally.

### Atomic declaration sealing

- Moved generated declaration/range registration work into the fallible seal
  lifecycle, before eager declaration hooks and final validation.
- Canonically sorted declarations and ranges before duplicate detection and
  snapshot publication, making snapshot meaning and declaration bytes
  independent of constructor order.
- Serialized concurrent snapshot requests and made them share one immutable
  snapshot. Late registration, duplicate declarations, recursive sealing, hook
  panic, and mutex poisoning remain distinct typed failures.
- Removed public collection and separately assembled snapshot functions that
  could expose unsealed registry views. Integrations now inspect
  `sealed_declaration_snapshot()`.

### Validation and format

- Added the exact two-libtest-thread regression, independent
  `MemoryRuntime<VectorMemory>` isolation/recovery tests, concurrent snapshot
  and runtime bootstrap tests, typed negative opens, failure publication tests,
  idempotence checks, and runtime-local diagnostics.
- Kept the stable-cell, protected commit-store, payload-envelope, allocation
  ledger, fixture, format marker, and format version bytes unchanged.
- Updated README, advanced and safety guidance, whitepaper operations, and
  rustdoc to distinguish linked declaration authority from concrete runtime
  ownership and to document bootstrap once per memory runtime.

## 0.11.1

This release tightens repository hygiene around the pre-1.0 hard-cut policy. It
does not change the current runtime API or durable format.

### Compatibility hygiene

- Audited the active API and decoder surface for deprecated forwarders,
  compatibility aliases, legacy modules, serde aliases/defaults, fallback
  readers, and migration shims. None remain.
- Removed test-only encodings of superseded envelope, record-field, and macro
  forms. The active suite now exercises only the current format and current
  authority boundaries.
- Removed current fixture and safety documentation that described superseded
  wire shapes; historical release notes remain the only record of them.

## 0.11.0

This is an intentional current-format and diagnostic-API hard cut. No legacy
decoder, compatibility shim, serde alias, or migration path was added.

### Durable format classification

- Added an explicit current ledger-format marker and version inside the
  protected payload envelope. Recognized pre-0.11 `ic-memory` payloads without
  the current discriminator now return typed
  `LedgerPayloadEnvelopeError::UnsupportedFormat` instead of falling through
  to a generic ledger decode or length error.
- Replaced the current golden fixtures in place and recomputed protected-slot
  checksums for the new envelope. Superseded fixtures and decoders are not
  retained.

### Machine-readable diagnostics

- Added stable machine-readable `DiagnosticCode` values alongside human
  messages in stable-cell, range-authority, and validation diagnostics. The
  doctor report no longer requires tooling to classify these failures by
  parsing prose.
- Added `DiagnosticFailure` and changed string-valued diagnostic errors to
  carry both a stable code and an operator-facing message.

### Authority ergonomics

- Allowed explicit macro authority arguments to use a shared compile-time
  string constant, reducing repeated-literal drift without restoring implicit
  ownership.

### WebAssembly size

- Measured equivalent `serde_cbor` and `ciborium` encode/decode Wasm probes;
  the raw optimized `ciborium` artifact was 55,288 bytes (41.19%) smaller. It
  remained 48,099 bytes (40.88%) smaller after `wasm-opt -Oz`.
- Confirmed that `crunchy` is present only in the all-target lockfile
  resolution and is not linked into the normal `wasm32-unknown-unknown`
  dependency graph. The complete method is recorded under `docs/audits/`.

## 0.10.0

This is an intentional current-format hard cut. No fallback decoder, missing-
field default, compatibility alias, or migration shim was added.

### Recovery hardening

- Fail closed when either present physical commit slot has an invalid marker or
  checksum. Recovery no longer falls back to an older generation, which could
  otherwise forget a newer allocation, retirement, or schema-history fact.
- Keep deterministic recovery for identical duplicate slots and select the
  highest generation only after every present slot validates.
- Add regressions proving a corrupt latest generation cannot roll allocation
  history back and a corrupt inactive slot cannot be silently overwritten.

### Decode and policy hardening

- Require every non-elided optional field in current durable CBOR records to be
  present. Explicit CBOR `null` remains the encoding of `None`; omission now
  fails closed instead of being interpreted as empty state.
- Apply caller-supplied default-runtime policy only to external declarations.
  The private `ic_memory.ledger.v1` allocation remains governed exclusively by
  ic-memory's internal namespace and range policy.

### State-model hard cut

- Changed `AllocationState::Retired` to
  `AllocationState::Retired { generation }` and removed the separate nullable
  `AllocationRecord::retired_generation` field and accessor. A retired record
  without a generation, or a live record with retirement metadata, is now
  unrepresentable.
- Removed the obsolete `MissingRetiredGeneration` and
  `UnexpectedRetiredGeneration` integrity errors.
- Replaced nullable diagnostic field pairs with explicit states:
  `CommitSlotDiagnostic` is now `Empty`, `Valid`, or `Invalid`;
  `CommitStoreDiagnostic::recovery` and
  `DiagnosticRangeAuthority::effective_authority` are `Result` values;
  corrupt stable-cell errors live in `DiagnosticStableCellStatus::Corrupt`;
  and `DiagnosticCheck` is now an enum carrying failure/not-run messages.
- Removed `DiagnosticCheckStatus`; `DiagnosticCheck` itself is the status.
- Replaced the current fixture set in place. Earlier pre-1.0 allocation-record
  and diagnostic shapes are intentionally rejected; there is no legacy decoder
  or in-crate migration path.

### Release checks and documentation

- Add WebAssembly test-target compilation to CI.
- Fix packaged README image and documentation links, and align recovery and
  capability wording across the safety guide and whitepaper.
- Keep stable keys and memory IDs unchanged while intentionally replacing the
  pre-0.10.0 allocation-ledger encoding.

## 0.9.0

This is an intentional hard-cut release. Removed APIs have no deprecated
forwarders, compatibility aliases, or legacy modules.

### Reduced public surface

- Removed the unused generic `AllocationSession`, `AllocationSessionError`,
  `StorageSubstrate`, and `LedgerAnchor` APIs. Persistence owners now use the
  opaque `CommittedAllocations` capability directly when authorizing their own
  storage-open path.
- Stopped exporting implementation-only physical recovery machinery:
  `ProtectedGenerationSlot`, `DualProtectedCommitStore`, `CommitSlotIndex`,
  `AuthoritativeSlot`, and `select_authoritative_slot`.
- Kept the concrete `DualCommitStore`, committed-generation DTOs, recovery
  errors, and diagnostics public for current stable-cell integrations.
- Narrowed `CommitStoreDiagnostic::from_store` to the concrete
  `DualCommitStore` instead of an extension trait.
- Added a repository-wide pre-1.0 hard-cut rule and removed redundant serde
  defaults from optional diagnostic fields so no annotation resembles an
  earlier-shape compatibility path.

### Durability clarification

- Clarified that both redundant commit slots are serialized inside one
  `ic-stable-structures::Cell` in the default runtime. ICP message execution
  provides atomic stable-memory commit and rollback; the embedded slot
  checksums provide fallback only from localized corruption when the enclosing
  record remains decodable.
- Preserved the 0.8 durable ledger, stable-cell, payload-envelope, and CBOR
  formats. Upgrading from 0.8 requires no stable-memory migration.

## 0.8.1

### Decode and ingestion hardening

- Reject trailing bytes after persisted CBOR ledger payloads and stable-cell
  ledger records instead of accepting a valid value prefix.
- Revalidate decoded `AllocationDeclaration` values when they enter the static
  declaration registry.
- Revalidate decoded `MemoryManagerAuthorityRecord` values when they enter the
  static range registry, and expose an explicit `validate()` method for other
  ingestion boundaries.
- Revalidate `AllocationRetirement` values before staging a retirement
  generation, and expose an explicit `validate()` method for decoded requests.

## 0.8.0

This is an intentional hard-cut release. It does not retain deprecated shims,
legacy macro forms, compatibility aliases, or renamed API forwarding methods.

### Breaking authority model

- Split allocation state into two distinct opaque types:
  - `ValidatedAllocations` is pre-commit validation state and cannot open
    storage. Its generation accessor is now `base_generation()`.
  - `CommittedAllocations` is the post-persistence capability accepted by
    `AllocationSession` and the default runtime's open path.
- Replaced `BootstrapCommit` with `PendingBootstrapCommit`. Generic persistence
  owners must durably write the mutated ledger record before calling
  `confirm_persisted()` to obtain `CommittedAllocations`.
- Removed `validated_allocations()`. The default runtime now exposes only
  `committed_allocations()`, published after its stable-cell write succeeds.
- Changed `AllocationSession::new` to require `CommittedAllocations` and renamed
  its state accessor from `validated()` to `committed()`.
- Renamed `RuntimeOpenError::StableKeyNotValidated` to
  `StableKeyNotCommitted`. `MemoryIdMismatch::validated_id` is now
  `committed_id`.

### Explicit authority registration

- Removed every implicit-authority form of `ic_memory_range!`,
  `ic_memory_declaration!`, and `ic_memory_key!`. Range and key declarations now
  require the same explicit stable `authority = "..."` value.
- Rejected all external `ic_memory.*` declarations and external attempts to use
  the reserved `ic-memory` authority identity.
- Replaced caller-controlled internal authority strings with private runtime
  provenance.
- Renamed `declaring_crate` fields and accessors to `authority` on static and
  diagnostic declaration APIs. No serde field alias is retained.
- Changed `StaticMemoryDeclaration::new` and
  `StaticMemoryRangeDeclaration::new` to return validation errors.

Before:

```rust,ignore
ic_memory::ic_memory_range!(start = 120, end = 129);
ic_memory::ic_memory_key!("app.users.v1", UsersStore, 120);
```

After:

```rust,ignore
ic_memory::ic_memory_range!(
    authority = "app",
    start = 120,
    end = 129,
);

ic_memory::ic_memory_key!(
    authority = "app",
    key = "app.users.v1",
    ty = UsersStore,
    id = 120,
);
```

### Manual bootstrap migration

Manual persistence owners must now make persistence confirmation explicit:

```rust,ignore
let pending = AllocationBootstrap::new(record.store_mut())
    .initialize_validate_and_commit(&genesis, declarations, &policy, committed_at)?;

persist_record(&record)?;
let committed = pending.confirm_persisted();
let session = AllocationSession::new(storage, committed);
```

Calling `confirm_persisted()` before the owning record is durably written
violates the protocol.

### Validation and storage hardening

- Validated reservation DTO invariants before invoking caller policy.
- Made `MemoryManagerRangeAuthority` deserialization validate every imported
  record and reject overlaps.
- Tightened committed ledger chronology: retirement must follow the final
  observation, schema history must begin at allocation creation, and schema
  changes cannot postdate the final observation.
- Validated retirement slot descriptors in `AllocationRetirement::new`.
- Removed all `ic_memory.*` governance allocations from published application
  capabilities while preserving them in durable ledger recovery state.

### Dependencies and packaging

- Replaced the unmaintained `serde_cbor` dependency with maintained `ciborium`.
  Current CBOR fixtures remain byte-for-byte stable without a compatibility
  decoder or legacy format path.
- Removed the `stable_structures` dependency re-export. Downstream code now
  imports `ic-stable-structures` directly.
- Reduced the published package from roughly 2.1 MiB compressed to about
  203 KiB by excluding an unused large decorative image.

---

## 0.7.5

### Runtime hardening

- Made default-runtime bootstrap return `RuntimeLockPoisoned` if the deferred
  eager-init hook queue lock is poisoned, instead of panicking.
- Made the default doctor report surface eager-init hook lock failures as a
  failed diagnostic check instead of panicking.
- Removed reachable production `expect(...)` paths from default ledger-cell
  initialization, stable-cell capacity sizing, and ledger schema-history
  staging.
- Added fallible `LedgerPayloadEnvelope::try_encode()` and used it from ledger
  commit paths so envelope length failures return `LedgerCommitError`.
- Replaced internal declaration-claim `unreachable!` arms with explicit
  validation and staging errors.

---

## 0.7.4

### Public API hygiene

- Hid constructor-bypassing fields on `AllocationRetirement`,
  `MemoryManagerAuthorityRecord`, and `SchemaMetadata`, replacing them with
  read accessors.
- Simplified `StaticMemoryRangeDeclaration::new` so the range authority comes
  only from the validated `MemoryManagerAuthorityRecord`.
- Made implementation modules private, keeping the intended public API on the
  crate root and updating exported macros to use root-level helpers.

### Code hygiene

- Centralized stable-cell ledger-record decoding for runtime reads and
  diagnostics.
- Consolidated static registry lock and sealed-state handling.
- Shared claim-conflict record lookup between validation and staging.
- Added strict unknown-field rejection to diagnostic export DTOs.
- Added a deterministic transition matrix test that checks committed ledger
  invariants across many declaration, reservation, and retirement sequences.

---

## 0.7.3

### Runtime hardening

- Kept the default runtime's internal ledger allocation in the durable ledger
  while removing it from the published/openable validated allocation set.
- Made public default-runtime opens reject `ic_memory.*` governance stable keys.
- Preflighted default ledger-cell writes so oversized records or failed stable
  memory growth return a bootstrap error before calling `Cell::set`.
- Revalidated full reservation declarations before staging reservation
  generations.
- Made static range declaration authority mismatches fail in release builds.
- Marked public error enums as non-exhaustive so future patch releases can add
  variants without breaking downstream wildcard matches.

---

## 0.7.2

### Diagnostics

- Added a default `MemoryManager` doctor report that combines stable-cell
  status, commit recovery, recovered ledger export, registered declarations,
  range authority, validation preflight, and live memory sizes.
- Documented default-runtime diagnostic behavior, including the fact that the
  doctor runs deferred `eager_init!` hooks before bootstrap and that custom
  policy diagnostics remain the framework adapter's responsibility.

---

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
