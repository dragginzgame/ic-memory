# ic-memory Recurring Audit: Code Hygiene

## Purpose

Catch small defects, drift, and maintainability regressions before they become architectural issues.

This audit is intentionally simple. It is not a security audit, compatibility audit, or design review.

## Assessment

This is the right recurring audit shape for `ic-memory`: it is narrow,
repeatable, and focused on the kinds of small quality failures that can weaken
the larger safety model over time. The strongest parts are the trust-boundary
questions, the emphasis on precise errors, and the insistence on negative tests.

The main weakness is that the current checklist can still produce a broad,
subjective review unless the auditor is forced to separate mechanical hygiene
from protocol design. `ic-memory` has many protocol-sensitive types, so a code
hygiene pass should identify confusing surfaces and missing guardrails, but it
should not silently become the temporal-compatibility audit or a redesign
proposal.

The best improvement is to make the audit more evidence-driven:

- require a public API inventory;
- require a trust-state inventory;
- require a panic/unwrap classification;
- require a serde/constructor classification;
- require each finding to state whether it is mechanical, behavioral, or design
  work.

## Scope

Audit the current `ic-memory` crate for:

1. Dead or misleading code
   - unused helpers
   - stale abstractions
   - obsolete comments
   - misleading names
   - public APIs that should be private
   - test-only code leaking into production API

2. Error hygiene
   - vague error variants
   - duplicated error types
   - errors that lose important context
   - panic paths in library/runtime code
   - unwrap/expect usage outside tests
   - errors that should distinguish corrupt / unsupported / invalid / policy-rejected

3. API hygiene
   - unnecessary public fields
   - unnecessary public constructors
   - authority-bearing types with too much surface
   - confusing advanced APIs
   - missing rustdoc on public protocol-sensitive items
   - names that obscure whether a value is raw, decoded, validated, recovered, or authoritative

4. Validation hygiene
   - validation duplicated inconsistently
   - validation missing at public boundaries
   - manual construction paths not covered by validation
   - serde-decoded values treated as trusted too early
   - tests that validate only the golden path

5. Test hygiene
   - missing regression tests for recent fixes
   - overly broad tests that do not assert the important invariant
   - tests coupled to implementation details rather than behavior
   - missing negative tests
   - missing panic/fail-closed tests
   - stale fixture names or comments

6. Documentation hygiene
   - README drift
   - ADVANCED.md drift
   - rustdoc drift
   - examples that use old API names
   - docs that describe behavior no longer enforced
   - docs that omit important authority/protocol caveats

7. Dependency and feature hygiene
   - unnecessary dependencies
   - unstable feature flags
   - test-only dependencies in normal builds
   - public API accidentally depending on optional features
   - workspace/package metadata drift

8. Module and ownership hygiene
   - files carrying too many unrelated responsibilities
   - tests embedded in modules that now obscure production code
   - TODOs that have become durable design decisions
   - helpers whose names no longer match their authority level
   - duplicated concepts between runtime, bootstrap, validation, and ledger code

## Commands

Run:

cargo fmt --all --check
cargo test -p ic-memory
cargo clippy -p ic-memory --all-targets -- -D warnings
cargo doc -p ic-memory --no-deps
cargo check -p ic-memory --all-features
cargo check -p ic-memory --no-default-features
git diff --check

Also run targeted searches:

rg "unwrap|expect|panic!|todo!|unimplemented!" src tests
rg "pub " src
rg "Deserialize|Serialize|Default|Clone|Copy" src
rg "compatibility|unsafe|advanced|deprecated|TODO|FIXME|HACK" src README.md ADVANCED.md
rg "pub\\(|pub(crate)|pub struct|pub enum|pub trait|pub fn" src
rg "from_slice|from_bytes|decode|deserialize|Deserialize" src
rg "Result<|thiserror|panic!" src

For the `pub` and serde searches, do not paste raw output into the report. Use
the output to build a short inventory and then identify only actionable issues.

## Method

1. Build a public API inventory.
   - List authority-bearing types separately from inert DTOs.
   - Note every public constructor for those types.
   - Note every public function that returns or consumes authority.

2. Build a decoded-data inventory.
   - List every `Deserialize` type.
   - Mark each as `inert`, `diagnostic`, `physical`, `ledger`, `declaration`,
     or `authority`.
   - Authority types should not deserialize. Decoded DTOs must be validated
     before use.

