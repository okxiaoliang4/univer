use crate::types::{
    js_value_to_json_value, MutationInfo, MutationInfoInternal, TransformListResult,
    TransformResult, TransformResultInternal,
};
use js_sys::{Array, Object, Reflect};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

pub mod insert_col;
pub mod insert_row;
pub mod mutation_transform;
pub mod remove_col;
pub mod remove_rows;
pub mod set_range_values;

#[cfg(test)]
mod test_utils;

#[cfg(test)]
mod insert_col_test;
#[cfg(test)]
mod insert_row_test;
#[cfg(test)]
mod remove_col_test;
#[cfg(test)]
mod remove_rows_test;
#[cfg(test)]
mod set_range_values_test;
// #[cfg(test)]
// mod list_transform_test;

use mutation_transform::MutationTransform;

pub(crate) const NOOP_MUTATION_ID: &str = "__noop__";

// Core transform service that works with internal types
pub struct TransformServiceCore {
    // Store transform instances by mutation ID
    transforms: HashMap<String, Box<dyn TransformDispatch>>,
}

trait TransformDispatch: Send + Sync {
    fn dispatch(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal;
}

impl<T: MutationTransform + Send + Sync + 'static> TransformDispatch for T {
    fn dispatch(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
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

    pub fn transform_list(
        &self,
        m1_list: &[MutationInfoInternal],
        m2_list: &[MutationInfoInternal],
    ) -> (
        Vec<MutationInfoInternal>,
        Vec<MutationInfoInternal>,
        Option<String>,
    ) {
        if m1_list.is_empty() {
            return (Vec::new(), m2_list.to_vec(), None);
        }

        if m2_list.is_empty() {
            return (m1_list.to_vec(), Vec::new(), None);
        }

        let mut current_m1_list: Vec<MutationInfoInternal> = m1_list.to_vec();
        let mut m2_primes: Vec<MutationInfoInternal> = Vec::with_capacity(m2_list.len());

        for m2 in m2_list.iter() {
            let mut current_m2 = m2.clone();
            let mut new_m1_list: Vec<MutationInfoInternal> =
                Vec::with_capacity(current_m1_list.len());

            for m1 in current_m1_list.iter() {
                let result = self.transform(m1, &current_m2);

                if let Some(err) = result.error {
                    return (Vec::new(), Vec::new(), Some(err));
                }

                new_m1_list.push(result.m1_prime);
                current_m2 = result.m2_prime;
            }

            m2_primes.push(current_m2);
            current_m1_list = new_m1_list;
        }

        (current_m1_list, m2_primes, None)
    }

    fn register_transform<T: MutationTransform + Send + Sync + 'static>(&mut self) {
        let mutation_id = T::mutation_id().to_string();
        self.transforms.insert(mutation_id, Box::new(T::default()));
    }

    pub fn transform(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
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

    /// Transform two lists of mutations - WASM boundary converts JsValue arrays <-> Vec<MutationInfoInternal>
    pub fn transform_list(&self, m1_list: &JsValue, m2_list: &JsValue) -> TransformListResult {
        // Helper function to convert JS object to MutationInfoInternal
        let js_to_internal = |item: JsValue| -> Option<MutationInfoInternal> {
            if let Some(obj) = item.dyn_ref::<Object>() {
                let id = Reflect::get(obj, &JsValue::from_str("id"))
                    .ok()
                    .and_then(|v| v.as_string());
                let params = Reflect::get(obj, &JsValue::from_str("params"))
                    .ok()
                    .map(|v| js_value_to_json_value(&v));

                if let (Some(id), Some(params)) = (id, params) {
                    Some(MutationInfoInternal { id, params })
                } else {
                    None
                }
            } else {
                None
            }
        };

        // Convert JS arrays to Vec<MutationInfoInternal>
        let m1_internal_list: Vec<MutationInfoInternal> =
            if let Some(array) = m1_list.dyn_ref::<Array>() {
                (0..array.length())
                    .filter_map(|i| js_to_internal(array.get(i)))
                    .collect()
            } else {
                Vec::new()
            };

        let m2_internal_list: Vec<MutationInfoInternal> =
            if let Some(array) = m2_list.dyn_ref::<Array>() {
                (0..array.length())
                    .filter_map(|i| js_to_internal(array.get(i)))
                    .collect()
            } else {
                Vec::new()
            };

        // Call core transform_list
        let (m1_prime_list, m2_prime_list, error) = self
            .core
            .transform_list(&m1_internal_list, &m2_internal_list);

        // Convert back to wasm types
        let m1_prime_wasm: Vec<MutationInfo> = m1_prime_list
            .into_iter()
            .map(|m| MutationInfo::from(m))
            .collect();

        let m2_prime_wasm: Vec<MutationInfo> = m2_prime_list
            .into_iter()
            .map(|m| MutationInfo::from(m))
            .collect();

        TransformListResult {
            m1_prime_list: m1_prime_wasm,
            m2_prime_list: m2_prime_wasm,
            error,
        }
    }
}

// Server/test implementation uses core service directly
#[cfg(any(test, feature = "server"))]
impl TransformService {
    pub fn transform_internal(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> TransformResultInternal {
        self.core.transform(m1, m2)
    }

    pub fn transform_list_internal(
        &self,
        m1_list: &[MutationInfoInternal],
        m2_list: &[MutationInfoInternal],
    ) -> (
        Vec<MutationInfoInternal>,
        Vec<MutationInfoInternal>,
        Option<String>,
    ) {
        self.core.transform_list(m1_list, m2_list)
    }
}
