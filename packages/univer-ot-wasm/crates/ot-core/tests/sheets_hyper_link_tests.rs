//! Tests for sheets-hyper-link module transforms

use ot_core::{MutationInfo, TransformService};
use serde_json::json;

#[test]
fn test_add_hyper_link_vs_add_hyper_link_identity() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheets.mutation.add-hyper-link".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1"}),
    };
    let m2 = MutationInfo {
        id: "sheets.mutation.add-hyper-link".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1"}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_some()); // Identity
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_update_hyper_link_vs_update_hyper_link_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheets.mutation.update-hyper-link".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "link": "url1"}),
    };
    let m2 = MutationInfo {
        id: "sheets.mutation.update-hyper-link".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "link": "url2"}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_update_hyper_link_ref_vs_update_hyper_link_ref_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheets.mutation.update-hyper-link-ref".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "ref": "A1"}),
    };
    let m2 = MutationInfo {
        id: "sheets.mutation.update-hyper-link-ref".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "ref": "B2"}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_update_rich_hyper_link_vs_update_rich_hyper_link_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheets.mutation.update-rich-hyper-link".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "data": {}}),
    };
    let m2 = MutationInfo {
        id: "sheets.mutation.update-rich-hyper-link".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "data": {}}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}
