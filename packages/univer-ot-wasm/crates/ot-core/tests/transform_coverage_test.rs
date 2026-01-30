//! Transform coverage test - analyzes which mutation pairs have explicit registrations
//! and which are using the default identity fallback.

use ot_core::TransformService;

/// All mutation IDs in the system (88 total)
const ALL_MUTATIONS: &[&str] = &[
    // Sheets Core (53)
    "sheet.mutation.insert-row",
    "sheet.mutation.insert-col",
    "sheet.mutation.remove-rows",
    "sheet.mutation.remove-col",
    "sheet.mutation.set-range-values",
    "sheet.mutation.move-range",
    "sheet.mutation.move-rows",
    "sheet.mutation.move-columns",
    "sheet.mutation.add-worksheet-merge",
    "sheet.mutation.set-range-protection",
    "sheet.mutation.set-range-theme",
    "sheet.mutation.set.numfmt",
    "sheet.mutation.remove.numfmt",
    "sheet.mutation.set-frozen",
    "sheet.mutation.set-row-data",
    "sheet.mutation.insert-sheet",
    "sheet.mutation.set-workbook-name",
    "sheet.mutation.remove-worksheet-merge",
    "sheet.mutation.add-range-protection",
    "sheet.mutation.delete-range-protection",
    "sheet.mutation.add-range-theme",
    "sheet.mutation.remove-range-theme",
    "sheet.mutation.set-col-data",
    "sheet.mutation.remove-sheet",
    "sheet.mutation.set-worksheet-name",
    "sheet.mutation.set-worksheet-order",
    "sheet.mutation.set-worksheet-hidden",
    "sheet.mutation.copy-worksheet-end",
    "sheet.mutation.set-row-visible",
    "sheet.mutation.set-row-hidden",
    "sheet.mutation.set-col-visible",
    "sheet.mutation.set-col-hidden",
    "sheet.mutation.set-worksheet-row-height",
    "sheet.mutation.set-worksheet-row-is-auto-height",
    "sheet.mutation.set-worksheet-row-auto-height",
    "sheet.mutation.set-worksheet-col-width",
    "sheet.mutation.set-worksheet-row-count",
    "sheet.mutation.set-worksheet-column-count",
    "sheet.mutation.toggle-gridlines",
    "sheet.mutation.set-gridlines-color",
    "sheet.mutation.set-tab-color",
    "sheet.mutation.reorder-range",
    "sheet.mutation.empty",
    "sheet.mutation.add-worksheet-protection",
    "sheet.mutation.set-worksheet-protection",
    "sheet.mutation.delete-worksheet-protection",
    "sheet.mutation.set-worksheet-permission-points",
    "sheet.mutation.set-worksheet-default-style",
    "sheet.mutation.set-worksheet-right-to-left",
    "sheet.mutation.set-worksheet-range-theme-style",
    "sheet.mutation.remove-worksheet-range-theme-style",
    "sheet.mutation.register-worksheet-range-theme-style",
    "sheet.mutation.unregister-worksheet-range-theme-style",
    // Data Validation (3)
    "data-validation.mutation.addRule",
    "data-validation.mutation.removeRule",
    "data-validation.mutation.updateRule",
    // Conditional Formatting (4)
    "sheet.mutation.add-conditional-rule",
    "sheet.mutation.delete-conditional-rule",
    "sheet.mutation.set-conditional-rule",
    "sheet.mutation.move-conditional-rule",
    // Filter (4)
    "sheet.mutation.set-filter-range",
    "sheet.mutation.set-filter-criteria",
    "sheet.mutation.remove-filter",
    "sheet.mutation.re-calc-filter",
    // HyperLink (5)
    "sheets.mutation.add-hyper-link",
    "sheets.mutation.remove-hyper-link",
    "sheets.mutation.update-hyper-link",
    "sheets.mutation.update-hyper-link-ref",
    "sheets.mutation.update-rich-hyper-link",
    // Note (4)
    "sheet.mutation.update-note",
    "sheet.mutation.remove-note",
    "sheet.mutation.toggle-note-popup",
    "sheet.mutation.update-note-position",
    // Table (4)
    "sheet.mutation.add-table",
    "sheet.mutation.set-sheet-table",
    "sheet.mutation.set-table-filter",
    "sheet.mutation.delete-table",
    // Pivot Table (6)
    "sheet.mutation.add-pivot-table",
    "sheet.mutation.remove-pivot-table",
    "sheet.mutation.set-pivot-table-source-range",
    "sheet.mutation.set-pivot-table-target-cell",
    "sheet.mutation.set-pivot-table-fields-config",
    "sheet.mutation.set-pivot-table-calculated-data",
    // Thread Comment (5)
    "thread-comment.mutation.add-comment",
    "thread-comment.mutation.update-comment",
    "thread-comment.mutation.update-comment-ref",
    "thread-comment.mutation.resolve-comment",
    "thread-comment.mutation.delete-comment",
];

