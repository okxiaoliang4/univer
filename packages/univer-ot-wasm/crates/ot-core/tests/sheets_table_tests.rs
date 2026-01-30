//! Tests for sheets-table module transforms

use ot_core::{MutationInfo, TransformService};
use serde_json::json;

#[test]
fn test_add_table_vs_add_table_identity() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.add-table".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1"}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.add-table".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1"}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_some()); // Identity
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_set_table_vs_set_table_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-sheet-table".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "table": {}}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.set-sheet-table".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "table": {}}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_set_table_filter_vs_set_table_filter_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-table-filter".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "filter": {}}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.set-table-filter".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "filter": {}}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}
