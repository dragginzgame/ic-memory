# ic-memory

`ic-memory` is persistent allocation-governance infrastructure for Internet
Computer stable memory.

The core invariant is:

```text
stable_key -> allocation_slot forever
```

The crate is intentionally generic. It owns durable allocation facts, stable-key
parsing, allocation-slot descriptors, declaration snapshot sealing, ledger data
shapes, policy traits, substrate traits, protected generation commit mechanics,
diagnostics, and validated allocation sessions.

Framework-specific rules belong in adapter crates. Namespace ownership,
controller authorization, endpoint dispatch, install/upgrade lifecycle, schema
migration, and application policy are not generic `ic-memory` responsibilities.

## Status

`0.0.1` is the first standalone split from Canic. The public API is intended to
stabilize around persistent allocation ownership, but downstream frameworks
should still treat this line as early infrastructure while the extraction
finishes.

## What It Provides

- Canonical stable-key parsing.
- Allocation slot descriptors.
- Declaration collection and duplicate rejection.
- Allocation policy and substrate traits.
- Durable allocation history records.
- Generation-scoped staging and commit helpers.
- Tombstone and reservation lifecycle primitives.
- Rollback-safe allocation validation.
- Protected dual-slot commit recovery primitives.
- Read-only diagnostic export shapes.

## What It Does Not Own

- Framework namespace policy.
- Framework/app range ownership.
- Controller authorization.
- Canister endpoint dispatch.
- Runtime bootstrap orchestration.
- Store-level schema migrations.
- Internet Computer management-canister calls.

## Example Shape

```rust
use ic_memory::{
    AllocationDeclaration, DeclarationCollector, SchemaMetadata,
};

let mut declarations = DeclarationCollector::default();
let declaration = AllocationDeclaration::new(
    "app.orders.primary.v1",
    ic_memory::AllocationSlotDescriptor::memory_manager(100),
    Some("orders"),
    SchemaMetadata::default(),
)
.expect("valid allocation declaration");
declarations.push(declaration).expect("unique declaration");

let snapshot = declarations.seal().expect("valid declaration snapshot");
```

Opening a stable-memory handle is deliberately a separate phase. Frameworks
collect declarations, validate them against policy and historical ledger state,
publish a validated allocation session, and only then open substrate slots.
