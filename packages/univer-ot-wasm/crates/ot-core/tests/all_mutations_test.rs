use ot_core::TransformService;

/// Complete list of all JavaScript mutations that need Rust OT transforms.
/// This is the single source of truth for mutation coverage.
///
/// TDD: Add new mutations here first, then implement them.

// ============================================================================
// SHEETS CORE MUTATIONS (Base layer - structural and data operations)
// ============================================================================

const SHEETS_CORE_MUTATIONS: &[&str] = &[
    // Row/Column structure operations (already implemented)
    "sheet.mutation.insert-row",
    "sheet.mutation.insert-col",
    "sheet.mutation.remove-rows",
    "sheet.mutation.remove-col",

    // Data operations (already implemented)
    "sheet.mutation.set-range-values",
    "sheet.mutation.move-range",
    "sheet.mutation.move-rows",
    "sheet.mutation.move-columns",

    // Merge operations (already implemented)
    "sheet.mutation.add-worksheet-merge",
    "sheet.mutation.remove-worksheet-merge",  // NEW

    // Protection operations
    "sheet.mutation.set-range-protection",     // already implemented
    "sheet.mutation.add-range-protection",     // NEW
    "sheet.mutation.delete-range-protection",  // NEW

    // Theme/Style operations
    "sheet.mutation.set-range-theme",          // already implemented
    "sheet.mutation.add-range-theme",          // NEW
    "sheet.mutation.remove-range-theme",       // NEW

    // Formatting operations
    "sheet.mutation.set.numfmt",               // already implemented
    "sheet.mutation.remove.numfmt",            // Number format removal
    "sheet.mutation.set-row-data",             // already implemented
    "sheet.mutation.set-col-data",             // NEW
    "sheet.mutation.set-frozen",               // already implemented

    // Worksheet operations
    "sheet.mutation.insert-sheet",             // already implemented
    "sheet.mutation.remove-sheet",             // NEW
    "sheet.mutation.set-worksheet-name",       // NEW
    "sheet.mutation.set-worksheet-order",      // NEW
    "sheet.mutation.set-worksheet-hidden",     // NEW
    "sheet.mutation.copy-worksheet-end",       // NEW

    // Workbook operations
    "sheet.mutation.set-workbook-name",        // already implemented

    // Visibility operations
    "sheet.mutation.set-row-visible",          // NEW
    "sheet.mutation.set-row-hidden",           // NEW
    "sheet.mutation.set-col-visible",          // NEW
    "sheet.mutation.set-col-hidden",           // NEW

    // Dimension operations
    "sheet.mutation.set-worksheet-row-height",       // NEW
    "sheet.mutation.set-worksheet-row-is-auto-height", // NEW
    "sheet.mutation.set-worksheet-row-auto-height",  // NEW
    "sheet.mutation.set-worksheet-col-width",        // NEW
    "sheet.mutation.set-worksheet-row-count",        // NEW
    "sheet.mutation.set-worksheet-column-count",     // NEW

    // Grid operations
    "sheet.mutation.toggle-gridlines",         // NEW
    "sheet.mutation.set-gridlines-color",      // NEW

    // Tab operations
    "sheet.mutation.set-tab-color",            // NEW

    // Range reorder
    "sheet.mutation.reorder-range",            // NEW

    // Empty mutation (special case)
    "sheet.mutation.empty",                    // NEW

    // Worksheet protection
    "sheet.mutation.add-worksheet-protection",     // NEW
    "sheet.mutation.set-worksheet-protection",     // NEW
    "sheet.mutation.delete-worksheet-protection",  // NEW
    "sheet.mutation.set-worksheet-permission-points", // NEW

    // Worksheet style
    "sheet.mutation.set-worksheet-default-style",  // NEW
    "sheet.mutation.set-worksheet-right-to-left",  // NEW

    // Range theme style (worksheet level)
    "sheet.mutation.set-worksheet-range-theme-style",         // NEW
    "sheet.mutation.remove-worksheet-range-theme-style",      // NEW
    "sheet.mutation.register-worksheet-range-theme-style",    // NEW
    "sheet.mutation.unregister-worksheet-range-theme-style",  // NEW
];

// ============================================================================
// DATA VALIDATION MUTATIONS (already implemented)
// ============================================================================

const DATA_VALIDATION_MUTATIONS: &[&str] = &[
    "data-validation.mutation.addRule",
    "data-validation.mutation.removeRule",
    "data-validation.mutation.updateRule",
];

// ============================================================================
// CONDITIONAL FORMATTING MUTATIONS (already implemented)
// ============================================================================

const CONDITIONAL_FORMATTING_MUTATIONS: &[&str] = &[
    "sheet.mutation.add-conditional-rule",
    "sheet.mutation.delete-conditional-rule",
    "sheet.mutation.set-conditional-rule",
    "sheet.mutation.move-conditional-rule",
];

// ============================================================================
// SHEETS FILTER MUTATIONS (NEW)
// ============================================================================

