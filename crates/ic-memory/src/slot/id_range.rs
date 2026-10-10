use super::memory_manager::{
    MEMORY_MANAGER_INVALID_ID, MEMORY_MANAGER_MAX_ID, MEMORY_MANAGER_MIN_ID,
};
use serde::{Deserialize, Deserializer, Serialize, de::Error as _};

///
/// MemoryManagerIdRange
///
/// Inclusive range of usable `MemoryManager` virtual memory IDs.
/// Construction and deserialization reject reversed bounds and sentinel ID 255.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct MemoryManagerIdRange {
    start: u8,
    end: u8,
}

impl<'de> Deserialize<'de> for MemoryManagerIdRange {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename = "MemoryManagerIdRange", deny_unknown_fields)]
        struct Bounds {
            start: u8,
            end: u8,
        }

        let bounds = Bounds::deserialize(deserializer)?;
        Self::new(bounds.start, bounds.end).map_err(D::Error::custom)
    }
}

impl MemoryManagerIdRange {
    /// Construct and validate an inclusive `MemoryManager` ID range.
    pub const fn new(start: u8, end: u8) -> Result<Self, MemoryManagerRangeError> {
        if start > end {
            return Err(MemoryManagerRangeError::InvalidRange { start, end });
        }
        // Ordered bounds make a usable end sufficient to exclude the sentinel.
        if end == MEMORY_MANAGER_INVALID_ID {
            return Err(MemoryManagerRangeError::InvalidMemoryManagerId { id: end });
        }
        Ok(Self { start, end })
    }

    /// Return the full usable `MemoryManager` ID range.
    #[must_use]
    pub const fn all_usable() -> Self {
        Self {
            start: MEMORY_MANAGER_MIN_ID,
            end: MEMORY_MANAGER_MAX_ID,
        }
    }

    /// Return true when `id` is inside this inclusive range.
    #[must_use]
    pub const fn contains(&self, id: u8) -> bool {
        id >= self.start && id <= self.end
    }

    /// First usable ID in the range.
    #[must_use]
    pub const fn start(&self) -> u8 {
        self.start
    }

    /// Last usable ID in the range.
    #[must_use]
    pub const fn end(&self) -> u8 {
        self.end
    }
}

///
/// MemoryManagerRangeError
///
/// Invalid `MemoryManager` virtual memory ID range.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, thiserror::Error, PartialEq)]
pub enum MemoryManagerRangeError {
    /// Range bounds are reversed.
    #[error("MemoryManager ID range is invalid: start={start} end={end}")]
    InvalidRange {
        /// Requested first ID.
        start: u8,
        /// Requested last ID.
        end: u8,
    },
    /// ID 255 is the unallocated-bucket sentinel.
    #[error("MemoryManager ID {id} is not a usable allocation slot")]
    InvalidMemoryManagerId {
        /// Invalid MemoryManager ID.
        id: u8,
    },
}
