use serde::{Deserialize, Serialize};

const DIAGNOSTIC_STRING_MAX_BYTES: usize = 256;

/// Substrate identifier for `ic-stable-structures::MemoryManager` slots.
pub const MEMORY_MANAGER_SUBSTRATE: &str = "ic-stable-structures.memory_manager";

/// Descriptor version for current `MemoryManagerId` slots.
pub const MEMORY_MANAGER_DESCRIPTOR_VERSION: u32 = 1;

/// First usable `MemoryManager` virtual memory ID.
pub const MEMORY_MANAGER_MIN_ID: u8 = 0;

/// Last usable `MemoryManager` virtual memory ID.
pub const MEMORY_MANAGER_MAX_ID: u8 = 254;

/// `MemoryManager` unallocated-bucket sentinel. This is not a usable slot.
pub const MEMORY_MANAGER_INVALID_ID: u8 = u8::MAX;

/// Stable-key namespace prefix reserved for `ic-memory` allocation-governance infrastructure.
pub const IC_MEMORY_STABLE_KEY_PREFIX: &str = "ic_memory.";

/// Diagnostic owner label for `ic-memory` allocation-governance infrastructure.
pub const IC_MEMORY_AUTHORITY_OWNER: &str = "ic-memory";

/// Diagnostic purpose for the `ic-memory` allocation-governance authority range.
pub const IC_MEMORY_AUTHORITY_PURPOSE: &str = "ic-memory allocation-governance authority";

/// Stable key of the allocation ledger when backed by the current MemoryManager substrate.
pub const IC_MEMORY_LEDGER_STABLE_KEY: &str = "ic_memory.ledger.v1";

/// Diagnostic label of the allocation ledger when backed by the current MemoryManager substrate.
pub const IC_MEMORY_LEDGER_LABEL: &str = "MemoryLayoutLedger";

/// MemoryManager ID used by the allocation ledger in the current MemoryManager substrate.
pub const MEMORY_MANAGER_LEDGER_ID: u8 = MEMORY_MANAGER_MIN_ID;

/// Last MemoryManager ID reserved for `ic-memory` governance in the current substrate.
pub const MEMORY_MANAGER_GOVERNANCE_MAX_ID: u8 = 9;

/// Return true when `stable_key` belongs to the `ic-memory` namespace.
#[must_use]
pub fn is_ic_memory_stable_key(stable_key: &str) -> bool {
    stable_key.starts_with(IC_MEMORY_STABLE_KEY_PREFIX)
}

/// MemoryManager range reserved for `ic-memory` governance in the current substrate.
#[must_use]
pub const fn memory_manager_governance_range() -> MemoryManagerIdRange {
    MemoryManagerIdRange {
        start: MEMORY_MANAGER_MIN_ID,
        end: MEMORY_MANAGER_GOVERNANCE_MAX_ID,
    }
}

///
/// AllocationSlot
///
/// Durable physical allocation identity interpreted by a storage substrate.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum AllocationSlot {
    /// `ic-stable-structures::MemoryManager` virtual memory ID.
    MemoryManagerId(u8),
    /// Named substrate partition.
    NamedPartition(String),
    /// Forward-compatible custom slot descriptor.
    Custom {
        /// Substrate-defined slot kind.
        kind: String,
        /// Slot descriptor version.
        version: u32,
        /// Canonical substrate-defined value.
        value: Vec<u8>,
    },
}

///
/// AllocationSlotDescriptor
///
/// Encoded allocation slot persisted in the ledger.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct AllocationSlotDescriptor {
    /// Durable allocation slot.
    pub slot: AllocationSlot,
    /// Substrate identifier that interprets the slot.
    pub substrate: String,
    /// Descriptor encoding version.
    pub descriptor_version: u32,
}

impl AllocationSlotDescriptor {
    /// Construct a descriptor for a usable `MemoryManager` virtual memory ID.
    pub fn memory_manager(id: u8) -> Result<Self, MemoryManagerSlotError> {
        validate_memory_manager_id(id)?;
        Ok(Self::memory_manager_unchecked(id))
    }

    /// Construct a descriptor for a `MemoryManager` virtual memory ID without validating it.
    ///
    /// This exists for tests, diagnostics, and decoding untrusted durable DTOs.
    /// Use [`Self::memory_manager`] when constructing new allocation
    /// declarations.
    #[must_use]
    pub fn memory_manager_unchecked(id: u8) -> Self {
        Self {
            slot: AllocationSlot::MemoryManagerId(id),
            substrate: MEMORY_MANAGER_SUBSTRATE.to_string(),
            descriptor_version: MEMORY_MANAGER_DESCRIPTOR_VERSION,
        }
    }

    /// Construct a descriptor for a usable `MemoryManager` virtual memory ID.
    pub fn memory_manager_checked(id: u8) -> Result<Self, MemoryManagerSlotError> {
        Self::memory_manager(id)
    }

    /// Return the usable `MemoryManager` virtual memory ID represented by this descriptor.
    pub fn memory_manager_id(&self) -> Result<u8, MemoryManagerSlotError> {
        if self.substrate != MEMORY_MANAGER_SUBSTRATE {
            return Err(MemoryManagerSlotError::UnsupportedSubstrate {
                substrate: self.substrate.clone(),
            });
        }
        if self.descriptor_version != MEMORY_MANAGER_DESCRIPTOR_VERSION {
            return Err(MemoryManagerSlotError::UnsupportedDescriptorVersion {
                version: self.descriptor_version,
            });
        }

        let AllocationSlot::MemoryManagerId(id) = self.slot else {
            return Err(MemoryManagerSlotError::UnsupportedSlot);
        };

        validate_memory_manager_id(id)?;
        Ok(id)
    }
}

