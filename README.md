# ic-memory

`ic-memory` prevents Internet Computer stable-memory slots from being
accidentally reused, moved, or reassigned across canister upgrades.

It does this by keeping a durable allocation ledger that records which
canonical stable key owns which physical allocation slot. The invariant is:

```text
stable_key -> allocation_slot forever
```

More fully: once a logical stable key has been assigned to a physical
allocation slot, that key must never point to a different slot, and that slot
must never be reused for a different key, even after retirement.

This crate is not a replacement for `ic-stable-structures`. It is not a generic
memory abstraction. It is stable-memory allocation-governance infrastructure.
Use it before opening stable-memory handles, so a canister rejects an unsafe
layout before it can open the wrong stable memory for a logical store.

The non-negotiable invariants are recorded in [SAFETY.md](SAFETY.md).

## Status

`ic-memory` is early infrastructure extracted from Canic. The public API is
intended to stabilize around persistent allocation ownership, but framework
authors should still treat this line as young infrastructure while the
standalone boundary settles.

## The Bug Class

Multi-store canisters and frameworks often map logical stores onto physical
stable-memory slots such as `ic-stable-structures::MemoryManager` IDs.

For example, a canister may ship with this layout:

```text
v1:
  app.users.v1  -> MemoryManager ID 100
  app.orders.v1 -> MemoryManager ID 101
```

A bad upgrade can accidentally swap those IDs:

```text
bad v2:
  app.users.v1  -> MemoryManager ID 101
  app.orders.v1 -> MemoryManager ID 100
```

That upgrade may still compile and install. Rust's type system and
`ic-stable-structures` do not automatically know that `app.users.v1` used to
own ID 100. The canister can boot while opening the orders memory as users, or
the users memory as orders.

`ic-memory` exists to reject that mapping before memory handles are opened.

## Why This Exists With ic-stable-structures

`ic-stable-structures` provides stable data structures and memory abstractions.
It lets you store data in stable memory.

`ic-memory` records and validates durable ownership of stable-memory slots over
time. It answers a different question: is this logical store still opening the
same physical slot it has always owned?

The two crates are complementary. A framework can use `ic-memory` to validate
allocation ownership, then use `ic-stable-structures` to open and operate on the
validated memories.

## How It Fits

- `ic-stable-structures` stores data in stable memory.
- `ic-memory` governs which logical store is allowed to open which stable-memory
  slot.
- The framework or application decides namespace policy, range policy,
  lifecycle timing, controller authorization, and schema migration.

This crate owns allocation invariants, not framework policy. It is generic over
storage substrates: `MemoryManager` IDs are one supported slot descriptor shape,
not the whole design.

## Who Needs This?

You probably do not need `ic-memory` if your canister has one stable structure,
a small fixed hand-written layout, and no generated or framework-managed stable
stores.

You may need it if you are building:

- an IC framework
- a multi-store canister
- a generated canister platform
- a plugin/module system
- a canister family where stable-memory declarations evolve over time
- any system where accidental stable-memory ID reuse would be catastrophic

## Lifecycle

The intended flow is:

1. Declare expected stable-memory allocations with canonical stable keys.
2. Recover the historical allocation ledger.
3. Validate current declarations against policy and historical ownership.
4. Commit a new allocation generation.
5. Open physical memory only through a validated allocation session.
6. Export diagnostics when needed.

Opening memory handles is deliberately a later phase. Declaration and validation
happen first so slot drift is caught before the application touches stable data.

## Terminology

- Stable key: a canonical logical name for one durable store, such as
  `app.orders.v1`.
- Allocation slot: the physical stable-memory location a storage substrate can
  open, such as a `MemoryManager` ID.
- Allocation ledger: durable history of stable-key to allocation-slot ownership.
- Declaration: the current binary's claim that a stable key should own a slot.
- Generation: one committed version of the allocation ledger.
- Reservation: a slot/key pair held for future use but not yet active.
- Retirement / tombstone: an explicit historical marker that an allocation is no
  longer active.
- Validated allocation session: the capability produced after declarations pass
  policy and ledger-history validation.
- Storage substrate: the implementation that interprets slots and opens
  physical memory handles.