const SHEETS_FILTER_MUTATIONS: &[&str] = &[
    "sheet.mutation.set-sheets-filter-range",
    "sheet.mutation.set-sheets-filter-criteria",
    "sheet.mutation.remove-sheets-filter",
    "sheet.mutation.re-calc-sheets-filter",
];

// ============================================================================
// SHEETS HYPERLINK MUTATIONS (NEW)
// ============================================================================

const SHEETS_HYPERLINK_MUTATIONS: &[&str] = &[
    "sheets.mutation.add-hyper-link",
    "sheets.mutation.remove-hyper-link",
    "sheets.mutation.update-hyper-link",
    "sheets.mutation.update-hyper-link-ref",
    "sheets.mutation.update-rich-hyper-link",
];

// ============================================================================
// SHEETS NOTE MUTATIONS (NEW)
// ============================================================================

const SHEETS_NOTE_MUTATIONS: &[&str] = &[
    "sheet.mutation.update-note",
    "sheet.mutation.remove-note",
    "sheet.mutation.toggle-note-popup",
    "sheet.mutation.update-note-position",
];

// ============================================================================
// SHEETS TABLE MUTATIONS (NEW)
// ============================================================================

const SHEETS_TABLE_MUTATIONS: &[&str] = &[
    "sheet.mutation.add-table",
    "sheet.mutation.set-sheet-table",
    "sheet.mutation.set-table-filter",
    "sheet.mutation.delete-table",
];

// ============================================================================
// SHEETS PIVOT TABLE MUTATIONS (NEW)
// ============================================================================

const SHEETS_PIVOT_TABLE_MUTATIONS: &[&str] = &[
    "sheet.mutation.add-pivot-table",
    "sheet.mutation.remove-pivot-table",
    "sheet.mutation.set-pivot-table-source-range",
    "sheet.mutation.set-pivot-table-target-cell",
    "sheet.mutation.set-pivot-table-fields-config",
    "sheet.mutation.set-pivot-table-calculated-data",
];

// ============================================================================
// THREAD COMMENT MUTATIONS (NEW)
// ============================================================================

const THREAD_COMMENT_MUTATIONS: &[&str] = &[
    "thread-comment.mutation.add-comment",
    "thread-comment.mutation.update-comment",
    "thread-comment.mutation.update-comment-ref",
    "thread-comment.mutation.resolve-comment",
    "thread-comment.mutation.delete-comment",
];

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn get_all_mutations() -> Vec<&'static str> {
    let mut all = Vec::new();
    all.extend_from_slice(SHEETS_CORE_MUTATIONS);
    all.extend_from_slice(DATA_VALIDATION_MUTATIONS);
    all.extend_from_slice(CONDITIONAL_FORMATTING_MUTATIONS);
    all.extend_from_slice(SHEETS_FILTER_MUTATIONS);
    all.extend_from_slice(SHEETS_HYPERLINK_MUTATIONS);
    all.extend_from_slice(SHEETS_NOTE_MUTATIONS);
    all.extend_from_slice(SHEETS_TABLE_MUTATIONS);
    all.extend_from_slice(SHEETS_PIVOT_TABLE_MUTATIONS);
    all.extend_from_slice(THREAD_COMMENT_MUTATIONS);
    all
}

// ============================================================================
// TESTS
// ============================================================================

#[test]
fn test_all_mutations_count() {
    let all = get_all_mutations();
    let expected_count =
        SHEETS_CORE_MUTATIONS.len() +
        DATA_VALIDATION_MUTATIONS.len() +
        CONDITIONAL_FORMATTING_MUTATIONS.len() +
        SHEETS_FILTER_MUTATIONS.len() +
        SHEETS_HYPERLINK_MUTATIONS.len() +
        SHEETS_NOTE_MUTATIONS.len() +
        SHEETS_TABLE_MUTATIONS.len() +
        SHEETS_PIVOT_TABLE_MUTATIONS.len() +
        THREAD_COMMENT_MUTATIONS.len();

    println!("=== MUTATION COUNTS ===");
    println!("Sheets Core: {}", SHEETS_CORE_MUTATIONS.len());
    println!("Data Validation: {}", DATA_VALIDATION_MUTATIONS.len());
    println!("Conditional Formatting: {}", CONDITIONAL_FORMATTING_MUTATIONS.len());
    println!("Sheets Filter: {}", SHEETS_FILTER_MUTATIONS.len());
    println!("Sheets HyperLink: {}", SHEETS_HYPERLINK_MUTATIONS.len());
    println!("Sheets Note: {}", SHEETS_NOTE_MUTATIONS.len());
    println!("Sheets Table: {}", SHEETS_TABLE_MUTATIONS.len());
    println!("Sheets Pivot Table: {}", SHEETS_PIVOT_TABLE_MUTATIONS.len());
    println!("Thread Comment: {}", THREAD_COMMENT_MUTATIONS.len());
    println!("========================");
    println!("TOTAL: {}", expected_count);

    assert_eq!(all.len(), expected_count);
}

