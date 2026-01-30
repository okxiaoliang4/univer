use ot_core::TransformService;

/// Total number of mutations in the system (88 total)
/// - Sheets Core: 53 (including remove.numfmt)
/// - Data Validation: 3
/// - Conditional Formatting: 4
/// - Sheets Filter: 4
/// - Sheets HyperLink: 5
/// - Sheets Note: 4
/// - Sheets Table: 4
/// - Sheets Pivot Table: 6
/// - Thread Comment: 5
const TOTAL_MUTATIONS: usize = 88;

/// Expected number of registry entries for 100% coverage
/// = symmetric (n) + bidirectional pairs (n * (n-1))
/// = n + n*(n-1) = n*n
const EXPECTED_FULL_COVERAGE: usize = TOTAL_MUTATIONS + (TOTAL_MUTATIONS * (TOTAL_MUTATIONS - 1));

#[test]
fn test_registry_coverage() {
    let service = TransformService::new();
    let size = service.registry_size();

    println!("Total registered transforms: {}", size);
    println!("Expected for full coverage: {}", EXPECTED_FULL_COVERAGE);
    println!("Coverage: {:.1}%", (size as f64 / EXPECTED_FULL_COVERAGE as f64) * 100.0);

    // Verify 100% coverage
    assert_eq!(
        size, EXPECTED_FULL_COVERAGE,
        "Expected {} transforms for 100% coverage ({} mutations), got {}",
        EXPECTED_FULL_COVERAGE, TOTAL_MUTATIONS, size
    );
}

#[test]
fn test_registry_has_all_symmetric_transforms() {
    let service = TransformService::new();

    // Verify service creation works and has correct registry size
    assert_eq!(service.registry_size(), EXPECTED_FULL_COVERAGE);
}
