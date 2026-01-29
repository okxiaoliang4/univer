use ot_core::{MutationInfo, TransformService};
use serde_json::json;

#[test]
fn test_remove_rows_vs_remove_rows_no_overlap() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 7,
                "endColumn": 10
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 10,
                "startColumn": 0,
                "endRow": 12,
                "endColumn": 10
            }
        }),
    };

    let result = service.transform(&m1, &m2);
    assert!(result.error.is_none());

    // m2 is after m1, should shift down by 3
    let m2_prime = result.m2_prime.unwrap();
    assert_eq!(m2_prime.params["range"]["startRow"], 7);
    assert_eq!(m2_prime.params["range"]["endRow"], 9);
}

#[test]
fn test_remove_rows_vs_remove_rows_complete_overlap() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 10,
                "endColumn": 10
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 6,
                "startColumn": 0,
                "endRow": 8,
                "endColumn": 10
            }
        }),
    };

    let result = service.transform(&m1, &m2);
    assert!(result.error.is_none());

    // m2 is completely contained in m1, should become noop
    assert!(result.m2_prime.is_none());
}

#[test]
fn test_remove_rows_vs_insert_row() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 7,
                "endColumn": 10
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.insert-row".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 10,
                "startColumn": 0,
                "endRow": 10,
                "endColumn": 10
            }
        }),
    };

    let result = service.transform(&m1, &m2);
    assert!(result.error.is_none());

    // Insert at 10 should shift to 7 (10 - 3)
    let m2_prime = result.m2_prime.unwrap();
    assert_eq!(m2_prime.params["range"]["startRow"], 7);
}

#[test]
fn test_remove_rows_vs_set_range_values() {
    let service = TransformService::new();

    // Remove rows 5-7 (3 rows)
    let m1 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 7,
                "endColumn": 10
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.set-range-values".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "cellValue": {
                "4": { "0": { "v": "before" } },      // Before removed range, stays at 4
                "6": { "0": { "v": "in_removed" } },  // In removed range, should be deleted
                "8": { "0": { "v": "after" } },       // After removed range, shifts to 5
                "10": { "0": { "v": "after2" } }      // After removed range, shifts to 7
            }
        }),
    };

    let result = service.transform(&m1, &m2);
    assert!(result.error.is_none());

    let m2_prime = result.m2_prime.unwrap();
    
    // Row 4 should remain at 4
    assert!(m2_prime.params["cellValue"].get("4").is_some());
    assert_eq!(m2_prime.params["cellValue"]["4"]["0"]["v"], "before");
    
    // Row 6 was in the removed range, should be deleted
    let has_row_6 = m2_prime.params["cellValue"].get("6")
        .map(|v| !v.is_null() && v.as_object().map_or(false, |o| !o.is_empty()))
        .unwrap_or(false);
    assert!(!has_row_6, "Row 6 should be removed");
    
    // Row 8 should shift to row 5
    assert!(m2_prime.params["cellValue"].get("5").is_some());
    assert_eq!(m2_prime.params["cellValue"]["5"]["0"]["v"], "after");
    
    // Row 10 should shift to row 7
    assert!(m2_prime.params["cellValue"].get("7").is_some());
    assert_eq!(m2_prime.params["cellValue"]["7"]["0"]["v"], "after2");
}

#[test]
fn test_remove_rows_vs_remove_rows_partial_overlap() {
    let service = TransformService::new();

    // Remove rows 5-7
    let m1 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 7,
                "endColumn": 10
            }
        }),
    };

    // Remove rows 6-9 (overlaps with m1 on 6-7)
    let m2 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 6,
                "startColumn": 0,
                "endRow": 9,
                "endColumn": 10
            }
        }),
    };

    let result = service.transform(&m1, &m2);
    assert!(result.error.is_none());

    // m2 originally removes 6-9
    // After m1 removes 5-7:
    // - Rows 6-7 are already removed by m1
    // - m2 should now remove what was originally 8-9, which becomes 5-6 after m1's removal
    let m2_prime = result.m2_prime.unwrap();
    assert_eq!(m2_prime.params["range"]["startRow"], 5);
    assert_eq!(m2_prime.params["range"]["endRow"], 6);
}

// Parse error tests

#[test]
fn test_remove_rows_vs_remove_rows_parse_error_m1() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "invalid": "params"
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 7,
                "endColumn": 10
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_some());
    assert!(result.error.unwrap().contains("Failed to parse m1 params"));
}

#[test]
fn test_remove_rows_vs_remove_rows_parse_error_m2() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 7,
                "endColumn": 10
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "invalid": "params"
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_some());
    assert!(result.error.unwrap().contains("Failed to parse m2 params"));
}

#[test]
fn test_remove_rows_vs_remove_rows_m2_before_m1() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 10,
                "startColumn": 0,
                "endRow": 15,
                "endColumn": 10
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 3,
                "startColumn": 0,
                "endRow": 5,
                "endColumn": 10
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    // m2 is before m1, no changes needed
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());

    let m2_prime = result.m2_prime.unwrap();
    assert_eq!(m2_prime.params["range"]["startRow"], 3); // No change
    assert_eq!(m2_prime.params["range"]["endRow"], 5); // No change
}

