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

// Parse error tests - symmetric transforms fall back to identity
// Bidirectional transforms may return errors depending on which param fails

#[test]
fn test_insert_row_vs_insert_row_parse_error_m1_falls_back_to_identity() {
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

    // Symmetric transform: when parsing fails, falls back to identity
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none()); // No error - identity fallback is silent

    // m2 should be unchanged (identity)
    let m2_prime = result.m2_prime.unwrap();
    let start_row = m2_prime.params["range"]["startRow"].as_u64().unwrap();
    assert_eq!(start_row, 5); // No shift
}

#[test]
fn test_insert_row_vs_insert_row_parse_error_m2_falls_back_to_identity() {
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

    // Symmetric transform: when parsing fails, falls back to identity
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none()); // No error - identity fallback is silent

    // m1 should be unchanged (identity)
    let m1_prime = result.m1_prime.unwrap();
    let start_row = m1_prime.params["range"]["startRow"].as_u64().unwrap();
    assert_eq!(start_row, 5); // Unchanged
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

// Tests for insert-row vs set-range-values

#[test]
fn test_insert_row_vs_set_range_values_parse_error_m1_falls_back_to_identity() {
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

    // Bidirectional transform: when m1 parsing fails, falls back to identity
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none()); // No error - identity fallback
}

#[test]
fn test_insert_row_vs_set_range_values_parse_error_m2_falls_back_to_identity() {
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

    // Bidirectional transform: when m2 parsing fails, falls back to identity
    // This is more resilient - if we can't understand the params, we don't modify them
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
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