#[test]
fn test_all_mutations_have_symmetric_transforms() {
    let service = TransformService::new();
    let all = get_all_mutations();
    let mut missing = Vec::new();

    for &mutation in &all {
        // Check if symmetric transform exists (mutation vs itself)
        if !service.has_transform(mutation, mutation) {
            missing.push(mutation);
        }
    }

    if !missing.is_empty() {
        println!("\n=== MISSING SYMMETRIC TRANSFORMS ===");
        for m in &missing {
            println!("  - {}", m);
        }
        println!("Total missing: {}", missing.len());
        println!("=====================================\n");
    }

    assert!(
        missing.is_empty(),
        "Missing symmetric transforms for {} mutations. See output above.",
        missing.len()
    );
}

#[test]
fn test_all_mutation_pairs_have_transforms() {
    let service = TransformService::new();
    let all = get_all_mutations();
    let mut missing_pairs = Vec::new();

    for (i, &m1) in all.iter().enumerate() {
        for &m2 in all.iter().skip(i + 1) {
            // Check both directions
            let has_m1_m2 = service.has_transform(m1, m2);
            let has_m2_m1 = service.has_transform(m2, m1);

            if !has_m1_m2 {
                missing_pairs.push((m1, m2));
            }
            if !has_m2_m1 {
                missing_pairs.push((m2, m1));
            }
        }
    }

    if !missing_pairs.is_empty() {
        println!("\n=== MISSING BIDIRECTIONAL TRANSFORMS ===");
        for (m1, m2) in missing_pairs.iter().take(50) {
            println!("  {} vs {}", m1, m2);
        }
        if missing_pairs.len() > 50 {
            println!("  ... and {} more", missing_pairs.len() - 50);
        }
        println!("Total missing pairs: {}", missing_pairs.len());
        println!("=========================================\n");
    }

    assert!(
        missing_pairs.is_empty(),
        "Missing bidirectional transforms for {} pairs. See output above.",
        missing_pairs.len()
    );
}

#[test]
fn test_registry_full_coverage() {
    let service = TransformService::new();
    let all = get_all_mutations();
    let n = all.len();

    // Expected: n symmetric + n*(n-1) bidirectional = n + n*(n-1) = n*n
    let expected_entries = n + n * (n - 1);
    let actual_entries = service.registry_size();

    println!("\n=== REGISTRY COVERAGE ===");
    println!("Total mutations: {}", n);
    println!("Expected entries (n + n*(n-1)): {}", expected_entries);
    println!("Actual entries: {}", actual_entries);
    println!("Coverage: {:.1}%", (actual_entries as f64 / expected_entries as f64) * 100.0);
    println!("=========================\n");

    assert_eq!(
        actual_entries, expected_entries,
        "Registry should have {} entries for 100% coverage, but has {}",
        expected_entries, actual_entries
    );
}

/// Test to list which categories have full coverage
#[test]
fn test_category_coverage_report() {
    let service = TransformService::new();

    fn check_category_coverage(service: &TransformService, name: &str, mutations: &[&str]) -> (usize, usize) {
        let mut registered = 0;
        let total = mutations.len();

        for &m in mutations {
            if service.has_transform(m, m) {
                registered += 1;
            }
        }

        println!("{}: {}/{} ({:.0}%)",
            name,
            registered,
            total,
            (registered as f64 / total as f64) * 100.0
        );

        (registered, total)
    }

    println!("\n=== CATEGORY COVERAGE REPORT ===");
    let mut total_registered = 0;
    let mut total_mutations = 0;

    let (r, t) = check_category_coverage(&service, "Sheets Core", SHEETS_CORE_MUTATIONS);
    total_registered += r;
    total_mutations += t;

    let (r, t) = check_category_coverage(&service, "Data Validation", DATA_VALIDATION_MUTATIONS);
    total_registered += r;
    total_mutations += t;

    let (r, t) = check_category_coverage(&service, "Conditional Formatting", CONDITIONAL_FORMATTING_MUTATIONS);
    total_registered += r;
    total_mutations += t;

    let (r, t) = check_category_coverage(&service, "Sheets Filter", SHEETS_FILTER_MUTATIONS);
    total_registered += r;
    total_mutations += t;

    let (r, t) = check_category_coverage(&service, "Sheets HyperLink", SHEETS_HYPERLINK_MUTATIONS);
    total_registered += r;
    total_mutations += t;

    let (r, t) = check_category_coverage(&service, "Sheets Note", SHEETS_NOTE_MUTATIONS);
    total_registered += r;
    total_mutations += t;

    let (r, t) = check_category_coverage(&service, "Sheets Table", SHEETS_TABLE_MUTATIONS);
    total_registered += r;
    total_mutations += t;

    let (r, t) = check_category_coverage(&service, "Sheets Pivot Table", SHEETS_PIVOT_TABLE_MUTATIONS);
    total_registered += r;
    total_mutations += t;

    let (r, t) = check_category_coverage(&service, "Thread Comment", THREAD_COMMENT_MUTATIONS);
    total_registered += r;
    total_mutations += t;

    println!("================================");
    println!("TOTAL: {}/{} ({:.0}%)",
        total_registered,
        total_mutations,
        (total_registered as f64 / total_mutations as f64) * 100.0
    );
    println!("================================\n");
}
