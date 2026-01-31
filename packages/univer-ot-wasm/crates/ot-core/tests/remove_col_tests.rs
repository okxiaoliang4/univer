use ot_core::{MutationInfo, TransformService};
use serde_json::json;

#[test]
fn test_remove_col_vs_remove_col_no_overlap() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-col".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "startColumn": 5,
                "endRow": 10,
                "endColumn": 7
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.remove-col".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "startColumn": 10,
                "endRow": 10,
                "endColumn": 12
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());

    // m2's columns should be shifted left by the count of removed columns in m1
    let m2_prime = result.m2_prime.unwrap();
    let range = m2_prime.params["range"].as_object().unwrap();
    assert_eq!(range["startColumn"].as_u64().unwrap(), 7); // 10 - 3 = 7
}

#[test]
fn test_remove_col_vs_remove_col_complete_overlap() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-col".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "startColumn": 5,
                "endRow": 10,
                "endColumn": 10
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.remove-col".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "startColumn": 6,
                "endRow": 10,
                "endColumn": 8
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_none()); // m2 is completely removed by m1
    assert!(result.error.is_none());
}

#[test]
fn test_remove_col_vs_remove_col_partial_overlap() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-col".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "startColumn": 5,
                "endRow": 10,
                "endColumn": 7
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.remove-col".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "startColumn": 6,
                "endRow": 10,
                "endColumn": 9
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());

    // Partial overlap - m2 should be adjusted
    let m2_prime = result.m2_prime.unwrap();
    let range = m2_prime.params["range"].as_object().unwrap();
    assert_eq!(range["startColumn"].as_u64().unwrap(), 5);
}

#[test]
fn test_remove_col_vs_set_range_values() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-col".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "startColumn": 5,
                "endRow": 10,
                "endColumn": 7
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.set-range-values".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "cellValue": {
                "0": {
                    "4": { "v": "before range" },
                    "6": { "v": "in range (removed)" },
                    "8": { "v": "after range" },
                    "10": { "v": "way after" }
                }
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());

    // Columns in removed range should be deleted, others shifted
    let m2_prime = result.m2_prime.unwrap();
    let cell_value = m2_prime.params["cellValue"].as_object().unwrap();
    let row_0 = cell_value["0"].as_object().unwrap();

    assert!(row_0.contains_key("4")); // Before range, no change
    assert!(!row_0.contains_key("6")); // In removed range, deleted
    assert!(row_0.contains_key("5")); // Column 8 shifted to 5 (8 - 3 = 5)
    assert!(row_0.contains_key("7")); // Column 10 shifted to 7 (10 - 3 = 7)
}

#[test]
fn test_remove_col_different_worksheets() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-col".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "startColumn": 5,
                "endRow": 10,
                "endColumn": 7
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.remove-col".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet2",
            "range": {
                "startRow": 0,
                "startColumn": 5,
                "endRow": 10,
                "endColumn": 7
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    // Different worksheets - identity transform
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

// Test parse error paths

#[test]
fn test_remove_col_vs_remove_col_parse_error_m1() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-col".to_string(),
        params: json!({
            "invalid": "params"
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.remove-col".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "startColumn": 5,
                "endRow": 10,
                "endColumn": 7
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
fn test_remove_col_vs_remove_col_parse_error_m2() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-col".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "startColumn": 5,
                "endRow": 10,
                "endColumn": 7
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.remove-col".to_string(),
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
fn test_remove_col_vs_remove_col_m2_before_m1() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-col".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "startColumn": 10,
                "endRow": 10,
                "endColumn": 15
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.remove-col".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "startColumn": 3,
                "endRow": 10,
                "endColumn": 5
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    // m2 is before m1, no changes needed
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());

    let m2_prime = result.m2_prime.unwrap();
    let range = m2_prime.params["range"].as_object().unwrap();
    assert_eq!(range["startColumn"].as_u64().unwrap(), 3); // No change
    assert_eq!(range["endColumn"].as_u64().unwrap(), 5); // No change
}

#[test]
fn test_remove_col_vs_remove_col_partial_overlap_left() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-col".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "startColumn": 5,
                "endRow": 10,
                "endColumn": 10
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.remove-col".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "startColumn": 3,
                "endRow": 10,
                "endColumn": 7
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());

    // m2 starts before m1, overlaps, should be truncated
    let m2_prime = result.m2_prime.unwrap();
    let range = m2_prime.params["range"].as_object().unwrap();
    assert_eq!(range["startColumn"].as_u64().unwrap(), 3);
    assert_eq!(range["endColumn"].as_u64().unwrap(), 4); // Truncated at m1_start - 1
}

#[test]
fn test_remove_col_vs_remove_col_partial_overlap_both_sides() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-col".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "startColumn": 5,
                "endRow": 10,
                "endColumn": 7
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.remove-col".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "startColumn": 3,
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
    let range = m2_prime.params["range"].as_object().unwrap();
    assert_eq!(range["endColumn"].as_u64().unwrap(), 7); // Shifted by m1_count (3)
}

// Tests for remove-col vs set-range-values

#[test]
fn test_remove_col_vs_set_range_values_parse_error_m1() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-col".to_string(),
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
fn test_remove_col_vs_set_range_values_parse_error_m2() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-col".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "startColumn": 5,
                "endRow": 10,
                "endColumn": 7
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
fn test_remove_col_vs_set_range_values_different_workbooks() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-col".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "startColumn": 5,
                "endRow": 10,
                "endColumn": 7
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

// Tests for remove-col vs insert-col

#[test]
fn test_remove_col_vs_insert_col() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-col".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "startColumn": 5,
                "endRow": 10,
                "endColumn": 7
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.insert-col".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "startColumn": 10,
                "endRow": 10,
                "endColumn": 12
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());

    // Insert after remove, should be shifted left
    let m2_prime = result.m2_prime.unwrap();
    let range = m2_prime.params["range"].as_object().unwrap();
    assert_eq!(range["startColumn"].as_u64().unwrap(), 7); // 10 - 3 = 7
}

#[test]
fn test_remove_col_vs_insert_col_inside_remove_range() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-col".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "startColumn": 5,
                "endRow": 10,
                "endColumn": 10
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.insert-col".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "startColumn": 7,
                "endRow": 10,
                "endColumn": 9
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());

    // Insert inside remove range, should be moved to remove_start
    let m2_prime = result.m2_prime.unwrap();
    let range = m2_prime.params["range"].as_object().unwrap();
    assert_eq!(range["startColumn"].as_u64().unwrap(), 5);
}

#[test]
fn test_remove_col_vs_insert_col_parse_error_m1() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-col".to_string(),
        params: json!({
            "invalid": "params"
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.insert-col".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "startColumn": 5,
                "endRow": 10,
                "endColumn": 7
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
fn test_remove_col_vs_insert_col_parse_error_m2() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-col".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "startColumn": 5,
                "endRow": 10,
                "endColumn": 7
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.insert-col".to_string(),
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
fn test_remove_col_vs_insert_col_different_workbooks() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.remove-col".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "startColumn": 5,
                "endRow": 10,
                "endColumn": 7
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.insert-col".to_string(),
        params: json!({
            "unitId": "workbook2",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "startColumn": 5,
                "endRow": 10,
                "endColumn": 7
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    // Different workbooks - identity transform
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

