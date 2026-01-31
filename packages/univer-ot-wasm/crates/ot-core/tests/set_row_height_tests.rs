//! Tests for SetWorksheetRowHeightMutation transforms
//!
//! SetRowHeight uses **conflict scope resolution**: operations on different rows don't conflict.

use serde_json::json;
use ot_core::{MutationInfo, TransformService};

// ============================================================================
// SetRowHeight vs SetRowHeight: Conflict Scope Resolution
// ============================================================================

/// Test that SetRowHeight operations on different rows don't conflict
#[test]
fn test_set_row_height_vs_set_row_height_different_rows() {
    let service = TransformService::new();

    // m1: Set row 0 height to 10
    let m1 = MutationInfo {
        id: "sheet.mutation.set-worksheet-row-height".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ranges": [{"startRow": 0, "endRow": 0, "startColumn": 0, "endColumn": 10}],
            "rowHeight": 10.0
        }),
    };

    // m2: Set row 2 height to 30
    let m2 = MutationInfo {
        id: "sheet.mutation.set-worksheet-row-height".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ranges": [{"startRow": 2, "endRow": 2, "startColumn": 0, "endColumn": 10}],
            "rowHeight": 30.0
        }),
    };

    let result = service.transform(&m1, &m2);

    // Both operations should execute (no conflict) because they affect different rows
    assert!(result.m1_prime.is_some(), "Expected m1 to be unchanged");
    assert!(result.m2_prime.is_some(), "Expected m2 to be unchanged");
    assert!(result.error.is_none());
}

/// Test that SetRowHeight operations on same row use LWW
#[test]
fn test_set_row_height_vs_set_row_height_same_row() {
    let service = TransformService::new();

    // m1: Set row 0 height to 10
    let m1 = MutationInfo {
        id: "sheet.mutation.set-worksheet-row-height".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ranges": [{"startRow": 0, "endRow": 0, "startColumn": 0, "endColumn": 10}],
            "rowHeight": 10.0
        }),
    };

    // m2: Set row 0 height to 30 (conflicting)
    let m2 = MutationInfo {
        id: "sheet.mutation.set-worksheet-row-height".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ranges": [{"startRow": 0, "endRow": 0, "startColumn": 0, "endColumn": 10}],
            "rowHeight": 30.0
        }),
    };

    let result = service.transform(&m1, &m2);

    // m1 should be removed (LWW) because they conflict on row 0
    assert!(result.m1_prime.is_none(), "Expected m1 to be removed for same row conflict");
    assert!(result.m2_prime.is_some(), "Expected m2 to be unchanged");
}

/// Test that SetRowHeight with HashMap doesn't conflict on different keys
#[test]
fn test_set_row_height_hashmap_different_keys() {
    let service = TransformService::new();

    // m1: Set row 0 height to 10 (using HashMap)
    let m1 = MutationInfo {
        id: "sheet.mutation.set-worksheet-row-height".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ranges": [{"startRow": 0, "endRow": 0, "startColumn": 0, "endColumn": 10}],
            "rowHeight": {"0": 10.0}
        }),
    };

    // m2: Set row 2 height to 30 (using HashMap)
    let m2 = MutationInfo {
        id: "sheet.mutation.set-worksheet-row-height".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ranges": [{"startRow": 2, "endRow": 2, "startColumn": 0, "endColumn": 10}],
            "rowHeight": {"2": 30.0}
        }),
    };

    let result = service.transform(&m1, &m2);

    // Both should execute because they affect different rows
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
}

// ============================================================================
// InsertRow vs SetRowHeight: Position Shifting
// ============================================================================

/// Test InsertRow shifts SetRowHeight ranges
#[test]
fn test_insert_row_shifts_set_row_height() {
    let service = TransformService::new();

    // m1: Insert 2 rows at row 1
    let m1 = MutationInfo {
        id: "sheet.mutation.insert-row".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {"startRow": 1, "endRow": 2, "startColumn": 0, "endColumn": 10},
            "unitId": "workbook1",
            "subUnitId": "sheet1"
        }),
    };

    // m2: Set row 2 height to 30
    let m2 = MutationInfo {
        id: "sheet.mutation.set-worksheet-row-height".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ranges": [{"startRow": 2, "endRow": 2, "startColumn": 0, "endColumn": 10}],
            "rowHeight": 30.0
        }),
    };

    let result = service.transform(&m1, &m2);

    if result.error.is_some() {
        eprintln!("Transform error: {:?}", result.error);
    }

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());

    // m2 ranges should be shifted to row 4 (2 + 2 inserted rows)
    let m2_prime = result.m2_prime.unwrap();
    eprintln!("Result m2: {:?}", serde_json::to_string_pretty(&m2_prime.params).unwrap());
    let ranges = m2_prime.params["ranges"].as_array().unwrap();
    assert_eq!(ranges[0]["startRow"], 4, "Expected startRow to be 4 (2 + 2 inserted rows), got {}", ranges[0]["startRow"]);
    assert_eq!(ranges[0]["endRow"], 4);
}

