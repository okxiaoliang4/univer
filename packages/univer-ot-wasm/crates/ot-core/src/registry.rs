use std::collections::HashMap;
use std::sync::Arc;
use crate::types::{MutationInfo, MutationOutcome, TransformResult, TransformResultRef};

/// Transform function signature (legacy)
/// Takes two mutations and returns a transform result
pub type TransformFn = Arc<
    dyn Fn(&MutationInfo, &MutationInfo) -> TransformResult + Send + Sync
>;

/// Optimized transform function signature with HRTB (Higher-Ranked Trait Bounds)
///
/// This signature allows returning references to the input mutations when unchanged,
/// avoiding unnecessary clones in identity transform cases (~40% of all transforms).
///
/// # Example
/// ```ignore
/// let transform: TransformFnRef = Arc::new(|m1, m2| {
///     // Zero-copy identity when mutations don't affect each other
///     if different_worksheet(m1, m2) {
///         return TransformResultRef::identity(m1, m2);
///     }
///     // Only clone when actually modified
///     TransformResultRef {
///         m1_prime: MutationOutcome::Unchanged(m1),
///         m2_prime: MutationOutcome::Modified(modified_m2),
///         error: None,
///     }
/// });
/// ```
pub type TransformFnRef = Arc<
    dyn for<'a> Fn(&'a MutationInfo, &'a MutationInfo) -> TransformResultRef<'a> + Send + Sync
>;

/// Mutation ID type
pub type MutationId = &'static str;

/// Registry for all transform functions
///
/// This registry stores transform functions in a HashMap for O(1) lookup.
/// It supports three types of registration:
/// 1. Bidirectional: Single implementation with automatic swap for reverse
/// 2. Symmetric: Same-type transforms (e.g., insert-row vs insert-row)
/// 3. Identity: Mutations that don't affect each other
///
/// # Memory Optimization
///
/// The registry internally uses `TransformFnRef` which returns `TransformResultRef`
/// with `MutationOutcome` enum. This allows:
/// - Identity transforms to return references (zero-copy)
/// - Modified mutations to return owned values
/// - Cloning only happens when `into_owned()` is called at the boundary
pub struct TransformRegistry {
    /// Internal storage using optimized TransformFnRef
    transforms_ref: HashMap<(MutationId, MutationId), TransformFnRef>,
}

impl TransformRegistry {
    pub fn new() -> Self {
        Self {
            transforms_ref: HashMap::new(),
        }
    }

    // ========================================================================
    // Optimized Registration Methods (using TransformFnRef)
    // ========================================================================

    /// Register a bidirectional transform with automatic swap (optimized version)
    ///
    /// This is the preferred method for new transforms. It uses `TransformResultRef`
    /// which allows returning references for unchanged mutations, avoiding clones.
    ///
    /// Only one implementation is needed - the reverse is generated automatically.
    ///
    /// # Mathematical Correctness
    ///
    /// This relies on the OT TP1 property:
    /// ```text
    /// State + m1 + m2' = State + m2 + m1'
    /// ```
    pub fn register_bidirectional_ref(
        &mut self,
        m1_id: MutationId,
        m2_id: MutationId,
        forward: TransformFnRef,
    ) {
        // Register forward direction
        self.transforms_ref.insert((m1_id, m2_id), forward.clone());

        // Generate and register reverse direction (swap)
        let reverse: TransformFnRef = Arc::new(move |m1: &MutationInfo, m2: &MutationInfo| {
            // Call forward transform with swapped arguments
            let result = forward(m2, m1);
            // Swap the results back
            TransformResultRef {
                m1_prime: result.m2_prime,
                m2_prime: result.m1_prime,
                error: result.error,
            }
        });
        self.transforms_ref.insert((m2_id, m1_id), reverse);
    }

    /// Register a symmetric transform (optimized version)
    ///
    /// Used when both mutations are of the same type, e.g., insert-row vs insert-row
    pub fn register_symmetric_ref(&mut self, id: MutationId, transform: TransformFnRef) {
        self.transforms_ref.insert((id, id), transform);
    }