///
/// MemoryManagerSlotError
///
/// Invalid or unsupported `MemoryManager` allocation slot descriptor.
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum MemoryManagerSlotError {
    /// Descriptor is not a `MemoryManagerId` slot.
    #[error("allocation slot is not a MemoryManager virtual memory ID")]
    UnsupportedSlot,
    /// Descriptor is attached to another substrate.
    #[error("allocation slot substrate '{substrate}' is not supported as a MemoryManager slot")]
    UnsupportedSubstrate {
        /// Unsupported substrate identifier.
        substrate: String,
    },
    /// Descriptor uses an unsupported encoding version.
    #[error("MemoryManager slot descriptor version {version} is unsupported")]
    UnsupportedDescriptorVersion {
        /// Unsupported descriptor version.
        version: u32,
    },
    /// ID 255 is the unallocated-bucket sentinel.
    #[error("MemoryManager ID {id} is not a usable allocation slot")]
    InvalidMemoryManagerId {
        /// Invalid MemoryManager ID.
        id: u8,
    },
}

///
/// MemoryManagerIdRange
///
/// Inclusive range of usable `MemoryManager` virtual memory IDs.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MemoryManagerIdRange {
    start: u8,
    end: u8,
}

impl MemoryManagerIdRange {
    /// Construct and validate an inclusive `MemoryManager` ID range.
    pub const fn new(start: u8, end: u8) -> Result<Self, MemoryManagerRangeError> {
        if start > end {
            return Err(MemoryManagerRangeError::InvalidRange { start, end });
        }
        if start == MEMORY_MANAGER_INVALID_ID {
            return Err(MemoryManagerRangeError::InvalidMemoryManagerId { id: start });
        }
        if end == MEMORY_MANAGER_INVALID_ID {
            return Err(MemoryManagerRangeError::InvalidMemoryManagerId { id: end });
        }
        Ok(Self { start, end })
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

///
/// MemoryManagerRangeMode
///
/// Diagnostic policy mode for a `MemoryManager` authority range.
///
/// These modes describe policy authority, not durable allocation state.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum MemoryManagerRangeMode {
    /// Range is reserved for authority-owned framework or infrastructure use.
    ///
    /// Reserved does not mean every ID in the range has been allocated.
    Reserved,
    /// Range is allowed for authority-governed application allocation use.
    ///
    /// Allowed does not allocate any ID in the range.
    Allowed,
}

///
/// MemoryManagerAuthorityRecord
///
/// Ordered diagnostic authority record for a `MemoryManager` ID range.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MemoryManagerAuthorityRecord {
    /// Inclusive range governed by this authority.
    pub range: MemoryManagerIdRange,
    /// Stable printable ASCII authority identifier.
    pub authority: String,
    /// Policy mode for this authority range.
    pub mode: MemoryManagerRangeMode,
    /// Optional stable printable ASCII diagnostic purpose.
    pub purpose: Option<String>,
}

///
/// MemoryManagerRangeAuthority
///
/// Substrate-specific range authority policy helper for `MemoryManager` IDs.
///
/// This helper records policy and diagnostic authority ranges only. It never
/// mutates the allocation ledger, and it does not allocate or reserve durable
/// stable-memory slots. Durable allocation remains the generic ledger mapping
/// from stable key to allocation slot.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct MemoryManagerRangeAuthority {
    authorities: Vec<MemoryManagerAuthorityRecord>,
}

