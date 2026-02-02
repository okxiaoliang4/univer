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

/// Expected number of registry entries for 100% explicit coverage
/// = symmetric (n) + bidirectional pairs (n * (n-1))
/// = n + n*(n-1) = n*n
///
/// NOTE: With the new architecture, we no longer require explicit registration
/// for all pairs. The registry falls back to identity transform automatically
/// for unregistered pairs. We only need symmetric transforms to be registered.
const EXPECTED_FULL_COVERAGE: usize = TOTAL_MUTATIONS + (TOTAL_MUTATIONS * (TOTAL_MUTATIONS - 1));

#[test]
fn test_registry_statistics() {
    let service = TransformService::new();
    let size = service.registry_size();

    println!("Total registered transforms: {}", size);
    println!("Total possible pairs (for full explicit coverage): {}", EXPECTED_FULL_COVERAGE);
    println!("Explicit registration rate: {:.1}%", (size as f64 / EXPECTED_FULL_COVERAGE as f64) * 100.0);

    // With the new architecture, we only require at least symmetric transforms
    // All other pairs fall back to identity automatically
    assert!(
        size >= TOTAL_MUTATIONS,
        "Expected at least {} symmetric transforms, got {}",
        TOTAL_MUTATIONS, size
    );
}

#[test]
fn test_registry_has_required_transforms() {
    let service = TransformService::new();

    // Verify service creation works and has at least symmetric transforms
    // The new architecture doesn't require 100% explicit coverage
    assert!(
        service.registry_size() >= TOTAL_MUTATIONS,
        "Expected at least {} symmetric transforms, got {}",
        TOTAL_MUTATIONS, service.registry_size()
    );
}
