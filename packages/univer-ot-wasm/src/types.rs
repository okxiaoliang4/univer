use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use js_sys::JSON;

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

// Specific mutation parameter types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetRangeValuesMutationParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,

    #[serde(rename = "cellValue", alias = "cell_value")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell_value: Option<ObjectMatrixPrimitiveType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertRowMutationParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,

    pub range: Range,

    #[serde(rename = "rowInfo", alias = "row_info")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_info: Option<ObjectArrayPrimitiveType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertColMutationParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,

    pub range: Range,

    #[serde(rename = "colInfo", alias = "col_info")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub col_info: Option<ObjectArrayPrimitiveType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveRowsMutationParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,

    pub range: Range,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveColMutationParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,

    pub range: Range,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub h: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hd: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub w: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hd: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectMatrixPrimitiveType {
    #[serde(flatten)]
    pub data: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectArrayPrimitiveType {
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
    pub m1_prime: MutationInfoInternal,
    pub m2_prime: MutationInfoInternal,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// WASM-bindgen exposed types (using JsValue)
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct TransformResult {
    #[wasm_bindgen(getter_with_clone)]
    pub m1_prime: MutationInfo,
    #[wasm_bindgen(getter_with_clone)]
    pub m2_prime: MutationInfo,
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
    pub fn new(m1_prime: MutationInfo, m2_prime: MutationInfo, error: Option<String>) -> Self {
        Self {
            m1_prime,
            m2_prime,
            error,
        }
    }
}

// Helper function to convert serde_json::Value to JsValue as plain object (not Map)
fn json_value_to_js_value(value: &serde_json::Value) -> JsValue {
    // Serialize to JSON string first, then parse it back to ensure plain objects
    match serde_json::to_string(value) {
        Ok(json_str) => {
            // Parse JSON string to get plain JavaScript object
            JSON::parse(&json_str).unwrap_or(JsValue::NULL)
        }
        Err(_) => JsValue::NULL,
    }
}

// Conversion helpers
impl From<TransformResultInternal> for TransformResult {
    fn from(internal: TransformResultInternal) -> Self {
        TransformResult {
            m1_prime: MutationInfo {
                id: internal.m1_prime.id,
                params: json_value_to_js_value(&internal.m1_prime.params),
            },
            m2_prime: MutationInfo {
                id: internal.m2_prime.id,
                params: json_value_to_js_value(&internal.m2_prime.params),
            },
            error: internal.error,
        }
    }
}

impl From<&MutationInfo> for MutationInfoInternal {
    fn from(info: &MutationInfo) -> Self {
        MutationInfoInternal {
            id: info.id.clone(),
            params: serde_wasm_bindgen::from_value(info.params.clone())
                .unwrap_or(serde_json::Value::Null),
        }
    }
}

// Error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformError {
    pub message: String,
    pub code: u32,
}