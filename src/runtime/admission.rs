use crate::{
    AllocationLedger, AllocationSlotDescriptor, AllocationState, MemoryRequest, SchemaMetadata,
    SealedDeclarationSnapshot, StableKey,
};

///
/// RecoveredAllocationMetadata
///
/// Validated allocation evidence borrowed during bootstrap preparation. This
/// metadata grants no memory access and contains no application payload or
/// historical authority identity. Host grants supply current authorization.
///

#[derive(Clone, Copy, Debug)]
pub struct RecoveredAllocationMetadata<'a> {
    /// Durable allocation identity.
    pub stable_key: &'a StableKey,
    /// Persisted assignment, not permission to open it.
    pub slot: &'a AllocationSlotDescriptor,
    /// Current generic allocation lifecycle state.
    pub state: AllocationState,
    /// Latest diagnostic schema metadata, not application schema validation.
    pub schema: &'a SchemaMetadata,
}

///
/// BootstrapAdmissionError
///
/// Historical declaration completion rejected before staging or persistence.
///

#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum BootstrapAdmissionError {
    #[error("historical key {0} is unknown")]
    Unknown(StableKey),
    #[error("historical key {0} is retired")]
    Retired(StableKey),
    #[error("key {0} is already declared or selected")]
    Duplicate(StableKey),
    #[error("completed declarations exceed 254 external allocations")]
    TooManyDeclarations,
    #[error("historical selection {stable_key} by {authority} lacks a current grant: {source}")]
    Range {
        stable_key: StableKey,
        authority: String,
        source: crate::MemoryManagerRangeAuthorityError,
    },
    #[error(transparent)]
    Registry(#[from] crate::StaticMemoryDeclarationError),
}

///
/// BootstrapAdmission
///
/// Bounded preparation context supplied only after validated ledger recovery.
/// Consumers may reject identity transitions or explicitly include known
/// historical allocations before the existing resolve/validate/commit boundary.
/// No memory handles or mutable recovered state are exposed. Failed selections
/// poison this attempt even if a consumer ignores their returned errors.
///

pub struct BootstrapAdmission<'a> {
    ledger: &'a AllocationLedger,
    declarations: &'a SealedDeclarationSnapshot,
    selected: Vec<MemoryRequest>,
    failure: Option<BootstrapAdmissionError>,
}

impl<'a> BootstrapAdmission<'a> {
    pub(super) const fn new(
        ledger: &'a AllocationLedger,
        declarations: &'a SealedDeclarationSnapshot,
    ) -> Self {
        Self {
            ledger,
            declarations,
            selected: Vec::new(),
            failure: None,
        }
    }

    /// Original sealed input; preparation cannot remove declarations or add grants.
    #[must_use]
    pub const fn declarations(&self) -> &SealedDeclarationSnapshot {
        self.declarations
    }

    /// At most 255 validated allocation summaries, including governance records.
    ///
    /// # Panics
    ///
    /// Panics only if an internal validated-ledger invariant is broken.
    #[must_use]
    pub fn recovered_allocations(
        &self,
    ) -> impl ExactSizeIterator<Item = RecoveredAllocationMetadata<'_>> {
        self.ledger
            .allocation_history()
            .records()
            .iter()
            .map(|record| RecoveredAllocationMetadata {
                stable_key: record.stable_key(),
                slot: record.slot(),
                state: record.state(),
                schema: record
                    .schema_history()
                    .last()
                    .expect("validated schema history")
                    .schema(),
            })
    }

    /// Whether the original input or an earlier selection already names this key.
    #[must_use]
    pub fn is_declared(&self, key: &StableKey) -> bool {
        self.declarations
            .allocation_snapshot()
            .declarations()
            .iter()
            .any(|d| d.stable_key() == key)
            || self
                .declarations
                .requests()
                .iter()
                .chain(&self.selected)
                .any(|r| r.stable_key() == key)
    }

    /// Include a known, nonretired key under an explicit current host grant.
    /// Retains its slot and latest schema metadata. Final current policy and all
    /// ordinary collision/retirement checks still run after preparation.
    pub fn include_historical(
        &mut self,
        authority: &str,
        stable_key: &str,
    ) -> Result<(), BootstrapAdmissionError> {
        if let Some(error) = &self.failure {
            return Err(error.clone());
        }
        let result = self.select(authority, stable_key);
        if let Err(error) = &result {
            self.failure = Some(error.clone());
        }
        result
    }

    fn select(&mut self, authority: &str, stable_key: &str) -> Result<(), BootstrapAdmissionError> {
        // Constructor bounds names before they can enter selection diagnostics.
        let request = MemoryRequest::new(authority, stable_key, SchemaMetadata::default())?;
        let key = request.stable_key();
        if self.is_declared(key) {
            return Err(BootstrapAdmissionError::Duplicate(key.clone()));
        }
        if self.declarations.registered_declarations().len()
            + self.declarations.requests().len()
            + self.selected.len()
            >= 254
        {
            return Err(BootstrapAdmissionError::TooManyDeclarations);
        }
        let record = self
            .ledger
            .allocation_history()
            .records()
            .iter()
            .find(|r| r.stable_key() == key)
            .ok_or_else(|| BootstrapAdmissionError::Unknown(key.clone()))?;
        if matches!(record.state(), AllocationState::Retired { .. }) {
            return Err(BootstrapAdmissionError::Retired(key.clone()));
        }
        self.declarations
            .range_authority()
            .validate_slot_authority(record.slot(), authority)
            .map_err(|source| BootstrapAdmissionError::Range {
                stable_key: key.clone(),
                authority: authority.to_string(),
                source,
            })?;
        self.selected.push(MemoryRequest::new(
            authority,
            stable_key,
            record
                .schema_history()
                .last()
                .expect("validated schema history")
                .schema()
                .clone(),
        )?);
        Ok(())
    }

    pub(super) fn complete(self) -> Result<SealedDeclarationSnapshot, BootstrapAdmissionError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        if self.selected.is_empty() {
            return Ok(self.declarations.clone());
        }
        let mut requests = self.declarations.requests().to_vec();
        requests.extend(self.selected);
        Ok(SealedDeclarationSnapshot::new(
            self.declarations.registered_declarations(),
            self.declarations.registered_ranges(),
            &requests,
        )?)
    }
}
