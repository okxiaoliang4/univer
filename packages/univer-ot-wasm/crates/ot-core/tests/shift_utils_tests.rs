use ot_core::{ObjectMatrixPrimitiveType, Range, ICellData};
use ot_core::utils::shift::*;
use serde_json::json;
use std::collections::HashMap;

// Helper function to create cell data from JSON
fn create_cell_data(value: &str) -> ICellData {
    serde_json::from_value(json!({"v": value})).unwrap()
}

// Helper to create ObjectMatrixPrimitiveType from simple string values
fn create_cell_value(data: &[(&str, &[(&str, &str)])]) -> ObjectMatrixPrimitiveType {
    let mut result: ObjectMatrixPrimitiveType = HashMap::new();
    for (row_key, cols) in data {
        let mut col_map: HashMap<String, ICellData> = HashMap::new();
        for (col_key, value) in *cols {
            col_map.insert(col_key.to_string(), create_cell_data(value));
        }
        result.insert(row_key.to_string(), col_map);
    }
    result
}

// Tests for shift_row_keys_for_insert

#[test]
fn test_shift_row_keys_for_insert_basic() {
    let mut cell_value = create_cell_value(&[
        ("0", &[("0", "A1")]),
        ("5", &[("0", "A6")]),
        ("10", &[("0", "A11")]),
    ]);

    shift_row_keys_for_insert(&mut cell_value, 5, 3);

    assert!(cell_value.contains_key("0")); // Before insert point, unchanged
    assert!(cell_value.contains_key("8")); // Row 5 shifted to 8
    assert!(cell_value.contains_key("13")); // Row 10 shifted to 13
    assert!(!cell_value.contains_key("5")); // Old key removed
    assert!(!cell_value.contains_key("10")); // Old key removed
}

#[test]
fn test_shift_row_keys_for_insert_non_numeric_keys() {
    let mut cell_value = create_cell_value(&[
        ("0", &[("0", "A1")]),
        ("nonNumeric", &[("0", "value")]),
        ("5", &[("0", "A6")]),
    ]);

    shift_row_keys_for_insert(&mut cell_value, 3, 2);

    assert!(cell_value.contains_key("nonNumeric")); // Non-numeric keys unchanged
    assert!(cell_value.contains_key("0")); // Before insert point
    assert!(cell_value.contains_key("7")); // Row 5 shifted to 7
}

#[test]
fn test_shift_row_keys_for_insert_at_start() {
    let mut cell_value = create_cell_value(&[
        ("0", &[("0", "A1")]),
        ("1", &[("0", "A2")]),
    ]);

    shift_row_keys_for_insert(&mut cell_value, 0, 5);

    assert!(cell_value.contains_key("5")); // Row 0 shifted to 5
    assert!(cell_value.contains_key("6")); // Row 1 shifted to 6
}

// Tests for shift_row_keys_for_remove

#[test]
fn test_shift_row_keys_for_remove_basic() {
    let mut cell_value = create_cell_value(&[
        ("0", &[("0", "A1")]),
        ("5", &[("0", "A6")]),
        ("7", &[("0", "A8")]),
        ("10", &[("0", "A11")]),
    ]);

    shift_row_keys_for_remove(&mut cell_value, 5, 7);

    assert!(cell_value.contains_key("0")); // Before remove range
    assert!(!cell_value.contains_key("5")); // Removed
    assert!(cell_value.contains_key("7")); // Row 10 shifted to 7 (10 - 3)
    assert!(!cell_value.contains_key("10")); // Row 10 was shifted
}

#[test]
fn test_shift_row_keys_for_remove_non_numeric_keys() {
    let mut cell_value = create_cell_value(&[
        ("0", &[("0", "A1")]),
        ("nonNumeric", &[("0", "value")]),
        ("10", &[("0", "A11")]),
    ]);

    shift_row_keys_for_remove(&mut cell_value, 2, 5);

    assert!(cell_value.contains_key("nonNumeric")); // Non-numeric keys unchanged
    assert!(cell_value.contains_key("0")); // Before remove range
    assert!(cell_value.contains_key("6")); // Row 10 shifted to 6
}

#[test]
fn test_shift_row_keys_for_remove_at_boundary() {
    let mut cell_value = create_cell_value(&[
        ("4", &[("0", "A5")]),
        ("5", &[("0", "A6")]),
        ("6", &[("0", "A7")]),
    ]);

    shift_row_keys_for_remove(&mut cell_value, 5, 5);

    assert!(cell_value.contains_key("4")); // Before remove
    assert!(cell_value.contains_key("5")); // Row 6 shifted to 5 (6 - 1)
    assert!(!cell_value.contains_key("6")); // Row 6 was shifted
}

