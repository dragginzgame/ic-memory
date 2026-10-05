<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-memory/ic-memory-readme-header.svg" alt="IC Memory — Stops upgrades from mixing up stored data" width="100%">
</p>

# ic-memory documentation

The documents in this directory serve different audiences. Use this index to
distinguish current integration guidance from historical engineering evidence.

## Start here

- [Project overview and quick start](../README.md) explains in plain language
  what `ic-memory` protects, when it is useful, and how to integrate it.
- [Frequently asked questions](../README.md#frequently-asked-questions) answers
  common scope, lifecycle, retirement, and allocation-style questions.
- [Advanced integration](../ADVANCED.md) covers runtime ownership, custom
  policies, range authority, recovery, and manual bootstrap.
- [Safety invariants](../SAFETY.md) defines the properties that implementation
  changes must preserve.
- [Host support and qualification](host-support.md) declares native prerequisites,
  supported macOS hosts, and qualification gaps.

## Current focused guides

- [Operations and diagnostics](operations.md) explains allocation reports,
  bucket configuration, doctor reports, and growth failures.
- [Troubleshooting](troubleshooting.md) maps common symptoms and typed errors
  to safe next actions without resetting durable state.
- [Key-only allocation and bounded recovery](key-only-recovery.md) defines
  automatic placement, host adoption, omitted-store inspection, and recovery
  limits.
- [Recovered-metadata admission](recovered-admission.md) defines how a host can
  select known historical stores before committing one bootstrap generation.

When current guides disagree, the crate's public API and safety invariants are
authoritative. This project is pre-1.0 and maintains only its current API and
durable format.

The package version is declared in [Cargo.toml](../Cargo.toml); current guides
describe the maintained implementation without separate release stamps.
Historical qualification records retain the exact versions and source snapshots
they checked.

## Integration and qualification records

These documents record specific consumer source snapshots, environments, and
test results. They are evidence for the versions named in each document, not a
promise that a later consumer graph or deployment was requalified.

- [Issue reconciliation](issue-reconciliation.md)
- [Runtime IO and maintainer improvements qualification](runtime-io-qualification.md)
- [Bounded ledger codec qualification](ledger-codec-qualification.md)
- [Shared release workflow qualification](release-workflow-qualification.md)
- [Canic checked-slot qualification for 0.25.0](consumer-qualification-0.25.0-canic.md)
- [Canic memory qualification for 0.24.6](consumer-qualification-0.24.6-canic.md)
- [IcyDB qualification for 0.24.4](consumer-qualification-0.24.4.md)
- [IcyDB qualification for 0.24.3](consumer-qualification-0.24.3.md)
- [Consumer qualification for 0.20.0](consumer-qualification-0.20.0.md)

## Historical design and measurement records

These reports explain why earlier changes were made and retain their original
measurements. Use the current guides above for present-day API instructions.

- [CANIC-162 physical allocation study](canic162-memory-attribution.md)
- [Opaque ledger payload encoding](opaque-ledger-payloads.md)
- [Logical bootstrap cleanup](logical-bootstrap-cleanup.md)
- [Wasm codec size audit](audits/wasm-codec-size-2026-07-14.md)
- [Recurring audit records](audits/recurring/)

Measurement CSVs under `measurements/` belong to the historical reports that
link to them.

## Maintainers

- [Detailed 0.26 release notes](changelog/0.26.md)
- [Detailed 0.25 release notes](changelog/0.25.md)
- [Release guide](../RELEASING.md)
- [Current wire fixtures](../fixtures/current/README.md)

### Documentation release checklist

Before publishing a release that changes the API, durable format, terminology,
or workflow:

- update the crate version and README dependency example together;
- update examples, diagrams, captions, and alt text when terminology changes;
- run doctests with warnings denied;
- verify every local Markdown link and heading anchor;
- confirm current guidance does not retain a superseded pre-1.0 API or wire
  shape; and
- keep historical measurements and qualification claims labeled with their
  original version and scope.
