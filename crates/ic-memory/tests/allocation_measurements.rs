//! Explicit native allocation measurements; not IC instruction benchmarks.
use ic_memory::{
    AllocationLedger, DualCommitStore, GenericRangePolicy, LedgerCommitError, LedgerCommitStore,
    LedgerPayloadEnvelope, MemoryManagerConfig, MemoryRuntime, SealedDeclarationSnapshot,
    StableCellLedgerRecord,
    ic_stable_structures::{
        Memory, Storable, VectorMemory,
        memory_manager::{MemoryId, MemoryManager},
    },
};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
    hint::black_box,
    time::Instant,
};

#[derive(Clone, Copy, Default)]
struct Counts {
    enabled: bool,
    allocations: usize,
    reallocations: usize,
    live: usize,
    peak: usize,
}

thread_local! {
    static COUNTS: Cell<Counts> = const { Cell::new(Counts {
        enabled: false, allocations: 0, reallocations: 0, live: 0, peak: 0,
    }) };
}

fn account(old: usize, new: usize, reallocation: bool) {
    let _ = COUNTS.try_with(|cell| {
        let mut counts = cell.get();
        if counts.enabled {
            counts.live = counts.live - old + new;
            counts.peak = counts.peak.max(counts.live);
            counts.allocations += usize::from(old == 0 && new != 0);
            counts.reallocations += usize::from(reallocation);
            cell.set(counts);
        }
    });
}

struct MeasuredAllocator;

#[expect(
    unsafe_code,
    reason = "count this thread's System allocator operations in an explicit benchmark"
)]
unsafe impl GlobalAlloc for MeasuredAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: Preserve System's allocator contract and arguments.
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            account(0, layout.size(), false);
        }
        ptr
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        account(layout.size(), 0, false);
        // SAFETY: Forward the allocation and its matching layout unchanged.
        unsafe { System.dealloc(ptr, layout) };
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        // SAFETY: Forward the valid allocation and requested size unchanged.
        let result = unsafe { System.realloc(ptr, layout, size) };
        if !result.is_null() {
            account(layout.size(), size, true);
        }
        result
    }
}

#[global_allocator]
static ALLOCATOR: MeasuredAllocator = MeasuredAllocator;

fn measure(phase: &str, generations: u64, iterations: usize, mut operation: impl FnMut()) {
    COUNTS.with(|cell| {
        cell.set(Counts {
            enabled: true,
            ..Counts::default()
        });
    });
    let start = Instant::now();
    for _ in 0..iterations {
        operation();
    }
    let elapsed = start.elapsed().as_micros();
    let counts = COUNTS.with(|cell| {
        let counts = cell.get();
        cell.set(Counts::default());
        counts
    });
    assert_eq!(counts.live, 0, "all measured allocations must be released");
    println!(
        "{phase},{generations},{iterations},{},{},{},{elapsed}",
        counts.allocations, counts.reallocations, counts.peak
    );
}

