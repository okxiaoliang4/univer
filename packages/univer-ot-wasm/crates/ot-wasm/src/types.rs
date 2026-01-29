use wasm_bindgen::prelude::*;
use ot_core::{MutationInfo, TransformResult};
use serde_wasm_bindgen::Serializer;
use serde::Serialize;

// ============================================================================
// WASM Boundary Types (using JsValue)
// ============================================================================

/// WASM-exposed MutationInfo with JsValue params
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmMutationInfo {
    #[wasm_bindgen(getter_with_clone)]
    pub id: String,
    #[wasm_bindgen(getter_with_clone)]
    pub params: JsValue,
}

#[wasm_bindgen]
impl WasmMutationInfo {
    #[wasm_bindgen(constructor)]
    pub fn new(id: String, params: JsValue) -> Self {
        Self { id, params }
    }
}

/// WASM-exposed TransformResult
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmTransformResult {
    #[wasm_bindgen(getter_with_clone)]
    pub m1_prime: Option<WasmMutationInfo>,
    #[wasm_bindgen(getter_with_clone)]
    pub m2_prime: Option<WasmMutationInfo>,
    #[wasm_bindgen(getter_with_clone)]
    pub error: Option<String>,
}

#[wasm_bindgen]
impl WasmTransformResult {
    #[wasm_bindgen(constructor)]
    pub fn new(
        m1_prime: Option<WasmMutationInfo>,
        m2_prime: Option<WasmMutationInfo>,
        error: Option<String>,
    ) -> Self {
        Self {
            m1_prime,
            m2_prime,
            error,
        }
    }
}

/// WASM-exposed list transform result
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmTransformListResult {
    #[wasm_bindgen(getter_with_clone)]
    pub m1_prime_list: Vec<WasmMutationInfo>,
    #[wasm_bindgen(getter_with_clone)]
    pub m2_prime_list: Vec<WasmMutationInfo>,
    #[wasm_bindgen(getter_with_clone)]
    pub error: Option<String>,
}

#[wasm_bindgen]
impl WasmTransformListResult {
    #[wasm_bindgen(constructor)]
    pub fn new(
        m1_prime_list: Vec<WasmMutationInfo>,
        m2_prime_list: Vec<WasmMutationInfo>,
        error: Option<String>,
    ) -> Self {
        Self {
            m1_prime_list,
            m2_prime_list,
            error,
        }
    }
}

/// WASM-exposed compose result
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmComposeResult {
    #[wasm_bindgen(getter_with_clone)]
    pub mutations: Vec<WasmMutationInfo>,
}

#[wasm_bindgen]
impl WasmComposeResult {
    #[wasm_bindgen(constructor)]
    pub fn new(mutations: Vec<WasmMutationInfo>) -> Self {
        Self { mutations }
    }
}

// ============================================================================
// Conversion Functions
// ============================================================================

/// Convert serde_json::Value to JsValue as plain object (not Map)
/// Uses json_compatible serializer to ensure plain JavaScript objects
pub fn json_value_to_js_value(value: &serde_json::Value) -> JsValue {
    let serializer = Serializer::json_compatible();
    value.serialize(&serializer).unwrap_or(JsValue::NULL)
}

/// Convert JsValue to serde_json::Value
/// Efficient conversion for wasm boundary using serde-wasm-bindgen
pub fn js_value_to_json_value(value: &JsValue) -> serde_json::Value {
    if value.is_null() || value.is_undefined() {
        return serde_json::Value::Null;
    }

    match serde_wasm_bindgen::from_value(value.clone()) {
        Ok(json) => json,
        Err(_) => {
            // Fallback: try JSON stringify/parse
            let json_string = js_sys::JSON::stringify(value)
                .ok()
                .and_then(|v| v.as_string());
            if let Some(json_string) = json_string {
                serde_json::from_str(&json_string).unwrap_or(serde_json::Value::Null)
            } else {
                serde_json::Value::Null
            }
        }
    }
}

// ============================================================================
// Type Conversions
// ============================================================================

impl From<MutationInfo> for WasmMutationInfo {
    fn from(info: MutationInfo) -> Self {
        WasmMutationInfo {
            id: info.id,
            params: json_value_to_js_value(&info.params),
        }
    }
}

impl From<&WasmMutationInfo> for MutationInfo {
    fn from(info: &WasmMutationInfo) -> Self {
        MutationInfo {
            id: info.id.clone(),
            params: js_value_to_json_value(&info.params),
        }
    }
}

impl From<TransformResult> for WasmTransformResult {
    fn from(result: TransformResult) -> Self {
        WasmTransformResult {
            m1_prime: result.m1_prime.map(|m| m.into()),
            m2_prime: result.m2_prime.map(|m| m.into()),
            error: result.error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_mutation_info_construction() {
        let info = WasmMutationInfo::new(
            "test.mutation".to_string(),
            JsValue::NULL,
        );
        assert_eq!(info.id, "test.mutation");
    }

    #[test]
    fn test_wasm_transform_result_construction() {
        let result = WasmTransformResult::new(None, None, None);
        assert!(result.m1_prime.is_none());
        assert!(result.m2_prime.is_none());
        assert!(result.error.is_none());
    }
}
