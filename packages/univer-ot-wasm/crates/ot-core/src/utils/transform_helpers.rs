//! Common transform helper functions
//!
//! This module provides reusable transform functions (LWW, identity) as static Arc instances
//! to avoid repeatedly creating identical closures across different transform modules.

use crate::registry::TransformFnRef;
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef};
use once_cell::sync::Lazy;
use std::sync::Arc;

/// Static Last-Write-Wins (LWW) transform function
///
/// In LWW strategy, m2 always wins, m1 is removed.
/// This is useful for mutations that modify the same property,
/// where the later operation should override the earlier one.
///
/// # Transform behavior
/// - m1_prime: Removed (m2 wins)
/// - m2_prime: Unchanged
static LWW_TRANSFORM: Lazy<TransformFnRef> = Lazy::new(|| {
    Arc::new(|_m1: &MutationInfo, m2: &MutationInfo| {
        // LWW: m2 wins, m1 is removed
        TransformResultRef {
            m1_prime: MutationOutcome::Removed,
            m2_prime: MutationOutcome::Unchanged(m2),
            error: None,
        }
    })
});

/// Static identity transform function
///
/// Identity transform means both mutations can proceed unchanged,
/// as they don't conflict with each other.
///
/// # Transform behavior
/// - m1_prime: Unchanged
/// - m2_prime: Unchanged
static IDENTITY_TRANSFORM: Lazy<TransformFnRef> = Lazy::new(|| {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        TransformResultRef::identity(m1, m2)
    })
});

/// Returns a clone of the shared LWW transform function
///
/// Use this instead of creating a new `Arc::new(...)` for LWW behavior
/// in your transform registrations.
///
/// # Example
/// ```rust
/// use crate::utils::transform_helpers::lww_transform;
///
/// registry.register_symmetric_ref(MUTATION_ID, lww_transform());
/// ```
#[inline]
pub fn lww_transform() -> TransformFnRef {
    Arc::clone(&LWW_TRANSFORM)
}

/// Returns a clone of the shared identity transform function
///
/// Use this instead of creating a new `Arc::new(...)` for identity behavior
/// in your transform registrations.
///
/// # Example
/// ```rust
/// use crate::utils::transform_helpers::identity_transform;
///
/// registry.register_symmetric_ref(MUTATION_ID, identity_transform());
/// registry.register_bidirectional_ref(M1_ID, M2_ID, identity_transform());
/// ```
#[inline]
pub fn identity_transform() -> TransformFnRef {
    Arc::clone(&IDENTITY_TRANSFORM)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MutationInfo;
    use serde_json::json;

    #[test]
    fn test_lww_transform() {
        let m1 = MutationInfo {
            id: "test.mutation.1".to_string(),
            params: json!({"value": 1}),
        };
        let m2 = MutationInfo {
            id: "test.mutation.2".to_string(),
            params: json!({"value": 2}),
        };

        let transform = lww_transform();
        let result = transform(&m1, &m2);

        // LWW: m2 wins, m1 is removed
        assert!(matches!(result.m1_prime, MutationOutcome::Removed));
        assert!(matches!(result.m2_prime, MutationOutcome::Unchanged(_)));
        assert!(result.error.is_none());
    }

    #[test]
    fn test_identity_transform() {
        let m1 = MutationInfo {
            id: "test.mutation.1".to_string(),
            params: json!({"value": 1}),
        };
        let m2 = MutationInfo {
            id: "test.mutation.2".to_string(),
            params: json!({"value": 2}),
        };

        let transform = identity_transform();
        let result = transform(&m1, &m2);

        // Identity: both unchanged
        assert!(matches!(result.m1_prime, MutationOutcome::Unchanged(_)));
        assert!(matches!(result.m2_prime, MutationOutcome::Unchanged(_)));
        assert!(result.error.is_none());
    }

    #[test]
    fn test_arc_sharing() {
        // Verify that calling the functions multiple times returns clones of the same Arc
        let lww1 = lww_transform();
        let lww2 = lww_transform();

        // Both should point to the same underlying function
        assert_eq!(Arc::strong_count(&lww1), Arc::strong_count(&lww2));

        let id1 = identity_transform();
        let id2 = identity_transform();

        // Both should point to the same underlying function
        assert_eq!(Arc::strong_count(&id1), Arc::strong_count(&id2));
    }
}
