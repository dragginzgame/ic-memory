use crate::{
    AllocationDeclaration, AllocationLedger, AllocationRecord, AllocationState, MemoryRequest,
    SchemaMetadata, SealedDeclarationSnapshot, StableKey,
};

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
    #[error(transparent)]
    Pool(#[from] crate::MemoryAllocationPoolError),
    #[error(transparent)]
    Declarations(#[from] crate::MemoryDeclarationError),
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
    pool: &'a crate::MemoryAllocationPool,
    selected: Vec<AllocationDeclaration>,
    failure: Option<BootstrapAdmissionError>,
}

impl<'a> BootstrapAdmission<'a> {
    pub(super) const fn new(
        ledger: &'a AllocationLedger,
        declarations: &'a SealedDeclarationSnapshot,
        pool: &'a crate::MemoryAllocationPool,
    ) -> Self {
        Self {
            ledger,
            declarations,
            pool,
            selected: Vec::new(),
            failure: None,
        }
    }

    /// Original sealed input; preparation cannot remove declarations or change host policy.
    #[must_use]
    pub const fn declarations(&self) -> &SealedDeclarationSnapshot {
        self.declarations
    }

    /// Borrow at most 255 validated allocation records, including governance.
    /// Records grant no memory access and contain no historical authority label;
    /// current host namespace grants remain the authorization owner.
    #[must_use = "recovered allocation metadata is inspected only when the iterator is consumed"]
    pub fn recovered_allocations(&self) -> impl ExactSizeIterator<Item = &AllocationRecord> {
        self.ledger.records().iter()
    }

    /// Whether the original input or an earlier selection already names this key.
    #[must_use]
    pub fn is_declared(&self, key: &StableKey) -> bool {
        // Sealing owns canonical source request keys. Selections retain callback
        // order, so only that unsealed tail needs a scan.
        key.as_str() == crate::IC_MEMORY_LEDGER_STABLE_KEY
            || self
                .declarations
                .requests()
                .binary_search_by(|request| request.stable_key().cmp(key))
                .is_ok()
            || self
                .selected
                .iter()
                .any(|declaration| declaration.stable_key() == key)
    }

    /// Include a known, nonretired key under an explicit current host namespace grant.
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
        if self.declarations.requests().len() + self.selected.len() >= 254 {
            return Err(BootstrapAdmissionError::TooManyDeclarations);
        }
        let record = self
            .ledger
            .records()
            .iter()
            .find(|r| r.stable_key() == key)
            .ok_or_else(|| BootstrapAdmissionError::Unknown(key.clone()))?;
        if matches!(record.state(), AllocationState::Retired) {
            return Err(BootstrapAdmissionError::Retired(key.clone()));
        }
        self.pool.validate_authority(key, authority)?;
        self.pool.validate_id(record.slot().id())?;
        self.selected
            .push(request.into_declaration(record.slot().clone(), record.schema().clone()));
        Ok(())
    }

    pub(super) fn complete(self) -> Result<Vec<AllocationDeclaration>, BootstrapAdmissionError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        Ok(self.selected)
    }
}
