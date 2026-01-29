//! OT Core - Pure Rust Operational Transformation Logic
//!
//! This crate contains the core OT (Operational Transformation) logic for Univer.
//! It is completely independent of WASM bindings and can be used in both:
//! - WASM environments (via the ot-wasm crate)
//! - Server environments (via the ot-server crate)
//!
//! # Architecture
//!
//! The OT system is built around a HashMap-based registry that supports:
//! - **Bidirectional Registration**: Single implementation with automatic swap
//! - **Symmetric Registration**: Same-type transforms (e.g., insert-row vs insert-row)
//! - **Identity Registration**: Mutations that don't affect each other
//!
//! # Example
//!
//! ```rust
//! use ot_core::{TransformService, MutationInfo};
//! use serde_json::json;
//!
//! let service = TransformService::new();
//!
//! let m1 = MutationInfo {
//!     id: "sheet.mutation.insert-row".to_string(),
//!     params: json!({
//!         "unitId": "workbook1",
//!         "subUnitId": "sheet1",
//!         "range": {
//!             "startRow": 5,
//!             "startColumn": 0,
//!             "endRow": 5,
//!             "endColumn": 10
//!         }
//!     }),
//! };
//!
//! let m2 = MutationInfo {
//!     id: "sheet.mutation.set-range-values".to_string(),
//!     params: json!({
//!         "unitId": "workbook1",
//!         "subUnitId": "sheet1",
//!         "cellValue": {}
//!     }),
//! };
//!
//! let result = service.transform(&m1, &m2);
//! ```

mod registry;
mod types;
mod params;
pub mod transforms;
pub mod utils;

pub use registry::{TransformRegistry, TransformFn, MutationId};
pub use types::*;
pub use params::*;

/// The main service for performing OT transforms
///
/// This service maintains a registry of all transforms and provides
/// high-level APIs for transforming mutations.
pub struct TransformService {
    registry: TransformRegistry,
}

impl TransformService {
    /// Create a new TransformService with all transforms registered
    pub fn new() -> Self {
        let mut registry = TransformRegistry::new();
        transforms::register_all(&mut registry);
        Self { registry }
    }

    /// Transform two concurrent mutations according to OT rules
    ///
    /// Returns the transformed versions of both mutations such that:
    /// ```text
    /// State + m1 + m2' = State + m2 + m1'
    /// ```
    pub fn transform(&self, m1: &MutationInfo, m2: &MutationInfo) -> TransformResult {
        self.registry.transform(m1, m2)
    }

    /// Transform a list of mutations against another list
    ///
    /// This is useful for transforming a sequence of local mutations against
    /// a sequence of remote mutations in collaborative editing scenarios.
    pub fn transform_list(
        &self,
        m1_list: &[MutationInfo],
        m2_list: &[MutationInfo],
    ) -> (Vec<MutationInfo>, Vec<MutationInfo>, Option<String>) {
        let mut m1_prime_list = Vec::new();
        let mut m2_prime_list = Vec::new();

        // Transform m1_list against m2_list
        let m1_temp = m1_list.to_vec();
        let mut m2_temp = m2_list.to_vec();

        for m1 in &m1_temp {
            let mut m1_current = m1.clone();
            let mut m2_transformed = Vec::new();

            for m2 in &m2_temp {
                let result = self.transform(&m1_current, m2);
                if let Some(error) = result.error {
                    return (m1_prime_list, m2_prime_list, Some(error));
                }

                if let Some(m1_prime) = result.m1_prime {
                    m1_current = m1_prime;
                }

                if let Some(m2_prime) = result.m2_prime {
                    m2_transformed.push(m2_prime);
                }
            }

            m1_prime_list.push(m1_current);
            m2_temp = m2_transformed;
        }

        m2_prime_list = m2_temp;

        (m1_prime_list, m2_prime_list, None)
    }

    /// Compose a list of mutations into a single optimized list
    ///
    /// This can help reduce the number of mutations that need to be stored
    /// and transmitted in collaborative editing scenarios.
    pub fn compose(&self, mutations: &[MutationInfo]) -> Vec<MutationInfo> {
        // TODO: Implement composition logic
        // For now, just return the original list
        mutations.to_vec()
    }

    /// Get the number of registered transforms
    pub fn registry_size(&self) -> usize {
        self.registry.len()
    }
}

impl Default for TransformService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_service_creation() {
        let service = TransformService::new();
        assert!(service.registry_size() > 0);
    }

    #[test]
    fn test_identity_transform() {
        let service = TransformService::new();

        let m1 = MutationInfo {
            id: "unknown.mutation".to_string(),
            params: json!({}),
        };

        let m2 = MutationInfo {
            id: "another.unknown.mutation".to_string(),
            params: json!({}),
        };

        let result = service.transform(&m1, &m2);
        assert!(result.m1_prime.is_some());
        assert!(result.m2_prime.is_some());
        assert!(result.error.is_none());
    }

    #[test]
    fn test_transform_list() {
        let service = TransformService::new();

        let m1_list = vec![
            MutationInfo {
                id: "test.mutation.1".to_string(),
                params: json!({}),
            },
            MutationInfo {
                id: "test.mutation.2".to_string(),
                params: json!({}),
            },
        ];

        let m2_list = vec![
            MutationInfo {
                id: "test.mutation.3".to_string(),
                params: json!({}),
            },
        ];

        let (m1_prime, m2_prime, error) = service.transform_list(&m1_list, &m2_list);
        assert!(error.is_none());
        assert_eq!(m1_prime.len(), 2);
        assert_eq!(m2_prime.len(), 1);
    }

    #[test]
    fn test_transform_list_with_error() {
        // Test that transform_list returns error when a transform fails
        // This covers line 108: return (m1_prime_list, m2_prime_list, Some(error));
        let service = TransformService::new();

        let m1_list = vec![
            MutationInfo {
                id: "sheet.mutation.set-range-values".to_string(),
                params: json!({"invalid": "params"}), // Invalid params to trigger error
            },
        ];

        let m2_list = vec![
            MutationInfo {
                id: "sheet.mutation.set-range-values".to_string(),
                params: json!({
                    "unitId": "workbook1",
                    "subUnitId": "sheet1",
                    "cellValue": {}
                }),
            },
        ];

        let (m1_prime, m2_prime, error) = service.transform_list(&m1_list, &m2_list);
        assert!(error.is_some());
        assert!(error.unwrap().contains("Failed to parse"));
        // When error occurs, both lists should be partially populated
        assert!(m1_prime.is_empty()); // m1 hasn't been added yet when error occurs
        assert!(m2_prime.is_empty());
    }

    #[test]
    fn test_compose() {
        // Test the compose function (currently a stub)
        // This covers lines 133, 136
        let service = TransformService::new();

        let mutations = vec![
            MutationInfo {
                id: "test.mutation.1".to_string(),
                params: json!({}),
            },
            MutationInfo {
                id: "test.mutation.2".to_string(),
                params: json!({}),
            },
        ];

        let composed = service.compose(&mutations);
        assert_eq!(composed.len(), 2); // Currently just returns the original list
        assert_eq!(composed[0].id, "test.mutation.1");
        assert_eq!(composed[1].id, "test.mutation.2");
    }

    #[test]
    fn test_default_trait() {
        // Test the Default trait implementation
        // This covers lines 146-147
        let service = TransformService::default();
        assert!(service.registry_size() > 0);

        // Verify it's equivalent to new()
        let service_new = TransformService::new();
        assert_eq!(service.registry_size(), service_new.registry_size());
    }
}
