use crate::types::{MutationInfoInternal, TransformResultInternal};

/// Core trait for mutation transforms
/// All transforms operate on core types (MutationInfoInternal/TransformResultInternal)
/// which use serde_json::Value internally for zero-copy efficiency
pub trait MutationTransform: Default {
    /// Get the mutation ID string for this mutation type
    fn mutation_id() -> &'static str;

    /// Transform this mutation type against set-range-values mutation
    fn transform_with_set_range_values(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal;
    
    /// Transform this mutation type against insert-row mutation
    fn transform_with_insert_row(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal;
    
    /// Transform this mutation type against insert-col mutation
    fn transform_with_insert_col(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal;
    
    /// Transform this mutation type against remove-rows mutation
    fn transform_with_remove_rows(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal;
    
    /// Transform this mutation type against remove-col mutation
    fn transform_with_remove_col(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal;
}
