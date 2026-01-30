//! Verify that all mutation pairs are properly registered

use ot_core::{MutationInfo, TransformService};
use serde_json::json;

#[test]
fn test_verify_new_mutations_registered() {
    let service = TransformService::new();

    println!("\n========================================");
    println!("Verifying New Mutations Registration");
    println!("========================================\n");

    // Test some of the "missing" pairs from the coverage test
    let test_pairs = vec![
        // insert-row vs new mutations
        ("sheet.mutation.insert-row", "sheet.mutation.set-tab-color"),
        ("sheet.mutation.insert-row", "sheet.mutation.toggle-gridlines"),
        ("sheet.mutation.insert-row", "sheet.mutation.set-worksheet-name"),

        // New mutations vs new mutations
        ("sheet.mutation.set-tab-color", "sheet.mutation.toggle-gridlines"),
        ("sheet.mutation.set-row-visible", "sheet.mutation.set-col-visible"),
        ("sheet.mutation.set-worksheet-row-height", "sheet.mutation.set-worksheet-col-width"),

        // Data validation vs new mutations
        ("data-validation.mutation.addRule", "sheet.mutation.set-tab-color"),
        ("data-validation.mutation.addRule", "sheet.mutation.toggle-gridlines"),

        // Conditional formatting vs new mutations
        ("sheet.mutation.add-conditional-rule", "sheet.mutation.set-tab-color"),
        ("sheet.mutation.add-conditional-rule", "sheet.mutation.toggle-gridlines"),
    ];

    let mut all_registered = true;

    for (m1_id, m2_id) in test_pairs {
        let m1 = MutationInfo {
            id: m1_id.to_string(),
            params: json!({"unitId": "w1", "subUnitId": "s1"}),
        };
        let m2 = MutationInfo {
            id: m2_id.to_string(),
            params: json!({"unitId": "w1", "subUnitId": "s1"}),
        };

        let result = service.transform(&m1, &m2);

        // If transforms work without panic, they are registered
        let registered = result.m1_prime.is_some() || result.m2_prime.is_some();

        let m1_short = m1_id.split('.').last().unwrap_or(m1_id);
        let m2_short = m2_id.split('.').last().unwrap_or(m2_id);

        if registered {
            println!("✓ {} ↔ {} - Registered", m1_short, m2_short);
        } else {
            println!("✗ {} ↔ {} - NOT registered", m1_short, m2_short);
            all_registered = false;
        }
    }

    println!("\n========================================");
    if all_registered {
        println!("✓ All tested pairs are registered");
    } else {
        println!("✗ Some pairs are missing registration");
    }
    println!("========================================\n");

    assert!(all_registered, "Some mutation pairs are not properly registered");
}

#[test]
fn test_verify_all_88_mutations_self_transform() {
    let service = TransformService::new();

    println!("\n========================================");
    println!("Verifying All 88 Mutations Self-Transform");
    println!("========================================\n");

    let all_mutations = vec![
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

    let mut all_work = true;
    let mut count = 0;

    for mutation_id in &all_mutations {
        let m = MutationInfo {
            id: mutation_id.to_string(),
            params: json!({"unitId": "w1", "subUnitId": "s1"}),
        };

        let result = service.transform(&m, &m);

        if result.m1_prime.is_some() || result.m2_prime.is_some() {
            count += 1;
        } else {
            let short = mutation_id.split('.').last().unwrap_or(mutation_id);
            println!("✗ {} - Self-transform failed", short);
            all_work = false;
        }
    }

    println!("Verified: {}/{} mutations have working self-transforms", count, all_mutations.len());
    println!("\n========================================");
    if all_work {
        println!("✓ All 88 mutations have working self-transforms");
    } else {
        println!("✗ Some mutations are missing self-transforms");
    }
    println!("========================================\n");

    assert!(all_work, "Some mutations don't have working self-transforms");
    assert_eq!(count, 88, "Expected 88 mutations with self-transforms");
}
