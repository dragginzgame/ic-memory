//! Disposable native fixtures, not Toko live attribution or IC instruction costs.
#[path = "support/metered.rs"]
mod metered;
use ic_memory::{
    AllocationPolicy, AllocationSlotDescriptor, MemoryAllocations, MemoryManagerConfig,
    MemoryRuntime, PolicyIdentity, PolicyIdentityError, RuntimeBootstrapPolicy, RuntimeMemory,
    StableKey,
};
use ic_stable_structures::{Cell, Memory, StableVec};
use metered::Metered;
use std::{convert::Infallible, hint::black_box, time::Instant};

struct Allow;
impl AllocationPolicy for Allow {
    type Error = Infallible;
    fn validate_key(&self, _: &StableKey) -> Result<(), Infallible> {
        Ok(())
    }
    fn validate_slot(&self, _: &StableKey, _: &AllocationSlotDescriptor) -> Result<(), Infallible> {
        Ok(())
    }
    fn validate_reserved_slot(
        &self,
        _: &StableKey,
        _: &AllocationSlotDescriptor,
    ) -> Result<(), Infallible> {
        Ok(())
    }
}
impl RuntimeBootstrapPolicy for Allow {
    fn runtime_bootstrap_identity(&self) -> Result<PolicyIdentity, PolicyIdentityError> {
        PolicyIdentity::new("allocation-measurements", 1)
    }
}
fn record(
    bucket: u16,
    phase: &str,
    counts: metered::Counts,
    elapsed: u128,
    report: &MemoryAllocations,
) {
    assert_eq!(
        report.physical_extent.bytes,
        report.manager_metadata_bytes + report.allocated_bucket_bytes + report.unmanaged_bytes
    );
    println!(
        "{bucket},{phase},{},{},{},{},{},{},{},{},{},{},{elapsed}",
        report.physical_extent.bytes,
        report.virtual_extent.bytes,
        report.allocated_buckets,
        report.memories[0].allocated_bytes,
        counts.grows,
        counts.grown_pages,
        counts.reads,
        counts.read_bytes,
        counts.writes,
        counts.write_bytes
    );
}
// Keep timed phases and their reset boundaries together for measurement review.
#[allow(clippy::too_many_lines)]
fn main() {
    ic_memory::register_static_memory_manager_range(
        100,
        139,
        "fixture",
        ic_memory::MemoryManagerRangeMode::Reserved,
        None,
    )
    .unwrap();
    for id in 100..140 {
        ic_memory::register_static_memory_manager_declaration(
            id,
            "fixture",
            "small",
            format!("fixture.store{id}.v1"),
        )
        .unwrap();
    }
    let declarations = ic_memory::sealed_declaration_snapshot().unwrap();
    println!(
        "bucket_pages,phase,physical_bytes,virtual_bytes,buckets,ledger_bucket_bytes,grows,grown_pages,reads,read_bytes,writes,write_bytes,elapsed_us"
    );
    for bucket in [128, 16, 8, 1] {
        let memory = Metered::default();
        let start = Instant::now();
        let mut runtime = MemoryRuntime::new_with_config(
            memory.clone(),
            MemoryManagerConfig::new(bucket).unwrap(),
        )
        .unwrap();
        record(
            bucket,
            "construction",
            memory.counts(),
            start.elapsed().as_micros(),
            &runtime.memory_allocations().unwrap(),
        );
        memory.reset();
        let start = Instant::now();
        runtime.bootstrap(&declarations, &Allow).unwrap();
        let elapsed = start.elapsed().as_micros();
        record(
            bucket,
            "bootstrap",
            memory.counts(),
            elapsed,
            &runtime.memory_allocations().unwrap(),
        );
        memory.reset();
        let start = Instant::now();
        for id in 100..128 {
            let handle = runtime
                .open_memory(&format!("fixture.store{id}.v1"), id)
                .unwrap();
            black_box(Cell::init(handle, 7_u64));
        }
        let elapsed = start.elapsed().as_micros();
        record(
            bucket,
            "28_small_cells",
            memory.counts(),
            elapsed,
            &runtime.memory_allocations().unwrap(),
        );
        let unopened = runtime.memory_allocations().unwrap();
        assert_eq!(unopened.memories[139].allocated_bytes, 0);
        let handle = runtime.open_memory("fixture.store128.v1", 128).unwrap();
        let growing = StableVec::<[u8; 1024], RuntimeMemory<Metered>>::init(handle.clone());
        memory.reset();
        let start = Instant::now();
        for index in 0..8192_u64 {
            let mut value = [0; 1024];
            value[..8].copy_from_slice(&index.to_le_bytes());
            growing.push(&value);
        }
        let elapsed = start.elapsed().as_micros();
        record(
            bucket,
            "8192_vec_rows",
            memory.counts(),
            elapsed,
            &runtime.memory_allocations().unwrap(),
        );
        memory.reset();
        let start = Instant::now();
        for index in 0..20_000_u64 {
            let row = (index * 7919) % 8192;
            let value = growing.get(row).unwrap();
            assert_eq!(&value[..8], &row.to_le_bytes());
            black_box(value);
        }
        let elapsed = start.elapsed().as_micros();
        record(
            bucket,
            "20000_random_gets",
            memory.counts(),
            elapsed,
            &runtime.memory_allocations().unwrap(),
        );
        memory.reset();
        let start = Instant::now();
        for _ in 0..20_000 {
            let mut bytes = [0; 32];
            handle.read(u64::from(bucket) * 65_536 - 16, &mut bytes);
            black_box(bytes);
        }
        let elapsed = start.elapsed().as_micros();
        record(
            bucket,
            "20000_crossing_reads",
            memory.counts(),
            elapsed,
            &runtime.memory_allocations().unwrap(),
        );
        drop(growing);
        drop(handle);
        let before = runtime.memory_allocations().unwrap();
        drop(runtime);
        memory.reset();
        let start = Instant::now();
        let mut runtime = MemoryRuntime::new(memory.clone()).unwrap();
        let report = runtime.memory_allocations().unwrap();
        assert_eq!(report.physical_extent, before.physical_extent);
        assert!(report.unknown_binding_bytes > 0);
        assert_eq!(memory.counts().writes, 0);
        assert_eq!(memory.counts().grows, 0);
        record(
            bucket,
            "reopen_unbound",
            memory.counts(),
            start.elapsed().as_micros(),
            &report,
        );
        runtime.bootstrap(&declarations, &Allow).unwrap();
        let recovered = runtime.memory_allocations().unwrap();
        memory.reset();
        let start = Instant::now();
        for _ in 0..100 {
            assert_eq!(runtime.memory_allocations().unwrap(), recovered);
        }
        record(
            bucket,
            "100_readonly_replays",
            memory.counts(),
            start.elapsed().as_micros(),
            &recovered,
        );
        assert_eq!(memory.counts().writes, 0);
        assert_eq!(memory.counts().grows, 0);
        memory.reset();
        let start = Instant::now();
        for _ in 0..64 {
            drop(runtime);
            runtime = MemoryRuntime::new(memory.clone()).unwrap();
            runtime.bootstrap(&declarations, &Allow).unwrap();
        }
        record(
            bucket,
            "64_bootstrap_generations",
            memory.counts(),
            start.elapsed().as_micros(),
            &runtime.memory_allocations().unwrap(),
        );
        memory.reset();
        let start = Instant::now();
        let historical = runtime.memory_allocations().unwrap();
        record(
            bucket,
            "history_independent_report",
            memory.counts(),
            start.elapsed().as_micros(),
            &historical,
        );
        assert_eq!(memory.counts().read_bytes, 34_848);
        assert_eq!(memory.counts().writes, 0);
        assert_eq!(memory.counts().grows, 0);
    }
}
