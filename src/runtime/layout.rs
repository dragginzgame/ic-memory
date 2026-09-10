//! Read-only adapter for the documented ic-stable-structures 0.7.2 MGR V1
//! layout. No ledger or payload reads. Keep the dependency pinned when changing
//! this adapter; layout changes require a new review, not a fallback decoder.
use super::RuntimeConstructionError;
use crate::WASM_PAGE_SIZE_BYTES;
use ic_stable_structures::Memory;

pub(super) const IDS: usize = 255;
pub(super) const BUCKETS: usize = 32_768;
pub(super) const HEADER_BYTES: usize = 40 + IDS * 8;
pub(super) const METADATA_BYTES: usize = HEADER_BYTES + BUCKETS;

///
/// MemoryManagerLayoutError
///
/// Invalid or unsupported persisted manager metadata. No writes are performed
/// when reporting these failures. Memory implementations must obey [`Memory`].
///

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum MemoryManagerLayoutError {
    /// This adapter supports the little-endian IC/current native layout only.
    #[error("unsupported big-endian manager layout")]
    UnsupportedByteOrder,
    /// The persisted bucket size is zero.
    #[error("persisted bucket size is zero")]
    ZeroBucketSize,
    /// Reserved header fields are not recognized by this adapter.
    #[error("nonzero reserved manager header bytes")]
    ReservedHeader,
    /// The allocated count exceeds the finite bucket table.
    #[error("allocated bucket count {count} exceeds 32768")]
    BucketCount { count: u16 },
    /// Allocated buckets must be a dense prefix and all later entries unused.
    #[error("invalid bucket table entry at index {index}")]
    BucketTable { index: u16 },
    /// Virtual extent disagrees with the number of assigned buckets.
    #[error("virtual extent and bucket count disagree for memory ID {id}")]
    VirtualExtent { id: u8 },
    /// Backing memory does not cover every assigned bucket.
    #[error("physical extent {physical_pages} pages is below assigned end {required_pages}")]
    TruncatedBacking {
        physical_pages: u64,
        required_pages: u64,
    },
    /// Backing size cannot be represented in bytes.
    #[error("backing memory extent overflows bytes")]
    ExtentOverflow,
    /// A live manager's cached state differs from persisted metadata.
    #[error("persisted manager metadata differs from runtime authority")]
    RuntimeMismatch,
}

pub(super) struct Layout {
    pub physical_pages: u64,
    pub bucket_pages: u16,
    pub allocated_buckets: u16,
    pub pages: [u64; IDS],
    pub buckets: [u16; IDS],
}

pub(super) fn read<M: Memory>(memory: &M) -> Result<Layout, RuntimeConstructionError> {
    if cfg!(target_endian = "big") {
        return Err(MemoryManagerLayoutError::UnsupportedByteOrder.into());
    }
    let physical_pages = memory.size();
    physical_pages
        .checked_mul(WASM_PAGE_SIZE_BYTES)
        .ok_or(MemoryManagerLayoutError::ExtentOverflow)?;
    // The caller handles empty memory. Even one page covers all metadata.
    if physical_pages == 0 {
        return Err(MemoryManagerLayoutError::TruncatedBacking {
            physical_pages,
            required_pages: 1,
        }
        .into());
    }
    let mut header = [0; HEADER_BYTES];
    memory.read(0, &mut header);
    let observed_magic = [header[0], header[1], header[2]];
    if observed_magic != *b"MGR" {
        return Err(RuntimeConstructionError::ForeignMemory { observed_magic });
    }
    if header[3] != 1 {
        return Err(RuntimeConstructionError::UnsupportedMemoryManagerVersion {
            observed: header[3],
            supported: 1,
        });
    }
    if header[8..40].iter().any(|byte| *byte != 0) {
        return Err(MemoryManagerLayoutError::ReservedHeader.into());
    }
    let allocated_buckets = u16::from_le_bytes([header[4], header[5]]);
    let bucket_pages = u16::from_le_bytes([header[6], header[7]]);
    if bucket_pages == 0 {
        return Err(MemoryManagerLayoutError::ZeroBucketSize.into());
    }
    if usize::from(allocated_buckets) > BUCKETS {
        return Err(MemoryManagerLayoutError::BucketCount {
            count: allocated_buckets,
        }
        .into());
    }
    let required_pages = 1 + u64::from(allocated_buckets) * u64::from(bucket_pages);
    if physical_pages < required_pages {
        return Err(MemoryManagerLayoutError::TruncatedBacking {
            physical_pages,
            required_pages,
        }
        .into());
    }
    // Fixed allocation and read bound, independent of physical size/history.
    let mut table = vec![0; BUCKETS];
    memory.read(HEADER_BYTES as u64, &mut table);
    let mut buckets = [0_u16; IDS];
    for (index, id) in (0_u16..32_768).zip(table) {
        if (index < allocated_buckets) != (id != 255) {
            return Err(MemoryManagerLayoutError::BucketTable { index }.into());
        }
        if id != 255 {
            buckets[usize::from(id)] += 1;
        }
    }
    let mut pages = [0; IDS];
    for (id, value) in (0_u8..255).zip(&mut pages) {
        let offset = 40 + usize::from(id) * 8;
        *value = u64::from_le_bytes(header[offset..offset + 8].try_into().expect("eight bytes"));
        if value.div_ceil(u64::from(bucket_pages)) != u64::from(buckets[usize::from(id)]) {
            return Err(MemoryManagerLayoutError::VirtualExtent { id }.into());
        }
    }
    Ok(Layout {
        physical_pages,
        bucket_pages,
        allocated_buckets,
        pages,
        buckets,
    })
}
