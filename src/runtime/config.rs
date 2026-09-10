use super::RuntimeConstructionError;

///
/// MemoryManagerConfig
///
/// Immutable bucket allocation policy for a runtime. The default remains 128
/// Wasm pages (8 MiB). Smaller sizes trade less rounding slack for a lower
/// 32,768-bucket capacity and more frequent growth. This policy is separate
/// from application allocation authorization and never grants memory access.
///
/// Explicit construction checks existing memory for an exact match before any
/// effects. This is a same-release setting, not a migration or shrink operation.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryManagerConfig {
    bucket_size_pages: u16,
}

impl Default for MemoryManagerConfig {
    fn default() -> Self {
        Self {
            bucket_size_pages: 128,
        }
    }
}

impl MemoryManagerConfig {
    pub(super) const fn from_validated(bucket_size_pages: u16) -> Self {
        Self { bucket_size_pages }
    }

    /// Validate a nonzero bucket size in Wasm pages before construction effects.
    pub const fn new(bucket_size_pages: u16) -> Result<Self, RuntimeConstructionError> {
        if bucket_size_pages == 0 {
            return Err(RuntimeConstructionError::InvalidBucketSize);
        }
        Ok(Self { bucket_size_pages })
    }

    /// Return the requested bucket size in pages.
    #[must_use]
    pub const fn bucket_size_pages(self) -> u16 {
        self.bucket_size_pages
    }
}
