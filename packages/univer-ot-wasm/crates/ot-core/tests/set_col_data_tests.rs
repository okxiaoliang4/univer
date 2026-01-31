//! Tests for SetColDataMutation transforms with column operations
//!
//! SetColDataMutation has `column_data: HashMap<String, Option<IColumnData>>` where
//! keys are column indices. When InsertCol/RemoveCol happens, these keys need adjustment.

use ot_core::{MutationInfo, TransformService};
use serde_json::json;

// ============================================================================
// SetColDataMutation vs InsertColMutation
// ============================================================================

#[test]
fn test_insert_col_vs_set_col_data_shifts_keys() {
    let service = TransformService::new();

    // Insert 2 columns at column 5
    let m1 = MutationInfo {
        id: "sheet.mutation.insert-col".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "startColumn": 5,
                "endRow": 10,
                "endColumn": 6  // Insert 2 columns (5, 6)
            }
        }),
    };

    // SetColData affects columns 3, 7, 10
    let m2 = MutationInfo {
        id: "sheet.mutation.set-col-data".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "columnData": {
                "3": { "w": 100 },   // Before insert, no shift
                "7": { "w": 150 },   // After insert, shift by 2 -> becomes "9"
                "10": { "w": 200 }   // After insert, shift by 2 -> becomes "12"
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none(), "Expected no error, got: {:?}", result.error);

    let m2_prime = result.m2_prime.unwrap();
    let col_data = m2_prime.params["columnData"].as_object().unwrap();

    // Column 3 (before insert at 5) should stay at "3"
    assert!(col_data.contains_key("3"), "Column 3 should remain unchanged");

    // Column 7 (>= insert at 5) should shift to "9" (7 + 2)
    assert!(col_data.contains_key("9"), "Column 7 should shift to 9");
    assert!(!col_data.contains_key("7"), "Column 7 should no longer exist");

    // Column 10 (>= insert at 5) should shift to "12" (10 + 2)
    assert!(col_data.contains_key("12"), "Column 10 should shift to 12");
    assert!(!col_data.contains_key("10"), "Column 10 should no longer exist");
}

#[test]
fn test_insert_col_vs_set_col_data_different_worksheets() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.insert-col".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "startColumn": 5,
                "endRow": 10,
                "endColumn": 5
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.set-col-data".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet2",  // Different sheet
            "columnData": {
                "7": { "w": 150 }
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    // Different worksheets - identity transform, no shift
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());

    let m2_prime = result.m2_prime.unwrap();
    let col_data = m2_prime.params["columnData"].as_object().unwrap();

    // No shift should happen
    assert!(col_data.contains_key("7"), "Column 7 should remain unchanged");
}

// ============================================================================
// SetColDataMutation vs RemoveColMutation
// ============================================================================

#[test]
fn test_remove_col_vs_set_col_data_shifts_keys() {
    let service = TransformService::new();

    // Remove columns 5-6 (2 columns)
    let m1 = MutationInfo {
        id: "sheet.mutation.remove-col".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "startColumn": 5,
                "endRow": 10,
                "endColumn": 6  // Remove 2 columns (5, 6)
            }
        }),
    };

    // SetColData affects columns 3, 5, 6, 10
    let m2 = MutationInfo {
        id: "sheet.mutation.set-col-data".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "columnData": {
                "3": { "w": 100 },   // Before remove range, no shift
                "5": { "w": 150 },   // In remove range, should be removed
                "6": { "w": 175 },   // In remove range, should be removed
                "10": { "w": 200 }   // After remove range, shift by -2 -> becomes "8"
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none(), "Expected no error, got: {:?}", result.error);

    let m2_prime = result.m2_prime.unwrap();
    let col_data = m2_prime.params["columnData"].as_object().unwrap();

    // Column 3 (before remove at 5) should stay at "3"
    assert!(col_data.contains_key("3"), "Column 3 should remain unchanged");

    // Columns 5 and 6 (in remove range) should be dropped
    assert!(!col_data.contains_key("5"), "Column 5 should be removed");
    assert!(!col_data.contains_key("6"), "Column 6 should be removed");

    // Column 10 (> remove end 6) should shift to "8" (10 - 2)
    assert!(col_data.contains_key("8"), "Column 10 should shift to 8");
    assert!(!col_data.contains_key("10"), "Column 10 should no longer exist");
}

// ============================================================================
// SetColDataMutation vs SetColDataMutation (symmetric - identity)
// ============================================================================

#[test]
fn test_set_col_data_vs_set_col_data_identity() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.set-col-data".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "columnData": {
                "5": { "w": 100 }
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.set-col-data".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "columnData": {
                "5": { "w": 200 }  // Same column, different width
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    // Both should execute (identity)
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}