/// Sheets core mutations only (53 total)
const SHEETS_CORE_MUTATIONS: &[&str] = &[
    "sheet.mutation.insert-row",
    "sheet.mutation.insert-col",
    "sheet.mutation.remove-rows",
    "sheet.mutation.remove-col",
    "sheet.mutation.set-range-values",
    "sheet.mutation.move-range",
    "sheet.mutation.move-rows",
    "sheet.mutation.move-columns",
    "sheet.mutation.add-worksheet-merge",
    "sheet.mutation.set-range-protection",
    "sheet.mutation.set-range-theme",
    "sheet.mutation.set.numfmt",
    "sheet.mutation.remove.numfmt",
    "sheet.mutation.set-frozen",
    "sheet.mutation.set-row-data",
    "sheet.mutation.insert-sheet",
    "sheet.mutation.set-workbook-name",
    "sheet.mutation.remove-worksheet-merge",
    "sheet.mutation.add-range-protection",
    "sheet.mutation.delete-range-protection",
    "sheet.mutation.add-range-theme",
    "sheet.mutation.remove-range-theme",
    "sheet.mutation.set-col-data",
    "sheet.mutation.remove-sheet",
    "sheet.mutation.set-worksheet-name",
    "sheet.mutation.set-worksheet-order",
    "sheet.mutation.set-worksheet-hidden",
    "sheet.mutation.copy-worksheet-end",
    "sheet.mutation.set-row-visible",
    "sheet.mutation.set-row-hidden",
    "sheet.mutation.set-col-visible",
    "sheet.mutation.set-col-hidden",
    "sheet.mutation.set-worksheet-row-height",
    "sheet.mutation.set-worksheet-row-is-auto-height",
    "sheet.mutation.set-worksheet-row-auto-height",
    "sheet.mutation.set-worksheet-col-width",
    "sheet.mutation.set-worksheet-row-count",
    "sheet.mutation.set-worksheet-column-count",
    "sheet.mutation.toggle-gridlines",
    "sheet.mutation.set-gridlines-color",
    "sheet.mutation.set-tab-color",
    "sheet.mutation.reorder-range",
    "sheet.mutation.empty",
    "sheet.mutation.add-worksheet-protection",
    "sheet.mutation.set-worksheet-protection",
    "sheet.mutation.delete-worksheet-protection",
    "sheet.mutation.set-worksheet-permission-points",
    "sheet.mutation.set-worksheet-default-style",
    "sheet.mutation.set-worksheet-right-to-left",
    "sheet.mutation.set-worksheet-range-theme-style",
    "sheet.mutation.remove-worksheet-range-theme-style",
    "sheet.mutation.register-worksheet-range-theme-style",
    "sheet.mutation.unregister-worksheet-range-theme-style",
];

