#![deny(unsafe_code, unsafe_op_in_unsafe_fn)]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc = include_str!("../README.md")]

//! Stable-memory allocation-governance primitives for Internet Computer
//! canister upgrades.
//!
//! `ic-memory` prevents stable-memory slot drift.
//!
//! Once a stable key is committed to a physical allocation slot, future binaries
//! must either reopen that same stable key on that same slot or declare a new
//! stable key.
//!
//! The crate records and validates durable ownership in both directions: an
//! active stable key cannot move to a different physical slot, and an active
//! physical slot cannot be reused by a different stable key.
//!
//! The intended runtime integration flow is:
//!
//! 1. Recover the persisted allocation ledger.
//! 2. Admit consumer identity and authorized historical selections from bounded
//!    metadata under the host's `RuntimeBootstrapPolicy`.
//! 3. Resolve key requests in the host-wide pool under current namespace grants,
//!    then validate retained key-to-ID bindings and application policy.
//! 4. Stage and durably persist the next generation.
//! 5. Only then open stable-memory handles through committed allocation
//!    authority.
//!
//! This crate owns allocation invariants, not framework policy. Namespace
//! rules, controller authorization, endpoint lifecycle, schema migrations, and
//! application validation belong to the framework or application.
//!
//! The host supplies one [`MemoryAllocationPool`] with explicit namespace grants
//! and physical exclusions. Linked components request permanent keys and name
//! their owner; they do not select numeric IDs or reserve component subranges.
//! Namespace grants are current host policy, not caller authentication or
//! persisted ownership labels. Application admission remains host-owned.
//!
//! Use these primitives before opening stable-memory handles. Integrations
//! should recover the historical ledger, declare the stores expected by the
//! current binary, admit recovered identity, resolve requests, validate against
//! history and policy, persist a new generation, and only then publish authority
//! before opening slots through the storage owner.
//!
//! Bounded physical attribution is available through
//! [`MemoryRuntime::memory_allocations`] and
//! [`MemoryRuntime::memory_allocation_summary`]. It reports actual persisted
//! buckets and explicit residuals without decoding retained ownership. Virtual
//! extent is not payload occupancy. Opens return [`RuntimeMemory`]; explicit
//! [`MemoryManagerConfig`] selects fresh-state buckets or checks a persisted
//! setting without migration. The default remains 128 pages.
//!
//! [`MemoryRuntime`] is the canonical owner for one backing memory instance. It
//! owns that memory's manager, ledger persistence, bootstrap lifecycle, committed
//! bindings, opens, and diagnostics. Each bootstrap attempt fallibly decodes
//! its ledger record and uses a temporary cell for capacity-checked writes.
//! Linked code contributes declarations to
//! one immutable [`SealedDeclarationSnapshot`], which is supplied to each
//! runtime independently.
//!
//! The host supplies the complete checked request snapshot, allocation pool and
//! admission policy directly to the runtime. Component factories and generators
//! contribute request lists; opening a store never registers a declaration.
//! Exactly one runtime owns bootstrap and persistence for each backing memory.
//!
//! `ic-stable-structures` `MemoryManager` IDs are the first-class supported
//! physical slot substrate. That ID domain is `u8`: IDs `0..=254` are usable,
//! and ID `255` is always the `ic-stable-structures` unallocated sentinel.
//! Protected current-snapshot commits and stable-cell encoding remain internal to the
//! runtime. Public diagnostics describe their state without granting a manual
//! commit or persistence-confirmation route.
//!
//! Hosts use [`DefaultMemoryImpl`], [`VectorMemory`] and [`Memory`] for backing
//! setup. [`RuntimeMemory`] is an exclusive allocation lease; production typed
//! collections consume these leases through IC Storage’s governed wrappers.

mod allocation_pool;
mod bounded_writer;
mod cbor;
mod constants;
mod declaration;
mod diagnostics;
mod hash;
mod key;
mod ledger;
mod physical;
mod policy;
mod requests;
mod runtime;
mod slot;
mod sort;
mod stable_cell;
mod text;
mod validation;

#[cfg(test)]
mod test_cbor {
    use serde::{Serialize, de::DeserializeOwned};

    pub use ciborium::Value;

