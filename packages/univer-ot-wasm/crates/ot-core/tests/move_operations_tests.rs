use ot_core::{MutationInfo, TransformService};
use serde_json::json;

// Move Rows Tests

#[test]
fn test_move_rows_vs_move_rows() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.move-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "sourceRange": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 7,
                "endColumn": 10
            },
            "targetRange": {
                "startRow": 10,
                "startColumn": 0,
                "endRow": 12,
                "endColumn": 10
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.move-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "sourceRange": {
                "startRow": 3,
                "startColumn": 0,
                "endRow": 4,
                "endColumn": 10
            },
            "targetRange": {
                "startRow": 8,
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
}

#[test]
fn test_move_rows_vs_insert_row() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.move-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "sourceRange": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 7,
                "endColumn": 10
            },
            "targetRange": {
                "startRow": 10,
                "startColumn": 0,
                "endRow": 12,
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
                "startRow": 3,
                "startColumn": 0,
                "endRow": 3,
                "endColumn": 10
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

#[test]
fn test_move_rows_vs_remove_rows() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.move-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "sourceRange": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 7,
                "endColumn": 10
            },
            "targetRange": {
                "startRow": 10,
                "startColumn": 0,
                "endRow": 12,
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
                "endRow": 4,
                "endColumn": 10
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

#[test]
fn test_move_rows_vs_set_range_values() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.move-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "sourceRange": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 7,
                "endColumn": 10
            },
            "targetRange": {
                "startRow": 10,
                "startColumn": 0,
                "endRow": 12,
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
                "6": {
                    "5": "value in source range"
                }
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

// Parse error tests

#[test]
fn test_move_rows_vs_move_rows_parse_error_m1() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.move-rows".to_string(),
        params: json!({
            "invalid": "params"
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.move-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "sourceRange": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 7,
                "endColumn": 10
            },
            "targetRange": {
                "startRow": 10,
                "startColumn": 0,
                "endRow": 12,
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
fn test_move_rows_vs_move_rows_parse_error_m2() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.move-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "sourceRange": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 7,
                "endColumn": 10
            },
            "targetRange": {
                "startRow": 10,
                "startColumn": 0,
                "endRow": 12,
                "endColumn": 10
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.move-rows".to_string(),
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

// Workbook/Sheet boundary tests

#[test]
fn test_move_rows_vs_move_rows_different_workbooks() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.move-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "sourceRange": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 7,
                "endColumn": 10
            },
            "targetRange": {
                "startRow": 10,
                "startColumn": 0,
                "endRow": 12,
                "endColumn": 10
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.move-rows".to_string(),
        params: json!({
            "unitId": "workbook2",
            "subUnitId": "sheet1",
            "sourceRange": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 7,
                "endColumn": 10
            },
            "targetRange": {
                "startRow": 10,
                "startColumn": 0,
                "endRow": 12,
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

#[test]
fn test_move_rows_vs_move_rows_different_worksheets() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.move-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "sourceRange": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 7,
                "endColumn": 10
            },
            "targetRange": {
                "startRow": 10,
                "startColumn": 0,
                "endRow": 12,
                "endColumn": 10
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.move-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet2",
            "sourceRange": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 7,
                "endColumn": 10
            },
            "targetRange": {
                "startRow": 10,
                "startColumn": 0,
                "endRow": 12,
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

// Move Columns Tests

#[test]
fn test_move_columns_vs_move_columns() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.move-columns".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "sourceRange": {
                "startRow": 0,
                "startColumn": 5,
                "endRow": 10,
                "endColumn": 7
            },
            "targetRange": {
                "startRow": 10,
                "startColumn": 0,
                "endRow": 12,
                "endColumn": 10
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.move-columns".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "sourceRange": {
                "startRow": 0,
                "startColumn": 3,
                "endRow": 10,
                "endColumn": 4
            },
            "targetRange": {
                "startRow": 8,
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
}

#[test]
fn test_move_columns_vs_insert_col() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.move-columns".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "sourceRange": {
                "startRow": 0,
                "startColumn": 5,
                "endRow": 10,
                "endColumn": 7
            },
            "targetRange": {
                "startRow": 10,
                "startColumn": 0,
                "endRow": 12,
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
                "startColumn": 3,
                "endRow": 10,
                "endColumn": 3
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

// Move Range Tests

#[test]
fn test_move_range_vs_move_range() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.move-range".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "fromRange": {
                "startRow": 5,
                "startColumn": 5,
                "endRow": 7,
                "endColumn": 7
            },
            "toRange": {
                "startRow": 10,
                "startColumn": 10,
                "endRow": 12,
                "endColumn": 12
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.move-range".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "fromRange": {
                "startRow": 3,
                "startColumn": 3,
                "endRow": 4,
                "endColumn": 4
            },
            "toRange": {
                "startRow": 8,
                "startColumn": 8,
                "endRow": 9,
                "endColumn": 9
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

#[test]
fn test_move_range_vs_insert_row() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.move-range".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "fromRange": {
                "startRow": 5,
                "startColumn": 5,
                "endRow": 7,
                "endColumn": 7
            },
            "toRange": {
                "startRow": 10,
                "startColumn": 10,
                "endRow": 12,
                "endColumn": 12
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.insert-row".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 3,
                "startColumn": 0,
                "endRow": 3,
                "endColumn": 10
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

#[test]
fn test_move_range_vs_set_range_values() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.move-range".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "fromRange": {
                "startRow": 5,
                "startColumn": 5,
                "endRow": 7,
                "endColumn": 7
            },
            "toRange": {
                "startRow": 10,
                "startColumn": 10,
                "endRow": 12,
                "endColumn": 12
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.set-range-values".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "cellValue": {
                "6": {
                    "6": "value in move range"
                }
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

// Double-check worksheet tests (snake_case aliases to bypass quick check)

#[test]
fn test_move_rows_vs_move_rows_double_check_different_worksheets() {
    let service = TransformService::new();

    // Use snake_case field names (aliases) - quick check returns None
    let m1 = MutationInfo {
        id: "sheet.mutation.move-rows".to_string(),
        params: json!({
            "unit_id": "workbook1",
            "sub_unit_id": "sheet1",
            "sourceRange": {
                "start_row": 5,
                "start_column": 0,
                "end_row": 7,
                "end_column": 10
            },
            "targetRange": {
                "start_row": 10,
                "start_column": 0,
                "end_row": 12,
                "end_column": 10
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.move-rows".to_string(),
        params: json!({
            "unit_id": "workbook1",
            "sub_unit_id": "sheet2",  // Different sheet
            "sourceRange": {
                "start_row": 5,
                "start_column": 0,
                "end_row": 7,
                "end_column": 10
            },
            "targetRange": {
                "start_row": 10,
                "start_column": 0,
                "end_row": 12,
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