impl MemoryManagerRangeAuthority {
    /// Create an empty `MemoryManager` range authority policy.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            authorities: Vec::new(),
        }
    }

    /// Build a range authority from diagnostic records.
    ///
    /// Records are validated with the same rules as the builder methods and
    /// stored in ascending range order.
    pub fn from_records(
        records: Vec<MemoryManagerAuthorityRecord>,
    ) -> Result<Self, MemoryManagerRangeAuthorityError> {
        let mut authority = Self::new();
        for record in records {
            authority = authority.insert_record(record)?;
        }
        Ok(authority)
    }

    /// Add a reserved authority range.
    ///
    /// Reserved is a policy authority mode. It does not allocate every ID in
    /// the range and does not write to the allocation ledger.
    pub fn reserve(
        self,
        range: MemoryManagerIdRange,
        authority: impl Into<String>,
    ) -> Result<Self, MemoryManagerRangeAuthorityError> {
        self.reserve_with_purpose(range, authority, None)
    }

    /// Add a reserved authority range with a diagnostic purpose.
    ///
    /// Reserved is a policy authority mode. It does not allocate every ID in
    /// the range and does not write to the allocation ledger.
    pub fn reserve_with_purpose(
        self,
        range: MemoryManagerIdRange,
        authority: impl Into<String>,
        purpose: Option<String>,
    ) -> Result<Self, MemoryManagerRangeAuthorityError> {
        self.insert(range, authority, MemoryManagerRangeMode::Reserved, purpose)
    }

    /// Add an allowed authority range.
    ///
    /// Allowed is a policy authority mode. It does not allocate any ID in the
    /// range and does not write to the allocation ledger.
    pub fn allow(
        self,
        range: MemoryManagerIdRange,
        authority: impl Into<String>,
    ) -> Result<Self, MemoryManagerRangeAuthorityError> {
        self.allow_with_purpose(range, authority, None)
    }

    /// Add an allowed authority range with a diagnostic purpose.
    ///
    /// Allowed is a policy authority mode. It does not allocate any ID in the
    /// range and does not write to the allocation ledger.
    pub fn allow_with_purpose(
        self,
        range: MemoryManagerIdRange,
        authority: impl Into<String>,
        purpose: Option<String>,
    ) -> Result<Self, MemoryManagerRangeAuthorityError> {
        self.insert(range, authority, MemoryManagerRangeMode::Allowed, purpose)
    }

    /// Validate that `slot` belongs to `expected_authority`.
    pub fn validate_slot_authority(
        &self,
        slot: &AllocationSlotDescriptor,
        expected_authority: &str,
    ) -> Result<&MemoryManagerAuthorityRecord, MemoryManagerRangeAuthorityError> {
        let id = slot
            .memory_manager_id()
            .map_err(MemoryManagerRangeAuthorityError::Slot)?;
        self.validate_id_authority(id, expected_authority)
    }

    /// Validate that `slot` belongs to `expected_authority` with `expected_mode`.
    pub fn validate_slot_authority_mode(
        &self,
        slot: &AllocationSlotDescriptor,
        expected_authority: &str,
        expected_mode: MemoryManagerRangeMode,
    ) -> Result<&MemoryManagerAuthorityRecord, MemoryManagerRangeAuthorityError> {
        let id = slot
            .memory_manager_id()
            .map_err(MemoryManagerRangeAuthorityError::Slot)?;
        self.validate_id_authority_mode(id, expected_authority, expected_mode)
    }

    /// Validate that `id` belongs to `expected_authority`.
    pub fn validate_id_authority(
        &self,
        id: u8,
        expected_authority: &str,
    ) -> Result<&MemoryManagerAuthorityRecord, MemoryManagerRangeAuthorityError> {
        validate_diagnostic_string("expected_authority", expected_authority)?;
        let record = self.covering_record(id)?;

        if record.authority != expected_authority {
            return Err(MemoryManagerRangeAuthorityError::AuthorityMismatch {
                id,
                expected_authority: expected_authority.to_string(),
                actual_authority: record.authority.clone(),
            });
        }

        Ok(record)
    }

    /// Validate that `id` belongs to `expected_authority` with `expected_mode`.
    pub fn validate_id_authority_mode(
        &self,
        id: u8,
        expected_authority: &str,
        expected_mode: MemoryManagerRangeMode,
    ) -> Result<&MemoryManagerAuthorityRecord, MemoryManagerRangeAuthorityError> {
        let record = self.validate_id_authority(id, expected_authority)?;
        if record.mode != expected_mode {
            return Err(MemoryManagerRangeAuthorityError::ModeMismatch {
                id,
                authority: record.authority.clone(),
                expected_mode,
                actual_mode: record.mode,
            });
        }
        Ok(record)
    }

    /// Return the authority record that governs `id`, if any.
    pub fn authority_for_id(
        &self,
        id: u8,
    ) -> Result<Option<&MemoryManagerAuthorityRecord>, MemoryManagerRangeAuthorityError> {
        validate_memory_manager_id(id).map_err(MemoryManagerRangeAuthorityError::Slot)?;
        Ok(self
            .authorities
            .iter()
            .find(|record| record.range.contains(id)))
    }

    /// Ordered non-overlapping authority records.
    ///
    /// This is the stable diagnostic/export surface for the authority table.
    /// Records are returned in ascending range order and do not imply ledger
    /// allocation state.
    #[must_use]
    pub fn authorities(&self) -> &[MemoryManagerAuthorityRecord] {
        &self.authorities
    }

    /// Clone the ordered non-overlapping authority records for diagnostics.
    #[must_use]
    pub fn to_records(&self) -> Vec<MemoryManagerAuthorityRecord> {
        self.authorities.clone()
    }

    /// Validate that authority records exactly and contiguously cover `target`.
    ///
    /// All records must be inside `target`, and together they must form a
    /// gap-free partition. This checks policy table coverage only and never
    /// changes allocation ledger state.
    pub fn validate_complete_coverage(
        &self,
        target: MemoryManagerIdRange,
    ) -> Result<(), MemoryManagerRangeAuthorityError> {
        if self.authorities.is_empty() {
            return Err(MemoryManagerRangeAuthorityError::MissingCoverage {
                start: target.start(),
                end: target.end(),
            });
        }

        for record in &self.authorities {
            if record.range.start() < target.start() || record.range.end() > target.end() {
                return Err(
                    MemoryManagerRangeAuthorityError::RangeOutsideCoverageTarget {
                        start: record.range.start(),
                        end: record.range.end(),
                        target_start: target.start(),
                        target_end: target.end(),
                    },
                );
            }
        }

        let mut next_uncovered = u16::from(target.start());
        let target_end = u16::from(target.end());
        for record in &self.authorities {
            let record_start = u16::from(record.range.start());
            let record_end = u16::from(record.range.end());

            if record_start > next_uncovered {
                return Err(MemoryManagerRangeAuthorityError::MissingCoverage {
                    start: u8::try_from(next_uncovered).expect("valid MemoryManager ID"),
                    end: record.range.start() - 1,
                });
            }

            if record_end >= next_uncovered {
                next_uncovered = record_end + 1;
            }
        }

        if next_uncovered <= target_end {
            return Err(MemoryManagerRangeAuthorityError::MissingCoverage {
                start: u8::try_from(next_uncovered).expect("valid MemoryManager ID"),
                end: target.end(),
            });
        }

        Ok(())
    }

    fn insert(
        self,
        range: MemoryManagerIdRange,
        authority: impl Into<String>,
        mode: MemoryManagerRangeMode,
        purpose: Option<String>,
    ) -> Result<Self, MemoryManagerRangeAuthorityError> {
        let record = MemoryManagerAuthorityRecord {
            range,
            authority: authority.into(),
            mode,
            purpose,
        };
        self.insert_record(record)
    }

    fn insert_record(
        mut self,
        record: MemoryManagerAuthorityRecord,
    ) -> Result<Self, MemoryManagerRangeAuthorityError> {
        validate_diagnostic_string("authority", &record.authority)?;
        if let Some(purpose) = &record.purpose {
            validate_diagnostic_string("purpose", purpose)?;
        }

        for existing in &self.authorities {
            if ranges_overlap(existing.range, record.range) {
                return Err(MemoryManagerRangeAuthorityError::OverlappingRanges {
                    existing_start: existing.range.start(),
                    existing_end: existing.range.end(),
                    candidate_start: record.range.start(),
                    candidate_end: record.range.end(),
                });
            }
        }

        self.authorities.push(record);
        self.authorities.sort_by_key(|record| record.range.start());
        Ok(self)
    }

    fn covering_record(
        &self,
        id: u8,
    ) -> Result<&MemoryManagerAuthorityRecord, MemoryManagerRangeAuthorityError> {
        let Some(record) = self.authority_for_id(id)? else {
            return Err(MemoryManagerRangeAuthorityError::UnclaimedId { id });
        };
        Ok(record)
    }
}

