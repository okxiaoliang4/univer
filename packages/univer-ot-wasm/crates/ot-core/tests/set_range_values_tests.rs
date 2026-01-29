use ot_core::{MutationInfo, TransformService};
use serde_json::json;

#[test]
fn test_set_range_values_no_conflict() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.set-range-values".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "cellValue": {
                "0": { "0": { "v": "A1" } }
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.set-range-values".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "cellValue": {
                "0": { "1": { "v": "B1" } }
            }
        }),
    };

    let result = service.transform(&m1, &m2);
    assert!(result.error.is_none());

    // No conflict, both should keep their values
    let m1_prime = result.m1_prime.unwrap();
    let m2_prime = result.m2_prime.unwrap();

    assert!(m1_prime.params["cellValue"]["0"]["0"].is_object());
    assert!(m2_prime.params["cellValue"]["0"]["1"].is_object());
}

#[test]
fn test_set_range_values_lww_conflict_single_cell() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.set-range-values".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "cellValue": {
                "0": { "0": { "v": "m1_value" } }
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.set-range-values".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "cellValue": {
                "0": { "0": { "v": "m2_value" } }
            }
        }),
    };

    let result = service.transform(&m1, &m2);
    assert!(result.error.is_none());

    // LWW: m2 wins, so m1_prime should not have the cell
    let m1_prime = result.m1_prime.unwrap();
    let m2_prime = result.m2_prime.unwrap();

    // m1_prime should have no cell value (conflict resolved in favor of m2)
    if let Some(cell_value) = m1_prime.params.get("cellValue") {
        if !cell_value.is_null() {
            // If cellValue exists, check that row 0 col 0 is not present
            if let Some(row) = cell_value.get("0") {
                assert!(row.get("0").is_none() || row.get("0").unwrap().is_null());
            }
        }
    }

    // m2_prime should keep its value
    assert_eq!(m2_prime.params["cellValue"]["0"]["0"]["v"], "m2_value");
}

#[test]
fn test_set_range_values_lww_partial_conflict() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.set-range-values".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "cellValue": {
                "0": { 
                    "0": { "v": "A1_m1" },
                    "1": { "v": "B1_m1" }
                }
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
                    "1": { "v": "B1_m2" },  // Conflict with m1
                    "2": { "v": "C1_m2" }   // No conflict
                }
            }
        }),
    };

    let result = service.transform(&m1, &m2);
    assert!(result.error.is_none());

    let m1_prime = result.m1_prime.unwrap();
    let m2_prime = result.m2_prime.unwrap();

    // m1_prime should keep A1 (no conflict) but lose B1 (conflict)
    assert_eq!(m1_prime.params["cellValue"]["0"]["0"]["v"], "A1_m1");
    // B1 should be removed from m1_prime or not present
    if let Some(b1) = m1_prime.params["cellValue"]["0"].get("1") {
        assert!(b1.is_null());
    }

    // m2_prime should keep both its cells
    assert_eq!(m2_prime.params["cellValue"]["0"]["1"]["v"], "B1_m2");
    assert_eq!(m2_prime.params["cellValue"]["0"]["2"]["v"], "C1_m2");
}

#[test]
fn test_set_range_values_lww_multiple_rows() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.set-range-values".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "cellValue": {
                "0": { "0": { "v": "A1_m1" } },
                "1": { "0": { "v": "A2_m1" } }
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.set-range-values".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "cellValue": {
                "1": { "0": { "v": "A2_m2" } }, // Conflict
                "2": { "0": { "v": "A3_m2" } }  // No conflict
            }
        }),
    };

    let result = service.transform(&m1, &m2);
    assert!(result.error.is_none());

    let m1_prime = result.m1_prime.unwrap();
    let m2_prime = result.m2_prime.unwrap();

    // m1_prime should keep row 0 but lose row 1
    assert_eq!(m1_prime.params["cellValue"]["0"]["0"]["v"], "A1_m1");
    // Row 1 should not be in m1_prime or be null
    if let Some(row1) = m1_prime.params["cellValue"].get("1") {
        if !row1.is_null() {
            if let Some(col0) = row1.get("0") {
                assert!(col0.is_null());
            }
        }
    }

    // m2_prime should keep both rows
    assert_eq!(m2_prime.params["cellValue"]["1"]["0"]["v"], "A2_m2");
    assert_eq!(m2_prime.params["cellValue"]["2"]["0"]["v"], "A3_m2");
}

