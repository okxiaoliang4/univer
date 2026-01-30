//! Tests for sheets dimensions module transforms

use ot_core::{MutationInfo, TransformService};
use serde_json::json;

#[test]
fn test_set_row_height_vs_set_row_height_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-worksheet-row-height".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "row": 0, "height": 20}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.set-worksheet-row-height".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "row": 0, "height": 40}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_set_row_is_auto_height_vs_set_row_is_auto_height_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-worksheet-row-is-auto-height".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "row": 0, "auto": true}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.set-worksheet-row-is-auto-height".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "row": 0, "auto": false}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_set_row_auto_height_vs_set_row_auto_height_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-worksheet-row-auto-height".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "row": 0}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.set-worksheet-row-auto-height".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "row": 0}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_set_col_width_vs_set_col_width_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-worksheet-col-width".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "col": 0, "width": 100}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.set-worksheet-col-width".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "col": 0, "width": 200}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_set_row_count_vs_set_row_count_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-worksheet-row-count".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "count": 100}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.set-worksheet-row-count".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "count": 200}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_set_col_count_vs_set_col_count_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-worksheet-column-count".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "count": 26}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.set-worksheet-column-count".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "count": 52}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_dimensions_intra_module_row_height_vs_col_width() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-worksheet-row-height".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "row": 0, "height": 20}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.set-worksheet-col-width".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "col": 0, "width": 100}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_some()); // Intra-module identity
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_dimensions_vs_filter_cross_module_identity() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-worksheet-row-height".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "row": 0, "height": 20}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.remove-filter".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1"}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_some()); // Cross-module identity
    assert!(result.m2_prime.is_some());
}
