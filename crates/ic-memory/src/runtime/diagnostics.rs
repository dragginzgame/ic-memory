use super::{MemoryRuntime, RuntimeDiagnosticError};
use crate::{
    DiagnosticExport, DiagnosticMemorySize, RecoveredLedger, physical::CommitSnapshotDiagnostic,
};
use ic_stable_structures::Memory;

impl<M: Memory> MemoryRuntime<M> {
    /// Export this runtime's recovered ledger and live virtual-memory sizes.
    pub fn diagnostic_export(&self) -> Result<DiagnosticExport, RuntimeDiagnosticError> {
        if !self.is_bootstrapped() {
            return Err(RuntimeDiagnosticError::NotBootstrapped);
        }
        let recovered = self.ledger_record_from_memory()?.recover()?;
        Ok(self.recovered_diagnostic_export(recovered))
    }

    /// Diagnose protected commit recovery from this runtime's ledger memory.
    ///
    /// This operation is available before bootstrap when the stable-cell
    /// envelope is readable or the ledger memory is empty.
    pub fn commit_recovery_diagnostic(
        &self,
    ) -> Result<CommitSnapshotDiagnostic, RuntimeDiagnosticError> {
        let record = self.ledger_record_from_memory()?;
        Ok(record.diagnostic())
    }

    fn recovered_diagnostic_export(&self, recovered: RecoveredLedger) -> DiagnosticExport {
        let mut export = DiagnosticExport::from_owned_ledger(recovered.into_ledger());
        for record in &mut export.records {
            record.memory_size = Some(DiagnosticMemorySize::from_wasm_pages(
                self.memory_size_pages(record.allocation.slot().id()),
            ));
        }
        export
    }
}
