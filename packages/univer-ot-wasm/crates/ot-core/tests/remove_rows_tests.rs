use ot_core::{MutationInfo, TransformService};
use serde_json::json;

// NOTE: remove-rows vs remove-rows uses identity transform (symmetric)
// The current architecture uses identity for symmetric transforms
// and relies on automatic fallback for unregistered pairs.

// ============================================================================
// remove-rows vs remove-rows (symmetric - identity transform)
// ============================================================================

#[test]
fn test_remove_rows_vs_remove_rows_identity() {
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

    // Identity transform - both mutations pass through unchanged
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());

    // Verify identity - m2 should be unchanged
    let m2_prime = result.m2_prime.unwrap();
    assert_eq!(m2_prime.params["range"]["startRow"], 10); // Unchanged (identity)
    assert_eq!(m2_prime.params["range"]["endRow"], 12); // Unchanged (identity)
}

#[test]
fn test_remove_rows_vs_remove_rows_different_worksheets() {
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

// Symmetric transforms use identity_transform() which doesn't parse params
// Parse errors are silently ignored, falling back to identity
#[test]
fn test_remove_rows_vs_remove_rows_parse_error_m1_falls_back_to_identity() {
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

    // Symmetric transform: identity_transform() doesn't parse, so no error
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none()); // No error - identity fallback is silent

    // m2 should be unchanged (identity)
    let m2_prime = result.m2_prime.unwrap();
    assert_eq!(m2_prime.params["range"]["startRow"], 5); // Unchanged
}

#[test]
fn test_remove_rows_vs_remove_rows_parse_error_m2_falls_back_to_identity() {
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

    // Symmetric transform: identity_transform() doesn't parse, so no error
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none()); // No error - identity fallback is silent

    // m1 should be unchanged (identity)
    let m1_prime = result.m1_prime.unwrap();
    assert_eq!(m1_prime.params["range"]["startRow"], 5); // Unchanged
}

// ============================================================================
// remove-rows vs set-range-values (bidirectional - actual transform logic)
// ============================================================================

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
fn test_remove_rows_vs_set_range_values_different_worksheets() {
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
fn test_remove_rows_vs_set_range_values_parse_error_m1_falls_back_to_identity() {
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

    // Bidirectional transform: m1 parse failure falls back to identity
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none()); // No error - identity fallback
}

// Bidirectional transforms: m2 parse errors fall back to identity for resilience
#[test]
fn test_remove_rows_vs_set_range_values_parse_error_m2_falls_back_to_identity() {
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

    // Bidirectional transform: m2 parse failure falls back to identity
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none()); // Falls back to identity, no error
}

// ============================================================================
// remove-rows vs insert-row (falls back to identity - no explicit transform)
// ============================================================================

// NOTE: remove-rows vs insert-row is not explicitly registered, so it falls back
// to identity transform automatically per the new architecture

#[test]
fn test_remove_rows_vs_insert_row_identity() {
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

    // Falls back to identity - no explicit transform registered
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());

    // Identity - m2 should be unchanged
    let m2_prime = result.m2_prime.unwrap();
    assert_eq!(m2_prime.params["range"]["startRow"], 10); // Unchanged (identity)
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