#[test]
fn test_analyze_transform_coverage() {
    let service = TransformService::new();

    println!("\n========================================");
    println!("Transform Coverage Analysis");
    println!("========================================\n");

    println!("Total mutations: {}", ALL_MUTATIONS.len());
    println!("Total registry entries: {}", service.registry_size());

    // Calculate expected pairs
    let n = ALL_MUTATIONS.len();
    let expected_symmetric = n;
    let expected_pairs = n * (n - 1) / 2;
    let expected_total = expected_symmetric + expected_pairs * 2; // Each pair registers both directions

    println!("\nTheoretical maximum:");
    println!("  - {} symmetric (self vs self)", expected_symmetric);
    println!("  - {} unique pairs × 2 directions = {} entries", expected_pairs, expected_pairs * 2);
    println!("  - Total if all registered: {}", expected_total);

    println!("\n----------------------------------------");
    println!("Registry Statistics:");
    println!("  Current: {} / {} ({:.1}%)",
        service.registry_size(),
        expected_total,
        (service.registry_size() as f64 / expected_total as f64) * 100.0
    );
    println!("----------------------------------------\n");
}

#[test]
fn test_list_missing_sheets_core_transforms() {
    println!("\n========================================");
    println!("Missing Sheets Core Transform Pairs");
    println!("========================================\n");

    // Based on code analysis, these are the registered pairs:
    let registered_pairs: Vec<(&str, &str)> = vec![
        // insert-row
        ("sheet.mutation.insert-row", "sheet.mutation.insert-row"),
        ("sheet.mutation.insert-row", "sheet.mutation.set-range-values"),

        // insert-col
        ("sheet.mutation.insert-col", "sheet.mutation.insert-col"),
        ("sheet.mutation.insert-col", "sheet.mutation.set-range-values"),
        ("sheet.mutation.insert-col", "sheet.mutation.insert-row"),

        // remove-rows
        ("sheet.mutation.remove-rows", "sheet.mutation.remove-rows"),
        ("sheet.mutation.remove-rows", "sheet.mutation.set-range-values"),
        ("sheet.mutation.remove-rows", "sheet.mutation.insert-row"),
        ("sheet.mutation.remove-rows", "sheet.mutation.insert-col"),
        ("sheet.mutation.remove-rows", "sheet.mutation.remove-col"),

        // remove-col
        ("sheet.mutation.remove-col", "sheet.mutation.remove-col"),
        ("sheet.mutation.remove-col", "sheet.mutation.set-range-values"),
        ("sheet.mutation.remove-col", "sheet.mutation.insert-col"),
        ("sheet.mutation.remove-col", "sheet.mutation.insert-row"),

        // set-range-values
        ("sheet.mutation.set-range-values", "sheet.mutation.set-range-values"),

        // move-range
        ("sheet.mutation.move-range", "sheet.mutation.move-range"),
        ("sheet.mutation.move-range", "sheet.mutation.insert-row"),
        ("sheet.mutation.move-range", "sheet.mutation.insert-col"),
        ("sheet.mutation.move-range", "sheet.mutation.set-range-values"),

        // move-rows
        ("sheet.mutation.move-rows", "sheet.mutation.move-rows"),
        ("sheet.mutation.move-rows", "sheet.mutation.insert-row"),
        ("sheet.mutation.move-rows", "sheet.mutation.remove-rows"),
        ("sheet.mutation.move-rows", "sheet.mutation.set-range-values"),
        ("sheet.mutation.move-rows", "sheet.mutation.insert-col"),
        ("sheet.mutation.move-rows", "sheet.mutation.remove-col"),
        ("sheet.mutation.move-rows", "sheet.mutation.move-columns"),

        // move-columns
        ("sheet.mutation.move-columns", "sheet.mutation.move-columns"),
        ("sheet.mutation.move-columns", "sheet.mutation.insert-row"),
        ("sheet.mutation.move-columns", "sheet.mutation.remove-rows"),
        ("sheet.mutation.move-columns", "sheet.mutation.set-range-values"),

        // merge
        ("sheet.mutation.add-worksheet-merge", "sheet.mutation.add-worksheet-merge"),
        ("sheet.mutation.add-worksheet-merge", "sheet.mutation.insert-row"),
        ("sheet.mutation.add-worksheet-merge", "sheet.mutation.insert-col"),

        // protection
        ("sheet.mutation.set-range-protection", "sheet.mutation.set-range-protection"),
        ("sheet.mutation.set-range-protection", "sheet.mutation.insert-row"),
        ("sheet.mutation.set-range-protection", "sheet.mutation.insert-col"),

        // theme
        ("sheet.mutation.set-range-theme", "sheet.mutation.set-range-theme"),

        // numfmt
        ("sheet.mutation.set.numfmt", "sheet.mutation.set.numfmt"),
        ("sheet.mutation.set.numfmt", "sheet.mutation.insert-row"),
        ("sheet.mutation.set.numfmt", "sheet.mutation.insert-col"),
        ("sheet.mutation.set.numfmt", "sheet.mutation.set-range-values"),

        // frozen
        ("sheet.mutation.set-frozen", "sheet.mutation.set-frozen"),
        ("sheet.mutation.set-frozen", "sheet.mutation.insert-row"),
        ("sheet.mutation.set-frozen", "sheet.mutation.insert-col"),
        ("sheet.mutation.set-frozen", "sheet.mutation.set-range-values"),

        // row_col_data
        ("sheet.mutation.set-row-data", "sheet.mutation.set-row-data"),
        ("sheet.mutation.set-row-data", "sheet.mutation.insert-row"),
        ("sheet.mutation.set-row-data", "sheet.mutation.set-range-values"),

        // worksheet
        ("sheet.mutation.insert-sheet", "sheet.mutation.insert-sheet"),

        // workbook
        ("sheet.mutation.set-workbook-name", "sheet.mutation.set-workbook-name"),
    ];

    // Convert to a set for easier lookup (order doesn't matter for pairs)
    let mut registered_set = std::collections::HashSet::new();
    for (a, b) in &registered_pairs {
        registered_set.insert((*a, *b));
        registered_set.insert((*b, *a)); // Add reverse too
    }

    // Find missing pairs
    let mut missing = Vec::new();
    for (i, m1) in SHEETS_CORE_MUTATIONS.iter().enumerate() {
        for m2 in SHEETS_CORE_MUTATIONS.iter().skip(i) {
            if !registered_set.contains(&(*m1, *m2)) {
                missing.push((*m1, *m2));
            }
        }
    }

    println!("Registered pairs (sheets core): {}", registered_pairs.len());
    println!("Missing pairs (sheets core): {}", missing.len());
    println!("\nMissing transform pairs:");
    println!("----------------------------------------");

    for (m1, m2) in &missing {
        let m1_short = m1.split('.').last().unwrap_or(m1);
        let m2_short = m2.split('.').last().unwrap_or(m2);
        println!("  {} ↔ {}", m1_short, m2_short);
    }

    println!("\n========================================");
    println!("Summary: {} missing pairs need to be registered", missing.len());
    println!("========================================\n");
}

