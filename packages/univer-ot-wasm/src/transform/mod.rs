use crate::types::{
    js_value_to_json_value, ComposeResult, MutationInfo, MutationInfoInternal, TransformListResult,
    TransformResult, TransformResultInternal,
};
use crate::{wasm_log_debug, wasm_log_error, wasm_log_info, wasm_log_warn};
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
#[cfg(test)]
mod list_transform_test;

use mutation_transform::MutationTransform;


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

    fn compose_dispatch(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> Vec<MutationInfoInternal>;
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
                m1_prime: Some(m1.clone()),
                m2_prime: Some(m2.clone()),
                error: None,
            },
        }
    }

    fn compose_dispatch(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> Vec<MutationInfoInternal> {
        self.compose(m1, m2)
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
        wasm_log_info!(
            "transform_list start m1_len={} m2_len={}",
            m1_list.len(),
            m2_list.len()
        );

        if m1_list.is_empty() {
            return (Vec::new(), m2_list.to_vec(), None);
        }

        if m2_list.is_empty() {
            return (m1_list.to_vec(), Vec::new(), None);
        }

        let mut current_m1_list: Vec<MutationInfoInternal> = m1_list.to_vec();
        let mut m2_primes: Vec<MutationInfoInternal> = Vec::with_capacity(m2_list.len());

        for (_m2_index, m2) in m2_list.iter().enumerate() {
            let mut current_m2: Option<MutationInfoInternal> = Some(m2.clone());
            let mut new_m1_list: Vec<MutationInfoInternal> =
                Vec::with_capacity(current_m1_list.len());

            wasm_log_debug!("transform_list m2 index={} id={}", _m2_index, m2.id);

            for (_m1_index, m1) in current_m1_list.iter().enumerate() {
                match current_m2.as_ref() {
                    Some(m2_value) => {
                        wasm_log_debug!(
                            "transform_list pair m1_index={} m2_index={} m1_id={} m2_id={} ",
                            _m1_index,
                            _m2_index,
                            m1.id,
                            m2_value.id
                        );

                        let result = self.transform(m1, m2_value);

                        if let Some(err) = result.error {
                            wasm_log_error!(
                                "transform_list error m1_id={} m2_id={} err={}",
                                m1.id,
                                m2_value.id,
                                err
                            );
                            return (Vec::new(), Vec::new(), Some(err));
                        }

                        if let Some(m1_prime) = result.m1_prime {
                            new_m1_list.push(m1_prime);
                        }
                        current_m2 = result.m2_prime;
                    }
                    None => {
                        new_m1_list.push(m1.clone());
                    }
                }
            }

            if let Some(m2_prime) = current_m2 {
                m2_primes.push(m2_prime);
            }
            current_m1_list = new_m1_list;
        }

        wasm_log_info!(
            "transform_list done m1_primes_len={} m2_primes_len={}",
            current_m1_list.len(),
            m2_primes.len()
        );
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
        wasm_log_debug!(
            "transform m1={:?} m2={:?}",
            m1,
            m2
        );
        if let Some(transform) = self.transforms.get(&m1.id) {
            return transform.dispatch(m1, m2);
        }

        // No algorithm found, return identity transform
        TransformResultInternal {
            m1_prime: Some(m1.clone()),
            m2_prime: Some(m2.clone()),
            error: None,
        }
    }

    pub fn compose(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> Vec<MutationInfoInternal> {
        if m1.id != m2.id {
            return vec![m1.clone(), m2.clone()];
        }

        if let Some(transform) = self.transforms.get(&m1.id) {
            return transform.compose_dispatch(m1, m2);
        }

        vec![m1.clone(), m2.clone()]
    }

    pub fn compose_list(&self, list: &[MutationInfoInternal]) -> Vec<MutationInfoInternal> {
        let mut result: Vec<MutationInfoInternal> = Vec::new();
        for mutation in list.iter() {
            let current = mutation.clone();
            if let Some(last) = result.pop() {
                let composed = self.compose(&last, &current);
                if composed.len() == 1 {
                    result.push(composed[0].clone());
                } else {
                    result.push(last);
                    result.push(current);
                }
            } else {
                result.push(current);
            }
        }
        result
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

        wasm_log_debug!(
            "transform wasm m1_id={} m2_id={} m1_params={} m2_params={}",
            m1_internal.id,
            m2_internal.id,
            m1_internal.params,
            m2_internal.params
        );

        // Call core transform
        let result = self.core.transform(&m1_internal, &m2_internal);

        wasm_log_debug!(
            "transform wasm result m1_prime_id={:?} m2_prime_id={:?} error={:?}",
            result.m1_prime.as_ref().map(|m| m.id.as_str()),
            result.m2_prime.as_ref().map(|m| m.id.as_str()),
            result.error
        );

        // Convert back to wasm types
        TransformResult::from(result)
    }

    fn js_value_to_internal_list(&self, list: &JsValue) -> Vec<MutationInfoInternal> {
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
                    wasm_log_warn!("transform_list drop item missing id/params");
                    None
                }
            } else {
                wasm_log_warn!("transform_list drop non-object item");
                None
            }
        };

        if let Some(array) = list.dyn_ref::<Array>() {
            (0..array.length())
                .filter_map(|i| js_to_internal(array.get(i)))
                .collect()
        } else {
            wasm_log_warn!("transform_list input is not array");
            Vec::new()
        }
    }

    /// Transform two lists of mutations - WASM boundary converts JsValue arrays <-> Vec<MutationInfoInternal>
    pub fn transform_list(&self, m1_list: &JsValue, m2_list: &JsValue) -> TransformListResult {
        let m1_internal_list = self.js_value_to_internal_list(m1_list);
        let m2_internal_list = self.js_value_to_internal_list(m2_list);

        wasm_log_info!(
            "transform_list wasm m1_len={} m2_len={}",
            m1_internal_list.len(),
            m2_internal_list.len()
        );

        // Call core transform_list
        let (m1_prime_list, m2_prime_list, error) = self
            .core
            .transform_list(&m1_internal_list, &m2_internal_list);

        wasm_log_info!(
            "transform_list wasm result m1_primes_len={} m2_primes_len={} error={:?}",
            m1_prime_list.len(),
            m2_prime_list.len(),
            error
        );

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

    /// Compose two mutations - WASM boundary converts JsValue <-> serde_json::Value
    pub fn compose(&self, m1: &MutationInfo, m2: &MutationInfo) -> ComposeResult {
        let m1_internal = MutationInfoInternal {
            id: m1.id.clone(),
            params: js_value_to_json_value(&m1.params),
        };
        let m2_internal = MutationInfoInternal {
            id: m2.id.clone(),
            params: js_value_to_json_value(&m2.params),
        };

        let result = self.core.compose(&m1_internal, &m2_internal);
        let mutations = result.into_iter().map(MutationInfo::from).collect();
        ComposeResult { mutations }
    }

    /// Compose list of mutations - WASM boundary converts JsValue arrays <-> Vec<MutationInfoInternal>
    pub fn compose_list(&self, list: &JsValue) -> ComposeResult {
        let internal_list = self.js_value_to_internal_list(list);
        let result = self.core.compose_list(&internal_list);
        let mutations = result.into_iter().map(MutationInfo::from).collect();
        ComposeResult { mutations }
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

    pub fn compose_internal(
        &self,
        m1: &MutationInfoInternal,
        m2: &MutationInfoInternal,
    ) -> Vec<MutationInfoInternal> {
        self.core.compose(m1, m2)
    }

    pub fn compose_list_internal(
        &self,
        list: &[MutationInfoInternal],
    ) -> Vec<MutationInfoInternal> {
        self.core.compose_list(list)
    }
}