#[test]
fn test_set_range_values_different_worksheets() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.set-range-values".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "cellValue": {
                "0": { "0": { "v": "test" } }
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.set-range-values".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet2",
            "cellValue": {
                "0": { "0": { "v": "test" } }
            }
        }),
    };

    let result = service.transform(&m1, &m2);
    assert!(result.error.is_none());

    // Different worksheets, no transform
    let m1_prime = result.m1_prime.unwrap();
    let m2_prime = result.m2_prime.unwrap();

    assert_eq!(m1_prime.params, m1.params);
    assert_eq!(m2_prime.params, m2.params);
}

#[test]
fn test_set_range_values_different_workbooks() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.set-range-values".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "cellValue": {
                "0": { "0": { "v": "test" } }
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.set-range-values".to_string(),
        params: json!({
            "unitId": "workbook2",
            "subUnitId": "sheet1",
            "cellValue": {
                "0": { "0": { "v": "test" } }
            }
        }),
    };

    let result = service.transform(&m1, &m2);
    assert!(result.error.is_none());

    // Different workbooks - identity transform
    let m1_prime = result.m1_prime.unwrap();
    let m2_prime = result.m2_prime.unwrap();

    assert_eq!(m1_prime.params, m1.params);
    assert_eq!(m2_prime.params, m2.params);
}

// Parse error tests

#[test]
fn test_set_range_values_parse_error_m1() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.set-range-values".to_string(),
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
fn test_set_range_values_parse_error_m2() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.set-range-values".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "cellValue": {}
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
fn test_set_range_values_complete_conflict_all_cells_removed() {
    let service = TransformService::new();

    // m1 sets cell A1
    let m1 = MutationInfo {
        id: "sheet.mutation.set-range-values".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "cellValue": {
                "0": { "0": { "v": "m1_A1" } }
            }
        }),
    };

    // m2 also sets cell A1 (complete conflict)
    let m2 = MutationInfo {
        id: "sheet.mutation.set-range-values".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "cellValue": {
                "0": { "0": { "v": "m2_A1" } }
            }
        }),
    };

    let result = service.transform(&m1, &m2);
    assert!(result.error.is_none());

    let m1_prime = result.m1_prime.unwrap();
    let m2_prime = result.m2_prime.unwrap();

    // m1_prime should have cellValue set to null or empty (all cells removed)
    if let Some(cell_value) = m1_prime.params.get("cellValue") {
        if cell_value.is_null() {
            // Perfect - cellValue is null
            assert!(cell_value.is_null());
        } else if let Some(obj) = cell_value.as_object() {
            // Or it's an empty object or row "0" doesn't exist
            assert!(obj.is_empty() || !obj.contains_key("0"));
        }
    }

    // m2_prime should keep its value (LWW winner)
    assert_eq!(m2_prime.params["cellValue"]["0"]["0"]["v"], "m2_A1");
}

// Double-check worksheet tests (snake_case aliases to bypass quick check)

#[test]
fn test_set_range_values_double_check_different_worksheets() {
    let service = TransformService::new();

    // Use snake_case field names (aliases) - quick check returns None
    let m1 = MutationInfo {
        id: "sheet.mutation.set-range-values".to_string(),
        params: json!({
            "unit_id": "workbook1",
            "sub_unit_id": "sheet1",
            "cellValue": {
                "0": { "0": { "v": "test" } }
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.set-range-values".to_string(),
        params: json!({
            "unit_id": "workbook1",
            "sub_unit_id": "sheet2",  // Different sheet
            "cellValue": {
                "0": { "0": { "v": "test" } }
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    // Different worksheets - identity transform (via double-check path)
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}
