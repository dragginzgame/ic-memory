use crate::{
    constants::WASM_PAGE_SIZE_BYTES,
    ledger::{AllocationLedger, AllocationRecord},
};
use serde::Serialize;

///
/// DiagnosticExport
///
/// Read-only machine-readable allocation ledger export.
///
/// Reports support serialization only; applications own any diagnostic input
/// protocol. This type is never used for durable ledger recovery.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DiagnosticExport {
    /// Current committed generation.
    pub current_generation: u64,
    /// Allocation records.
    pub records: Vec<DiagnosticRecord>,
}

impl DiagnosticExport {
    /// Build a read-only diagnostic export from an allocation ledger.
    ///
    /// Records start unmeasured. The exporter can fill their size fields from
    /// its backing-memory observations; this DTO constructor neither recovers
    /// nor measures memory. Physical recovery status is available separately
    /// from the runtime.
    #[must_use]
    #[cfg(any(test, feature = "test-support"))]
    pub fn from_ledger(ledger: &AllocationLedger) -> Self {
        Self::from_records(ledger.current_generation, ledger.records.iter().cloned())
    }

    // Normal exports consume their decoded records. Borrowed exports copy
    // records directly into the same projection without an intermediate ledger.
    pub(crate) fn from_owned_ledger(ledger: AllocationLedger) -> Self {
        Self::from_records(ledger.current_generation, ledger.records.into_iter())
    }

    fn from_records(
        current_generation: u64,
        records: impl Iterator<Item = AllocationRecord>,
    ) -> Self {
        Self {
            current_generation,
            records: records
                .map(|allocation| DiagnosticRecord {
                    allocation,
                    memory_size: None,
                })
                .collect(),
        }
    }
}

///
/// DiagnosticRecord
///
/// Read-only diagnostic allocation record.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DiagnosticRecord {
    /// Allocation record.
    pub allocation: AllocationRecord,
    /// Live backing memory size, when the exporter measured one.
    ///
    /// This is allocation size reported by the backing memory, not logical user
    /// payload size inside the stable structure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_size: Option<DiagnosticMemorySize>,
}

///
/// DiagnosticMemorySize
///
/// Live size reported by a backing stable memory.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct DiagnosticMemorySize {
    /// WebAssembly pages reported by the memory.
    pub wasm_pages: u64,
}

impl DiagnosticMemorySize {
    /// Build a size from a WebAssembly page count.
    #[must_use]
    pub const fn from_wasm_pages(wasm_pages: u64) -> Self {
        Self { wasm_pages }
    }

    /// Bytes represented by the page count, saturating on an unrepresentable size.
    #[must_use]
    pub const fn bytes(self) -> u64 {
        self.wasm_pages.saturating_mul(WASM_PAGE_SIZE_BYTES)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AllocationDeclaration, MemoryManagerSlot, ledger::AllocationRecord};

    #[test]
    fn diagnostic_export_copies_ledger_records() {
        let declaration = AllocationDeclaration::new(
            "app.users.v1",
            MemoryManagerSlot::new(100).expect("usable slot"),
        )
        .expect("declaration");
        let ledger = AllocationLedger {
            current_generation: 3,
            records: vec![AllocationRecord::from_declaration(&declaration)],
        };

        let export = DiagnosticExport::from_ledger(&ledger);

        assert_eq!(export.current_generation, 3);
        assert_eq!(export.records.len(), 1);
        assert_eq!(export.records[0].memory_size, None);

        assert_eq!(export, DiagnosticExport::from_owned_ledger(ledger));
        let wire = serde_json::to_value(&export).expect("diagnostic JSON");

        assert_eq!(
            wire,
            serde_json::json!({
                "current_generation": 3,
                "records": [{"allocation": {"stable_key": "app.users.v1", "slot": 100}}]
            })
        );
    }

    #[test]
    fn diagnostic_export_can_include_memory_sizes() {
        let declaration = AllocationDeclaration::new(
            "app.users.v1",
            MemoryManagerSlot::new(100).expect("usable slot"),
        )
        .expect("declaration");
        let ledger = AllocationLedger {
            current_generation: 3,
            records: vec![AllocationRecord::from_declaration(&declaration)],
        };

        let mut export = DiagnosticExport::from_ledger(&ledger);
        export.records[0].memory_size = Some(DiagnosticMemorySize::from_wasm_pages(2));

        assert_eq!(
            export.records[0].memory_size,
            Some(DiagnosticMemorySize { wasm_pages: 2 })
        );
        let wire = serde_json::to_value(&export).expect("diagnostic JSON");
        assert_eq!(
            wire["records"][0]["memory_size"],
            serde_json::json!({"wasm_pages": 2})
        );
    }
}
