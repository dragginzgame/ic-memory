use super::{MemoryManagerLayoutError, MemoryRuntime, RuntimeDiagnosticError, layout};
use crate::{DiagnosticMemorySize, MEMORY_MANAGER_LEDGER_ID, WASM_PAGE_SIZE_BYTES};
use ic_stable_structures::Memory;
use serde::Serialize;

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
    pub virtual_extent: DiagnosticMemorySize,
    pub allocated_buckets: u16,
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
    /// Numeric accounting shared with the summary-only report.
    pub summary: MemoryAllocationSummary,
    pub memories: Vec<MemoryAllocation>,
}

///
/// MemoryBindingSummary
///
/// Numeric bucket allocation and slack for one binding provenance partition.
/// These values measure capacity, never payload occupancy.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct MemoryBindingSummary {
    pub allocated_buckets: u16,
    pub virtual_extent: DiagnosticMemorySize,
}

///
/// MemoryAllocationSummary
///
/// Bounded numeric allocation accounting without per-ID rows, stable keys,
/// owners or pool metadata. Uses the same validated metadata and conservation
/// equations as [`MemoryAllocations`]. Binding groups describe current runtime
/// provenance; unknown includes omitted or retired keys without reading history.
/// No payload occupancy is available.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct MemoryAllocationSummary {
    /// In-memory committed generation; unavailable before bootstrap.
    pub current_generation: Option<u64>,
    /// Actual persisted bucket size, never a requested/default assumption.
    pub bucket_size_pages: u16,
    pub allocated_buckets: u16,
    pub physical_extent: DiagnosticMemorySize,
    pub virtual_extent: DiagnosticMemorySize,
    pub current_binding: MemoryBindingSummary,
    pub ledger_binding: MemoryBindingSummary,
    pub unknown_binding: MemoryBindingSummary,
}

impl MemoryAllocation {
    /// Bucket capacity assigned to this ID under the measured bucket geometry.
    #[must_use]
    pub const fn allocated_bytes(&self, bucket_size_pages: u16) -> u64 {
        self.allocated_buckets as u64 * bucket_size_pages as u64 * WASM_PAGE_SIZE_BYTES
    }

    /// Assigned capacity beyond this ID's virtual extent, not live payload occupancy.
    /// Use the containing report's measured `bucket_size_pages`.
    #[must_use]
    pub const fn bucket_slack_bytes(&self, bucket_size_pages: u16) -> u64 {
        self.allocated_bytes(bucket_size_pages) - self.virtual_extent.bytes()
    }
}

impl MemoryBindingSummary {
    /// Assigned capacity for this provenance group under the measured geometry.
    #[must_use]
    pub const fn allocated_bytes(self, bucket_size_pages: u16) -> u64 {
        self.allocated_buckets as u64 * bucket_size_pages as u64 * WASM_PAGE_SIZE_BYTES
    }

    /// Assigned capacity beyond the group's virtual extents.
    /// Use the containing report's measured `bucket_size_pages`.
    #[must_use]
    pub const fn bucket_slack_bytes(self, bucket_size_pages: u16) -> u64 {
        self.allocated_bytes(bucket_size_pages) - self.virtual_extent.bytes()
    }
}

impl MemoryAllocationSummary {
    /// Supported, validated manager layout identity.
    pub const MANAGER_LAYOUT_VERSION: u8 = 1;
    /// Finite table capacity, excluding metadata and backing limits.
    pub const BUCKET_CAPACITY: u32 = layout::BUCKET_CAPACITY as u32;
    /// The complete metadata page, including header, table and padding.
    pub const MANAGER_METADATA_BYTES: u64 = WASM_PAGE_SIZE_BYTES;
    /// Fixed manager header bytes.
    pub const MANAGER_HEADER_BYTES: u64 = layout::HEADER_BYTES as u64;
    /// Fixed bucket ownership table bytes.
    pub const MANAGER_BUCKET_TABLE_BYTES: u64 = layout::BUCKETS as u64;
    /// Remaining metadata-page bytes.
    pub const MANAGER_PADDING_BYTES: u64 = WASM_PAGE_SIZE_BYTES - layout::METADATA_BYTES as u64;
    /// Exact metadata read budget, independent of ledger history and payload.
    pub const METADATA_BYTES_READ: u64 = layout::METADATA_BYTES as u64;
    /// Number of usable IDs, including empty IDs and governance.
    pub const MEMORIES_MEASURED: u16 = crate::MEMORY_MANAGER_INVALID_ID as u16;

    /// Bytes in each measured bucket.
    #[must_use]
    pub const fn bucket_size_bytes(&self) -> u64 {
        self.bucket_size_pages as u64 * WASM_PAGE_SIZE_BYTES
    }

    /// Available bucket ownership entries, not backing-memory availability.
    #[must_use]
    pub const fn remaining_buckets(&self) -> u32 {
        Self::BUCKET_CAPACITY - self.allocated_buckets as u32
    }

