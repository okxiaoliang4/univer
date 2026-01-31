use ot_core::{MutationInfo, TransformService};
use serde_json::json;

#[test]
fn test_insert_col_vs_insert_col_same_position() {
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

    let result = service.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());

    // m2 should be shifted right by 1 column
    let m2_prime = result.m2_prime.unwrap();
    let range = m2_prime.params["range"].as_object().unwrap();
    assert_eq!(range["startColumn"].as_u64().unwrap(), 6);
}

#[test]
fn test_insert_col_vs_insert_col_different_positions() {
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
        id: "sheet.mutation.insert-col".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "startColumn": 10,
                "endRow": 10,
                "endColumn": 10
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());

    // m2 should be shifted right by 1 column
    let m2_prime = result.m2_prime.unwrap();
    let range = m2_prime.params["range"].as_object().unwrap();
    assert_eq!(range["startColumn"].as_u64().unwrap(), 11);
}

#[test]
fn test_insert_col_vs_set_range_values() {
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
        id: "sheet.mutation.set-range-values".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "cellValue": {
                "0": {
                    "6": { "v": "value at column 6" }
                }
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none(), "Expected no error, got: {:?}", result.error);

    // Cell at column 6 should be shifted to column 7
    let m2_prime = result.m2_prime.unwrap();
    let cell_value = m2_prime.params["cellValue"].as_object().unwrap();
    let row_0 = cell_value["0"].as_object().unwrap();
    assert!(row_0.contains_key("7")); // Column shifted from 6 to 7
}

#[test]
fn test_insert_col_different_worksheets() {
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
        id: "sheet.mutation.insert-col".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet2",
            "range": {
                "startRow": 0,
                "startColumn": 5,
                "endRow": 10,
                "endColumn": 5
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    // Different worksheets - identity transform
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());

    let m2_prime = result.m2_prime.unwrap();
    let range = m2_prime.params["range"].as_object().unwrap();
    assert_eq!(range["startColumn"].as_u64().unwrap(), 5); // No shift
}

#[test]
fn test_insert_col_vs_insert_row_identity() {
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

    // Different dimensions - identity transform
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

// Parse error tests

#[test]
fn test_insert_col_vs_insert_col_parse_error_m1() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.insert-col".to_string(),
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
                "endColumn": 5
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
fn test_insert_col_vs_insert_col_parse_error_m2() {
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

// Tests for insert-col vs set-range-values

#[test]
fn test_insert_col_vs_set_range_values_parse_error_m1() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.insert-col".to_string(),
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
fn test_insert_col_vs_set_range_values_parse_error_m2() {
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
fn test_insert_col_vs_set_range_values_different_workbooks() {
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
