use ot_core::{MutationInfo, TransformService};
use serde_json::json;

// NOTE: remove_col vs remove_col uses identity transform (symmetric)
// The current architecture uses identity for symmetric transforms
// and relies on automatic fallback for unregistered pairs.

// ============================================================================
// remove-col vs remove-col (symmetric - identity transform)
// ============================================================================

#[test]
fn test_remove_col_vs_remove_col_identity() {
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

    // Identity transform - both mutations pass through unchanged
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());

    // Verify identity - m2 should be unchanged
    let m2_prime = result.m2_prime.unwrap();
    let range = m2_prime.params["range"].as_object().unwrap();
    assert_eq!(range["startColumn"].as_u64().unwrap(), 10); // Unchanged (identity)
}

#[test]
fn test_remove_col_vs_remove_col_different_worksheets() {
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

// Symmetric transforms use identity_transform() which doesn't parse params
// Parse errors are silently ignored, falling back to identity
#[test]
fn test_remove_col_vs_remove_col_parse_error_m1_falls_back_to_identity() {
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

    // Symmetric transform: identity_transform() doesn't parse, so no error
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none()); // No error - identity fallback is silent

    // m2 should be unchanged (identity)
    let m2_prime = result.m2_prime.unwrap();
    let range = m2_prime.params["range"].as_object().unwrap();
    assert_eq!(range["startColumn"].as_u64().unwrap(), 5); // Unchanged
}

#[test]
fn test_remove_col_vs_remove_col_parse_error_m2_falls_back_to_identity() {
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

    // Symmetric transform: identity_transform() doesn't parse, so no error
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none()); // No error - identity fallback is silent

    // m1 should be unchanged (identity)
    let m1_prime = result.m1_prime.unwrap();
    let range = m1_prime.params["range"].as_object().unwrap();
    assert_eq!(range["startColumn"].as_u64().unwrap(), 5); // Unchanged
}

// ============================================================================
// remove-col vs set-range-values (bidirectional - actual transform logic)
// ============================================================================

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
fn test_remove_col_vs_set_range_values_different_worksheets() {
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
            "subUnitId": "sheet2",
            "cellValue": {}
        }),
    };

    let result = service.transform(&m1, &m2);

    // Different worksheets - identity transform
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

// Bidirectional transforms: m1 parse errors fall back to identity silently
#[test]
fn test_remove_col_vs_set_range_values_parse_error_m1_falls_back_to_identity() {
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

    // Bidirectional transform: m1 parse failure falls back to identity
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none()); // No error - identity fallback
}

// Bidirectional transforms: m2 parse errors fall back to identity for resilience
#[test]
fn test_remove_col_vs_set_range_values_parse_error_m2_falls_back_to_identity() {
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

    // Bidirectional transform: m2 parse failure falls back to identity
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none()); // Falls back to identity, no error
}

// ============================================================================
// remove-col vs insert-col (falls back to identity - no explicit transform)
// ============================================================================

// NOTE: remove-col vs insert-col is not explicitly registered, so it falls back
// to identity transform automatically per the new architecture

#[test]
fn test_remove_col_vs_insert_col_identity() {
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

    // Falls back to identity - no explicit transform registered
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());

    // Identity - m2 should be unchanged
    let m2_prime = result.m2_prime.unwrap();
    let range = m2_prime.params["range"].as_object().unwrap();
    assert_eq!(range["startColumn"].as_u64().unwrap(), 10); // Unchanged (identity)
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