Retirement is a tombstone, not a free-list operation. A retired stable key/slot
pair remains historically owned and cannot be reused for a different key. This
preserves rollback safety, diagnostics, and historical ABI integrity.

## Non-Goals

`ic-memory` does not own:

- stable data-structure schemas
- schema migrations
- controller authorization
- IC management-canister calls
- endpoint dispatch
- framework namespace/range policy
- application-level data validation

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

Decoded durable ledgers and public DTO structs are untrusted until validated.
Recovery and commit paths use strict committed-ledger validation before a ledger
can become authoritative.

## Example: Declaration Phase

This example demonstrates lifecycle phase 1: collect the current binary's
expected stable-memory allocations. It does not open memory.

```rust
use ic_memory::{
    AllocationDeclaration, AllocationSlotDescriptor, DeclarationCollector, SchemaMetadata,
};

let mut declarations = DeclarationCollector::default();
let declaration = AllocationDeclaration::new(
    "app.orders.v1",
    AllocationSlotDescriptor::memory_manager(100).expect("usable slot"),
    Some("orders".to_string()),
    SchemaMetadata::default(),
)
.expect("valid allocation declaration");

declarations.push(declaration);

let snapshot = declarations.seal().expect("valid declaration snapshot");
assert_eq!(snapshot.len(), 1);
```

## Example: Rejecting Slot Drift

This example demonstrates lifecycle phase 3: validate the current declarations
against historical ownership. The bad upgrade tries to move `app.users.v1` from
MemoryManager ID 100 to ID 101, so validation fails before an allocation session
can open any memory handles.

```rust
use ic_memory::{
    AllocationDeclaration, AllocationHistory, AllocationLedger, AllocationPolicy,
    AllocationRecord, AllocationSlotDescriptor, AllocationState, AllocationValidationError,
    CURRENT_LEDGER_SCHEMA_VERSION, CURRENT_PHYSICAL_FORMAT_ID, DeclarationSnapshot,
    GenerationRecord, SchemaMetadata, StableKey, validate_allocations,
};
use std::convert::Infallible;

#[derive(Debug)]
struct AllowAllPolicy;

impl AllocationPolicy for AllowAllPolicy {
    type Error = Infallible;

    fn validate_key(&self, _key: &StableKey) -> Result<(), Self::Error> {
        Ok(())
    }

    fn validate_slot(
        &self,
        _key: &StableKey,
        _slot: &AllocationSlotDescriptor,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn validate_reserved_slot(
        &self,
        _key: &StableKey,
        _slot: &AllocationSlotDescriptor,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn declaration(key: &str, memory_manager_id: u8) -> AllocationDeclaration {
    AllocationDeclaration::new(
        key,
        AllocationSlotDescriptor::memory_manager(memory_manager_id).expect("usable slot"),
        None,
        SchemaMetadata::default(),
    )
    .expect("valid declaration")
}

let historical_ledger = AllocationLedger {
    ledger_schema_version: CURRENT_LEDGER_SCHEMA_VERSION,
    physical_format_id: CURRENT_PHYSICAL_FORMAT_ID,
    current_generation: 1,
    allocation_history: AllocationHistory {
        records: vec![
            AllocationRecord::from_declaration(
                1,
                declaration("app.users.v1", 100),
                AllocationState::Active,
            ),
            AllocationRecord::from_declaration(
                1,
                declaration("app.orders.v1", 101),
                AllocationState::Active,
            ),
        ],
        generations: vec![GenerationRecord {
            generation: 1,
            parent_generation: Some(0),
            runtime_fingerprint: None,
            declaration_count: 2,
            committed_at: None,
        }],
    },
};

let bad_v2 = DeclarationSnapshot::new(vec![
    declaration("app.users.v1", 101),
    declaration("app.orders.v1", 100),
])
.expect("duplicate-free declarations");

let error = validate_allocations(&historical_ledger, bad_v2, &AllowAllPolicy)
    .expect_err("slot drift must be rejected");

assert!(matches!(
    error,
    AllocationValidationError::StableKeySlotConflict { .. }
));
```

The protected physical checksum detects torn writes and accidental corruption.
It is not a cryptographic integrity mechanism and must not be treated as
adversarial tamper resistance.