    /// Maximum assigned bucket capacity, excluding metadata.
    #[must_use]
    pub const fn maximum_bucket_bytes(&self) -> u64 {
        Self::BUCKET_CAPACITY as u64 * self.bucket_size_bytes()
    }

    /// Assigned bucket capacity across all IDs.
    #[must_use]
    pub const fn allocated_bucket_bytes(&self) -> u64 {
        self.allocated_buckets as u64 * self.bucket_size_bytes()
    }

    /// Assigned capacity beyond all virtual extents.
    #[must_use]
    pub const fn bucket_slack_bytes(&self) -> u64 {
        self.allocated_bucket_bytes() - self.virtual_extent.bytes()
    }

    /// Assigned bytes belonging to current declarations or governance.
    #[must_use]
    pub const fn known_binding_bytes(&self) -> u64 {
        (self.current_binding.allocated_buckets as u64
            + self.ledger_binding.allocated_buckets as u64)
            * self.bucket_size_bytes()
    }

    /// Assigned bytes with no current declaration binding.
    #[must_use]
    pub const fn unknown_binding_bytes(&self) -> u64 {
        self.unknown_binding.allocated_bytes(self.bucket_size_pages)
    }

    /// Backing bytes after metadata and the assigned bucket region.
    #[must_use]
    pub const fn unmanaged_bytes(&self) -> u64 {
        self.physical_extent.bytes() - Self::MANAGER_METADATA_BYTES - self.allocated_bucket_bytes()
    }
}

impl<M: Memory> MemoryRuntime<M> {
    /// Measure all IDs with a fixed metadata read and bounded current bindings.
    ///
    /// Reads at most 34,848 backing bytes. Never initializes stores, decodes the ledger, writes,
    /// grows memory, or advances a generation. Available before bootstrap;
    /// current provenance totals are then unavailable. Host labels and pool
    /// eligibility are projected separately from borrowed runtime bindings.
    pub fn memory_allocations(&self) -> Result<MemoryAllocations, RuntimeDiagnosticError> {
        let (measured, summary) = self.measure_allocations()?;
        let mut memories = Vec::with_capacity(layout::IDS);
        for id in 0..255_u8 {
            let index = usize::from(id);
            let virtual_extent = DiagnosticMemorySize::from_wasm_pages(measured.pages[index]);
            memories.push(MemoryAllocation {
                memory_manager_id: id,
                virtual_extent,
                allocated_buckets: measured.buckets[index],
            });
        }
        Ok(MemoryAllocations { summary, memories })
    }

    /// Measure numeric totals and binding partitions without constructing per-ID
    /// rows or copying keys, owners or pool metadata. Reads at most 34,848 metadata
    /// bytes; no ledger history, writes, growth, or generation changes occur.
    pub fn memory_allocation_summary(
        &self,
    ) -> Result<MemoryAllocationSummary, RuntimeDiagnosticError> {
        self.measure_allocations().map(|(_, summary)| summary)
    }

    fn measure_allocations(
        &self,
    ) -> Result<(layout::Layout, MemoryAllocationSummary), RuntimeDiagnosticError> {
        let declarations = self.committed_allocations().ok();
        let measured = layout::read(self.growth.backing.as_ref())?;
        let live_buckets = self
            .growth
            .allocated_buckets
            .try_borrow()
            .map_err(|_| RuntimeDiagnosticError::ReentrantAccess)?;
        if measured.bucket_pages != self.growth.bucket_size_pages
            || measured.allocated_buckets != *live_buckets
        {
            return Err(super::RuntimeConstructionError::Layout(
                MemoryManagerLayoutError::RuntimeMismatch,
            )
            .into());
        }
        let mut current = [false; layout::IDS];
        if let Some(snapshot) = declarations {
            for declaration in snapshot {
                let id = declaration.slot().id();
                current[usize::from(id)] = true;
            }
        }
        let mut groups = [MemoryBindingSummary::default(); 3];
        let mut total_pages = 0;
        for id in 0..255_u8 {
            let index = usize::from(id);
            if self.memory_size_pages(id) != measured.pages[index] {
                return Err(super::RuntimeConstructionError::Layout(
                    MemoryManagerLayoutError::RuntimeMismatch,
                )
                .into());
            }
            let group = if id == MEMORY_MANAGER_LEDGER_ID {
                1
            } else if current[index] {
                0
            } else {
                2
            };
            groups[group].allocated_buckets += measured.buckets[index];
            groups[group].virtual_extent.wasm_pages += measured.pages[index];
            total_pages += measured.pages[index];
        }
        let physical_extent = DiagnosticMemorySize::from_wasm_pages(measured.physical_pages);
        let virtual_extent = DiagnosticMemorySize::from_wasm_pages(total_pages);
        let summary = MemoryAllocationSummary {
            current_generation: self.committed_generation().ok(),
            bucket_size_pages: measured.bucket_pages,
            allocated_buckets: measured.allocated_buckets,
            physical_extent,
            virtual_extent,
            current_binding: groups[0],
            ledger_binding: groups[1],
            unknown_binding: groups[2],
        };
        Ok((measured, summary))
    }
}