#[test]
fn test_remove_rows_vs_remove_rows_partial_overlap_left() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 10,
                "endColumn": 10
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 3,
                "startColumn": 0,
                "endRow": 7,
                "endColumn": 10
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());

    // m2 starts before m1, overlaps, should be truncated
    let m2_prime = result.m2_prime.unwrap();
    assert_eq!(m2_prime.params["range"]["startRow"], 3);
    assert_eq!(m2_prime.params["range"]["endRow"], 4); // Truncated at m1_start - 1
}

#[test]
fn test_remove_rows_vs_remove_rows_partial_overlap_both_sides() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 7,
                "endColumn": 10
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 3,
                "startColumn": 0,
                "endRow": 10,
                "endColumn": 10
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());

    // m2 surrounds m1, should be adjusted on end
    let m2_prime = result.m2_prime.unwrap();
    assert_eq!(m2_prime.params["range"]["endRow"], 7); // Shifted by m1_count (3)
}

#[test]
fn test_remove_rows_different_worksheets() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 7,
                "endColumn": 10
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet2",
            "range": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 7,
                "endColumn": 10
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    // Different worksheets - identity transform
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

// Tests for remove-rows vs set-range-values

#[test]
fn test_remove_rows_vs_set_range_values_parse_error_m1() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "invalid": "params"
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.set-range-values".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "cellValue": {}
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_some());
    assert!(result.error.unwrap().contains("Failed to parse m1 params"));
}

#[test]
fn test_remove_rows_vs_set_range_values_parse_error_m2() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 7,
                "endColumn": 10
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.set-range-values".to_string(),
        params: json!({
            "invalid": "params"
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_some());
    assert!(result.error.unwrap().contains("Failed to parse m2 params"));
}

#[test]
fn test_remove_rows_vs_set_range_values_different_workbooks() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 7,
                "endColumn": 10
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.set-range-values".to_string(),
        params: json!({
            "unitId": "workbook2",
            "subUnitId": "sheet1",
            "cellValue": {}
        }),
    };

    let result = service.transform(&m1, &m2);

    // Different workbooks - identity transform
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

// Tests for remove-rows vs insert-row

#[test]
fn test_remove_rows_vs_insert_row_inside_remove_range() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 10,
                "endColumn": 10
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.insert-row".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 7,
                "startColumn": 0,
                "endRow": 9,
                "endColumn": 10
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());

    // Insert inside remove range, should be moved to remove_start
    let m2_prime = result.m2_prime.unwrap();
    assert_eq!(m2_prime.params["range"]["startRow"], 5);
}

#[test]
fn test_remove_rows_vs_insert_row_parse_error_m1() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "invalid": "params"
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.insert-row".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 7,
                "endColumn": 10
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_some());
    assert!(result.error.unwrap().contains("Failed to parse m1 params"));
}

#[test]
fn test_remove_rows_vs_insert_row_parse_error_m2() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 7,
                "endColumn": 10
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.insert-row".to_string(),
        params: json!({
            "invalid": "params"
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_some());
    assert!(result.error.unwrap().contains("Failed to parse m2 params"));
}

#[test]
fn test_remove_rows_vs_insert_row_different_workbooks() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 7,
                "endColumn": 10
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.insert-row".to_string(),
        params: json!({
            "unitId": "workbook2",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 7,
                "endColumn": 10
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    // Different workbooks - identity transform
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

// Double-check worksheet tests (snake_case aliases to bypass quick check)

#[test]
fn test_remove_rows_vs_remove_rows_double_check_different_worksheets() {
    let service = TransformService::new();

    // Use snake_case field names (aliases) - quick check returns None
    let m1 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "unit_id": "workbook1",
            "sub_unit_id": "sheet1",
            "range": {
                "start_row": 5,
                "start_column": 0,
                "end_row": 7,
                "end_column": 10
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "unit_id": "workbook1",
            "sub_unit_id": "sheet2",  // Different sheet
            "range": {
                "start_row": 5,
                "start_column": 0,
                "end_row": 7,
                "end_column": 10
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    // Different worksheets - identity transform (via double-check path)
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

#[test]
fn test_remove_rows_vs_set_range_values_double_check_different_worksheets() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "unit_id": "workbook1",
            "sub_unit_id": "sheet1",
            "range": {
                "start_row": 5,
                "start_column": 0,
                "end_row": 7,
                "end_column": 10
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.set-range-values".to_string(),
        params: json!({
            "unit_id": "workbook1",
            "sub_unit_id": "sheet2",  // Different sheet
            "cellValue": {}
        }),
    };

    let result = service.transform(&m1, &m2);

    // Different worksheets - identity transform (via double-check path)
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

#[test]
fn test_remove_rows_vs_insert_row_double_check_different_worksheets() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "unit_id": "workbook1",
            "sub_unit_id": "sheet1",
            "range": {
                "start_row": 5,
                "start_column": 0,
                "end_row": 7,
                "end_column": 10
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.insert-row".to_string(),
        params: json!({
            "unit_id": "workbook1",
            "sub_unit_id": "sheet2",  // Different sheet
            "range": {
                "start_row": 5,
                "start_column": 0,
                "end_row": 7,
                "end_column": 10
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    // Different worksheets - identity transform (via double-check path)
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}