// Tests for shift_col_keys_for_insert

#[test]
fn test_shift_col_keys_for_insert_basic() {
    let mut cell_value = create_cell_value(&[
        ("0", &[("0", "A1"), ("5", "F1"), ("10", "K1")]),
    ]);

    shift_col_keys_for_insert(&mut cell_value, 5, 3);

    let row = cell_value.get("0").unwrap();
    assert!(row.contains_key("0")); // Before insert point
    assert!(row.contains_key("8")); // Col 5 shifted to 8
    assert!(row.contains_key("13")); // Col 10 shifted to 13
}

#[test]
fn test_shift_col_keys_for_insert_non_numeric_keys() {
    let mut cell_value = create_cell_value(&[
        ("0", &[("0", "A1"), ("nonNumeric", "value"), ("5", "F1")]),
    ]);

    shift_col_keys_for_insert(&mut cell_value, 3, 2);

    let row = cell_value.get("0").unwrap();
    assert!(row.contains_key("nonNumeric")); // Non-numeric keys unchanged
    assert!(row.contains_key("0")); // Before insert point
    assert!(row.contains_key("7")); // Col 5 shifted to 7
}

#[test]
fn test_shift_col_keys_for_insert_empty_row_after_shift() {
    let mut cell_value: ObjectMatrixPrimitiveType = HashMap::new();
    cell_value.insert("0".to_string(), HashMap::new());

    shift_col_keys_for_insert(&mut cell_value, 5, 3);

    assert!(!cell_value.contains_key("0")); // Empty rows removed
}

// Tests for shift_col_keys_for_remove

#[test]
fn test_shift_col_keys_for_remove_basic() {
    let mut cell_value = create_cell_value(&[
        ("0", &[("0", "A1"), ("5", "F1"), ("7", "H1"), ("10", "K1")]),
    ]);

    shift_col_keys_for_remove(&mut cell_value, 5, 7);

    let row = cell_value.get("0").unwrap();
    assert!(row.contains_key("0")); // Before remove range
    assert!(!row.contains_key("5")); // Removed
    assert!(row.contains_key("7")); // Col 10 shifted to 7 (10 - 3)
    assert!(!row.contains_key("10")); // Col 10 was shifted
}

#[test]
fn test_shift_col_keys_for_remove_non_numeric_keys() {
    let mut cell_value = create_cell_value(&[
        ("0", &[("0", "A1"), ("nonNumeric", "value"), ("10", "K1")]),
    ]);

    shift_col_keys_for_remove(&mut cell_value, 2, 5);

    let row = cell_value.get("0").unwrap();
    assert!(row.contains_key("nonNumeric")); // Non-numeric keys unchanged
    assert!(row.contains_key("0")); // Before remove range
    assert!(row.contains_key("6")); // Col 10 shifted to 6
}

#[test]
fn test_shift_col_keys_for_remove_empty_row_after_removal() {
    let mut cell_value = create_cell_value(&[
        ("0", &[("5", "F1")]),
    ]);

    shift_col_keys_for_remove(&mut cell_value, 5, 5);

    assert!(!cell_value.contains_key("0")); // Empty rows removed
}

// Tests for shift_range_rows_for_insert

#[test]
fn test_shift_range_rows_for_insert_before() {
    let mut range = Range {
        start_row: 10,
        start_column: 0,
        end_row: 15,
        end_column: 5,
        range_type: None,
    };

    shift_range_rows_for_insert(&mut range, 5, 3);

    assert_eq!(range.start_row, 13); // Shifted by 3
    assert_eq!(range.end_row, 18); // Shifted by 3
}

#[test]
fn test_shift_range_rows_for_insert_inside() {
    let mut range = Range {
        start_row: 5,
        start_column: 0,
        end_row: 15,
        end_column: 5,
        range_type: None,
    };

    shift_range_rows_for_insert(&mut range, 10, 3);

    assert_eq!(range.start_row, 5); // Start unchanged
    assert_eq!(range.end_row, 18); // End expanded by 3
}

#[test]
fn test_shift_range_rows_for_insert_after() {
    let mut range = Range {
        start_row: 5,
        start_column: 0,
        end_row: 8,
        end_column: 5,
        range_type: None,
    };

    shift_range_rows_for_insert(&mut range, 10, 3);

    assert_eq!(range.start_row, 5); // Unchanged
    assert_eq!(range.end_row, 8); // Unchanged
}

// Tests for shift_range_rows_for_remove

