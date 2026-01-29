use ot_core::{MutationInfo, MutationOutcome, TransformResult, TransformResultRef};
use serde_json::json;

// ============================================================================
// MutationOutcome Tests
// ============================================================================

#[test]
fn test_mutation_outcome_unchanged_into_owned() {
    let mutation = MutationInfo {
        id: "test.mutation".to_string(),
        params: json!({"key": "value"}),
    };

    let outcome = MutationOutcome::Unchanged(&mutation);
    let owned = outcome.into_owned();

    assert!(owned.is_some());
    let m = owned.unwrap();
    assert_eq!(m.id, "test.mutation");
}

#[test]
fn test_mutation_outcome_modified_into_owned() {
    let mutation = MutationInfo {
        id: "test.mutation".to_string(),
        params: json!({"key": "value"}),
    };

    let outcome = MutationOutcome::Modified(mutation);
    let owned = outcome.into_owned();

    assert!(owned.is_some());
    let m = owned.unwrap();
    assert_eq!(m.id, "test.mutation");
}

#[test]
fn test_mutation_outcome_removed_into_owned() {
    let outcome: MutationOutcome = MutationOutcome::Removed;
    let owned = outcome.into_owned();

    assert!(owned.is_none());
}

#[test]
fn test_mutation_outcome_is_unchanged() {
    let mutation = MutationInfo {
        id: "test".to_string(),
        params: json!({}),
    };

    let unchanged = MutationOutcome::Unchanged(&mutation);
    let modified = MutationOutcome::Modified(mutation.clone());
    let removed: MutationOutcome = MutationOutcome::Removed;

    assert!(unchanged.is_unchanged());
    assert!(!modified.is_unchanged());
    assert!(!removed.is_unchanged());
}

#[test]
fn test_mutation_outcome_is_modified() {
    let mutation = MutationInfo {
        id: "test".to_string(),
        params: json!({}),
    };

    let unchanged = MutationOutcome::Unchanged(&mutation);
    let modified = MutationOutcome::Modified(mutation.clone());
    let removed: MutationOutcome = MutationOutcome::Removed;

    assert!(!unchanged.is_modified());
    assert!(modified.is_modified());
    assert!(!removed.is_modified());
}

#[test]
fn test_mutation_outcome_is_removed() {
    let mutation = MutationInfo {
        id: "test".to_string(),
        params: json!({}),
    };

    let unchanged = MutationOutcome::Unchanged(&mutation);
    let modified = MutationOutcome::Modified(mutation.clone());
    let removed: MutationOutcome = MutationOutcome::Removed;

    assert!(!unchanged.is_removed());
    assert!(!modified.is_removed());
    assert!(removed.is_removed());
}

#[test]
fn test_mutation_outcome_is_some() {
    let mutation = MutationInfo {
        id: "test".to_string(),
        params: json!({}),
    };

    let unchanged = MutationOutcome::Unchanged(&mutation);
    let modified = MutationOutcome::Modified(mutation.clone());
    let removed: MutationOutcome = MutationOutcome::Removed;

    assert!(unchanged.is_some());
    assert!(modified.is_some());
    assert!(!removed.is_some());
}

#[test]
fn test_mutation_outcome_is_none() {
    let mutation = MutationInfo {
        id: "test".to_string(),
        params: json!({}),
    };

    let unchanged = MutationOutcome::Unchanged(&mutation);
    let modified = MutationOutcome::Modified(mutation.clone());
    let removed: MutationOutcome = MutationOutcome::Removed;

    assert!(!unchanged.is_none());
    assert!(!modified.is_none());
    assert!(removed.is_none());
}

#[test]
fn test_mutation_outcome_unwrap_unchanged() {
    let mutation = MutationInfo {
        id: "test.mutation".to_string(),
        params: json!({"key": "value"}),
    };

    let outcome = MutationOutcome::Unchanged(&mutation);
    let unwrapped = outcome.unwrap();

    assert_eq!(unwrapped.id, "test.mutation");
}

#[test]
fn test_mutation_outcome_unwrap_modified() {
    let mutation = MutationInfo {
        id: "test.mutation".to_string(),
        params: json!({"key": "value"}),
    };

    let outcome = MutationOutcome::Modified(mutation);
    let unwrapped = outcome.unwrap();

    assert_eq!(unwrapped.id, "test.mutation");
}

