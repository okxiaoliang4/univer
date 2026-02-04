//! Tests for sheets-filter module transforms

use ot_core::{MutationInfo, TransformService};
use serde_json::json;

#[test]
fn test_set_sheets_filter_range_vs_set_sheets_filter_range_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-sheets-filter-range".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "range": {"startRow": 0, "endRow": 10}}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.set-sheets-filter-range".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "range": {"startRow": 0, "endRow": 20}}),
    };
    let result = service.transform(&m1, &m2);
    // LWW: m2 wins, m1 is removed
    assert!(result.m1_prime.is_none());
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_set_sheets_filter_criteria_vs_set_sheets_filter_criteria_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-sheets-filter-criteria".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "col": 0, "criteria": {}}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.set-sheets-filter-criteria".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "col": 0, "criteria": {}}),
    };
    let result = service.transform(&m1, &m2);
    // LWW: m2 wins, m1 is removed
    assert!(result.m1_prime.is_none());
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_remove_sheets_filter_vs_remove_sheets_filter_identity() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.remove-sheets-filter".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1"}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.remove-sheets-filter".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1"}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_some()); // Identity
    assert!(result.m2_prime.is_some());
}
