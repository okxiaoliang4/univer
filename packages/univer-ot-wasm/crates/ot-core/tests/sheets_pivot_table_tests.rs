//! Tests for sheets-pivot-table module transforms

use ot_core::{MutationInfo, TransformService};
use serde_json::json;

#[test]
fn test_add_pivot_table_vs_add_pivot_table_identity() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.add-pivot-table".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1"}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.add-pivot-table".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1"}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_some()); // Identity
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_set_pivot_table_source_range_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-pivot-table-source-range".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "range": {}}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.set-pivot-table-source-range".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "range": {}}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_set_pivot_table_target_cell_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-pivot-table-target-cell".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "cell": "A1"}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.set-pivot-table-target-cell".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "cell": "B2"}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_set_pivot_table_fields_config_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-pivot-table-fields-config".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "config": {}}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.set-pivot-table-fields-config".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "config": {}}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_set_pivot_table_calculated_data_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.set-pivot-table-calculated-data".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "data": {}}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.set-pivot-table-calculated-data".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "data": {}}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}