#[test]
fn test_list_missing_cross_module_transforms() {
    println!("\n========================================");
    println!("Missing Cross-Module Transform Pairs");
    println!("========================================\n");

    let data_validation = &[
        "data-validation.mutation.addRule",
        "data-validation.mutation.removeRule",
        "data-validation.mutation.updateRule",
    ];

    let conditional_formatting = &[
        "sheet.mutation.add-conditional-rule",
        "sheet.mutation.delete-conditional-rule",
        "sheet.mutation.set-conditional-rule",
        "sheet.mutation.move-conditional-rule",
    ];

    // Registered cross-module pairs (from code analysis)
    let registered_dv_sheets: Vec<(&str, &str)> = vec![
        ("data-validation.mutation.addRule", "sheet.mutation.insert-row"),
        ("data-validation.mutation.addRule", "sheet.mutation.insert-col"),
        ("data-validation.mutation.addRule", "sheet.mutation.set-range-values"),
        ("data-validation.mutation.removeRule", "sheet.mutation.insert-row"),
        ("data-validation.mutation.removeRule", "sheet.mutation.set-range-values"),
        ("data-validation.mutation.updateRule", "sheet.mutation.insert-row"),
        ("data-validation.mutation.updateRule", "sheet.mutation.set-range-values"),
    ];

    let registered_cf_sheets: Vec<(&str, &str)> = vec![
        ("sheet.mutation.add-conditional-rule", "sheet.mutation.insert-row"),
        ("sheet.mutation.add-conditional-rule", "sheet.mutation.insert-col"),
        ("sheet.mutation.add-conditional-rule", "sheet.mutation.set-range-values"),
        ("sheet.mutation.delete-conditional-rule", "sheet.mutation.insert-row"),
        ("sheet.mutation.delete-conditional-rule", "sheet.mutation.set-range-values"),
        ("sheet.mutation.set-conditional-rule", "sheet.mutation.insert-row"),
        ("sheet.mutation.set-conditional-rule", "sheet.mutation.set-range-values"),
    ];

    // Convert to sets
    let mut dv_registered = std::collections::HashSet::new();
    for (a, b) in &registered_dv_sheets {
        dv_registered.insert((*a, *b));
        dv_registered.insert((*b, *a));
    }

    let mut cf_registered = std::collections::HashSet::new();
    for (a, b) in &registered_cf_sheets {
        cf_registered.insert((*a, *b));
        cf_registered.insert((*b, *a));
    }

    // Find missing data-validation vs sheets pairs
    println!("Data Validation vs Sheets Core:");
    println!("----------------------------------------");
    let mut missing_dv = Vec::new();
    for dv in data_validation {
        for sheet in SHEETS_CORE_MUTATIONS {
            if !dv_registered.contains(&(*dv, *sheet)) {
                missing_dv.push((*dv, *sheet));
            }
        }
    }

    for (dv, sheet) in &missing_dv {
        let dv_short = dv.split('.').last().unwrap_or(dv);
        let sheet_short = sheet.split('.').last().unwrap_or(sheet);
        println!("  {} ↔ {}", dv_short, sheet_short);
    }
    println!("  Total missing: {}\n", missing_dv.len());

    // Find missing conditional-formatting vs sheets pairs
    println!("Conditional Formatting vs Sheets Core:");
    println!("----------------------------------------");
    let mut missing_cf = Vec::new();
    for cf in conditional_formatting {
        for sheet in SHEETS_CORE_MUTATIONS {
            // Skip if both are conditional formatting (already handled internally)
            if sheet.contains("conditional") {
                continue;
            }
            if !cf_registered.contains(&(*cf, *sheet)) {
                missing_cf.push((*cf, *sheet));
            }
        }
    }

    for (cf, sheet) in &missing_cf {
        let cf_short = cf.split('.').last().unwrap_or(cf);
        let sheet_short = sheet.split('.').last().unwrap_or(sheet);
        println!("  {} ↔ {}", cf_short, sheet_short);
    }
    println!("  Total missing: {}\n", missing_cf.len());

    // Data Validation vs Conditional Formatting
    println!("Data Validation vs Conditional Formatting:");
    println!("----------------------------------------");
    let mut missing_dv_cf = Vec::new();
    for dv in data_validation {
        for cf in conditional_formatting {
            // These modules typically don't interact, but we should register identity
            missing_dv_cf.push((*dv, *cf));
        }
    }

    for (dv, cf) in &missing_dv_cf {
        let dv_short = dv.split('.').last().unwrap_or(dv);
        let cf_short = cf.split('.').last().unwrap_or(cf);
        println!("  {} ↔ {}", dv_short, cf_short);
    }
    println!("  Total missing: {}\n", missing_dv_cf.len());

    println!("========================================");
    println!("Total cross-module missing: {}", missing_dv.len() + missing_cf.len() + missing_dv_cf.len());
    println!("========================================\n");
}
