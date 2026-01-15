use crate::types::{MutationInfo, TransformResult};
#[cfg(any(test, feature = "server"))]
use crate::types::{MutationInfoInternal, TransformResultInternal};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

pub mod mutation_transform;
pub mod set_range_values;
pub mod insert_row;
pub mod insert_col;
pub mod remove_rows;
pub mod remove_col;

#[cfg(test)]
mod test_utils;

#[cfg(test)]
mod set_range_values_test;
#[cfg(test)]
mod insert_row_test;
#[cfg(test)]
mod remove_rows_test;
#[cfg(test)]
mod insert_col_test;
#[cfg(test)]
mod remove_col_test;

use mutation_transform::MutationTransform;

type TransformAlgorithm = fn(&MutationInfo, &MutationInfo) -> TransformResult;

#[wasm_bindgen]
pub struct TransformService {
    algorithms: HashMap<String, HashMap<String, TransformAlgorithm>>,
}

#[wasm_bindgen]
impl TransformService {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let mut service = Self {
            algorithms: HashMap::new(),
        };

        // Register all mutation transforms
        // Adding a new mutation type only requires:
        // 1. Implementing MutationTransform trait
        // 2. Adding one line here: service.register_mutation_transforms::<NewMutationTransform>();
        service.register_mutation_transforms::<set_range_values::SetRangeValuesTransform>();
        service.register_mutation_transforms::<insert_row::InsertRowTransform>();
        service.register_mutation_transforms::<insert_col::InsertColTransform>();
        service.register_mutation_transforms::<remove_rows::RemoveRowsTransform>();
        service.register_mutation_transforms::<remove_col::RemoveColTransform>();

        service
    }

    /// Register transform methods for a mutation type that implements MutationTransform
    /// This method automatically registers all transform methods by calling create_wrapper
    /// for each known mutation type. The create_wrapper function handles dispatching to
    /// the correct trait method based on the mutation type.
    fn register_mutation_transforms<T: MutationTransform>(&mut self) {
        let mutation_id = T::mutation_id().to_string();
        let mut transforms = HashMap::new();

        // Automatically register transforms with all known mutation types
        // The create_wrapper function handles dispatching to the correct trait method
        transforms.insert(
            set_range_values::SetRangeValuesTransform::mutation_id().to_string(),
            Self::create_wrapper::<T, set_range_values::SetRangeValuesTransform>(),
        );
        transforms.insert(
            insert_row::InsertRowTransform::mutation_id().to_string(),
            Self::create_wrapper::<T, insert_row::InsertRowTransform>(),
        );
        transforms.insert(
            insert_col::InsertColTransform::mutation_id().to_string(),
            Self::create_wrapper::<T, insert_col::InsertColTransform>(),
        );
        transforms.insert(
            remove_rows::RemoveRowsTransform::mutation_id().to_string(),
            Self::create_wrapper::<T, remove_rows::RemoveRowsTransform>(),
        );
        transforms.insert(
            remove_col::RemoveColTransform::mutation_id().to_string(),
            Self::create_wrapper::<T, remove_col::RemoveColTransform>(),
        );

        self.algorithms.insert(mutation_id, transforms);
    }

    /// Create a wrapper function that dispatches to the correct trait method based on U's mutation ID
    /// This function uses compile-time dispatch to call the appropriate trait method.
    /// The match statement is necessary because we need to know which trait method to call
    /// based on the mutation type U.
    fn create_wrapper<T: MutationTransform, U: MutationTransform>() -> TransformAlgorithm {
        // Dispatch to the correct trait method based on U's mutation ID
        // Directly return TransformResult with JsValue params to avoid serialization/deserialization overhead
        match U::mutation_id() {
            id if id == set_range_values::SetRangeValuesTransform::mutation_id() => {
                |m1, m2| {
                    // m1 and m2 are already MutationInfo with JsValue params, no conversion needed
                    T::default().transform_with_set_range_values(m1, m2)
                }
            }
            id if id == insert_row::InsertRowTransform::mutation_id() => {
                |m1, m2| {
                    T::default().transform_with_insert_row(m1, m2)
                }
            }
            id if id == insert_col::InsertColTransform::mutation_id() => {
                |m1, m2| {
                    T::default().transform_with_insert_col(m1, m2)
                }
            }
            id if id == remove_rows::RemoveRowsTransform::mutation_id() => {
                |m1, m2| {
                    T::default().transform_with_remove_rows(m1, m2)
                }
            }
            id if id == remove_col::RemoveColTransform::mutation_id() => {
                |m1, m2| {
                    T::default().transform_with_remove_col(m1, m2)
                }
            }
            _ => |m1, m2| TransformResult {
                m1_prime: MutationInfo {
                    id: m1.id.clone(),
                    params: m1.params.clone(),
                },
                m2_prime: MutationInfo {
                    id: m2.id.clone(),
                    params: m2.params.clone(),
                },
                error: None,
            },
        }
    }

    pub fn transform(&self, m1: &MutationInfo, m2: &MutationInfo) -> TransformResult {
        // Directly use MutationInfo with JsValue params to avoid serialization/deserialization overhead
        if let Some(m1_algorithms) = self.algorithms.get(&m1.id) {
            if let Some(algorithm) = m1_algorithms.get(&m2.id) {
                return algorithm(m1, m2);
            }
        }

        // No algorithm found, return identity transform
        TransformResult {
            m1_prime: MutationInfo {
                id: m1.id.clone(),
                params: m1.params.clone(),
            },
            m2_prime: MutationInfo {
                id: m2.id.clone(),
                params: m2.params.clone(),
            },
            error: None,
        }
    }

    // Note: get_algorithms removed as wasm-bindgen doesn't support returning borrowed references
    // Use transform() method instead to interact with the service
}

