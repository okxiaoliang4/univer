use crate::types::{MutationInfo, TransformResult};
#[cfg(any(test, feature = "server"))]
use crate::types::{MutationInfoInternal, TransformResultInternal};
use crate::transform::mutation_transform::MutationTransform;

#[derive(Default)]
pub struct InsertColTransform;

impl MutationTransform for InsertColTransform {
    fn mutation_id() -> &'static str {
        "sheet.mutation.insert-col"
    }

    fn transform_with_set_range_values(&self, m1: &MutationInfo, m2: &MutationInfo) -> TransformResult {
        // Insert col doesn't need to change when set_range_values happens
        TransformResult {
            m1_prime: m1.clone(),
            m2_prime: m2.clone(),
            error: None,
        }
    }

    fn transform_with_insert_row(&self, m1: &MutationInfo, m2: &MutationInfo) -> TransformResult {
        // Insert col and insert row are independent
        TransformResult {
            m1_prime: m1.clone(),
            m2_prime: m2.clone(),
            error: None,
        }
    }

    fn transform_with_insert_col(&self, m1: &MutationInfo, m2: &MutationInfo) -> TransformResult {
        // Similar to insert_row::transform_with_insert_row but for columns
        // For now, return identity transform
        TransformResult {
            m1_prime: m1.clone(),
            m2_prime: m2.clone(),
            error: None,
        }
    }

    fn transform_with_remove_rows(&self, m1: &MutationInfo, m2: &MutationInfo) -> TransformResult {
        // Insert col and remove rows are independent
        TransformResult {
            m1_prime: m1.clone(),
            m2_prime: m2.clone(),
            error: None,
        }
    }

    fn transform_with_remove_col(&self, m1: &MutationInfo, m2: &MutationInfo) -> TransformResult {
        // Similar to insert_row::transform_with_remove_rows but for columns
        // For now, return identity transform
        TransformResult {
            m1_prime: m1.clone(),
            m2_prime: m2.clone(),
            error: None,
        }
    }

    // Internal methods that work with MutationInfoInternal (for Rust unit tests and server)
    #[cfg(any(test, feature = "server"))]
    fn transform_internal_with_set_range_values(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
        TransformResultInternal {
            m1_prime: m1.clone(),
            m2_prime: m2.clone(),
            error: None,
        }
    }

    #[cfg(any(test, feature = "server"))]
    fn transform_internal_with_insert_row(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
        TransformResultInternal {
            m1_prime: m1.clone(),
            m2_prime: m2.clone(),
            error: None,
        }
    }

    #[cfg(any(test, feature = "server"))]
    fn transform_internal_with_insert_col(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
        TransformResultInternal {
            m1_prime: m1.clone(),
            m2_prime: m2.clone(),
            error: None,
        }
    }

    #[cfg(any(test, feature = "server"))]
    fn transform_internal_with_remove_rows(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
        TransformResultInternal {
            m1_prime: m1.clone(),
            m2_prime: m2.clone(),
            error: None,
        }
    }

    #[cfg(any(test, feature = "server"))]
    fn transform_internal_with_remove_col(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
        TransformResultInternal {
            m1_prime: m1.clone(),
            m2_prime: m2.clone(),
            error: None,
        }
    }
}

