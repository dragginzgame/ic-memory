use super::{
    MemoryManagerLayoutError, MemoryRuntime, RuntimeDiagnosticError, RuntimeLifecycle, layout,
};
use crate::{
    DiagnosticMemorySize, IC_MEMORY_AUTHORITY_OWNER, IC_MEMORY_LEDGER_STABLE_KEY,
    MEMORY_MANAGER_LEDGER_ID, MemoryManagerRangeMode, WASM_PAGE_SIZE_BYTES,
};
use ic_stable_structures::Memory;
use serde::Serialize;

///
/// AllocationBinding
///
/// Source of a stable-key binding. Unknown includes retired or absent current
/// declarations: this report never recovers historical ownership. The ledger
/// variant identifies the substrate-reserved slot, not validation of its payload.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub enum AllocationBinding {
    /// A declaration bound by this runtime's successful bootstrap.
    Current { stable_key: String, owner: String },
    /// The ic-memory ledger's reserved allocation.
    Ledger { stable_key: String, owner: String },
    /// No current stable-key binding is known; this does not mean unused.
    Unknown,
}

///
/// AllocationRangeClaim
///
/// Current range policy metadata. A range claim is not a stable-key binding,
/// historical ownership claim, or permission to open a memory handle.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AllocationRangeClaim {
    /// Declaring range authority.
    pub authority: String,
    /// Whether the range requires an explicit reserved-slot policy decision.
    pub mode: MemoryManagerRangeMode,
}

///
/// MemoryAllocation
///
/// Measured allocation of one usable ID, including zero-size IDs. Virtual
/// extent is addressable capacity, never live payload occupancy. Bucket slack
/// is assigned bucket capacity beyond virtual extent; it says nothing about
/// unused bytes inside the virtual extent.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MemoryAllocation {
    pub memory_manager_id: u8,
    pub binding: AllocationBinding,
    pub range_claim: Option<AllocationRangeClaim>,
    pub virtual_extent: DiagnosticMemorySize,
    pub allocated_buckets: u16,
    pub allocated_bytes: u64,
    pub bucket_slack_bytes: u64,
    /// Unavailable: neither manager metadata nor virtual extent measures payload.
    pub payload_bytes: Option<u64>,
}

///
/// MemoryAllocations
///
/// Owned, bounded, read-only physical allocation accounting for one runtime's
/// backing memory. Exactly 255 entries are ordered by ID. Numeric sizes are
/// measured from validated persisted metadata or exact arithmetic on those
/// measurements; none are estimates. Physical extent is the supplied backing
/// memory's extent, which is the IC stable extent only for that backing type.
///
/// Conservation: `physical_extent.bytes = manager_metadata_bytes +
/// allocated_bucket_bytes + unmanaged_bytes`. Also `allocated_bucket_bytes =
/// sum(memories.allocated_bytes) = virtual_extent.bytes + bucket_slack_bytes`.
/// Known binding bytes include the reserved ledger slot; unknown binding bytes
/// include allocations whose historical owners were deliberately not decoded.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MemoryAllocations {
    /// In-memory committed generation; unavailable before bootstrap.
    pub current_generation: Option<u64>,
    pub manager_layout_version: u8,
    /// Actual persisted bucket size, never a requested/default assumption.
    pub bucket_size_pages: u16,
    pub bucket_size_bytes: u64,
    pub bucket_capacity: u32,
    pub allocated_buckets: u16,
    pub remaining_buckets: u32,
    /// Finite table capacity, excluding the metadata page and backing limits.
    pub maximum_bucket_bytes: u64,
    pub physical_extent: DiagnosticMemorySize,
    pub virtual_extent: DiagnosticMemorySize,
    /// The complete first page, including header, table, and padding.
    pub manager_metadata_bytes: u64,
    pub manager_header_bytes: u64,
    pub manager_bucket_table_bytes: u64,
    pub manager_padding_bytes: u64,
    pub allocated_bucket_bytes: u64,
    pub bucket_slack_bytes: u64,
    pub known_binding_bytes: u64,
    pub unknown_binding_bytes: u64,
    /// Backing bytes after the assigned bucket region; ownership is unknown.
    pub unmanaged_bytes: u64,
    /// Exact fixed metadata read budget; no ledger history or payload is read.
    pub metadata_bytes_read: u64,
    pub memories: Vec<MemoryAllocation>,
}

