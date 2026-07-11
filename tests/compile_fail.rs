#[test]
fn hard_cut_authority_boundaries_are_compile_time_enforced() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/fabricate_allocation_records.rs");
    tests.compile_fail("tests/ui/fabricate_committed_allocations.rs");
    tests.compile_fail("tests/ui/fabricate_recovered_ledger.rs");
    tests.compile_fail("tests/ui/implicit_authority_macro_forms.rs");
    tests.compile_fail("tests/ui/open_with_precommit_allocations.rs");
    tests.compile_fail("tests/ui/fabricate_validated_allocations.rs");
}
