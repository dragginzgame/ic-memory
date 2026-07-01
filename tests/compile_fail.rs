#[test]
fn authority_capabilities_are_not_externally_fabricable() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/fabricate_allocation_records.rs");
    tests.compile_fail("tests/ui/fabricate_recovered_ledger.rs");
    tests.compile_fail("tests/ui/fabricate_validated_allocations.rs");
}
