use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::prelude::*;

#[allow(unused_imports)]
pub use crate::mutations::types::{
    ColumnData, InsertColMutationParams, InsertRowMutationParams, RemoveColMutationParams,
    RemoveRowsMutationParams, RowData, SetRangeValuesMutationParams,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v: Option<CellValue>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub t: Option<CellValueType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub f: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub s: Option<StyleData>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub p: Option<DocumentData>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub si: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<CustomData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CellValue {
    String(String),
    Number(f64),
    Boolean(bool),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CellValueType {
    String = 1,
    Number = 2,
    Boolean = 3,
    ForceString = 4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ff: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub fs: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub b: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub i: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cl: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomData {
    #[serde(flatten)]
    pub data: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range {
    #[serde(rename = "startRow", alias = "start_row")]
    pub start_row: u32,
    #[serde(rename = "startColumn", alias = "start_column")]
    pub start_column: u32,
    #[serde(rename = "endRow", alias = "end_row")]
    pub end_row: u32,
    #[serde(rename = "endColumn", alias = "end_column")]
    pub end_column: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitParams {
    #[serde(rename = "unitId", alias = "unit_id")]
    pub unit_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubUnitParams {
    #[serde(rename = "unitId", alias = "unit_id")]
    pub unit_id: String,
    #[serde(rename = "subUnitId", alias = "sub_unit_id")]
    pub sub_unit_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectMatrixPrimitiveType {
    #[serde(flatten)]
    pub data: serde_json::Map<String, serde_json::Value>,
}

// Internal types for Rust code (using serde_json::Value)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationInfoInternal {
    pub id: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformResultInternal {
    pub m1_prime: Option<MutationInfoInternal>,
    pub m2_prime: Option<MutationInfoInternal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// WASM-bindgen exposed types (using JsValue)
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct TransformResult {
    #[wasm_bindgen(getter_with_clone)]
    pub m1_prime: Option<MutationInfo>,
    #[wasm_bindgen(getter_with_clone)]
    pub m2_prime: Option<MutationInfo>,
    #[wasm_bindgen(getter_with_clone)]
    pub error: Option<String>,
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct MutationInfo {
    #[wasm_bindgen(getter_with_clone)]
    pub id: String,
    #[wasm_bindgen(getter_with_clone)]
    pub params: JsValue,
}

#[wasm_bindgen]
impl MutationInfo {
    #[wasm_bindgen(constructor)]
    pub fn new(id: String, params: JsValue) -> Self {
        Self { id, params }
    }
}

#[wasm_bindgen]
impl TransformResult {
    #[wasm_bindgen(constructor)]
    pub fn new(
        m1_prime: Option<MutationInfo>,
        m2_prime: Option<MutationInfo>,
        error: Option<String>,
    ) -> Self {
        Self {
            m1_prime,
            m2_prime,
            error,
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct TransformListResult {
    #[wasm_bindgen(getter_with_clone)]
    pub m1_prime_list: Vec<MutationInfo>,
    #[wasm_bindgen(getter_with_clone)]
    pub m2_prime_list: Vec<MutationInfo>,
    #[wasm_bindgen(getter_with_clone)]
    pub error: Option<String>,
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct ComposeResult {
    #[wasm_bindgen(getter_with_clone)]
    pub mutations: Vec<MutationInfo>,
}

#[wasm_bindgen]
impl TransformListResult {
    #[wasm_bindgen(constructor)]
    pub fn new(
        m1_prime_list: Vec<MutationInfo>,
        m2_prime_list: Vec<MutationInfo>,
        error: Option<String>,
    ) -> Self {
        Self {
            m1_prime_list,
            m2_prime_list,
            error,
        }
    }
}

#[wasm_bindgen]
impl ComposeResult {
    #[wasm_bindgen(constructor)]
    pub fn new(mutations: Vec<MutationInfo>) -> Self {
        Self { mutations }
    }
}

/// Convert serde_json::Value to JsValue as plain object (not Map)
/// Uses json_compatible serializer to ensure plain JavaScript objects instead of Map objects
/// This is more efficient than JSON string roundtrips
pub fn json_value_to_js_value(value: &serde_json::Value) -> JsValue {
    // Use json_compatible serializer to avoid Map objects
    // This ensures HashMap<String, ...> serializes as plain objects, not JavaScript Maps
    let serializer = Serializer::json_compatible();
    value.serialize(&serializer).unwrap_or(JsValue::NULL)
}

/// Convert JsValue to serde_json::Value
/// Efficient conversion for wasm boundary using serde-wasm-bindgen
/// Falls back to JSON stringify/parse to handle undefined values or non-plain objects.
pub fn js_value_to_json_value(value: &JsValue) -> serde_json::Value {
    if value.is_null() || value.is_undefined() {
        return serde_json::Value::Null;
    }

    match serde_wasm_bindgen::from_value(value.clone()) {
        Ok(json) => json,
        Err(_) => {
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

/// Convert any Serialize type to JsValue as plain object (not Map)
/// Uses serde-wasm-bindgen directly with json_compatible serializer for better performance
/// This ensures WASM returns plain JavaScript objects instead of Map objects
/// which can cause serialization issues on the JavaScript side
///
/// # Why this is needed:
/// By default, `serde_wasm_bindgen::to_value` may return Map objects for nested structures,
/// which fail to serialize properly with JSON.stringify() in JavaScript (becomes {})
/// Using `Serializer::json_compatible()` ensures plain objects while maintaining performance
pub fn to_plain_js_value<T: Serialize>(value: &T) -> JsValue {
    // Use serde-wasm-bindgen directly with json_compatible serializer
    // This is more efficient than going through serde_json::Value first
    let serializer = Serializer::json_compatible();
    value.serialize(&serializer).unwrap_or_else(|_| {
        // Return error as JsValue string if serialization fails
        JsValue::from_str(&format!("Serialization error"))
    })
}

// Conversion helpers
impl From<TransformResultInternal> for TransformResult {
    fn from(internal: TransformResultInternal) -> Self {
        TransformResult {
            m1_prime: internal.m1_prime.map(|mutation| MutationInfo {
                id: mutation.id,
                params: json_value_to_js_value(&mutation.params),
            }),
            m2_prime: internal.m2_prime.map(|mutation| MutationInfo {
                id: mutation.id,
                params: json_value_to_js_value(&mutation.params),
            }),
            error: internal.error,
        }
    }
}

impl From<&MutationInfo> for MutationInfoInternal {
    fn from(info: &MutationInfo) -> Self {
        MutationInfoInternal {
            id: info.id.clone(),
            params: js_value_to_json_value(&info.params),
        }
    }
}

impl From<MutationInfoInternal> for MutationInfo {
    fn from(internal: MutationInfoInternal) -> Self {
        MutationInfo {
            id: internal.id,
            params: json_value_to_js_value(&internal.params),
        }
    }
}

// Error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformError {
    pub message: String,
    pub code: u32,
}
