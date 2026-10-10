//! Installed IO/lifecycle qualification fixture, not an application integration API.
//! Raw replies avoid adding a CDK or Candid dependency to the library graph.

#[cfg(target_arch = "wasm32")]
#[expect(
    unsafe_code,
    reason = "test-only raw IC system calls and Memory::read_unsafe"
)]
mod canister {
    use super::pool;
    use ic_memory::{
        GenericAllocationPolicy, MemoryManagerConfig, bootstrap_default_memory_manager_with_config,
        committed_allocations, default_memory_manager_memory_allocation_summary,
        ic_stable_structures::Memory, open_default_memory_manager_memory,
    };

    const PAGE: u64 = 65_536;
    const ROWS: &str = "io_qualification.rows.v1";
    const NEIGHBOR: &str = "io_qualification.neighbor.v1";

    ic_memory::ic_memory_declaration!(
        authority = "io_qualification",
        key = "io_qualification.neighbor.v1"
    );
    ic_memory::ic_memory_declaration!(
        authority = "io_qualification",
        key = "io_qualification.rows.v1"
    );

    #[link(wasm_import_module = "ic0")]
    unsafe extern "C" {
        fn msg_arg_data_size() -> usize;
        fn msg_arg_data_copy(dst: usize, offset: usize, size: usize);
        fn msg_reply_data_append(src: usize, size: usize);
        fn msg_reply();
        fn performance_counter(counter_type: u32) -> u64;
    }

    fn arguments<const N: usize>() -> [u8; N] {
        let mut bytes = [0; N];
        // SAFETY: The fixed-size destination is writable; assert before copying.
        unsafe {
            assert_eq!(msg_arg_data_size(), N);
            msg_arg_data_copy(bytes.as_mut_ptr() as usize, 0, N);
        }
        bytes
    }

    fn reply(bytes: &[u8]) {
        // SAFETY: The source remains valid until the system copies it.
        unsafe {
            msg_reply_data_append(bytes.as_ptr() as usize, bytes.len());
            msg_reply();
        }
    }

    fn bootstrap(mode: u8) {
        let pages = if mode == 2 { 16 } else { 8 };
        bootstrap_default_memory_manager_with_config(
            MemoryManagerConfig::new(pages).unwrap(),
            &pool(),
            &GenericAllocationPolicy,
        )
        .unwrap();
        assert_ne!(mode, 1, "injected trap after durable bootstrap write");
    }

    #[unsafe(export_name = "canister_init")]
    extern "C" fn init() {
        bootstrap(0);
        let neighbor = open_default_memory_manager_memory(NEIGHBOR).unwrap();
        neighbor.grow(8).unwrap();
        neighbor.write(8 * PAGE - 1, &[0x6D]);
        let rows = open_default_memory_manager_memory(ROWS).unwrap();
        rows.grow(1).unwrap();
        rows.write(0, &[0x42]);
    }

    #[unsafe(export_name = "canister_post_upgrade")]
    extern "C" fn post_upgrade() {
        bootstrap(arguments::<1>()[0]);
    }

    #[unsafe(export_name = "canister_query snapshot")]
    extern "C" fn snapshot() {
        let mut bytes = committed_allocations()
            .unwrap()
            .generation()
            .to_le_bytes()
            .to_vec();
        // Check assigned identities independently of payload retention.
        assert_eq!(
            ic_memory::default_memory_manager_memory_id(NEIGHBOR).unwrap(),
            10
        );
        assert_eq!(
            ic_memory::default_memory_manager_memory_id(ROWS).unwrap(),
            11
        );
        let rows = open_default_memory_manager_memory(ROWS).unwrap();
        let neighbor = open_default_memory_manager_memory(NEIGHBOR).unwrap();
        let mut markers = [0; 2];
        rows.read(0, &mut markers[..1]);
        neighbor.read(8 * PAGE - 1, &mut markers[1..]);
        bytes.extend_from_slice(&markers);
        let summary = default_memory_manager_memory_allocation_summary().unwrap();
        bytes.extend_from_slice(&summary.physical_extent.wasm_pages.to_le_bytes());
        bytes.extend_from_slice(&rows.size().to_le_bytes());
        reply(&bytes);
    }

    #[unsafe(export_name = "canister_update io")]
    extern "C" fn io() {
        let args = arguments::<10>();
        let operation = args[0];
        let count = usize::from(args[1]);
        assert!(count <= 2);
        let offset = u64::from_le_bytes(args[2..].try_into().unwrap());
        let rows = open_default_memory_manager_memory(ROWS).unwrap();
        // Warm the cache and dirty valid state before the candidate failure.
        // A trap must roll this mutation back as well as reject the invalid IO.
        rows.write(0, &[0x99]);
        let mut dst = [0; 2];
        // SAFETY: Counter zero is supported in update execution.
        let start = unsafe { performance_counter(0) };
        for _ in 0..1000 {
            match operation {
                0 => rows.read(offset, &mut dst[..count]),
                // SAFETY: count fits the separate writable destination.
                1 => unsafe { rows.read_unsafe(offset, dst.as_mut_ptr(), count) },
                2 => rows.write(offset, &[0xA5; 2][..count]),
                _ => panic!("unknown IO operation"),
            }
        }
        // SAFETY: Same counter/execution as above.
        let instructions = unsafe { performance_counter(0) } - start;
        let mut bytes = dst.to_vec();
        bytes.extend_from_slice(&instructions.to_le_bytes());
        reply(&bytes);
    }
}

#[cfg(target_arch = "wasm32")]
fn pool() -> ic_memory::MemoryAllocationPool {
    ic_memory::MemoryAllocationPool::new(
        vec![ic_memory::MemoryAuthority::new("io_qualification", "io_qualification.").unwrap()],
        vec![],
    )
    .unwrap()
}
