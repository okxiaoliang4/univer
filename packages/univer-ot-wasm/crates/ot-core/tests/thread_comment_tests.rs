//! Tests for thread-comment module transforms

use ot_core::{MutationInfo, TransformService};
use serde_json::json;

#[test]
fn test_add_comment_vs_add_comment_identity() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "thread-comment.mutation.add-comment".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1"}),
    };
    let m2 = MutationInfo {
        id: "thread-comment.mutation.add-comment".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1"}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_some()); // Identity
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_update_comment_vs_update_comment_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "thread-comment.mutation.update-comment".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "text": "comment1"}),
    };
    let m2 = MutationInfo {
        id: "thread-comment.mutation.update-comment".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "text": "comment2"}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_update_comment_ref_vs_update_comment_ref_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "thread-comment.mutation.update-comment-ref".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "ref": "A1"}),
    };
    let m2 = MutationInfo {
        id: "thread-comment.mutation.update-comment-ref".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "ref": "B2"}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_resolve_comment_vs_resolve_comment_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "thread-comment.mutation.resolve-comment".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "resolved": true}),
    };
    let m2 = MutationInfo {
        id: "thread-comment.mutation.resolve-comment".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "resolved": false}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}
