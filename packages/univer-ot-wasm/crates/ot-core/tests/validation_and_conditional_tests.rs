use ot_core::{MutationInfo, TransformService};
use serde_json::json;

// Data Validation Tests

#[test]
fn test_add_rule_vs_add_rule_identity() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "data-validation.mutation.addRule".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ruleId": "rule1",
            "ranges": [{
                "startRow": 0,
                "startColumn": 0,
                "endRow": 10,
                "endColumn": 0
            }]
        }),
    };

    let m2 = MutationInfo {
        id: "data-validation.mutation.addRule".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ruleId": "rule2",
            "ranges": [{
                "startRow": 0,
                "startColumn": 1,
                "endRow": 10,
                "endColumn": 1
            }]
        }),
    };

    let result = service.transform(&m1, &m2);

    // Identity - both succeed
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

#[test]
fn test_add_rule_vs_remove_rule_identity() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "data-validation.mutation.addRule".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ruleId": "rule1",
            "ranges": []
        }),
    };

    let m2 = MutationInfo {
        id: "data-validation.mutation.removeRule".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ruleId": "rule2"
        }),
    };

    let result = service.transform(&m1, &m2);

    // Identity - different rules
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

#[test]
fn test_add_rule_vs_update_rule_identity() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "data-validation.mutation.addRule".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ruleId": "rule1",
            "ranges": []
        }),
    };

    let m2 = MutationInfo {
        id: "data-validation.mutation.updateRule".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ruleId": "rule2",
            "ranges": []
        }),
    };

    let result = service.transform(&m1, &m2);

    // Identity - different rules
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

#[test]
fn test_remove_rule_vs_remove_rule_identity() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "data-validation.mutation.removeRule".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ruleId": "rule1"
        }),
    };

    let m2 = MutationInfo {
        id: "data-validation.mutation.removeRule".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ruleId": "rule2"
        }),
    };

    let result = service.transform(&m1, &m2);

    // Identity - different rules
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

#[test]
fn test_update_rule_vs_update_rule_lww() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "data-validation.mutation.updateRule".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ruleId": "rule1",
            "ranges": []
        }),
    };

    let m2 = MutationInfo {
        id: "data-validation.mutation.updateRule".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ruleId": "rule1",
            "ranges": []
        }),
    };

    let result = service.transform(&m1, &m2);

    // LWW: m2 wins
    assert!(result.m1_prime.is_none());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

#[test]
fn test_add_rule_vs_insert_row_identity() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "data-validation.mutation.addRule".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ruleId": "rule1",
            "ranges": []
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.insert-row".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 5,
                "endColumn": 10
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    // Identity - different operations
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

// Conditional Formatting Tests

#[test]
fn test_add_conditional_rule_vs_add_conditional_rule_identity() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.add-conditional-rule".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ruleId": "cf_rule1",
            "ranges": []
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.add-conditional-rule".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ruleId": "cf_rule2",
            "ranges": []
        }),
    };

    let result = service.transform(&m1, &m2);

    // Identity - both succeed
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

#[test]
fn test_add_conditional_rule_vs_delete_conditional_rule_identity() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.add-conditional-rule".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ruleId": "cf_rule1",
            "ranges": []
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.delete-conditional-rule".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ruleId": "cf_rule2"
        }),
    };

    let result = service.transform(&m1, &m2);

    // Identity - different rules
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

#[test]
fn test_set_conditional_rule_vs_set_conditional_rule_lww() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.set-conditional-rule".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ruleId": "cf_rule1",
            "ranges": []
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.set-conditional-rule".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ruleId": "cf_rule1",
            "ranges": []
        }),
    };

    let result = service.transform(&m1, &m2);

    // LWW: m2 wins
    assert!(result.m1_prime.is_none());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

#[test]
fn test_move_conditional_rule_vs_move_conditional_rule_identity() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.move-conditional-rule".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ruleId": "cf_rule1",
            "targetIndex": 5
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.move-conditional-rule".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ruleId": "cf_rule2",
            "targetIndex": 3
        }),
    };

    let result = service.transform(&m1, &m2);

    // Identity - different rules
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

#[test]
fn test_add_conditional_rule_vs_set_conditional_rule_identity() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.add-conditional-rule".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ruleId": "cf_rule1",
            "ranges": []
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.set-conditional-rule".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ruleId": "cf_rule2",
            "ranges": []
        }),
    };

    let result = service.transform(&m1, &m2);

    // Identity - different rules
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

#[test]
fn test_add_conditional_rule_vs_insert_row_identity() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.add-conditional-rule".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ruleId": "cf_rule1",
            "ranges": []
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.insert-row".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 5,
                "endColumn": 10
            }
        }),
    };

    let result = service.transform(&m1, &m2);

    // Identity - different operations
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}
