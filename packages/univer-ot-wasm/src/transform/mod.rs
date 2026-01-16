use crate::types::{MutationInfo, TransformResult, MutationInfoInternal, TransformResultInternal};
use crate::types::{js_value_to_json_value};
use std::collections::HashMap;
use std::sync::Arc;
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

// Core transform service that works with internal types
pub struct TransformServiceCore {
    // Store transform instances by mutation ID
    transforms: HashMap<String, Box<dyn TransformDispatch>>,
}

trait TransformDispatch: Send + Sync {
    fn dispatch(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal;
}

impl<T: MutationTransform + Send + Sync + 'static> TransformDispatch for T {
    fn dispatch(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
        match m2.id.as_str() {
            id if id == set_range_values::SetRangeValuesTransform::mutation_id() => {
                self.transform_with_set_range_values(m1, m2)
            }
            id if id == insert_row::InsertRowTransform::mutation_id() => {
                self.transform_with_insert_row(m1, m2)
            }
            id if id == insert_col::InsertColTransform::mutation_id() => {
                self.transform_with_insert_col(m1, m2)
            }
            id if id == remove_rows::RemoveRowsTransform::mutation_id() => {
                self.transform_with_remove_rows(m1, m2)
            }
            id if id == remove_col::RemoveColTransform::mutation_id() => {
                self.transform_with_remove_col(m1, m2)
            }
            _ => TransformResultInternal {
                m1_prime: m1.clone(),
                m2_prime: m2.clone(),
                error: None,
            },
        }
    }
}

impl TransformServiceCore {
    pub fn new() -> Self {
        let mut service = Self {
            transforms: HashMap::new(),
        };

        // Register all transforms - add new transforms here
        service.register_transform::<set_range_values::SetRangeValuesTransform>();
        service.register_transform::<insert_row::InsertRowTransform>();
        service.register_transform::<insert_col::InsertColTransform>();
        service.register_transform::<remove_rows::RemoveRowsTransform>();
        service.register_transform::<remove_col::RemoveColTransform>();

        service
    }

    fn register_transform<T: MutationTransform + Send + Sync + 'static>(&mut self) {
        let mutation_id = T::mutation_id().to_string();
        self.transforms.insert(mutation_id, Box::new(T::default()));
    }

    pub fn transform(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
        if let Some(transform) = self.transforms.get(&m1.id) {
            return transform.dispatch(m1, m2);
        }

        // No algorithm found, return identity transform
        TransformResultInternal {
            m1_prime: m1.clone(),
            m2_prime: m2.clone(),
            error: None,
        }
    }
}

// WASM wrapper that converts at the boundary
#[wasm_bindgen]
pub struct TransformService {
    core: TransformServiceCore,
}

#[wasm_bindgen]
impl TransformService {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            core: TransformServiceCore::new(),
        }
    }

    /// Transform two mutations - WASM boundary converts JsValue <-> serde_json::Value
    pub fn transform(&self, m1: &MutationInfo, m2: &MutationInfo) -> TransformResult {
        // Convert wasm types to core types
        let m1_internal = MutationInfoInternal {
            id: m1.id.clone(),
            params: js_value_to_json_value(&m1.params),
        };
        let m2_internal = MutationInfoInternal {
            id: m2.id.clone(),
            params: js_value_to_json_value(&m2.params),
        };

        // Call core transform
        let result = self.core.transform(&m1_internal, &m2_internal);

        // Convert back to wasm types
        TransformResult::from(result)
    }
}

// Server/test implementation uses core service directly
#[cfg(any(test, feature = "server"))]
impl TransformService {
    /// Internal method that works with MutationInfoInternal (for Rust unit tests and server)
    pub fn transform_internal(&self, m1: &MutationInfoInternal, m2: &MutationInfoInternal) -> TransformResultInternal {
        self.core.transform(m1, m2)
    }
}
