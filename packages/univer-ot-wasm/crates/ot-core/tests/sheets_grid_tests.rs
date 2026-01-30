//! Tests for sheets grid module transforms

use ot_core::{MutationInfo, TransformService};
use serde_json::json;

#[test]
fn test_toggle_gridlines_vs_toggle_gridlines_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.toggle-gridlines".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "show": true}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.toggle-gridlines".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "show": false}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_set_gridlines_color_vs_set_gridlines_color_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-gridlines-color".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "color": "#000000"}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.set-gridlines-color".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "color": "#FFFFFF"}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_grid_intra_module_gridlines_vs_color() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.toggle-gridlines".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "show": true}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.set-gridlines-color".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "color": "#000"}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_some()); // Intra-module identity
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_grid_vs_filter_cross_module_identity() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.toggle-gridlines".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "show": true}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.remove-filter".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1"}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_some()); // Cross-module identity
    assert!(result.m2_prime.is_some());
}