///
/// MemoryManagerRangeAuthorityError
///
/// Invalid `MemoryManager` range authority policy.
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum MemoryManagerRangeAuthorityError {
    /// Slot descriptor is not a usable `MemoryManager` ID slot.
    #[error("{0}")]
    Slot(#[from] MemoryManagerSlotError),
    /// Authority range overlaps an existing range.
    #[error(
        "MemoryManager authority range {candidate_start}-{candidate_end} overlaps existing range {existing_start}-{existing_end}"
    )]
    OverlappingRanges {
        /// Existing range start.
        existing_start: u8,
        /// Existing range end.
        existing_end: u8,
        /// Candidate range start.
        candidate_start: u8,
        /// Candidate range end.
        candidate_end: u8,
    },
    /// Authority or purpose text failed diagnostic string validation.
    #[error("{field} {reason}")]
    InvalidDiagnosticString {
        /// Diagnostic field name.
        field: &'static str,
        /// Validation failure.
        reason: &'static str,
    },
    /// No authority range covers the requested ID.
    #[error("MemoryManager ID {id} is not covered by an authority range")]
    UnclaimedId {
        /// Unclaimed MemoryManager ID.
        id: u8,
    },
    /// Slot is governed by a different authority.
    #[error(
        "MemoryManager ID {id} belongs to authority '{actual_authority}', not '{expected_authority}'"
    )]
    AuthorityMismatch {
        /// MemoryManager ID.
        id: u8,
        /// Expected authority identifier.
        expected_authority: String,
        /// Actual authority identifier.
        actual_authority: String,
    },
    /// Slot is governed by the expected authority with a different mode.
    #[error(
        "MemoryManager ID {id} belongs to authority '{authority}' with mode {actual_mode:?}, not {expected_mode:?}"
    )]
    ModeMismatch {
        /// MemoryManager ID.
        id: u8,
        /// Authority identifier.
        authority: String,
        /// Expected authority mode.
        expected_mode: MemoryManagerRangeMode,
        /// Actual authority mode.
        actual_mode: MemoryManagerRangeMode,
    },
    /// A complete coverage target has no authority records for part of it.
    #[error("MemoryManager authority coverage is missing range {start}-{end}")]
    MissingCoverage {
        /// First missing ID.
        start: u8,
        /// Last missing ID.
        end: u8,
    },
    /// An authority record lies outside the complete coverage target.
    #[error(
        "MemoryManager authority range {start}-{end} is outside coverage target {target_start}-{target_end}"
    )]
    RangeOutsideCoverageTarget {
        /// Authority range start.
        start: u8,
        /// Authority range end.
        end: u8,
        /// Coverage target start.
        target_start: u8,
        /// Coverage target end.
        target_end: u8,
    },
}

/// Validate that a `MemoryManager` ID is usable as an allocation slot.
pub const fn validate_memory_manager_id(id: u8) -> Result<(), MemoryManagerSlotError> {
    if id == MEMORY_MANAGER_INVALID_ID {
        return Err(MemoryManagerSlotError::InvalidMemoryManagerId { id });
    }
    Ok(())
}

const fn ranges_overlap(left: MemoryManagerIdRange, right: MemoryManagerIdRange) -> bool {
    left.start() <= right.end() && right.start() <= left.end()
}