#[test]
#[should_panic(expected = "called `MutationOutcome::unwrap()` on a `Removed` value")]
fn test_mutation_outcome_unwrap_removed_panics() {
    let outcome: MutationOutcome = MutationOutcome::Removed;
    let _ = outcome.unwrap();
}

#[test]
fn test_mutation_outcome_as_ref_unchanged() {
    let mutation = MutationInfo {
        id: "test.mutation".to_string(),
        params: json!({"key": "value"}),
    };

    let outcome = MutationOutcome::Unchanged(&mutation);
    let reference = outcome.as_ref();

    assert!(reference.is_some());
    assert_eq!(reference.unwrap().id, "test.mutation");
}

#[test]
fn test_mutation_outcome_as_ref_modified() {
    let mutation = MutationInfo {
        id: "test.mutation".to_string(),
        params: json!({"key": "value"}),
    };

    let outcome = MutationOutcome::Modified(mutation);
    let reference = outcome.as_ref();

    assert!(reference.is_some());
    assert_eq!(reference.unwrap().id, "test.mutation");
}

#[test]
fn test_mutation_outcome_as_ref_removed() {
    let outcome: MutationOutcome = MutationOutcome::Removed;
    let reference = outcome.as_ref();

    assert!(reference.is_none());
}

// ============================================================================
// TransformResultRef Tests
// ============================================================================

#[test]
fn test_transform_result_ref_identity() {
    let m1 = MutationInfo {
        id: "mutation1".to_string(),
        params: json!({"key": "value1"}),
    };
    let m2 = MutationInfo {
        id: "mutation2".to_string(),
        params: json!({"key": "value2"}),
    };

    let result = TransformResultRef::identity(&m1, &m2);

    assert!(result.m1_prime.is_unchanged());
    assert!(result.m2_prime.is_unchanged());
    assert!(result.error.is_none());
}

#[test]
fn test_transform_result_ref_parse_error() {
    let m1 = MutationInfo {
        id: "mutation1".to_string(),
        params: json!({"key": "value1"}),
    };
    let m2 = MutationInfo {
        id: "mutation2".to_string(),
        params: json!({"key": "value2"}),
    };

    let result = TransformResultRef::parse_error(&m1, &m2, "Test error message");

    assert!(result.m1_prime.is_unchanged());
    assert!(result.m2_prime.is_unchanged());
    assert!(result.error.is_some());
    assert_eq!(result.error.unwrap(), "Test error message");
}

#[test]
fn test_transform_result_ref_into_owned() {
    let m1 = MutationInfo {
        id: "mutation1".to_string(),
        params: json!({"key": "value1"}),
    };
    let m2 = MutationInfo {
        id: "mutation2".to_string(),
        params: json!({"key": "value2"}),
    };

    let result_ref = TransformResultRef::identity(&m1, &m2);
    let result: TransformResult = result_ref.into_owned();

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert_eq!(result.m1_prime.unwrap().id, "mutation1");
    assert_eq!(result.m2_prime.unwrap().id, "mutation2");
    assert!(result.error.is_none());
}

#[test]
fn test_transform_result_ref_into_owned_with_removed() {
    let m1 = MutationInfo {
        id: "mutation1".to_string(),
        params: json!({}),
    };

    let result_ref = TransformResultRef {
        m1_prime: MutationOutcome::Removed,
        m2_prime: MutationOutcome::Unchanged(&m1),
        error: None,
    };
    let result: TransformResult = result_ref.into_owned();

    assert!(result.m1_prime.is_none());
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_transform_result_ref_into_owned_with_modified() {
    let m1 = MutationInfo {
        id: "mutation1".to_string(),
        params: json!({}),
    };
    let m2_modified = MutationInfo {
        id: "mutation2_modified".to_string(),
        params: json!({"modified": true}),
    };

    let result_ref = TransformResultRef {
        m1_prime: MutationOutcome::Unchanged(&m1),
        m2_prime: MutationOutcome::Modified(m2_modified),
        error: None,
    };
    let result: TransformResult = result_ref.into_owned();

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert_eq!(result.m2_prime.unwrap().id, "mutation2_modified");
}
