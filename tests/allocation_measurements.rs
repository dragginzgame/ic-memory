//! Explicit native allocation measurements; not IC instruction benchmarks.
use ic_memory::{
    AllocationHistory, AllocationLedger, GenerationRecord, LedgerCommitStore,
    StableCellLedgerRecord, ic_stable_structures::Storable,
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

fn ledger(generations: u64) -> AllocationLedger {
    ledger_with_fingerprint(generations, None)
}

fn ledger_with_fingerprint(generations: u64, fingerprint: Option<&str>) -> AllocationLedger {
    let records: Vec<_> = (1..=generations)
        .map(|generation| {
            GenerationRecord::new(
                generation,
                generation - 1,
                fingerprint.map(str::to_owned),
                0,
                None,
            )
            .unwrap()
        })
        .collect();
    let history: AllocationHistory = serde_json::from_value(serde_json::json!({
        "records": [], "generations": records,
    }))
    .unwrap();
    AllocationLedger::new_committed(generations, history).unwrap()
}

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
    for generations in [1, 64, 1024, 16_384, 65_536] {
        let ledger = ledger(generations);
        let mut store = LedgerCommitStore::default();
        store.commit(&self::ledger(generations - 1)).unwrap();
        store.commit(&ledger).unwrap();
        let record = StableCellLedgerRecord::new(store);
        measure("encode", generations, 10, || {
            drop(black_box(record.to_bytes()));
        });
        measure("validate", generations, 10, || {
            black_box(ledger.validate_committed_integrity()).unwrap();
        });
        measure("recover", generations, 10, || {
            drop(black_box(record.store().recover().unwrap()));
        });
        measure("commit", generations, 10, || {
            let mut store = LedgerCommitStore::default();
            drop(black_box(store.commit(&ledger).unwrap()));
        });
    }

    // Histories with metadata exercise the byte ceiling separately from the
    // generation-count ceiling. Fixtures and their strings are not measured.
    for fingerprint_len in [128, 256] {
        let fingerprint = "x".repeat(fingerprint_len);
        let generations = 65_536;
        let ledger = ledger_with_fingerprint(generations, Some(&fingerprint));
        let phase = format!("commit_fingerprint_{fingerprint_len}");
        measure(&phase, generations, 10, || {
            let mut store = LedgerCommitStore::default();
            let result = black_box(store.commit(&ledger));
            assert_eq!(result.is_ok(), fingerprint_len == 128);
        });
    }
}