3. Build a panic inventory.
   - Classify each `panic!`, `unwrap`, and `expect` as test-only, constructor
     invariant, impossible-by-type, or bug.
   - Runtime/library paths that inspect stable memory should prefer classified
     errors over panics.

4. Build an error inventory.
   - Confirm errors distinguish corruption, unsupported versions, invalid input,
     policy rejection, and not-yet-bootstrapped state where that distinction
     affects operator action.

5. Build a test inventory.
   - For each recent fix, identify its regression test.
   - For each public validation boundary, identify at least one negative test.
   - Prefer behavior names over implementation names in new tests.

6. Classify each finding.
   - `Mechanical`: rename, visibility, rustdoc, stale comment, test name.
   - `Behavioral`: fail-closed behavior, validation ordering, error split.
   - `Design`: protocol evolution, migration model, long-term storage format.

Only mechanical and clearly safe behavioral findings should be fixed during the
audit. Design findings should be linked to design docs or follow-up issues.

## Key ic-memory Questions

For every public item, ask:

- Is this raw data, decoded data, validated data, recovered state, or authority?
- Does the type name make that distinction obvious?
- Can a downstream caller misuse it to bypass validation?
- Should this be `pub`, `pub(crate)`, or private?
- Does rustdoc explain the trust boundary?
- Is this public because downstream crates need it, or because tests once needed
  it?

For every error path, ask:

- Does this fail closed?
- Would an operator know what happened?
- Is corrupt state distinct from unsupported future state?
- Is policy rejection distinct from integrity rejection?
- Is panic avoided in library/runtime paths?
- Does the error name match the layer that reports it?

For every test, ask:

- What invariant does this prove?
- Would it catch a regression?
- Does it test failure as well as success?
- Does it protect a recently fixed bug?
- Would a future maintainer understand which invariant the test protects from
  the test name alone?

For every document and example, ask:

- Does the README show the shortest normal path?
- Is advanced protocol or recovery material in `ADVANCED.md` or `SAFETY.md`
  instead of the README?
- Do examples use `rust` fenced code blocks?
- Do docs distinguish `ic-memory` generic mechanics from Canic policy?

## Deliverable

Write a concise report with:

1. Executive summary
2. Risk score 0-10
3. Findings grouped by High / Medium / Low / Cleanup
4. For each finding:
   - file:path:line
   - issue
   - why it matters
   - recommended fix
   - suggested regression test, if applicable
5. A checklist of quick fixes suitable for one PR
6. A separate list of issues that should be deferred to design work
7. A short "Audit Quality" section:
   - what the audit is confident about;
   - what it did not inspect;
   - what would make the next pass stronger.

Use this severity guide:

- `High`: a public API or runtime path can plausibly be misused to bypass
  validation, panic on stable-memory input, or hide an operator-actionable error.
- `Medium`: stale naming, documentation, or module shape can mislead future
  maintainers around authority, recovery, or validation.
- `Low`: local cleanup, test clarity, or ergonomics issues with limited blast
  radius.
- `Cleanup`: mechanical formatting, dead comments, duplicate fixtures, or
  naming polish.

## Non-goals

Do not redesign the protocol.
Do not propose large migration architecture changes unless code hygiene reveals a concrete immediate hazard.
Do not make speculative security claims.
Do not change code unless the fix is mechanical and obviously safe.

## Expected Output Style

Be direct and practical.

Prefer findings like:

- “make this private”
- “rename this”
- “add rustdoc here”
- “split this error”
- “replace expect with Result”
- “add a negative test”
- “delete stale helper”

Avoid broad architecture commentary unless it directly follows from code hygiene.

## Common False Positives

- A public DTO is not automatically a bug. It is a bug only if callers can treat
  it as authority without validation.
- A `Deserialize` derive is not automatically a bug. It is a bug when the decoded
  value can cross into authority without deep validation.
- An advanced API is not automatically a bug. It is a hygiene issue when rustdoc
  does not clearly state the trust boundary and intended users.
- A large module is not automatically a bug. It becomes a hygiene issue when
  unrelated responsibilities make review or invariant tracing difficult.
- A missing test is not automatically high severity. Severity should follow the
  invariant the test would protect.