impl<M: Memory> MemoryRuntime<M> {
    /// Measure all IDs with a fixed metadata read and bounded current bindings.
    ///
    /// Reads at most 34,848 backing bytes. Never initializes stores, decodes the ledger, writes,
    /// grows memory, or advances a generation. Available before bootstrap;
    /// current declaration/range bindings are then unavailable.
    pub fn memory_allocations(&self) -> Result<MemoryAllocations, RuntimeDiagnosticError> {
        let declarations = match &self.lifecycle {
            RuntimeLifecycle::Unbootstrapped => None,
            RuntimeLifecycle::Bootstrapped { binding, .. } => Some(&binding.declarations),
        };
        // Bound collection before reading metadata or copying any declarations.
        if declarations.is_some_and(|snapshot| {
            snapshot.registered_declarations().len() >= layout::IDS
                || snapshot.range_authority().authorities().len() > layout::IDS
        }) {
            return Err(RuntimeDiagnosticError::AllocationBound);
        }
        let measured = layout::read(self.backing.as_ref())?;
        if measured.bucket_pages != self.bucket_size_pages {
            return Err(super::RuntimeConstructionError::Layout(
                MemoryManagerLayoutError::RuntimeMismatch,
            )
            .into());
        }
        let bucket_size_bytes = u64::from(measured.bucket_pages) * WASM_PAGE_SIZE_BYTES;
        let mut memories = Vec::with_capacity(layout::IDS);
        let mut total_pages = 0;
        let mut known_binding_bytes = 0;
        for id in 0..255_u8 {
            let index = usize::from(id);
            if self.memory(id).size() != measured.pages[index] {
                return Err(super::RuntimeConstructionError::Layout(
                    MemoryManagerLayoutError::RuntimeMismatch,
                )
                .into());
            }
            let (binding, range_claim) = current_binding(id, declarations);
            let virtual_extent = DiagnosticMemorySize::from_wasm_pages(measured.pages[index]);
            let allocated_bytes = u64::from(measured.buckets[index]) * bucket_size_bytes;
            if !matches!(binding, AllocationBinding::Unknown) {
                known_binding_bytes += allocated_bytes;
            }
            total_pages += measured.pages[index];
            memories.push(MemoryAllocation {
                memory_manager_id: id,
                binding,
                range_claim,
                virtual_extent,
                allocated_buckets: measured.buckets[index],
                allocated_bytes,
                bucket_slack_bytes: allocated_bytes - virtual_extent.bytes,
                payload_bytes: None,
            });
        }
        let allocated_bucket_bytes = u64::from(measured.allocated_buckets) * bucket_size_bytes;
        let physical_extent = DiagnosticMemorySize::from_wasm_pages(measured.physical_pages);
        let virtual_extent = DiagnosticMemorySize::from_wasm_pages(total_pages);
        Ok(MemoryAllocations {
            current_generation: self
                .committed_allocations()
                .ok()
                .map(crate::CommittedAllocations::generation),
            manager_layout_version: 1,
            bucket_size_pages: measured.bucket_pages,
            bucket_size_bytes,
            bucket_capacity: 32_768,
            allocated_buckets: measured.allocated_buckets,
            remaining_buckets: 32_768 - u32::from(measured.allocated_buckets),
            maximum_bucket_bytes: 32_768 * bucket_size_bytes,
            physical_extent,
            virtual_extent,
            manager_metadata_bytes: WASM_PAGE_SIZE_BYTES,
            manager_header_bytes: layout::HEADER_BYTES as u64,
            manager_bucket_table_bytes: layout::BUCKETS as u64,
            manager_padding_bytes: WASM_PAGE_SIZE_BYTES - layout::METADATA_BYTES as u64,
            allocated_bucket_bytes,
            bucket_slack_bytes: allocated_bucket_bytes - virtual_extent.bytes,
            known_binding_bytes,
            unknown_binding_bytes: allocated_bucket_bytes - known_binding_bytes,
            unmanaged_bytes: physical_extent.bytes - WASM_PAGE_SIZE_BYTES - allocated_bucket_bytes,
            metadata_bytes_read: layout::METADATA_BYTES as u64,
            memories,
        })
    }
}

fn current_binding(
    id: u8,
    declarations: Option<&crate::SealedDeclarationSnapshot>,
) -> (AllocationBinding, Option<AllocationRangeClaim>) {
    let mut binding = AllocationBinding::Unknown;
    let mut range_claim = None;
    if let Some(snapshot) = declarations {
        if let Some(registration) = snapshot
            .registered_declarations()
            .iter()
            .find(|registration| registration.declaration().slot().memory_manager_id() == Ok(id))
        {
            binding = AllocationBinding::Current {
                stable_key: registration.declaration().stable_key().as_str().to_string(),
                owner: registration.authority().to_string(),
            };
        }
        if let Some(claim) = snapshot
            .range_authority()
            .authorities()
            .iter()
            .find(|claim| claim.range().contains(id))
        {
            range_claim = Some(AllocationRangeClaim {
                authority: claim.authority().to_string(),
                mode: claim.mode(),
            });
        }
    }
    if id == MEMORY_MANAGER_LEDGER_ID {
        binding = AllocationBinding::Ledger {
            stable_key: IC_MEMORY_LEDGER_STABLE_KEY.to_string(),
            owner: IC_MEMORY_AUTHORITY_OWNER.to_string(),
        };
    }
    (binding, range_claim)
}
