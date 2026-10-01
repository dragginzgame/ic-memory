use ic_memory::ic_stable_structures::Memory;
use ic_memory::{
    GenericRangePolicy, MemoryManagerConfig, RuntimeBootstrapError, RuntimeConstructionError,
    RuntimeOpenError, RuntimeStateError, bootstrap_default_memory_manager,
    bootstrap_default_memory_manager_with_config, committed_allocations,
    default_memory_manager_memory_allocation_summary, default_memory_manager_memory_allocations,
    default_memory_manager_memory_id, is_default_memory_manager_bootstrapped,
    open_default_memory_manager_memory, open_default_memory_manager_memory_by_key,
    verify_default_memory_manager_authority,
};

const AUTHORITY: &str = "default_config";
const KEY: &str = "default_config.rows.v1";
const MEMORY_ID: u8 = 150;

ic_memory::ic_memory_range!(authority = AUTHORITY, start = MEMORY_ID, end = MEMORY_ID);
ic_memory::ic_memory_declaration!(
    authority = AUTHORITY,
    key = "default_config.rows.v1",
    label = "rows",
    id = MEMORY_ID,
);

#[test]
fn observations_allow_configured_generic_bootstrap_and_repeated_adoption() {
    for pages in [4, 16, 128] {
        // Each setting needs a fresh backing memory and TLS runtime.
        std::thread::spawn(move || {
            assert!(!is_default_memory_manager_bootstrapped().unwrap());
            assert_eq!(
                committed_allocations(),
                Err(RuntimeOpenError::NotBootstrapped)
            );
            assert!(matches!(
                open_default_memory_manager_memory(KEY, MEMORY_ID),
                Err(RuntimeOpenError::NotBootstrapped)
            ));
            assert!(matches!(
                open_default_memory_manager_memory_by_key(KEY),
                Err(RuntimeOpenError::NotBootstrapped)
            ));
            assert!(matches!(
                default_memory_manager_memory_allocations(),
                Err(ic_memory::RuntimeDiagnosticError::NotBootstrapped)
            ));
            assert_eq!(
                default_memory_manager_memory_id(KEY),
                Err(RuntimeOpenError::NotBootstrapped)
            );
            let declarations = ic_memory::sealed_declaration_snapshot().unwrap();
            assert_eq!(
                verify_default_memory_manager_authority(&declarations, AUTHORITY),
                Err(ic_memory::RuntimeAdoptionError::Open(
                    RuntimeOpenError::NotBootstrapped
                ))
            );
            assert!(matches!(
                default_memory_manager_memory_allocation_summary(),
                Err(ic_memory::RuntimeDiagnosticError::NotBootstrapped)
            ));
            let config = MemoryManagerConfig::new(pages).unwrap();
            let committed =
                bootstrap_default_memory_manager_with_config(config, &GenericRangePolicy).unwrap();
            assert!(is_default_memory_manager_bootstrapped().unwrap());
            assert_eq!(committed_allocations().unwrap(), committed);
            assert_eq!(default_memory_manager_memory_id(KEY), Ok(MEMORY_ID));
            verify_default_memory_manager_authority(&declarations, AUTHORITY).unwrap();

            let memory = open_default_memory_manager_memory(KEY, MEMORY_ID).unwrap();
            assert_eq!(memory.grow(1), Ok(0));
            memory.write(0, &[7, 8, 9]);
            let before = default_memory_manager_memory_allocations().unwrap();
            assert_eq!(before.bucket_size_pages, pages);
            let summary = default_memory_manager_memory_allocation_summary().unwrap();
            assert_eq!(summary.physical_extent, before.physical_extent);
            assert_eq!(summary.current_generation, Some(committed.generation()));
            assert_eq!(
                bootstrap_default_memory_manager_with_config(config, &GenericRangePolicy).unwrap(),
                committed
            );
            // Both public choices use the same built-in policy identity.
            assert_eq!(bootstrap_default_memory_manager().unwrap(), committed);
            let different = MemoryManagerConfig::new(pages + 1).unwrap();
            assert!(matches!(
                bootstrap_default_memory_manager_with_config(different, &GenericRangePolicy),
                Err(RuntimeBootstrapError::State(RuntimeStateError::Construction(
                    RuntimeConstructionError::BucketSizeMismatch { persisted, requested }
                ))) if persisted == pages && requested == pages + 1
            ));
            assert_eq!(committed_allocations().unwrap(), committed);
            assert_eq!(default_memory_manager_memory_allocations().unwrap(), before);
            let mut bytes = [0; 3];
            memory.read(0, &mut bytes);
            assert_eq!(bytes, [7, 8, 9]);
        })
        .join()
        .unwrap();
    }
}