#[test]
#[ignore = "explicit matched allocation measurement, run in release mode with one test thread"]
fn ledger_encoding_and_validation_allocations() {
    println!("phase,generations,iterations,allocations,reallocations,peak_bytes,elapsed_us");
    for count in [0, 1, 64, 255] {
        for generations in [1, 1024, u64::MAX] {
            let records = (0..count)
                .map(|id| {
                    serde_json::json!({
                        "stable_key": format!("app.store{id}.v1"),
                        "slot": { "slot": { "MemoryManagerId": id } },
                        "state": "Active", "schema": { "schema_version": u32::MAX }
                    })
                })
                .collect::<Vec<_>>();
            let ledger = AllocationLedger::new(
                generations,
                serde_json::from_value(serde_json::Value::Array(records)).unwrap(),
            )
            .unwrap();
            let mut store = LedgerCommitStore::default();
            store.commit(&ledger).unwrap();
            let record = StableCellLedgerRecord::new(store);
            measure(&format!("encode_records_{count}"), generations, 10, || {
                drop(black_box(record.to_bytes()));
            });
            measure(
                &format!("validate_records_{count}"),
                generations,
                10,
                || {
                    black_box(ledger.validate_integrity()).unwrap();
                },
            );
            measure(&format!("recover_records_{count}"), generations, 10, || {
                drop(black_box(record.store().recover().unwrap()));
            });
            measure(&format!("commit_records_{count}"), generations, 10, || {
                let mut store = LedgerCommitStore::default();
                drop(black_box(store.commit(&ledger).unwrap()));
            });
        }
    }
}
// Only the stable key differs from a valid current ledger. The physical store
// owns a correctly checksummed opaque envelope; text must be checked by logical
// recovery, not by decoding the enclosing physical DTO.
fn key_text_store(text_bytes: usize) -> LedgerCommitStore {
    #[derive(serde::Serialize)]
    struct Store<'a> {
        physical: &'a DualCommitStore,
    }

    let mut dto = serde_json::json!({"current_generation":1,"records":[{"stable_key":"app.rows.v1","slot":{"slot":{"MemoryManagerId":100}},"state":"Active","schema":{"schema_version":null}}]});
    dto["records"][0]["stable_key"] =
        serde_json::Value::String(format!("app.{}.v1", "x".repeat(text_bytes - 7)));
    let mut payload = Vec::new();
    ciborium::into_writer(&dto, &mut payload).unwrap();
    let envelope = LedgerPayloadEnvelope::current(payload)
        .try_encode()
        .unwrap();
    let mut physical = DualCommitStore::default();
    physical.commit_payload_at_generation(1, envelope).unwrap();
    let mut record = Vec::new();
    ciborium::into_writer(
        &Store {
            physical: &physical,
        },
        &mut record,
    )
    .unwrap();
    ciborium::from_reader(record.as_slice()).unwrap()
}

#[test]
#[ignore = "explicit matched text recovery allocation measurement, run in release mode with one test thread"]
fn ledger_text_recovery_allocations() {
    println!("phase,generations,iterations,allocations,reallocations,peak_bytes,elapsed_us");
    for text_bytes in [128, 129, 1024, 8192, 32 * 1024] {
        let store = key_text_store(text_bytes);
        let phase = format!("recover_text_{text_bytes}");
        measure(&phase, 1, 10, || {
            let result = black_box(store.recover());
            if text_bytes <= 128 {
                assert_eq!(result.unwrap().current_generation(), 1);
            } else {
                assert!(matches!(result, Err(LedgerCommitError::Codec(_))));
            }
        });
    }
}

#[test]
#[ignore = "explicit matched diagnostics allocation measurement, run in release mode with one test thread"]
fn allocation_diagnostics_allocations() {
    println!("phase,generations,iterations,allocations,reallocations,peak_bytes,elapsed_us");
    let declarations = SealedDeclarationSnapshot::new(&[], &[], &[]).unwrap();
    for populated in [false, true] {
        let backing = VectorMemory::default();
        if populated {
            let manager = MemoryManager::init_with_bucket_size(backing.clone(), 1);
            for id in [100, 254] {
                assert_eq!(manager.get(MemoryId::new(id)).grow(2), 0);
            }
        }
        let mut runtime =
            MemoryRuntime::new_with_config(backing, MemoryManagerConfig::new(1).unwrap()).unwrap();
        let prefix = if populated { "populated" } else { "empty" };
        assert_eq!(
            runtime
                .memory_allocation_summary()
                .unwrap()
                .allocated_buckets,
            if populated { 4 } else { 0 }
        );
        for bootstrapped in [false, true] {
            if bootstrapped {
                runtime
                    .bootstrap(&declarations, &GenericRangePolicy)
                    .unwrap();
            }
            let generation = u64::from(bootstrapped);
            let current_generation = bootstrapped.then_some(generation);
            let state = if bootstrapped {
                "bootstrapped"
            } else {
                "unbootstrapped"
            };
            let summary_phase = format!("{prefix}_{state}_summary");
            measure(&summary_phase, generation, 100, || {
                let summary = black_box(runtime.memory_allocation_summary().unwrap());
                assert_eq!(summary.current_generation, current_generation);
            });
            let report_phase = format!("{prefix}_{state}_report");
            measure(&report_phase, generation, 100, || {
                let report = black_box(runtime.memory_allocations().unwrap());
                assert_eq!(report.memories.len(), 255);
                assert_eq!(report.current_generation, current_generation);
            });
        }
    }
}
