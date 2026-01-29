use ot_core::{MutationInfo, TransformRegistry, TransformResult};
use serde_json::json;
use std::sync::Arc;

// ============================================================================
// Legacy Registration Method Tests
// ============================================================================

#[test]
fn test_legacy_register_symmetric() {
    let mut registry = TransformRegistry::new();

    // Create a legacy TransformFn
    let legacy_transform = Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        TransformResult {
            m1_prime: Some(m1.clone()),
            m2_prime: Some(m2.clone()),
            error: None,
        }
    });

    // Register using legacy method
    registry.register_symmetric("test.mutation.legacy", legacy_transform);

    // Test that it works
    let m1 = MutationInfo {
        id: "test.mutation.legacy".to_string(),
        params: json!({"key": "value1"}),
    };
    let m2 = MutationInfo {
        id: "test.mutation.legacy".to_string(),
        params: json!({"key": "value2"}),
    };

    let result = registry.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

#[test]
fn test_legacy_register_symmetric_with_removed() {
    let mut registry = TransformRegistry::new();

    // Create a legacy TransformFn that removes m1 (LWW pattern)
    let legacy_transform = Arc::new(|_m1: &MutationInfo, m2: &MutationInfo| {
        TransformResult {
            m1_prime: None,  // This should become MutationOutcome::Removed
            m2_prime: Some(m2.clone()),
            error: None,
        }
    });

    registry.register_symmetric("test.mutation.lww", legacy_transform);

    let m1 = MutationInfo {
        id: "test.mutation.lww".to_string(),
        params: json!({"key": "value1"}),
    };
    let m2 = MutationInfo {
        id: "test.mutation.lww".to_string(),
        params: json!({"key": "value2"}),
    };

    let result = registry.transform(&m1, &m2);

    assert!(result.m1_prime.is_none());  // Should be removed
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

#[test]
fn test_legacy_register_bidirectional() {
    let mut registry = TransformRegistry::new();

    // Create a legacy TransformFn
    let legacy_transform = Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        TransformResult {
            m1_prime: Some(m1.clone()),
            m2_prime: Some(m2.clone()),
            error: None,
        }
    });

    // Register using legacy bidirectional method
    registry.register_bidirectional(
        "test.mutation.a",
        "test.mutation.b",
        legacy_transform,
    );

    // Test forward direction
    let m1 = MutationInfo {
        id: "test.mutation.a".to_string(),
        params: json!({"key": "value1"}),
    };
    let m2 = MutationInfo {
        id: "test.mutation.b".to_string(),
        params: json!({"key": "value2"}),
    };

    let result = registry.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());

    // Test reverse direction
    let m1 = MutationInfo {
        id: "test.mutation.b".to_string(),
        params: json!({"key": "value1"}),
    };
    let m2 = MutationInfo {
        id: "test.mutation.a".to_string(),
        params: json!({"key": "value2"}),
    };

    let result = registry.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

#[test]
fn test_legacy_register_bidirectional_with_removed() {
    let mut registry = TransformRegistry::new();

    // Create a legacy TransformFn that removes m1
    let legacy_transform = Arc::new(|_m1: &MutationInfo, m2: &MutationInfo| {
        TransformResult {
            m1_prime: None,  // Removed
            m2_prime: Some(m2.clone()),
            error: None,
        }
    });

    registry.register_bidirectional(
        "test.mutation.remove.a",
        "test.mutation.remove.b",
        legacy_transform,
    );

    let m1 = MutationInfo {
        id: "test.mutation.remove.a".to_string(),
        params: json!({}),
    };
    let m2 = MutationInfo {
        id: "test.mutation.remove.b".to_string(),
        params: json!({}),
    };

    let result = registry.transform(&m1, &m2);

    assert!(result.m1_prime.is_none());
    assert!(result.m2_prime.is_some());
}

#[test]
fn test_legacy_register_symmetric_with_both_removed() {
    let mut registry = TransformRegistry::new();

    // Create a legacy TransformFn that removes both
    let legacy_transform = Arc::new(|_m1: &MutationInfo, _m2: &MutationInfo| {
        TransformResult {
            m1_prime: None,
            m2_prime: None,
            error: None,
        }
    });

    registry.register_symmetric("test.mutation.both.removed", legacy_transform);

    let m1 = MutationInfo {
        id: "test.mutation.both.removed".to_string(),
        params: json!({}),
    };
    let m2 = MutationInfo {
        id: "test.mutation.both.removed".to_string(),
        params: json!({}),
    };

    let result = registry.transform(&m1, &m2);

    assert!(result.m1_prime.is_none());
    assert!(result.m2_prime.is_none());
}

#[test]
fn test_legacy_register_symmetric_with_error() {
    let mut registry = TransformRegistry::new();

    // Create a legacy TransformFn that returns an error
    let legacy_transform = Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        TransformResult {
            m1_prime: Some(m1.clone()),
            m2_prime: Some(m2.clone()),
            error: Some("Legacy error".to_string()),
        }
    });

    registry.register_symmetric("test.mutation.error", legacy_transform);

    let m1 = MutationInfo {
        id: "test.mutation.error".to_string(),
        params: json!({}),
    };
    let m2 = MutationInfo {
        id: "test.mutation.error".to_string(),
        params: json!({}),
    };

    let result = registry.transform(&m1, &m2);

    assert!(result.error.is_some());
    assert_eq!(result.error.unwrap(), "Legacy error");
}

#[test]
fn test_legacy_register_bidirectional_with_m2_removed() {
    let mut registry = TransformRegistry::new();

    // Create a legacy TransformFn that removes m2 (covers registry.rs line 151)
    let legacy_transform = Arc::new(|m1: &MutationInfo, _m2: &MutationInfo| {
        TransformResult {
            m1_prime: Some(m1.clone()),
            m2_prime: None,  // m2 is removed - this covers line 151
            error: None,
        }
    });

    registry.register_bidirectional(
        "test.mutation.m2remove.a",
        "test.mutation.m2remove.b",
        legacy_transform,
    );

    let m1 = MutationInfo {
        id: "test.mutation.m2remove.a".to_string(),
        params: json!({}),
    };
    let m2 = MutationInfo {
        id: "test.mutation.m2remove.b".to_string(),
        params: json!({}),
    };

    let result = registry.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_none());  // m2 should be removed
}
