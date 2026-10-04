use super::descriptor::AllocationSlotDescriptor;
use super::memory_manager::{
    MEMORY_MANAGER_INVALID_ID, MEMORY_MANAGER_MAX_ID, MEMORY_MANAGER_MIN_ID,
    MemoryManagerSlotError, validate_memory_manager_id,
};
use crate::text::validate_diagnostic_text;
use serde::{Deserialize, Deserializer, Serialize, de::Error as _};

///
/// MemoryManagerIdRange
///
/// Inclusive range of usable `MemoryManager` virtual memory IDs.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MemoryManagerIdRange {
    pub(crate) start: u8,
    pub(crate) end: u8,
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

    /// Validate this range's decoded bounds.
    pub const fn validate(&self) -> Result<(), MemoryManagerRangeError> {
        match Self::new(self.start, self.end) {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
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

///
/// MemoryManagerRangeMode
///
/// Allocation policy mode for a `MemoryManager` authority range.
///
/// These modes describe policy authority, not durable allocation state.
/// Only `Allowed` ranges supply fresh logical placements; neither mode allocates
/// memory by itself. [`crate::ic_memory_range!`] defaults to `Reserved` when its
/// `mode` argument is omitted.
///

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
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MemoryManagerAuthorityRecord {
    /// Inclusive range governed by this authority.
    pub(crate) range: MemoryManagerIdRange,
    /// Stable printable ASCII authority identifier.
    pub(crate) authority: String,
    /// Policy mode for this authority range.
    pub(crate) mode: MemoryManagerRangeMode,
    /// Optional stable printable ASCII diagnostic purpose.
    #[serde(deserialize_with = "crate::cbor::deserialize_present_option")]
    pub(crate) purpose: Option<String>,
}

impl MemoryManagerAuthorityRecord {
    /// Build a diagnostic authority record after validating printable metadata.
    pub fn new(
        range: MemoryManagerIdRange,
        authority: impl Into<String>,
        mode: MemoryManagerRangeMode,
        purpose: Option<String>,
    ) -> Result<Self, MemoryManagerRangeAuthorityError> {
        let record = Self {
            range,
            authority: authority.into(),
            mode,
            purpose,
        };
        validate_authority_record(&record)?;
        Ok(record)
    }

    /// Return the inclusive range governed by this authority.
    #[must_use]
    pub const fn range(&self) -> MemoryManagerIdRange {
        self.range
    }

    /// Return the stable printable ASCII authority identifier.
    #[must_use]
    pub fn authority(&self) -> &str {
        &self.authority
    }

    /// Return the policy mode for this authority range.
    #[must_use]
    pub const fn mode(&self) -> MemoryManagerRangeMode {
        self.mode
    }

    /// Return the optional stable printable ASCII diagnostic purpose.
    #[must_use]
    pub fn purpose(&self) -> Option<&str> {
        self.purpose.as_deref()
    }

    /// Validate constructor invariants after decode or manual assembly.
    pub fn validate(&self) -> Result<(), MemoryManagerRangeAuthorityError> {
        validate_authority_record(self)
    }
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
///
/// When used through the default runtime registry, registered ranges are
/// authoritative generic policy and are checked before caller-supplied
/// [`crate::AllocationPolicy`]. When no user ranges are registered, frameworks
/// can enforce fixed application claims through their own policy. Logical
/// placement and historical selection always require explicit grants; fresh
/// placements use only `Allowed` ranges.
///

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct MemoryManagerRangeAuthority {
    authorities: Vec<MemoryManagerAuthorityRecord>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MemoryManagerRangeAuthorityDto {
    authorities: Vec<MemoryManagerAuthorityRecord>,
}

impl<'de> Deserialize<'de> for MemoryManagerRangeAuthority {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let dto = MemoryManagerRangeAuthorityDto::deserialize(deserializer)?;
        Self::from_records(dto.authorities).map_err(D::Error::custom)
    }
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
    /// Each record is validated before insertion, overlaps are rejected, and
    /// accepted records are stored in ascending range order. Decoded records
    /// pass the same checks as records built with their checked constructor.
    pub fn from_records(
        records: Vec<MemoryManagerAuthorityRecord>,
    ) -> Result<Self, MemoryManagerRangeAuthorityError> {
        let mut authorities: Vec<MemoryManagerAuthorityRecord> = Vec::new();
        for record in records {
            validate_authority_record(&record)?;
            let insertion = authorities
                .partition_point(|existing| existing.range.start() < record.range.start());
            // Accepted ranges are ordered and disjoint. Only the predecessor
            // and successor can be the first overlap; check them in range order.
            let neighbours = insertion.saturating_sub(1)..(insertion + 1).min(authorities.len());
            for existing in &authorities[neighbours] {
                if ranges_overlap(existing.range, record.range) {
                    return Err(MemoryManagerRangeAuthorityError::OverlappingRanges {
                        existing_start: existing.range.start(),
                        existing_end: existing.range.end(),
                        candidate_start: record.range.start(),
                        candidate_end: record.range.end(),
                    });
                }
            }
            authorities.insert(insertion, record);
        }
        Ok(Self { authorities })
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

        let mut next_uncovered = target.start();
        for record in &self.authorities {
            if record.range.start() > next_uncovered {
                return Err(MemoryManagerRangeAuthorityError::MissingCoverage {
                    start: next_uncovered,
                    end: record.range.start() - 1,
                });
            }
            // Ranges are ordered and disjoint, with usable ends at most 254.
            // The next position therefore advances and fits through 255.
            next_uncovered = record.range.end() + 1;
        }

        if next_uncovered <= target.end() {
            return Err(MemoryManagerRangeAuthorityError::MissingCoverage {
                start: next_uncovered,
                end: target.end(),
            });
        }

        Ok(())
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

fn validate_authority_record(
    record: &MemoryManagerAuthorityRecord,
) -> Result<(), MemoryManagerRangeAuthorityError> {
    record.range.validate()?;
    validate_diagnostic_string("authority", &record.authority)?;
    if let Some(purpose) = &record.purpose {
        validate_diagnostic_string("purpose", purpose)?;
    }
    Ok(())
}

///
/// MemoryManagerRangeAuthorityError
///
/// Invalid `MemoryManager` range authority policy.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum MemoryManagerRangeAuthorityError {
    /// Authority range bounds are invalid.
    #[error(transparent)]
    Range(#[from] MemoryManagerRangeError),
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

const fn ranges_overlap(left: MemoryManagerIdRange, right: MemoryManagerIdRange) -> bool {
    left.start() <= right.end() && right.start() <= left.end()
}

fn validate_diagnostic_string(
    field: &'static str,
    value: &str,
) -> Result<(), MemoryManagerRangeAuthorityError> {
    validate_diagnostic_text(value).map_err(|error| {
        MemoryManagerRangeAuthorityError::InvalidDiagnosticString {
            field,
            reason: error.reason(),
        }
    })
}
