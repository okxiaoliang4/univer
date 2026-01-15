use crate::types::{MutationInfo, TransformResult};
#[cfg(any(test, feature = "server"))]
use crate::types::{MutationInfoInternal, TransformResultInternal};

/// Trait for all mutations that can be transformed with other mutations
pub trait MutationTransform: Default {
    /// Transform this mutation with SetRangeValues mutation
    fn transform_with_set_range_values(&self, m1: &MutationInfo, m2: &MutationInfo) -> TransformResult;

    /// Transform this mutation with InsertRow mutation
    fn transform_with_insert_row(&self, m1: &MutationInfo, m2: &MutationInfo) -> TransformResult;

    /// Transform this mutation with InsertCol mutation
    fn transform_with_insert_col(&self, m1: &MutationInfo, m2: &MutationInfo) -> TransformResult;

    /// Transform this mutation with RemoveRows mutation
    fn transform_with_remove_rows(&self, m1: &MutationInfo, m2: &MutationInfo) -> TransformResult;

    /// Transform this mutation with RemoveCol mutation
    fn transform_with_remove_col(&self, m1: &MutationInfo, m2: &MutationInfo) -> TransformResult;

    /// Get the mutation ID string for this mutation type
    fn mutation_id() -> &'static str;

    // Internal methods that work with MutationInfoInternal (for Rust unit tests and server)
    #[cfg(any(test, feature = "server"))]
    fn transform_internal_with_set_range_values(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
        // Default implementation: convert to wasm-bindgen types and call the trait method
        // This will fail in non-WASM test environments, so each implementer should override this
        unimplemented!("transform_internal_with_set_range_values must be implemented")
    }

    #[cfg(any(test, feature = "server"))]
    fn transform_internal_with_insert_row(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
        unimplemented!("transform_internal_with_insert_row must be implemented")
    }

    #[cfg(any(test, feature = "server"))]
    fn transform_internal_with_insert_col(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
        unimplemented!("transform_internal_with_insert_col must be implemented")
    }

    #[cfg(any(test, feature = "server"))]
    fn transform_internal_with_remove_rows(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
        unimplemented!("transform_internal_with_remove_rows must be implemented")
    }

    #[cfg(any(test, feature = "server"))]
    fn transform_internal_with_remove_col(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
        unimplemented!("transform_internal_with_remove_col must be implemented")
    }
}
