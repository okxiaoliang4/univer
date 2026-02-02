//! Tests for SetRowDataMutation transforms with row operations
//!
//! SetRowDataMutation has `row_data: HashMap<String, Option<IRowData>>` where
//! keys are row indices. When InsertRow/RemoveRows happens, these keys need adjustment.

use ot_core::{MutationInfo, TransformService};
use serde_json::json;

// ============================================================================
// SetRowDataMutation vs InsertRowMutation
// ============================================================================

#[test]
fn test_insert_row_vs_set_row_data_shifts_keys() {
    let service = TransformService::new();

    // Insert 2 rows at row 5
    let m1 = MutationInfo {
        id: "sheet.mutation.insert-row".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 6,  // Insert 2 rows (5, 6)
                "endColumn": 10
            }
        }),
    };

    // SetRowData affects rows 3, 7, 10
    let m2 = MutationInfo {
        id: "sheet.mutation.set-row-data".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "rowData": {
                "3": { "h": 30 },   // Before insert, no shift
                "7": { "h": 70 },   // After insert, shift by 2 -> becomes "9"
                "10": { "h": 100 }  // After insert, shift by 2 -> becomes "12"
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none(), "Expected no error, got: {:?}", result.error);

    let m2_prime = result.m2_prime.unwrap();
    let row_data = m2_prime.params["rowData"].as_object().unwrap();

    // Row 3 (before insert at 5) should stay at "3"
    assert!(row_data.contains_key("3"), "Row 3 should remain unchanged");

    // Row 7 (>= insert at 5) should shift to "9" (7 + 2)
    assert!(row_data.contains_key("9"), "Row 7 should shift to 9");
    assert!(!row_data.contains_key("7"), "Row 7 should no longer exist");

    // Row 10 (>= insert at 5) should shift to "12" (10 + 2)
    assert!(row_data.contains_key("12"), "Row 10 should shift to 12");
    assert!(!row_data.contains_key("10"), "Row 10 should no longer exist");
}

#[test]
fn test_insert_row_vs_set_row_data_different_worksheets() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.insert-row".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 5,
                "endColumn": 10
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.set-row-data".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet2",  // Different sheet
            "rowData": {
                "7": { "h": 70 }
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    // Different worksheets - identity transform, no shift
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());

    let m2_prime = result.m2_prime.unwrap();
    let row_data = m2_prime.params["rowData"].as_object().unwrap();

    // No shift should happen
    assert!(row_data.contains_key("7"), "Row 7 should remain unchanged");
}

// ============================================================================
// SetRowDataMutation vs RemoveRowsMutation
// ============================================================================

#[test]
fn test_remove_rows_vs_set_row_data_shifts_keys() {
    let service = TransformService::new();

    // Remove rows 5-6 (2 rows)
    let m1 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 6,  // Remove 2 rows (5, 6)
                "endColumn": 10
            }
        }),
    };

    // SetRowData affects rows 3, 5, 6, 10
    let m2 = MutationInfo {
        id: "sheet.mutation.set-row-data".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "rowData": {
                "3": { "h": 30 },   // Before remove range, no shift
                "5": { "h": 50 },   // In remove range, should be removed
                "6": { "h": 60 },   // In remove range, should be removed
                "10": { "h": 100 }  // After remove range, shift by -2 -> becomes "8"
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none(), "Expected no error, got: {:?}", result.error);

    let m2_prime = result.m2_prime.unwrap();
    let row_data = m2_prime.params["rowData"].as_object().unwrap();

    // Row 3 (before remove at 5) should stay at "3"
    assert!(row_data.contains_key("3"), "Row 3 should remain unchanged");

    // Rows 5 and 6 (in remove range) should be dropped
    assert!(!row_data.contains_key("5"), "Row 5 should be removed");
    assert!(!row_data.contains_key("6"), "Row 6 should be removed");

    // Row 10 (> remove end 6) should shift to "8" (10 - 2)
    assert!(row_data.contains_key("8"), "Row 10 should shift to 8");
    assert!(!row_data.contains_key("10"), "Row 10 should no longer exist");
}

// ============================================================================
// SetRowDataMutation vs SetRowDataMutation (symmetric - LWW)
// ============================================================================

#[test]
fn test_set_row_data_vs_set_row_data_lww() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.set-row-data".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "rowData": {
                "5": { "h": 50 }
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.set-row-data".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "rowData": {
                "5": { "h": 100 }  // Same row, different height
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    // Both should execute (identity for now, LWW would be better)
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

// ============================================================================
// Parse error tests
// ============================================================================

#[test]
fn test_insert_row_vs_set_row_data_parse_error_m2_falls_back_to_identity() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.insert-row".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 5,
                "endColumn": 10
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.set-row-data".to_string(),
        params: json!({
            "invalid": "params"
        }),
    };

    let result = service.transform(&m1, &m2);

    // With parse error, falls back to identity for resilience
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none()); // Falls back to identity, no error
}
