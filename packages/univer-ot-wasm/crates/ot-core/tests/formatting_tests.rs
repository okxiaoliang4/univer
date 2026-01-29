use ot_core::{MutationInfo, TransformService};
use serde_json::json;

// Merge Tests

#[test]
fn test_merge_vs_merge_lww() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.add-worksheet-merge".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "mergeData": {
                "range": {
                    "startRow": 0,
                    "startColumn": 0,
                    "endRow": 2,
                    "endColumn": 2
                }
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.add-worksheet-merge".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "mergeData": {
                "range": {
                    "startRow": 0,
                    "startColumn": 0,
                    "endRow": 3,
                    "endColumn": 3
                }
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    // Identity: both succeed
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

// Protection Tests

#[test]
fn test_protection_vs_protection_lww() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.set-range-protection".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "protection": {
                "range": {
                    "startRow": 0,
                    "startColumn": 0,
                    "endRow": 10,
                    "endColumn": 10
                },
                "locked": true
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.set-range-protection".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "protection": {
                "range": {
                    "startRow": 0,
                    "startColumn": 0,
                    "endRow": 10,
                    "endColumn": 10
                },
                "locked": false
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    // Identity: both succeed
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

// Theme Tests

#[test]
fn test_theme_vs_theme_lww() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.set-range-theme".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "theme": {
                "name": "blue",
                "colors": []
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.set-range-theme".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "theme": {
                "name": "red",
                "colors": []
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    // Identity: both succeed
    assert!(result.m1_prime.is_none());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

// Frozen Tests

#[test]
fn test_frozen_vs_frozen_lww() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.set-frozen".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "startRow": 0,
            "startColumn": 0,
            "ySplit": 1,
            "xSplit": 1
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.set-frozen".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "startRow": 0,
            "startColumn": 0,
            "ySplit": 2,
            "xSplit": 2
        }),
    };

    let result = service.transform(&m1, &m2);

    // LWW: m2 wins
    assert!(result.m1_prime.is_none());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

#[test]
fn test_frozen_vs_insert_row_identity() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.set-frozen".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "startRow": 0,
            "startColumn": 0,
            "ySplit": 1,
            "xSplit": 1
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

    // Identity transform
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

// Numfmt Tests

#[test]
fn test_numfmt_vs_numfmt_identity() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.set.numfmt".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "startColumn": 0,
                "endRow": 10,
                "endColumn": 10
            },
            "format": "0.00"
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.set.numfmt".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 5,
                "startColumn": 5,
                "endRow": 15,
                "endColumn": 15
            },
            "format": "0.000"
        }),
    };

    let result = service.transform(&m1, &m2);

    // Identity transform (both succeed)
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

// Row/Col Data Tests

#[test]
fn test_row_col_data_vs_row_col_data_lww() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.set-row-data".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "rowData": {
                "0": { "h": 50 }
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.set-row-data".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "rowData": {
                "0": { "h": 100 }
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    // Identity: both succeed
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

// Worksheet Tests

#[test]
fn test_worksheet_vs_worksheet_lww() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.insert-sheet".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "newsheet1",
            "index": 0,
            "sheet": { "name": "New Sheet 1" }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.insert-sheet".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "newsheet2",
            "index": 0,
            "sheet": { "name": "New Sheet 2" }
        }),
    };

    let result = service.transform(&m1, &m2);

    // Identity: both succeed
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

// Workbook Tests

#[test]
fn test_workbook_vs_workbook_lww() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.set-workbook-name".to_string(),
        params: json!({
            "unitId": "workbook1",
            "name": "My Workbook v1"
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.set-workbook-name".to_string(),
        params: json!({
            "unitId": "workbook1",
            "name": "My Workbook v2"
        }),
    };

    let result = service.transform(&m1, &m2);

    // Identity: both succeed
    assert!(result.m1_prime.is_none());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}