#[test]
fn test_shift_range_rows_for_remove_before() {
    let mut range = Range {
        start_row: 0,
        start_column: 0,
        end_row: 3,
        end_column: 5,
        range_type: None,
    };

    let result = shift_range_rows_for_remove(&mut range, 10, 15);

    assert!(result); // Range not removed
    assert_eq!(range.start_row, 0); // Unchanged
    assert_eq!(range.end_row, 3); // Unchanged
}

#[test]
fn test_shift_range_rows_for_remove_after() {
    let mut range = Range {
        start_row: 20,
        start_column: 0,
        end_row: 25,
        end_column: 5,
        range_type: None,
    };

    let result = shift_range_rows_for_remove(&mut range, 10, 15);

    assert!(result); // Range not removed
    assert_eq!(range.start_row, 14); // Shifted back by 6
    assert_eq!(range.end_row, 19); // Shifted back by 6
}

#[test]
fn test_shift_range_rows_for_remove_complete_overlap() {
    let mut range = Range {
        start_row: 10,
        start_column: 0,
        end_row: 15,
        end_column: 5,
        range_type: None,
    };

    let result = shift_range_rows_for_remove(&mut range, 5, 20);

    assert!(!result); // Range completely removed
}

#[test]
fn test_shift_range_rows_for_remove_partial_overlap_start() {
    let mut range = Range {
        start_row: 5,
        start_column: 0,
        end_row: 15,
        end_column: 5,
        range_type: None,
    };

    let result = shift_range_rows_for_remove(&mut range, 8, 12);

    assert!(result); // Range not completely removed
    assert_eq!(range.start_row, 5); // Start unchanged (remove starts after)
    assert_eq!(range.end_row, 10); // End adjusted: 5 + (11-5) - 1 = 10
}

#[test]
fn test_shift_range_rows_for_remove_partial_overlap_end() {
    let mut range = Range {
        start_row: 5,
        start_column: 0,
        end_row: 15,
        end_column: 5,
        range_type: None,
    };

    let result = shift_range_rows_for_remove(&mut range, 12, 20);

    assert!(result); // Range not completely removed
    assert_eq!(range.start_row, 5); // Unchanged start
    assert_eq!(range.end_row, 11); // Adjusted end
}

#[test]
fn test_shift_range_rows_for_remove_overlap_at_start() {
    // Test case where removal starts at range start with partial overlap
    // This covers line 140: range.start_row = range.start_row.saturating_sub(remove_count);
    let mut range = Range {
        start_row: 10,
        start_column: 0,
        end_row: 20,
        end_column: 5,
        range_type: None,
    };

    // Remove rows 10-14 (5 rows removed, starting exactly at range start)
    let result = shift_range_rows_for_remove(&mut range, 10, 14);

    assert!(result); // Range not completely removed (rows 15-20 remain)
    // Remaining: 11 - 5 = 6 rows
    // Since remove_start (10) <= range.start_row (10), start shifts down
    assert_eq!(range.start_row, 10 - 5); // start_row adjusted by remove_count
    assert_eq!(range.end_row, 5 + 6 - 1); // end = start + remaining - 1 = 10
}

// Tests for shift_range_cols_for_insert

#[test]
fn test_shift_range_cols_for_insert_before() {
    let mut range = Range {
        start_row: 0,
        start_column: 10,
        end_row: 5,
        end_column: 15,
        range_type: None,
    };

    shift_range_cols_for_insert(&mut range, 5, 3);

    assert_eq!(range.start_column, 13); // Shifted by 3
    assert_eq!(range.end_column, 18); // Shifted by 3
}

#[test]
fn test_shift_range_cols_for_insert_inside() {
    let mut range = Range {
        start_row: 0,
        start_column: 5,
        end_row: 5,
        end_column: 15,
        range_type: None,
    };

    shift_range_cols_for_insert(&mut range, 10, 3);

    assert_eq!(range.start_column, 5); // Start unchanged
    assert_eq!(range.end_column, 18); // End expanded by 3
}

#[test]
fn test_shift_range_cols_for_insert_after() {
    let mut range = Range {
        start_row: 0,
        start_column: 5,
        end_row: 5,
        end_column: 8,
        range_type: None,
    };

    shift_range_cols_for_insert(&mut range, 10, 3);

    assert_eq!(range.start_column, 5); // Unchanged
    assert_eq!(range.end_column, 8); // Unchanged
}

// Tests for shift_range_cols_for_remove

