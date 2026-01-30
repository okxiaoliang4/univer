//! Tests for other sheets module transforms (tab, reorder, empty, protection, style, theme, additional)

use ot_core::{MutationInfo, TransformService};
use serde_json::json;

// ============================================================================
// TAB TESTS
// ============================================================================

#[test]
fn test_set_tab_color_vs_set_tab_color_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-tab-color".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "color": "#FF0000"}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.set-tab-color".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "color": "#00FF00"}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}

// ============================================================================
// REORDER TESTS
// ============================================================================

#[test]
fn test_reorder_range_vs_reorder_range_identity() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.reorder-range".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "range": {}}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.reorder-range".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "range": {}}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_some()); // Identity
    assert!(result.m2_prime.is_some());
}

// ============================================================================
// EMPTY TESTS
// ============================================================================

#[test]
fn test_empty_vs_empty_identity() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.empty".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1"}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.empty".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1"}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_some()); // Identity
    assert!(result.m2_prime.is_some());
}

// ============================================================================
// WORKSHEET PROTECTION TESTS
// ============================================================================

#[test]
fn test_add_worksheet_protection_vs_add_worksheet_protection_identity() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.add-worksheet-protection".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1"}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.add-worksheet-protection".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1"}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_some()); // Identity
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_set_worksheet_protection_vs_set_worksheet_protection_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-worksheet-protection".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "protection": {}}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.set-worksheet-protection".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "protection": {}}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_set_permission_points_vs_set_permission_points_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-worksheet-permission-points".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "points": []}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.set-worksheet-permission-points".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "points": []}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}

// ============================================================================
// WORKSHEET STYLE TESTS
// ============================================================================

#[test]
fn test_set_default_style_vs_set_default_style_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-worksheet-default-style".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "style": {}}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.set-worksheet-default-style".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "style": {}}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_set_right_to_left_vs_set_right_to_left_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-worksheet-right-to-left".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "rtl": true}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.set-worksheet-right-to-left".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "rtl": false}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_worksheet_style_intra_module_default_vs_rtl() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-worksheet-default-style".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "style": {}}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.set-worksheet-right-to-left".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "rtl": true}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_some()); // Intra-module identity
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_worksheet_style_vs_filter_cross_module_identity() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-worksheet-default-style".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "style": {}}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.remove-filter".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1"}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_some()); // Cross-module identity
    assert!(result.m2_prime.is_some());
}

// ============================================================================
// RANGE THEME STYLE TESTS
// ============================================================================

#[test]
fn test_set_range_theme_style_vs_set_range_theme_style_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-worksheet-range-theme-style".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "style": {}}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.set-worksheet-range-theme-style".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "style": {}}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_remove_range_theme_style_vs_remove_range_theme_style_identity() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.remove-worksheet-range-theme-style".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1"}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.remove-worksheet-range-theme-style".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1"}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_some()); // Identity
    assert!(result.m2_prime.is_some());
}

// ============================================================================
// ADDITIONAL MUTATIONS TESTS
// ============================================================================

#[test]
fn test_remove_worksheet_merge_vs_remove_worksheet_merge_identity() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.remove-worksheet-merge".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1"}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.remove-worksheet-merge".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1"}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_some()); // Identity
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_add_range_protection_vs_add_range_protection_identity() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.add-range-protection".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1"}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.add-range-protection".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1"}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_some()); // Identity
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_set_worksheet_name_vs_set_worksheet_name_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-worksheet-name".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "name": "Sheet1"}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.set-worksheet-name".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "name": "Sheet2"}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_set_worksheet_order_vs_set_worksheet_order_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-worksheet-order".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "order": 0}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.set-worksheet-order".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "order": 1}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_set_worksheet_hidden_vs_set_worksheet_hidden_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-worksheet-hidden".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "hidden": true}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.set-worksheet-hidden".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "hidden": false}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}
