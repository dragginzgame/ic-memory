use crate::{Memory, VectorMemory, WASM_PAGE_SIZE_BYTES};

///
/// SparseMemoryImageError
///
/// Refused fixture bytes or backing allocation. No allocation authority is recovered.
///

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum SparseMemoryImageError {
    /// Page count exceeds the caller's ceiling or the host address range.
    #[error("fixture page count {pages} exceeds limit {maximum} or host address range")]
    PageLimit {
        /// Requested pages.
        pages: u64,
        /// Caller-selected maximum pages.
        maximum: u64,
    },
    /// A run cannot fit wholly inside the declared image.
    #[error("fixture run {run} is outside the declared image")]
    RunOutOfBounds {
        /// Zero-based run index.
        run: usize,
    },
    /// A run is not an even number of ASCII hexadecimal digits.
    #[error("fixture run {run} is not hexadecimal bytes")]
    InvalidHex {
        /// Zero-based run index.
        run: usize,
    },
    /// VectorMemory refused the admitted growth request.
    #[error("fixture backing refused growth")]
    GrowthFailed,
}

/// Expand sparse hexadecimal fixture runs into fresh zero-filled backing memory.
///
/// `pages` and `maximum_pages` are counts of 64 KiB Wasm pages. All run lengths,
/// offsets and hex digits are checked before allocating backing. Runs are written
/// in order; later runs replace earlier bytes if they overlap. Gaps stay zero.
/// The caller owns the fixture's provenance, JSON/schema, payload admission and
/// page ceiling. This creates only raw test backing; it never opens a runtime,
/// recovers authority, converts stored formats or modifies an existing image.
///
/// # Panics
/// Host allocation failure retains VectorMemory's allocation behavior.
pub fn memory_from_sparse_hex(
    pages: u64,
    runs: &[(u64, String)],
    maximum_pages: u64,
) -> Result<VectorMemory, SparseMemoryImageError> {
    let page_error = SparseMemoryImageError::PageLimit {
        pages,
        maximum: maximum_pages,
    };
    if pages > maximum_pages {
        return Err(page_error);
    }
    let bytes = pages
        .checked_mul(WASM_PAGE_SIZE_BYTES)
        .filter(|&bytes| usize::try_from(bytes).is_ok())
        .ok_or(page_error)?;
    for (run, (offset, hex)) in runs.iter().enumerate() {
        let length = u64::try_from(hex.len() / 2)
            .map_err(|_| SparseMemoryImageError::RunOutOfBounds { run })?;
        if offset.checked_add(length).is_none_or(|end| end > bytes) {
            return Err(SparseMemoryImageError::RunOutOfBounds { run });
        }
        if !hex.len().is_multiple_of(2) || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(SparseMemoryImageError::InvalidHex { run });
        }
    }
    let memory = VectorMemory::default();
    if memory.grow(pages) < 0 {
        return Err(SparseMemoryImageError::GrowthFailed);
    }
    for (offset, hex) in runs {
        let bytes: Vec<_> = (0..hex.len())
            .step_by(2)
            .map(|index| {
                u8::from_str_radix(&hex[index..index + 2], 16).expect("validated fixture hex")
            })
            .collect();
        memory.write(*offset, &bytes);
    }
    Ok(memory)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sparse_runs_preserve_gaps_page_boundaries_and_write_order() {
        let runs = [
            (3, "0123".into()),
            (65_535, "aB0C".into()),
            (65_536, "ef".into()),
        ];
        let memory = memory_from_sparse_hex(2, &runs, 2).unwrap();
        let mut expected = vec![0; 2 * 65_536];
        expected[3..5].copy_from_slice(&[1, 35]);
        expected[65_535..65_537].copy_from_slice(&[171, 239]);
        assert_eq!(*memory.borrow(), expected);
        assert_eq!(memory.size(), 2);
    }

    #[test]
    fn page_limits_refuse_before_backing_allocation() {
        for (pages, maximum) in [(1, 0), (3, 2), (u64::MAX, u64::MAX)] {
            assert_eq!(
                memory_from_sparse_hex(pages, &[], maximum).unwrap_err(),
                SparseMemoryImageError::PageLimit { pages, maximum }
            );
        }
    }

    #[test]
    fn invalid_runs_report_the_offending_index() {
        for (offset, hex, expected) in [
            (0, "a", SparseMemoryImageError::InvalidHex { run: 1 }),
            (0, "zz", SparseMemoryImageError::InvalidHex { run: 1 }),
            (0, "é", SparseMemoryImageError::InvalidHex { run: 1 }),
            (
                65_535,
                "aabb",
                SparseMemoryImageError::RunOutOfBounds { run: 1 },
            ),
            (
                65_537,
                "",
                SparseMemoryImageError::RunOutOfBounds { run: 1 },
            ),
            (
                u64::MAX,
                "00",
                SparseMemoryImageError::RunOutOfBounds { run: 1 },
            ),
        ] {
            let runs = [(0, "00".into()), (offset, hex.into())];
            assert_eq!(memory_from_sparse_hex(1, &runs, 1).unwrap_err(), expected);
        }
    }

    #[test]
    fn empty_images_and_empty_boundary_runs_are_admitted() {
        assert_eq!(memory_from_sparse_hex(0, &[], 0).unwrap().size(), 0);
        assert_eq!(
            memory_from_sparse_hex(1, &[(65_536, String::new())], 1)
                .unwrap()
                .size(),
            1
        );
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn substrate_growth_refusal_is_returned() {
        let pages = u64::try_from(i64::MAX).unwrap() / WASM_PAGE_SIZE_BYTES + 1;
        assert_eq!(
            memory_from_sparse_hex(pages, &[], pages).unwrap_err(),
            SparseMemoryImageError::GrowthFailed
        );
    }
}