// Separate impl block for test-only methods that don't use wasm-bindgen types
#[cfg(any(test, feature = "server"))]
impl TransformService {
    /// Internal method that works with MutationInfoInternal (for Rust unit tests and server)
    /// This bypasses the wasm-bindgen wrapper and directly calls the transform algorithms
    pub fn transform_internal(&self, m1: &crate::types::MutationInfoInternal, m2: &crate::types::MutationInfoInternal) -> crate::types::TransformResultInternal {
        // Directly call the transform algorithms using internal types
        // We need to manually dispatch based on mutation IDs since we can't use the wasm-bindgen wrappers in tests

        match (m1.id.as_str(), m2.id.as_str()) {
            ("sheet.mutation.set-range-values", "sheet.mutation.set-range-values") => {
                set_range_values::SetRangeValuesTransform::default()
                    .transform_internal_with_set_range_values(m1, m2)
            }
            ("sheet.mutation.set-range-values", "sheet.mutation.insert-row") => {
                set_range_values::SetRangeValuesTransform::default()
                    .transform_internal_with_insert_row(m1, m2)
            }
            ("sheet.mutation.set-range-values", "sheet.mutation.insert-col") => {
                set_range_values::SetRangeValuesTransform::default()
                    .transform_internal_with_insert_col(m1, m2)
            }
            ("sheet.mutation.set-range-values", "sheet.mutation.remove-rows") => {
                set_range_values::SetRangeValuesTransform::default()
                    .transform_internal_with_remove_rows(m1, m2)
            }
            ("sheet.mutation.set-range-values", "sheet.mutation.remove-col") => {
                set_range_values::SetRangeValuesTransform::default()
                    .transform_internal_with_remove_col(m1, m2)
            }
            ("sheet.mutation.insert-row", "sheet.mutation.set-range-values") => {
                insert_row::InsertRowTransform::default()
                    .transform_internal_with_set_range_values(m1, m2)
            }
            ("sheet.mutation.insert-row", "sheet.mutation.insert-row") => {
                insert_row::InsertRowTransform::default()
                    .transform_internal_with_insert_row(m1, m2)
            }
            ("sheet.mutation.insert-row", "sheet.mutation.insert-col") => {
                insert_row::InsertRowTransform::default()
                    .transform_internal_with_insert_col(m1, m2)
            }
            ("sheet.mutation.insert-row", "sheet.mutation.remove-rows") => {
                insert_row::InsertRowTransform::default()
                    .transform_internal_with_remove_rows(m1, m2)
            }
            ("sheet.mutation.insert-row", "sheet.mutation.remove-col") => {
                insert_row::InsertRowTransform::default()
                    .transform_internal_with_remove_col(m1, m2)
            }
            ("sheet.mutation.insert-col", "sheet.mutation.set-range-values") => {
                insert_col::InsertColTransform::default()
                    .transform_internal_with_set_range_values(m1, m2)
            }
            ("sheet.mutation.insert-col", "sheet.mutation.insert-row") => {
                insert_col::InsertColTransform::default()
                    .transform_internal_with_insert_row(m1, m2)
            }
            ("sheet.mutation.insert-col", "sheet.mutation.insert-col") => {
                insert_col::InsertColTransform::default()
                    .transform_internal_with_insert_col(m1, m2)
            }
            ("sheet.mutation.insert-col", "sheet.mutation.remove-rows") => {
                insert_col::InsertColTransform::default()
                    .transform_internal_with_remove_rows(m1, m2)
            }
            ("sheet.mutation.insert-col", "sheet.mutation.remove-col") => {
                insert_col::InsertColTransform::default()
                    .transform_internal_with_remove_col(m1, m2)
            }
            ("sheet.mutation.remove-rows", "sheet.mutation.set-range-values") => {
                remove_rows::RemoveRowsTransform::default()
                    .transform_internal_with_set_range_values(m1, m2)
            }
            ("sheet.mutation.remove-rows", "sheet.mutation.insert-row") => {
                remove_rows::RemoveRowsTransform::default()
                    .transform_internal_with_insert_row(m1, m2)
            }
            ("sheet.mutation.remove-rows", "sheet.mutation.insert-col") => {
                remove_rows::RemoveRowsTransform::default()
                    .transform_internal_with_insert_col(m1, m2)
            }
            ("sheet.mutation.remove-rows", "sheet.mutation.remove-rows") => {
                remove_rows::RemoveRowsTransform::default()
                    .transform_internal_with_remove_rows(m1, m2)
            }
            ("sheet.mutation.remove-rows", "sheet.mutation.remove-col") => {
                remove_rows::RemoveRowsTransform::default()
                    .transform_internal_with_remove_col(m1, m2)
            }
            ("sheet.mutation.remove-col", "sheet.mutation.set-range-values") => {
                remove_col::RemoveColTransform::default()
                    .transform_internal_with_set_range_values(m1, m2)
            }
            ("sheet.mutation.remove-col", "sheet.mutation.insert-row") => {
                remove_col::RemoveColTransform::default()
                    .transform_internal_with_insert_row(m1, m2)
            }
            ("sheet.mutation.remove-col", "sheet.mutation.insert-col") => {
                remove_col::RemoveColTransform::default()
                    .transform_internal_with_insert_col(m1, m2)
            }
            ("sheet.mutation.remove-col", "sheet.mutation.remove-rows") => {
                remove_col::RemoveColTransform::default()
                    .transform_internal_with_remove_rows(m1, m2)
            }
            ("sheet.mutation.remove-col", "sheet.mutation.remove-col") => {
                remove_col::RemoveColTransform::default()
                    .transform_internal_with_remove_col(m1, m2)
            }
            _ => TransformResultInternal {
                m1_prime: m1.clone(),
                m2_prime: m2.clone(),
                error: None,
            }
        }
    }
}
