# ic-memory

<p align="center">
  <img src="images/under-construction.gif" alt="Animated warning banner" width="400">
</p>

<p align="center">
  <strong>EARLY INFRASTRUCTURE: validate before opening stable memory.</strong>
</p>

---

`ic-memory` helps Internet Computer canisters avoid opening the wrong stable
memory after an upgrade.

The invariant:

> Once a stable key is committed to a physical allocation slot, future binaries
> must either reopen that same stable key on that same slot or declare a new
> stable key.

It remembers this mapping forever:

```text
logical store -> physical stable-memory slot
```

If a future version tries to move that store to a different slot, or reuse that
slot for a different store, `ic-memory` rejects the layout before stable-memory
handles are opened.

The meme is funny. The bug is not.

<p align="center">
  <img src="images/balloon-meme.jpg" alt="Meme showing ic-memory keeping ic-stable-structures stable memory allocations from drifting" width="375">
</p>

## Why Use It?

Use `ic-memory` when a canister has more than one stable store and the layout
can change over time.

It is most useful for:

- IC frameworks
- generated canisters
- multi-store canisters
- plugin or module systems
- canister families that evolve across releases
- any project where stable-memory ID reuse would be a serious bug

You probably do not need it for a tiny canister with one hand-written stable
structure and a fixed layout.

## What It Protects Against

The dangerous bug is slot drift.

Version 1 ships with:

```text
app.users.v1  -> MemoryManager ID 100
app.orders.v1 -> MemoryManager ID 101
```

A later upgrade accidentally ships with:

```text
app.users.v1  -> MemoryManager ID 101
app.orders.v1 -> MemoryManager ID 100
```

That can still compile. It can even install.

But now the canister may open orders data as users data, and users data as
orders data (or more likely just fail to deserialize anything.)

`ic-memory` catches that mismatch first.

It enforces both directions for active allocations:

- The same active stable key cannot move to a different physical slot.
- The same active physical slot cannot be reused by a different stable key.

<p align="center">
  <img src="images/dont-overwrite.png" alt="Retro computer warning: don't overwrite your memory" width="500">
</p>

## How It Fits

`ic-stable-structures` stores the data.

`ic-memory` checks that each logical store is still opening the same physical
slot it owned before.

A typical framework flow is:

1. Recover the saved allocation ledger.
2. Declare the stores this binary expects.
3. Validate those declarations against history and policy.
4. Commit the new generation.
5. Open stable-memory handles only after validation passes.

The important rule: validate layout before touching stable data.

Do not rawdog stable-memory IDs.

## Golden Path

The exact persistence and storage substrate are integration-specific. The safe
order is fixed:

```text
recover persisted allocation ledger
declare this binary's expected stable stores
validate declarations against ledger/history/policy
commit the new generation
only then open stable-memory handles
```

Minimal compiling sketch:

```rust
use ic_memory::{
    AllocationHistory, AllocationLedger, AllocationPolicy, CURRENT_LEDGER_SCHEMA_VERSION,
    CURRENT_PHYSICAL_FORMAT_ID, DeclarationCollector, LedgerCodec, LedgerCommitStore, StableKey,
    validate_allocations,
};
use std::cell::RefCell;

#[derive(Default)]
struct DemoCodec {
    ledgers: RefCell<Vec<AllocationLedger>>,
}

impl LedgerCodec for DemoCodec {
    type Error = &'static str;

    fn encode(&self, ledger: &AllocationLedger) -> Result<Vec<u8>, Self::Error> {
        let mut ledgers = self.ledgers.borrow_mut();
        let index = u64::try_from(ledgers.len()).map_err(|_| "too many ledgers")?;
        ledgers.push(ledger.clone());
        Ok(index.to_le_bytes().to_vec())
    }

    fn decode(&self, bytes: &[u8]) -> Result<AllocationLedger, Self::Error> {
        let bytes: [u8; 8] = bytes.try_into().map_err(|_| "invalid ledger index")?;
        let index =
            usize::try_from(u64::from_le_bytes(bytes)).map_err(|_| "invalid ledger index")?;
        self.ledgers
            .borrow()
            .get(index)
            .cloned()
            .ok_or("unknown ledger index")
    }
}

struct AllowAllPolicy;

impl AllocationPolicy for AllowAllPolicy {
    type Error = &'static str;

    fn validate_key(&self, _key: &StableKey) -> Result<(), Self::Error> {
        Ok(())
    }

    fn validate_slot(
        &self,
        _key: &StableKey,
        _slot: &ic_memory::AllocationSlotDescriptor,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn validate_reserved_slot(
        &self,
        _key: &StableKey,
        _slot: &ic_memory::AllocationSlotDescriptor,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

let codec = DemoCodec::default();
let mut store = LedgerCommitStore::default();
let genesis = AllocationLedger::new(
    CURRENT_LEDGER_SCHEMA_VERSION,
    CURRENT_PHYSICAL_FORMAT_ID,
    0,
    AllocationHistory::default(),
)
.expect("valid genesis ledger");

let ledger = store
    .recover_or_initialize(&codec, &genesis)
    .expect("recover or initialize allocation ledger");

let snapshot = DeclarationCollector::new()
    .with_memory_manager("app.orders.v1", 100, "orders")
    .expect("valid allocation declaration")
    .seal()
    .expect("valid declaration snapshot");

let validated =
    validate_allocations(&ledger, snapshot, &AllowAllPolicy).expect("layout is safe");

let next_ledger = ledger
    .stage_validated_generation(&validated, None)
    .expect("stage next allocation generation");

let _committed = store
    .commit(&next_ledger, &codec)
    .expect("commit allocation ledger before opening stable memory");

// Only now should the integration open stable-memory handles for `validated`.
```

