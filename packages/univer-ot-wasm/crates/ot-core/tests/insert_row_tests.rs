use ot_core::{MutationInfo, TransformService};
use serde_json::json;

#[test]
fn test_insert_row_vs_insert_row_same_position() {
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

    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());

    // m2's position should be shifted to 6
    let m2_prime = result.m2_prime.unwrap();
    let start_row = m2_prime.params["range"]["startRow"].as_u64().unwrap();
    assert_eq!(start_row, 6);
}

#[test]
fn test_insert_row_vs_insert_row_different_positions() {
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

    // m2 is after m1, so should shift to 11
    let m2_prime = result.m2_prime.unwrap();
    let start_row = m2_prime.params["range"]["startRow"].as_u64().unwrap();
    assert_eq!(start_row, 11);
}

#[test]
fn test_insert_row_vs_set_range_values() {
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
        id: "sheet.mutation.set-range-values".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "cellValue": {
                "5": { "0": { "v": "test" } }
            }
        }),
    };

    let result = service.transform(&m1, &m2);
    assert!(result.error.is_none());

    // Row 5 in cellValue should be shifted to row 6
    let m2_prime = result.m2_prime.unwrap();
    assert!(m2_prime.params["cellValue"]["6"].is_object());
    assert!(m2_prime.params["cellValue"]["5"].is_null());
}

#[test]
fn test_insert_row_vs_set_range_values_multiple_rows() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.insert-row".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 7, // Insert 3 rows
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
                "5": { "0": { "v": "test1" } },
                "6": { "0": { "v": "test2" } },
                "7": { "0": { "v": "test3" } }
            }
        }),
    };

    let result = service.transform(&m1, &m2);
    assert!(result.error.is_none());

    // All rows should shift by 3
    let m2_prime = result.m2_prime.unwrap();
    assert!(m2_prime.params["cellValue"]["8"].is_object());
    assert!(m2_prime.params["cellValue"]["9"].is_object());
    assert!(m2_prime.params["cellValue"]["10"].is_object());
}

#[test]
fn test_insert_row_different_worksheets() {
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
        id: "sheet.mutation.set-range-values".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet2", // Different sheet
            "cellValue": {
                "5": { "0": { "v": "test" } }
            }
        }),
    };

    let result = service.transform(&m1, &m2);
    assert!(result.error.is_none());

    // Different worksheets, no transform needed
    let m2_prime = result.m2_prime.unwrap();
    assert_eq!(m2_prime.params, m2.params);
}

// Parse error tests

#[test]
fn test_insert_row_vs_insert_row_parse_error_m1() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.insert-row".to_string(),
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
                "endRow": 5,
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
fn test_insert_row_vs_insert_row_parse_error_m2() {
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
fn test_insert_row_vs_insert_row_different_workbooks() {
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
        id: "sheet.mutation.insert-row".to_string(),
        params: json!({
            "unitId": "workbook2",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 5,
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

// Tests for double-check worksheet match (quick check returns None)
// These tests use aliased field names (snake_case) which serde can parse
// but the quick check function can't find, causing it to return None

#[test]
fn test_insert_row_vs_insert_row_double_check_different_worksheets() {
    let service = TransformService::new();

    // Use snake_case field names (aliases) - quick check returns None
    let m1 = MutationInfo {
        id: "sheet.mutation.insert-row".to_string(),
        params: json!({
            "unit_id": "workbook1",  // alias instead of unitId
            "sub_unit_id": "sheet1", // alias instead of subUnitId
            "range": {
                "start_row": 5,      // alias
                "start_column": 0,
                "end_row": 5,
                "end_column": 10
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.insert-row".to_string(),
        params: json!({
            "unit_id": "workbook1",  // Same workbook
            "sub_unit_id": "sheet2", // Different sheet - should trigger double-check
            "range": {
                "start_row": 5,
                "start_column": 0,
                "end_row": 5,
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
fn test_insert_row_vs_set_range_values_double_check_different_worksheets() {
    let service = TransformService::new();

    // Use snake_case field names (aliases) - quick check returns None
    let m1 = MutationInfo {
        id: "sheet.mutation.insert-row".to_string(),
        params: json!({
            "unit_id": "workbook1",
            "sub_unit_id": "sheet1",
            "range": {
                "start_row": 5,
                "start_column": 0,
                "end_row": 5,
                "end_column": 10
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.set-range-values".to_string(),
        params: json!({
            "unit_id": "workbook1",
            "sub_unit_id": "sheet2", // Different sheet
            "cellValue": {}
        }),
    };

    let result = service.transform(&m1, &m2);

    // Different worksheets - identity transform (via double-check path)
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

// Tests for insert-row vs set-range-values

#[test]
fn test_insert_row_vs_set_range_values_parse_error_m1() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.insert-row".to_string(),
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
fn test_insert_row_vs_set_range_values_parse_error_m2() {
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
fn test_insert_row_vs_set_range_values_different_workbooks() {
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