fn validate_diagnostic_string(
    field: &'static str,
    value: &str,
) -> Result<(), MemoryManagerRangeAuthorityError> {
    if value.is_empty() {
        return Err(MemoryManagerRangeAuthorityError::InvalidDiagnosticString {
            field,
            reason: "must not be empty",
        });
    }
    if value.len() > DIAGNOSTIC_STRING_MAX_BYTES {
        return Err(MemoryManagerRangeAuthorityError::InvalidDiagnosticString {
            field,
            reason: "must be at most 256 bytes",
        });
    }
    if !value.is_ascii() {
        return Err(MemoryManagerRangeAuthorityError::InvalidDiagnosticString {
            field,
            reason: "must be ASCII",
        });
    }
    if value.bytes().any(|byte| byte.is_ascii_control()) {
        return Err(MemoryManagerRangeAuthorityError::InvalidDiagnosticString {
            field,
            reason: "must not contain ASCII control characters",
        });
    }
    Ok(())
}

impl crate::policy::RangeAuthority for MemoryManagerRangeAuthority {
    type Error = MemoryManagerRangeAuthorityError;

    fn validate_slot(&self, slot: &AllocationSlotDescriptor) -> Result<(), Self::Error> {
        let id = slot.memory_manager_id()?;
        if self.authority_for_id(id)?.is_none() {
            return Err(MemoryManagerRangeAuthorityError::UnclaimedId { id });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_manager_checked_accepts_usable_ids() {
        assert!(AllocationSlotDescriptor::memory_manager_checked(MEMORY_MANAGER_MIN_ID).is_ok());
        assert!(AllocationSlotDescriptor::memory_manager_checked(MEMORY_MANAGER_MAX_ID).is_ok());
    }

    #[test]
    fn memory_manager_checked_rejects_sentinel() {
        let err = AllocationSlotDescriptor::memory_manager_checked(MEMORY_MANAGER_INVALID_ID)
            .expect_err("sentinel must fail");

        assert_eq!(
            err,
            MemoryManagerSlotError::InvalidMemoryManagerId {
                id: MEMORY_MANAGER_INVALID_ID
            }
        );
    }

    #[test]
    fn memory_manager_default_constructor_rejects_sentinel() {
        let err = AllocationSlotDescriptor::memory_manager(MEMORY_MANAGER_INVALID_ID)
            .expect_err("sentinel must fail");

        assert_eq!(
            err,
            MemoryManagerSlotError::InvalidMemoryManagerId {
                id: MEMORY_MANAGER_INVALID_ID
            }
        );
    }

    #[test]
    fn memory_manager_id_validates_descriptor_shape() {
        let slot = AllocationSlotDescriptor::memory_manager(42).expect("usable slot");
        assert_eq!(slot.memory_manager_id().expect("usable ID"), 42);

        let err = AllocationSlotDescriptor {
            slot: AllocationSlot::NamedPartition("ledger".to_string()),
            substrate: MEMORY_MANAGER_SUBSTRATE.to_string(),
            descriptor_version: MEMORY_MANAGER_DESCRIPTOR_VERSION,
        }
        .memory_manager_id()
        .expect_err("slot kind should fail");
        assert_eq!(err, MemoryManagerSlotError::UnsupportedSlot);

        let err = AllocationSlotDescriptor {
            slot: AllocationSlot::MemoryManagerId(42),
            substrate: "other".to_string(),
            descriptor_version: MEMORY_MANAGER_DESCRIPTOR_VERSION,
        }
        .memory_manager_id()
        .expect_err("substrate should fail");
        assert_eq!(
            err,
            MemoryManagerSlotError::UnsupportedSubstrate {
                substrate: "other".to_string()
            }
        );

        let err = AllocationSlotDescriptor {
            slot: AllocationSlot::MemoryManagerId(42),
            substrate: MEMORY_MANAGER_SUBSTRATE.to_string(),
            descriptor_version: MEMORY_MANAGER_DESCRIPTOR_VERSION + 1,
        }
        .memory_manager_id()
        .expect_err("version should fail");
        assert_eq!(
            err,
            MemoryManagerSlotError::UnsupportedDescriptorVersion {
                version: MEMORY_MANAGER_DESCRIPTOR_VERSION + 1
            }
        );
    }

    #[test]
    fn memory_manager_range_accepts_usable_ranges() {
        let range = MemoryManagerIdRange::new(MEMORY_MANAGER_MIN_ID, MEMORY_MANAGER_MAX_ID)
            .expect("usable full range");

        assert!(range.contains(MEMORY_MANAGER_MIN_ID));
        assert!(range.contains(MEMORY_MANAGER_MAX_ID));
        assert!(!range.contains(MEMORY_MANAGER_INVALID_ID));
    }

    #[test]
    fn memory_manager_governance_range_is_owned_by_ic_memory() {
        let range = memory_manager_governance_range();

        assert_eq!(range.start(), MEMORY_MANAGER_MIN_ID);
        assert_eq!(MEMORY_MANAGER_LEDGER_ID, range.start());
        assert!(range.contains(MEMORY_MANAGER_LEDGER_ID));
        assert!(is_ic_memory_stable_key(IC_MEMORY_LEDGER_STABLE_KEY));
        assert_eq!(IC_MEMORY_AUTHORITY_OWNER, "ic-memory");
    }

    #[test]
    fn memory_manager_range_rejects_reversed_bounds() {
        let err = MemoryManagerIdRange::new(10, 9).expect_err("reversed range");

        assert_eq!(
            err,
            MemoryManagerRangeError::InvalidRange { start: 10, end: 9 }
        );
    }

    #[test]
    fn memory_manager_range_rejects_sentinel_bounds() {
        let err =
            MemoryManagerIdRange::new(240, MEMORY_MANAGER_INVALID_ID).expect_err("sentinel range");

        assert_eq!(
            err,
            MemoryManagerRangeError::InvalidMemoryManagerId {
                id: MEMORY_MANAGER_INVALID_ID
            }
        );
    }

    #[test]
    fn memory_manager_range_authority_accepts_non_overlapping_construction() {
        let authority = MemoryManagerRangeAuthority::new()
            .reserve(memory_manager_governance_range(), IC_MEMORY_AUTHORITY_OWNER)
            .expect("ic-memory range")
            .reserve(
                MemoryManagerIdRange::new(10, 99).expect("framework range"),
                "framework",
            )
            .expect("framework range")
            .allow(
                MemoryManagerIdRange::new(100, MEMORY_MANAGER_MAX_ID).expect("app range"),
                "applications",
            )
            .expect("app range");

        assert_eq!(authority.authorities().len(), 3);
        assert_eq!(
            authority.authorities()[0].range,
            memory_manager_governance_range()
        );
        assert_eq!(
            authority.authorities()[0].mode,
            MemoryManagerRangeMode::Reserved
        );
        assert_eq!(authority.authorities()[1].range.start(), 10);
        assert_eq!(authority.authorities()[2].range.start(), 100);
    }

    #[test]
    fn memory_manager_range_authority_rejects_overlap() {
        let err = MemoryManagerRangeAuthority::new()
            .reserve(
                MemoryManagerIdRange::new(10, 99).expect("framework range"),
                "framework",
            )
            .expect("framework range")
            .allow(
                MemoryManagerIdRange::new(99, 120).expect("overlapping app range"),
                "applications",
            )
            .expect_err("overlap must fail");

        assert_eq!(
            err,
            MemoryManagerRangeAuthorityError::OverlappingRanges {
                existing_start: 10,
                existing_end: 99,
                candidate_start: 99,
                candidate_end: 120,
            }
        );
    }

    #[test]
    fn memory_manager_range_authority_rejects_invalid_diagnostic_strings() {
        let err = MemoryManagerRangeAuthority::new()
            .reserve(
                MemoryManagerIdRange::new(10, 99).expect("framework range"),
                "",
            )
            .expect_err("empty authority must fail");
        assert_eq!(
            err,
            MemoryManagerRangeAuthorityError::InvalidDiagnosticString {
                field: "authority",
                reason: "must not be empty",
            }
        );

        let err = MemoryManagerRangeAuthority::new()
            .allow_with_purpose(
                MemoryManagerIdRange::new(100, MEMORY_MANAGER_MAX_ID).expect("app range"),
                "applications",
                Some("bad\npurpose".to_string()),
            )
            .expect_err("control character purpose must fail");
        assert_eq!(
            err,
            MemoryManagerRangeAuthorityError::InvalidDiagnosticString {
                field: "purpose",
                reason: "must not contain ASCII control characters",
            }
        );
    }

    #[test]
    fn memory_manager_range_authority_rejects_sentinel_lookup() {
        let err = MemoryManagerRangeAuthority::new()
            .authority_for_id(MEMORY_MANAGER_INVALID_ID)
            .expect_err("sentinel lookup must fail");

        assert_eq!(
            err,
            MemoryManagerRangeAuthorityError::Slot(
                MemoryManagerSlotError::InvalidMemoryManagerId {
                    id: MEMORY_MANAGER_INVALID_ID
                }
            )
        );
    }

    #[test]
    fn memory_manager_range_authority_finds_authority_for_id() {
        let authority = MemoryManagerRangeAuthority::new()
            .allow(
                MemoryManagerIdRange::new(100, MEMORY_MANAGER_MAX_ID).expect("app range"),
                "applications",
            )
            .expect("app range")
            .reserve(memory_manager_governance_range(), IC_MEMORY_AUTHORITY_OWNER)
            .expect("ic-memory range");

        let record = authority
            .authority_for_id(100)
            .expect("valid ID")
            .expect("authority record");
        assert_eq!(record.authority, "applications");
        assert_eq!(record.mode, MemoryManagerRangeMode::Allowed);

        assert!(
            authority
                .authority_for_id(99)
                .expect("valid unclaimed ID")
                .is_none()
        );
    }

    #[test]
    fn memory_manager_range_authority_validates_slot_authority() {
        let authority = MemoryManagerRangeAuthority::new()
            .reserve(
                MemoryManagerIdRange::new(10, 99).expect("framework range"),
                "framework",
            )
            .expect("framework range")
            .allow(
                MemoryManagerIdRange::new(100, MEMORY_MANAGER_MAX_ID).expect("app range"),
                "applications",
            )
            .expect("app range");

        let record = authority
            .validate_slot_authority(
                &AllocationSlotDescriptor::memory_manager(42).expect("framework slot"),
                "framework",
            )
            .expect("framework authority");
        assert_eq!(record.mode, MemoryManagerRangeMode::Reserved);

        let err = authority
            .validate_slot_authority(
                &AllocationSlotDescriptor::memory_manager(42).expect("framework slot"),
                "applications",
            )
            .expect_err("wrong authority must fail");
        assert_eq!(
            err,
            MemoryManagerRangeAuthorityError::AuthorityMismatch {
                id: 42,
                expected_authority: "applications".to_string(),
                actual_authority: "framework".to_string(),
            }
        );
    }

    #[test]
    fn memory_manager_range_authority_validates_slot_authority_mode() {
        let authority = MemoryManagerRangeAuthority::new()
            .reserve(
                MemoryManagerIdRange::new(10, 99).expect("framework range"),
                "framework",
            )
            .expect("framework range")
            .allow(
                MemoryManagerIdRange::new(100, MEMORY_MANAGER_MAX_ID).expect("app range"),
                "applications",
            )
            .expect("app range");

        let record = authority
            .validate_slot_authority_mode(
                &AllocationSlotDescriptor::memory_manager(42).expect("framework slot"),
                "framework",
                MemoryManagerRangeMode::Reserved,
            )
            .expect("framework reserved authority");
        assert_eq!(record.authority, "framework");

        let err = authority
            .validate_slot_authority_mode(
                &AllocationSlotDescriptor::memory_manager(42).expect("framework slot"),
                "framework",
                MemoryManagerRangeMode::Allowed,
            )
            .expect_err("wrong mode must fail");
        assert_eq!(
            err,
            MemoryManagerRangeAuthorityError::ModeMismatch {
                id: 42,
                authority: "framework".to_string(),
                expected_mode: MemoryManagerRangeMode::Allowed,
                actual_mode: MemoryManagerRangeMode::Reserved,
            }
        );
    }

    #[test]
    fn memory_manager_range_authority_validates_id_authority() {
        let authority = MemoryManagerRangeAuthority::new()
            .reserve(
                MemoryManagerIdRange::new(10, 99).expect("framework range"),
                "framework",
            )
            .expect("framework range")
            .allow(
                MemoryManagerIdRange::new(100, MEMORY_MANAGER_MAX_ID).expect("app range"),
                "applications",
            )
            .expect("app range");

        assert_eq!(
            authority
                .validate_id_authority(100, "applications")
                .expect("application authority")
                .mode,
            MemoryManagerRangeMode::Allowed
        );
        assert_eq!(
            authority
                .validate_id_authority_mode(42, "framework", MemoryManagerRangeMode::Reserved)
                .expect("framework reserved authority")
                .range,
            MemoryManagerIdRange::new(10, 99).expect("framework range")
        );

        let err = authority
            .validate_id_authority_mode(100, "applications", MemoryManagerRangeMode::Reserved)
            .expect_err("wrong mode must fail");
        assert_eq!(
            err,
            MemoryManagerRangeAuthorityError::ModeMismatch {
                id: 100,
                authority: "applications".to_string(),
                expected_mode: MemoryManagerRangeMode::Reserved,
                actual_mode: MemoryManagerRangeMode::Allowed,
            }
        );
    }

    #[test]
    fn memory_manager_range_authority_reports_authority_mismatch_before_mode_mismatch() {
        let authority = MemoryManagerRangeAuthority::new()
            .allow(
                MemoryManagerIdRange::new(100, MEMORY_MANAGER_MAX_ID).expect("app range"),
                "applications",
            )
            .expect("app range");

        let err = authority
            .validate_id_authority_mode(100, "framework", MemoryManagerRangeMode::Reserved)
            .expect_err("authority mismatch must be distinct");
        assert_eq!(
            err,
            MemoryManagerRangeAuthorityError::AuthorityMismatch {
                id: 100,
                expected_authority: "framework".to_string(),
                actual_authority: "applications".to_string(),
            }
        );
    }

    #[test]
    fn memory_manager_range_authority_preserves_reserve_and_allow_modes() {
        let authority = MemoryManagerRangeAuthority::new()
            .reserve(
                MemoryManagerIdRange::new(10, 99).expect("framework range"),
                "framework",
            )
            .expect("framework range")
            .allow(
                MemoryManagerIdRange::new(100, MEMORY_MANAGER_MAX_ID).expect("app range"),
                "applications",
            )
            .expect("app range");

        assert_eq!(
            authority.authorities()[0].mode,
            MemoryManagerRangeMode::Reserved
        );
        assert_eq!(
            authority.authorities()[1].mode,
            MemoryManagerRangeMode::Allowed
        );
    }

    #[test]
    fn memory_manager_range_authority_validates_complete_coverage() {
        let authority = MemoryManagerRangeAuthority::new()
            .reserve(memory_manager_governance_range(), IC_MEMORY_AUTHORITY_OWNER)
            .expect("ic-memory range")
            .reserve(
                MemoryManagerIdRange::new(10, 99).expect("framework range"),
                "framework",
            )
            .expect("framework range")
            .allow(
                MemoryManagerIdRange::new(100, MEMORY_MANAGER_MAX_ID).expect("app range"),
                "applications",
            )
            .expect("app range");

        authority
            .validate_complete_coverage(
                MemoryManagerIdRange::new(MEMORY_MANAGER_MIN_ID, MEMORY_MANAGER_MAX_ID)
                    .expect("full range"),
            )
            .expect("complete coverage");
    }

    #[test]
    fn memory_manager_range_authority_rejects_complete_coverage_gaps() {
        let authority = MemoryManagerRangeAuthority::new()
            .reserve(memory_manager_governance_range(), IC_MEMORY_AUTHORITY_OWNER)
            .expect("ic-memory range")
            .allow(
                MemoryManagerIdRange::new(100, MEMORY_MANAGER_MAX_ID).expect("app range"),
                "applications",
            )
            .expect("app range");

        let err = authority
            .validate_complete_coverage(
                MemoryManagerIdRange::new(MEMORY_MANAGER_MIN_ID, MEMORY_MANAGER_MAX_ID)
                    .expect("full range"),
            )
            .expect_err("coverage gap must fail");
        assert_eq!(
            err,
            MemoryManagerRangeAuthorityError::MissingCoverage { start: 10, end: 99 }
        );

        let err = MemoryManagerRangeAuthority::new()
            .validate_complete_coverage(
                MemoryManagerIdRange::new(MEMORY_MANAGER_MIN_ID, MEMORY_MANAGER_MAX_ID)
                    .expect("full range"),
            )
            .expect_err("empty coverage must fail");
        assert_eq!(
            err,
            MemoryManagerRangeAuthorityError::MissingCoverage {
                start: MEMORY_MANAGER_MIN_ID,
                end: MEMORY_MANAGER_MAX_ID,
            }
        );
    }

    #[test]
    fn memory_manager_range_authority_rejects_complete_coverage_outside_target() {
        let authority = MemoryManagerRangeAuthority::new()
            .reserve(memory_manager_governance_range(), IC_MEMORY_AUTHORITY_OWNER)
            .expect("ic-memory range")
            .reserve(
                MemoryManagerIdRange::new(10, 99).expect("framework range"),
                "framework",
            )
            .expect("framework range");

        let err = authority
            .validate_complete_coverage(MemoryManagerIdRange::new(10, 99).expect("target range"))
            .expect_err("outside range must fail");
        assert_eq!(
            err,
            MemoryManagerRangeAuthorityError::RangeOutsideCoverageTarget {
                start: MEMORY_MANAGER_MIN_ID,
                end: MEMORY_MANAGER_GOVERNANCE_MAX_ID,
                target_start: 10,
                target_end: 99,
            }
        );
    }

    #[test]
    fn memory_manager_range_authority_from_records_sorts_and_validates() {
        let err = MemoryManagerRangeAuthority::from_records(vec![MemoryManagerAuthorityRecord {
            range: MemoryManagerIdRange::new(10, 99).expect("framework range"),
            authority: String::new(),
            mode: MemoryManagerRangeMode::Reserved,
            purpose: None,
        }])
        .expect_err("empty authority must fail");
        assert_eq!(
            err,
            MemoryManagerRangeAuthorityError::InvalidDiagnosticString {
                field: "authority",
                reason: "must not be empty",
            }
        );

        let authority = MemoryManagerRangeAuthority::from_records(vec![
            MemoryManagerAuthorityRecord {
                range: MemoryManagerIdRange::new(100, MEMORY_MANAGER_MAX_ID).expect("app range"),
                authority: "applications".to_string(),
                mode: MemoryManagerRangeMode::Allowed,
                purpose: Some("application stable stores".to_string()),
            },
            MemoryManagerAuthorityRecord {
                range: memory_manager_governance_range(),
                authority: IC_MEMORY_AUTHORITY_OWNER.to_string(),
                mode: MemoryManagerRangeMode::Reserved,
                purpose: Some(IC_MEMORY_AUTHORITY_PURPOSE.to_string()),
            },
        ])
        .expect("records");

        assert_eq!(
            authority.authorities()[0].authority,
            IC_MEMORY_AUTHORITY_OWNER
        );
        assert_eq!(authority.authorities()[1].authority, "applications");
    }

    #[test]
    fn memory_manager_range_authority_from_records_rejects_overlap() {
        let err = MemoryManagerRangeAuthority::from_records(vec![
            MemoryManagerAuthorityRecord {
                range: MemoryManagerIdRange::new(10, 99).expect("framework range"),
                authority: "framework".to_string(),
                mode: MemoryManagerRangeMode::Reserved,
                purpose: None,
            },
            MemoryManagerAuthorityRecord {
                range: MemoryManagerIdRange::new(90, 120).expect("overlap range"),
                authority: "applications".to_string(),
                mode: MemoryManagerRangeMode::Allowed,
                purpose: None,
            },
        ])
        .expect_err("overlap must fail");

        assert_eq!(
            err,
            MemoryManagerRangeAuthorityError::OverlappingRanges {
                existing_start: 10,
                existing_end: 99,
                candidate_start: 90,
                candidate_end: 120,
            }
        );
    }

    #[test]
    fn memory_manager_range_authority_diagnostic_export_is_stable() {
        let authority = MemoryManagerRangeAuthority::new()
            .allow_with_purpose(
                MemoryManagerIdRange::new(100, MEMORY_MANAGER_MAX_ID).expect("app range"),
                "applications",
                Some("application stable stores".to_string()),
            )
            .expect("app range")
            .reserve_with_purpose(
                memory_manager_governance_range(),
                IC_MEMORY_AUTHORITY_OWNER,
                Some(IC_MEMORY_AUTHORITY_PURPOSE.to_string()),
            )
            .expect("ic-memory range");

        assert_eq!(
            authority.to_records(),
            vec![
                MemoryManagerAuthorityRecord {
                    range: memory_manager_governance_range(),
                    authority: IC_MEMORY_AUTHORITY_OWNER.to_string(),
                    mode: MemoryManagerRangeMode::Reserved,
                    purpose: Some(IC_MEMORY_AUTHORITY_PURPOSE.to_string()),
                },
                MemoryManagerAuthorityRecord {
                    range: MemoryManagerIdRange::new(100, MEMORY_MANAGER_MAX_ID)
                        .expect("app range"),
                    authority: "applications".to_string(),
                    mode: MemoryManagerRangeMode::Allowed,
                    purpose: Some("application stable stores".to_string()),
                },
            ]
        );
    }
}