#[test]
fn test_shift_range_cols_for_remove_before() {
    let mut range = Range {
        start_row: 0,
        start_column: 0,
        end_row: 5,
        end_column: 3,
        range_type: None,
    };

    let result = shift_range_cols_for_remove(&mut range, 10, 15);

    assert!(result); // Range not removed
    assert_eq!(range.start_column, 0); // Unchanged
    assert_eq!(range.end_column, 3); // Unchanged
}

#[test]
fn test_shift_range_cols_for_remove_after() {
    let mut range = Range {
        start_row: 0,
        start_column: 20,
        end_row: 5,
        end_column: 25,
        range_type: None,
    };

    let result = shift_range_cols_for_remove(&mut range, 10, 15);

    assert!(result); // Range not removed
    assert_eq!(range.start_column, 14); // Shifted back by 6
    assert_eq!(range.end_column, 19); // Shifted back by 6
}

#[test]
fn test_shift_range_cols_for_remove_complete_overlap() {
    let mut range = Range {
        start_row: 0,
        start_column: 10,
        end_row: 5,
        end_column: 15,
        range_type: None,
    };

    let result = shift_range_cols_for_remove(&mut range, 5, 20);

    assert!(!result); // Range completely removed
}

#[test]
fn test_shift_range_cols_for_remove_partial_overlap_start() {
    let mut range = Range {
        start_row: 0,
        start_column: 5,
        end_row: 5,
        end_column: 15,
        range_type: None,
    };

    let result = shift_range_cols_for_remove(&mut range, 8, 12);

    assert!(result); // Range not completely removed
    assert_eq!(range.start_column, 5); // Start unchanged (remove starts after)
    assert_eq!(range.end_column, 10); // End adjusted: 5 + (11-5) - 1 = 10
}

#[test]
fn test_shift_range_cols_for_remove_partial_overlap_end() {
    let mut range = Range {
        start_row: 0,
        start_column: 5,
        end_row: 5,
        end_column: 15,
        range_type: None,
    };

    let result = shift_range_cols_for_remove(&mut range, 12, 20);

    assert!(result); // Range not completely removed
    assert_eq!(range.start_column, 5); // Unchanged start
    assert_eq!(range.end_column, 11); // Adjusted end
}

#[test]
fn test_shift_range_cols_for_remove_overlap_at_start() {
    // Test case where removal starts at range start with partial overlap
    // This covers line 177: range.start_column = range.start_column.saturating_sub(remove_count);
    let mut range = Range {
        start_row: 0,
        start_column: 10,
        end_row: 5,
        end_column: 20,
        range_type: None,
    };

    // Remove columns 10-14 (5 columns removed, starting exactly at range start)
    let result = shift_range_cols_for_remove(&mut range, 10, 14);

    assert!(result); // Range not completely removed (columns 15-20 remain)
    // Remaining: 11 - 5 = 6 columns
    // Since remove_start (10) <= range.start_column (10), start shifts left
    assert_eq!(range.start_column, 10 - 5); // start_column adjusted by remove_count
    assert_eq!(range.end_column, 5 + 6 - 1); // end = start + remaining - 1 = 10
}

// ============================================================================
// Params Utility Tests
// ============================================================================

use ot_core::utils::params::{get_cell_value_ref, get_cell_value_mut};

#[test]
fn test_get_cell_value_ref() {
    let params = json!({
        "unitId": "workbook1",
        "subUnitId": "sheet1",
        "cellValue": {
            "0": { "0": "A1" }
        }
    });

    let cell_value = get_cell_value_ref(&params);
    assert!(cell_value.is_some());
    assert!(cell_value.unwrap()["0"]["0"].as_str() == Some("A1"));
}

#[test]
fn test_get_cell_value_ref_missing() {
    let params = json!({
        "unitId": "workbook1",
        "subUnitId": "sheet1"
    });

    let cell_value = get_cell_value_ref(&params);
    assert!(cell_value.is_none());
}

#[test]
fn test_get_cell_value_mut() {
    let mut params = json!({
        "unitId": "workbook1",
        "subUnitId": "sheet1",
        "cellValue": {
            "0": { "0": "A1" }
        }
    });

    let cell_value = get_cell_value_mut(&mut params);
    assert!(cell_value.is_some());

    // Modify the value
    if let Some(cv) = cell_value {
        cv["0"]["0"] = json!("Modified");
    }

    // Verify modification
    assert_eq!(params["cellValue"]["0"]["0"], "Modified");
}

#[test]
fn test_get_cell_value_mut_missing() {
    let mut params = json!({
        "unitId": "workbook1",
        "subUnitId": "sheet1"
    });

    let cell_value = get_cell_value_mut(&mut params);
    assert!(cell_value.is_none());
}