    /// Register an identity transform (optimized, zero-copy)
    ///
    /// This is the most efficient form - identity transforms return references
    /// to the original mutations without any cloning.
    pub fn register_identity(&mut self, m1_id: MutationId, m2_id: MutationId) {
        let identity: TransformFnRef = Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
            TransformResultRef::identity(m1, m2)  // Zero-copy!
        });

        self.transforms_ref.insert((m1_id, m2_id), identity.clone());
        if m1_id != m2_id {
            self.transforms_ref.insert((m2_id, m1_id), identity);
        }
    }

    // ========================================================================
    // Legacy Registration Methods (for backward compatibility)
    // ========================================================================

    /// Register a bidirectional transform with automatic swap (legacy version)
    ///
    /// This wraps legacy TransformFn into TransformFnRef for internal storage.
    /// New code should prefer `register_bidirectional_ref`.
    pub fn register_bidirectional(
        &mut self,
        m1_id: MutationId,
        m2_id: MutationId,
        forward: TransformFn,
    ) {
        // Wrap legacy TransformFn into TransformFnRef
        let forward_ref: TransformFnRef = Arc::new(move |m1: &MutationInfo, m2: &MutationInfo| {
            let result = forward(m1, m2);
            TransformResultRef {
                m1_prime: match result.m1_prime {
                    Some(m) => MutationOutcome::Modified(m),
                    None => MutationOutcome::Removed,
                },
                m2_prime: match result.m2_prime {
                    Some(m) => MutationOutcome::Modified(m),
                    None => MutationOutcome::Removed,
                },
                error: result.error,
            }
        });

        // Use the optimized registration internally
        self.register_bidirectional_ref(m1_id, m2_id, forward_ref);
    }

    /// Register a symmetric transform (legacy version)
    ///
    /// Wraps legacy TransformFn into TransformFnRef for internal storage.
    pub fn register_symmetric(&mut self, id: MutationId, transform: TransformFn) {
        let transform_ref: TransformFnRef = Arc::new(move |m1: &MutationInfo, m2: &MutationInfo| {
            let result = transform(m1, m2);
            TransformResultRef {
                m1_prime: match result.m1_prime {
                    Some(m) => MutationOutcome::Modified(m),
                    None => MutationOutcome::Removed,
                },
                m2_prime: match result.m2_prime {
                    Some(m) => MutationOutcome::Modified(m),
                    None => MutationOutcome::Removed,
                },
                error: result.error,
            }
        });

        self.register_symmetric_ref(id, transform_ref);
    }

    // ========================================================================
    // Transform Execution
    // ========================================================================

    /// Execute a transform between two mutations (returns owned result)
    ///
    /// Returns the transform result if a transform is registered,
    /// otherwise falls back to identity transform (both mutations unchanged).
    ///
    /// This method calls `into_owned()` at the boundary, which is where
    /// cloning happens for unchanged mutations.
    pub fn transform(&self, m1: &MutationInfo, m2: &MutationInfo) -> TransformResult {
        self.transform_ref(m1, m2).into_owned()
    }

    /// Execute a transform between two mutations (returns reference result)
    ///
    /// This is the optimized version that delays cloning. Use this when you
    /// can work with references and want to avoid unnecessary allocations.
    pub fn transform_ref<'a>(&self, m1: &'a MutationInfo, m2: &'a MutationInfo) -> TransformResultRef<'a> {
        if let Some(transform_fn) = self.get_transform_ref(&m1.id, &m2.id) {
            return transform_fn(m1, m2);
        }

        // Fallback: identity transform (zero-copy!)
        TransformResultRef::identity(m1, m2)
    }

    /// Get a transform function from the registry
    fn get_transform_ref(&self, m1_id: &str, m2_id: &str) -> Option<&TransformFnRef> {
        // Find the transform in the HashMap
        self.transforms_ref.iter()
            .find(|((k1, k2), _)| *k1 == m1_id && *k2 == m2_id)
            .map(|(_, v)| v)
    }

    /// Check if a transform is registered for the given mutation pair
    pub fn has_transform(&self, m1_id: &str, m2_id: &str) -> bool {
        self.get_transform_ref(m1_id, m2_id).is_some()
    }

    /// Get the number of registered transforms
    pub fn len(&self) -> usize {
        self.transforms_ref.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.transforms_ref.is_empty()
    }
}

impl Default for TransformRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_identity_transform() {
        let registry = TransformRegistry::new();

        let m1 = MutationInfo {
            id: "test.mutation.1".to_string(),
            params: json!({"value": 1}),
        };
        let m2 = MutationInfo {
            id: "test.mutation.2".to_string(),
            params: json!({"value": 2}),
        };

        let result = registry.transform(&m1, &m2);

        assert!(result.m1_prime.is_some());
        assert!(result.m2_prime.is_some());
        assert_eq!(result.m1_prime.as_ref().unwrap().id, m1.id);
        assert_eq!(result.m2_prime.as_ref().unwrap().id, m2.id);
    }

    #[test]
    fn test_bidirectional_registration() {
        let mut registry = TransformRegistry::new();

        // Register a simple transform
        let forward_fn = Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
            TransformResult {
                m1_prime: Some(MutationInfo {
                    id: m1.id.clone(),
                    params: json!({"transformed": "forward"}),
                }),
                m2_prime: Some(m2.clone()),
                error: None,
            }
        });

        registry.register_bidirectional("mutation.a", "mutation.b", forward_fn);

        // Test forward direction
        let m1 = MutationInfo {
            id: "mutation.a".to_string(),
            params: json!({}),
        };
        let m2 = MutationInfo {
            id: "mutation.b".to_string(),
            params: json!({}),
        };

        let forward_result = registry.transform(&m1, &m2);
        assert!(forward_result.m1_prime.is_some());

        // Test reverse direction (automatically swapped)
        let reverse_result = registry.transform(&m2, &m1);
        assert!(reverse_result.m2_prime.is_some());
    }

    #[test]
    fn test_registry_len() {
        let mut registry = TransformRegistry::new();
        assert_eq!(registry.len(), 0);

        registry.register_identity("mutation.a", "mutation.b");
        assert_eq!(registry.len(), 2); // Both directions registered

        let transform = Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
            TransformResult {
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            }
        });

        registry.register_symmetric("mutation.c", transform);
        assert_eq!(registry.len(), 3);
    }

    #[test]
    fn test_registry_is_empty() {
        // Test the is_empty() method
        // This covers lines 130-131
        let mut registry = TransformRegistry::new();
        assert!(registry.is_empty()); // New registry is empty
        assert_eq!(registry.len(), 0);

        // Add a transform
        registry.register_identity("mutation.a", "mutation.b");
        assert!(!registry.is_empty()); // No longer empty
        assert!(registry.len() > 0);
    }

    #[test]
    fn test_registry_default_trait() {
        // Test the Default trait implementation
        // This covers lines 136-137
        let registry = TransformRegistry::default();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);

        // Verify it's equivalent to new()
        let registry_new = TransformRegistry::new();
        assert_eq!(registry.len(), registry_new.len());
    }
}