## Basic Declaration

Declare every stable store with a stable name and a physical slot:

```rust
use ic_memory::DeclarationCollector;

let snapshot = DeclarationCollector::new()
    .with_memory_manager("app.orders.v1", 100, "orders")
    .expect("valid allocation declaration")
    .seal()
    .expect("valid declaration snapshot");
assert_eq!(snapshot.len(), 1);
```

That snapshot is what you validate against the recovered ledger before opening
the store.

## Stable Keys

Stable keys are permanent logical store names. They should describe ownership
and purpose, not the current memory ID.

Format:

```text
namespace.component.store_or_role.vN
```

Rules:

- ASCII only.
- Lowercase only.
- Dot-separated segments.
- Each segment starts with a lowercase letter.
- Segments may contain lowercase letters, digits, and underscores.
- No whitespace, slashes, or hyphens.
- Must end with a nonzero version suffix such as `.v1` or `.v12`.
- Maximum length is 128 bytes.

Examples:

```rust
use ic_memory::StableKey;

StableKey::parse("app.orders.v1").expect("app key");
StableKey::parse("myapp.audit_log.v1").expect("app key");
StableKey::parse("framework.cache.index.v1").expect("framework key");
StableKey::parse("database.users.data.v1").expect("database key");
```

Suggested namespace conventions:

- `ic_memory.*` is reserved for `ic-memory` governance records.
- Application-owned stores can use an application namespace, such as
  `app.orders.v1` or `myapp.audit_log.v1`.
- Frameworks and generated stores should use namespaces they own, such as
  `framework.cache.index.v1` or `database.users.data.v1`.

Canic and IcyDB examples:

- `canic.core.*` is appropriate for Canic framework-owned stores.
- `icydb.<memory_namespace>.<store_name>.<role>.vN` works for generated IcyDB
  stores, such as `icydb.test_db.users.data.v1`.

Changing a key creates a new logical allocation identity. If the durable store
is the same, keep the stable key and update schema metadata instead.

## Range Authority

Range authority is policy metadata. It does not allocate stable-memory IDs and
does not write to the allocation ledger.

Packages should publish only the ranges they own:

```rust
use ic_memory::{
    IC_MEMORY_AUTHORITY_OWNER, MemoryManagerRangeAuthority, MemoryManagerRangeMode,
    memory_manager_governance_range,
};

let authority = MemoryManagerRangeAuthority::new()
    .reserve(memory_manager_governance_range(), IC_MEMORY_AUTHORITY_OWNER)
    .expect("ic-memory governance range")
    .reserve_ids(10, 99, "framework.example")
    .expect("framework range");

authority
    .validate_id_authority_mode(42, "framework.example", MemoryManagerRangeMode::Reserved)
    .expect("framework-owned ID");
```

An open stack composes records from multiple packages and rejects overlaps:

```rust
use ic_memory::MemoryManagerRangeAuthority;

let framework_records = MemoryManagerRangeAuthority::new()
    .reserve_ids(10, 99, "framework.example")
    .expect("framework range")
    .to_records();

let database_records = MemoryManagerRangeAuthority::new()
    .reserve_ids(120, 149, "database.framework")
    .expect("database range")
    .to_records();

let authority = MemoryManagerRangeAuthority::from_records(
    framework_records
        .into_iter()
        .chain(database_records)
        .collect(),
)
.expect("non-overlapping package ranges");

assert_eq!(authority.authorities().len(), 2);
```

A final closed policy may claim the remaining application space and require full
coverage:

```rust
use ic_memory::{
    IC_MEMORY_AUTHORITY_OWNER, MEMORY_MANAGER_MAX_ID, MemoryManagerIdRange,
    MemoryManagerRangeAuthority, memory_manager_governance_range,
};

let authority = MemoryManagerRangeAuthority::new()
    .reserve(memory_manager_governance_range(), IC_MEMORY_AUTHORITY_OWNER)
    .expect("ic-memory governance range")
    .reserve_ids(10, 99, "framework.example")
    .expect("framework range")
    .allow_ids(100, MEMORY_MANAGER_MAX_ID, "applications")
    .expect("application range");

authority
    .validate_complete_coverage(MemoryManagerIdRange::all_usable())
    .expect("closed policy covers every usable ID");
```

## Current MemoryManager Rules

For the built-in `ic-stable-structures::MemoryManager` slot descriptor:

- IDs `0..=254` are usable stable-memory slots.
- ID `255` is rejected because it is the unallocated sentinel.
- IDs `0..=9` are reserved for `ic-memory` governance.
- ID `0` is assigned to the allocation ledger.

The crate also exposes range-authority helpers for frameworks that want to split
ID ranges between infrastructure and application stores.

Canic currently uses `10..=99` as a framework-reserved example range. That is
Canic policy, not an `ic-memory` rule.

## What It Does Not Do

`ic-memory` does not replace `ic-stable-structures`.

It also does not handle:

- schema migrations
- schema compatibility or data semantics
- controller authorization
- application data validation
- endpoint routing
- IC management-canister calls
- malicious-controller protection
- disaster recovery

It only protects stable-memory allocation ownership.

## Status

`ic-memory` is early infrastructure extracted from Canic. The public API is
intended to stabilize around persistent allocation ownership, but framework
authors should still treat this line as young infrastructure while the
standalone boundary settles.

Earlier drafts exposed some durable DTO fields directly. Current versions use
checked constructors and accessors so invalid allocation state is harder to
construct accidentally.

The non-negotiable invariants are recorded in [SAFETY.md](SAFETY.md).

The protected physical checksum detects torn writes and accidental corruption.
It is not a cryptographic integrity mechanism and must not be treated as
adversarial tamper resistance.