    pub fn to_vec<T: Serialize>(
        value: &T,
    ) -> Result<Vec<u8>, ciborium::ser::Error<std::io::Error>> {
        let mut bytes = Vec::new();
        ciborium::into_writer(value, &mut bytes)?;
        Ok(bytes)
    }

    pub fn from_slice<T: DeserializeOwned>(
        bytes: &[u8],
    ) -> Result<T, ciborium::de::Error<std::io::Error>> {
        crate::cbor::from_slice_exact(bytes)
    }

    pub fn to_value<T: Serialize>(value: T) -> Result<Value, ciborium::value::Error> {
        Value::serialized(&value)
    }

    pub fn hex_fixture(contents: &str) -> Vec<u8> {
        let hex = contents
            .chars()
            .filter(|char| !char.is_whitespace())
            .collect::<String>();
        assert_eq!(hex.len() % 2, 0, "fixture hex must have byte pairs");
        hex.as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| {
                let pair = std::str::from_utf8(pair).expect("fixture hex is utf8");
                u8::from_str_radix(pair, 16).expect("fixture hex byte")
            })
            .collect()
    }
}

/// Backing setup and IO interfaces from the exact substrate dependency.
/// Hosts own backing custody; production collections belong behind IC Storage's
/// governed wrappers and consume [`RuntimeMemory`] leases. Import visibility is
/// not admission: implementing `Memory` does not construct a checked handle.
/// Collection codec traits belong to Storage; metric records and restoration
/// belong to IC Metrics Stable.
pub use ic_stable_structures::{DefaultMemoryImpl, Memory, VectorMemory};

pub use allocation_pool::{MemoryAllocationPool, MemoryAllocationPoolError, MemoryAuthority};
pub use constants::{
    MAX_LEDGER_BYTES, MAX_LEDGER_NESTING, MAX_LEDGER_RECORD_BYTES, WASM_PAGE_SIZE_BYTES,
};
pub use declaration::AllocationDeclaration;
pub use diagnostics::{DiagnosticExport, DiagnosticMemorySize, DiagnosticRecord};
pub use key::{StableKey, StableKeyError};
use ledger::{AllocationLedger, LedgerCommitStore, RecoveredLedger};
pub use ledger::{AllocationRecord, AllocationStageError, LedgerCommitError, LedgerIntegrityError};
pub use physical::{CommitRecoveryError, CommitSnapshotDiagnostic};
pub use policy::{PolicyIdentity, RuntimeBootstrapPolicy};
pub use requests::{MemoryDeclarationError, MemoryRequest, SealedDeclarationSnapshot};
pub use runtime::{
    BootstrapAdmission, BootstrapAdmissionError, GenericAllocationPolicy, MemoryAllocation,
    MemoryAllocationSummary, MemoryAllocations, MemoryBindingSummary, MemoryManagerConfig,
    MemoryManagerLayoutError, MemoryResolutionError, MemoryRuntime, RuntimeAdoptionError,
    RuntimeBootstrapError, RuntimeConstructionError, RuntimeDiagnosticError, RuntimeGrowError,
    RuntimeMemory, RuntimeOpenError,
};
pub use slot::{
    IC_MEMORY_AUTHORITY_OWNER, IC_MEMORY_LEDGER_STABLE_KEY, IC_MEMORY_STABLE_KEY_PREFIX,
    MEMORY_MANAGER_GOVERNANCE_MAX_ID, MEMORY_MANAGER_INVALID_ID, MEMORY_MANAGER_LEDGER_ID,
    MEMORY_MANAGER_MAX_ID, MEMORY_MANAGER_MIN_ID, MemoryManagerSlot, MemoryManagerSlotError,
    is_ic_memory_stable_key, validate_memory_manager_id,
};
pub use stable_cell::{
    STABLE_CELL_HEADER_SIZE, STABLE_CELL_LAYOUT_VERSION, STABLE_CELL_MAGIC,
    STABLE_CELL_VALUE_OFFSET, StableCellLedgerError, StableCellPayloadError,
    decode_stable_cell_payload,
};
#[cfg(test)]
use stable_cell::{decode_stable_cell_commit_store, decode_stable_cell_commit_store_from_memory};
pub use validation::AllocationValidationError;

#[cfg(test)]
mod allocation_measurements_tests;

#[cfg(any(test, feature = "test-support"))]
pub mod testing;
