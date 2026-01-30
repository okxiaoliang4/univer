//! Tests for sheets-note module transforms

use ot_core::{MutationInfo, TransformService};
use serde_json::json;

#[test]
fn test_update_note_vs_update_note_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.update-note".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "note": "note1"}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.update-note".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "note": "note2"}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_remove_note_vs_remove_note_identity() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.remove-note".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1"}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.remove-note".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1"}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_some()); // Identity
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_toggle_note_popup_vs_toggle_note_popup_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.toggle-note-popup".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "show": true}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.toggle-note-popup".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "show": false}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_update_note_position_vs_update_note_position_lww() {
    let service = TransformService::new();
    let m1 = MutationInfo {
        id: "sheet.mutation.update-note-position".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "x": 10, "y": 10}),
    };
    let m2 = MutationInfo {
        id: "sheet.mutation.update-note-position".to_string(),
        params: json!({"unitId": "w1", "subUnitId": "s1", "x": 20, "y": 20}),
    };
    let result = service.transform(&m1, &m2);
    assert!(result.m1_prime.is_none()); // LWW
    assert!(result.m2_prime.is_some());
}