/// Test InsertRow shifts SetRowHeight HashMap keys
#[test]
fn test_insert_row_shifts_set_row_height_hashmap() {
    let service = TransformService::new();

    // m1: Insert 2 rows at row 1
    let m1 = MutationInfo {
        id: "sheet.mutation.insert-row".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {"startRow": 1, "endRow": 2, "startColumn": 0, "endColumn": 10},
            "unitId": "workbook1",
            "subUnitId": "sheet1"
        }),
    };

    // m2: Set row 2 and 3 heights using HashMap
    let m2 = MutationInfo {
        id: "sheet.mutation.set-worksheet-row-height".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ranges": [{"startRow": 2, "endRow": 3, "startColumn": 0, "endColumn": 10}],
            "rowHeight": {"2": 30.0, "3": 40.0}
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m2_prime.is_some());

    // m2 HashMap keys should be shifted to "4" and "5"
    let m2_prime = result.m2_prime.unwrap();
    let row_height = m2_prime.params["rowHeight"].as_object().unwrap();
    assert!(row_height.contains_key("4"), "Expected key '4' after shift");
    assert!(row_height.contains_key("5"), "Expected key '5' after shift");
    assert!(!row_height.contains_key("2"), "Key '2' should be shifted");
    assert!(!row_height.contains_key("3"), "Key '3' should be shifted");
    assert_eq!(row_height["4"], 30.0);
    assert_eq!(row_height["5"], 40.0);
}

// ============================================================================
// RemoveRow vs SetRowHeight: Position Shifting and Removal
// ============================================================================

/// Test RemoveRow shifts SetRowHeight ranges
#[test]
fn test_remove_row_shifts_set_row_height() {
    let service = TransformService::new();

    // m1: Remove rows 1-2
    let m1 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {"startRow": 1, "endRow": 2, "startColumn": 0, "endColumn": 10},
            "unitId": "workbook1",
            "subUnitId": "sheet1"
        }),
    };

    // m2: Set row 4 height to 30
    let m2 = MutationInfo {
        id: "sheet.mutation.set-worksheet-row-height".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ranges": [{"startRow": 4, "endRow": 4, "startColumn": 0, "endColumn": 10}],
            "rowHeight": 30.0
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m2_prime.is_some());

    // m2 ranges should be shifted to row 2 (4 - 2 removed rows)
    let m2_prime = result.m2_prime.unwrap();
    let ranges = m2_prime.params["ranges"].as_array().unwrap();
    assert_eq!(ranges[0]["startRow"], 2);
    assert_eq!(ranges[0]["endRow"], 2);
}

/// Test RemoveRow removes SetRowHeight when rows are deleted
#[test]
fn test_remove_row_removes_set_row_height_in_range() {
    let service = TransformService::new();

    // m1: Remove rows 1-3
    let m1 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {"startRow": 1, "endRow": 3, "startColumn": 0, "endColumn": 10},
            "unitId": "workbook1",
            "subUnitId": "sheet1"
        }),
    };

    // m2: Set row 2 height to 30 (will be deleted)
    let m2 = MutationInfo {
        id: "sheet.mutation.set-worksheet-row-height".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ranges": [{"startRow": 2, "endRow": 2, "startColumn": 0, "endColumn": 10}],
            "rowHeight": 30.0
        }),
    };

    let result = service.transform(&m1, &m2);

    // m2 should be removed because row 2 is deleted
    assert!(result.m2_prime.is_none(), "Expected m2 to be removed when target row is deleted");
}

/// Test RemoveRow removes HashMap keys in deleted range
#[test]
fn test_remove_row_removes_set_row_height_hashmap_keys() {
    let service = TransformService::new();

    // m1: Remove rows 2-3
    let m1 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {"startRow": 2, "endRow": 3, "startColumn": 0, "endColumn": 10},
            "unitId": "workbook1",
            "subUnitId": "sheet1"
        }),
    };

    // m2: Set heights for rows 1, 3, 5 using HashMap
    let m2 = MutationInfo {
        id: "sheet.mutation.set-worksheet-row-height".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ranges": [{"startRow": 1, "endRow": 5, "startColumn": 0, "endColumn": 10}],
            "rowHeight": {"1": 20.0, "3": 30.0, "5": 50.0}
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m2_prime.is_some());

    // m2 HashMap should have key "3" removed and "5" shifted to "3"
    let m2_prime = result.m2_prime.unwrap();
    let row_height = m2_prime.params["rowHeight"].as_object().unwrap();
    assert!(row_height.contains_key("1"), "Row 1 should remain"); // unaffected
    assert!(!row_height.contains_key("2"), "Row 2 should not exist (wasn't in original)");
    assert!(row_height.contains_key("3"), "Should have '3' (shifted from '5')"); // shifted from "5"
    assert!(!row_height.contains_key("5"), "Row 5 should be shifted");
    assert_eq!(row_height["1"], 20.0);
    assert_eq!(row_height["3"], 50.0); // shifted from row 5
}

/// Test SetRowHeight on different worksheets doesn't transform
#[test]
fn test_set_row_height_different_worksheets() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.set-worksheet-row-height".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ranges": [{"startRow": 0, "endRow": 0, "startColumn": 0, "endColumn": 10}],
            "rowHeight": 10.0
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.set-worksheet-row-height".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet2",
            "ranges": [{"startRow": 0, "endRow": 0, "startColumn": 0, "endColumn": 10}],
            "rowHeight": 30.0
        }),
    };

    let result = service.transform(&m1, &m2);

    // Both should be unchanged (different worksheets)
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
}
