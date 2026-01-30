//! Tests for sheets visibility module transforms

use ot_core::{MutationInfo, TransformService};
use serde_json::json;

#[test]
fn test_set_row_visible_vs_set_row_visible_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-row-visible".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "row": 0, "visible": true}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.set-row-visible".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "row": 0, "visible": false}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_set_row_hidden_vs_set_row_hidden_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-row-hidden".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "row": 0}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.set-row-hidden".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "row": 0}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_set_col_visible_vs_set_col_visible_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-col-visible".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "col": 0}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.set-col-visible".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "col": 0}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_set_col_hidden_vs_set_col_hidden_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-col-hidden".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "col": 0}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.set-col-hidden".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "col": 0}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_visibility_intra_module_row_visible_vs_col_visible() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-row-visible".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "row": 0}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.set-col-visible".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "col": 0}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_some()); // Intra-module identity
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_visibility_vs_filter_cross_module_identity() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-row-visible".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "row": 0}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.remove-filter".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1"}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_some()); // Cross-module identity
    assert!(result.m2_prime.is_some());
}
