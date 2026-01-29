use wasm_bindgen::prelude::*;
use ot_core::TransformService as CoreTransformService;
use crate::types::{WasmMutationInfo, WasmTransformResult, WasmTransformListResult, WasmComposeResult};

/// WASM-exposed TransformService
///
/// This is a thin wrapper around ot-core::TransformService that handles
/// JsValue <-> serde_json::Value conversions at the WASM boundary.
#[wasm_bindgen]
pub struct TransformService {
    core_service: CoreTransformService,
}

#[wasm_bindgen]
impl TransformService {
    /// Create a new TransformService
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            core_service: CoreTransformService::new(),
        }
    }

    /// Transform two concurrent mutations
    ///
    /// # Arguments
    /// * `m1` - First mutation
    /// * `m2` - Second mutation
    ///
    /// # Returns
    /// TransformResult containing the transformed mutations
    #[wasm_bindgen]
    pub fn transform(&self, m1: &WasmMutationInfo, m2: &WasmMutationInfo) -> WasmTransformResult {
        // Convert WASM types to core types
        let core_m1: ot_core::MutationInfo = m1.into();
        let core_m2: ot_core::MutationInfo = m2.into();

        // Call core transform
        let core_result = self.core_service.transform(&core_m1, &core_m2);

        // Convert back to WASM types
        core_result.into()
    }

    /// Transform a list of mutations against another list
    ///
    /// # Arguments
    /// * `m1_list` - First list of mutations
    /// * `m2_list` - Second list of mutations
    ///
    /// # Returns
    /// TransformListResult containing both transformed lists
    #[wasm_bindgen(js_name = transformList)]
    pub fn transform_list(
        &self,
        m1_list: Vec<WasmMutationInfo>,
        m2_list: Vec<WasmMutationInfo>,
    ) -> WasmTransformListResult {
        // Convert WASM types to core types
        let core_m1_list: Vec<ot_core::MutationInfo> = m1_list
            .iter()
            .map(|m| m.into())
            .collect();

        let core_m2_list: Vec<ot_core::MutationInfo> = m2_list
            .iter()
            .map(|m| m.into())
            .collect();

        // Call core transform_list
        let (core_m1_prime, core_m2_prime, error) = self.core_service.transform_list(&core_m1_list, &core_m2_list);

        // Convert back to WASM types
        let wasm_m1_prime: Vec<WasmMutationInfo> = core_m1_prime
            .into_iter()
            .map(|m| m.into())
            .collect();

        let wasm_m2_prime: Vec<WasmMutationInfo> = core_m2_prime
            .into_iter()
            .map(|m| m.into())
            .collect();

        WasmTransformListResult::new(wasm_m1_prime, wasm_m2_prime, error)
    }

    /// Compose a list of mutations into an optimized list
    ///
    /// # Arguments
    /// * `mutations` - List of mutations to compose
    ///
    /// # Returns
    /// ComposeResult containing the composed mutations
    #[wasm_bindgen]
    pub fn compose(&self, mutations: Vec<WasmMutationInfo>) -> WasmComposeResult {
        // Convert WASM types to core types
        let core_mutations: Vec<ot_core::MutationInfo> = mutations
            .iter()
            .map(|m| m.into())
            .collect();

        // Call core compose
        let composed = self.core_service.compose(&core_mutations);

        // Convert back to WASM types
        let wasm_composed: Vec<WasmMutationInfo> = composed
            .into_iter()
            .map(|m| m.into())
            .collect();

        WasmComposeResult::new(wasm_composed)
    }

    /// Get the number of registered transforms (for debugging)
    #[wasm_bindgen(js_name = registrySize)]
    pub fn registry_size(&self) -> usize {
        self.core_service.registry_size()
    }
}

impl Default for TransformService {
    fn default() -> Self {
        Self::new()
    }
}
