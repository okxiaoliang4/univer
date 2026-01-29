use crate::types::{ObjectMatrixPrimitiveType, Range, SubUnitParams};
use serde::{Deserialize, Serialize};

// ============================================================================
// Core Mutation Parameters
// ============================================================================

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
    pub row_info: Option<Vec<RowData>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertColMutationParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,

    pub range: Range,

    #[serde(rename = "colInfo", alias = "col_info")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub col_info: Option<Vec<ColumnData>>,
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

// ============================================================================
// Move Operations
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveRowsMutationParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,

    #[serde(rename = "sourceRange", alias = "source_range")]
    pub source_range: Range,

    #[serde(rename = "targetRange", alias = "target_range")]
    pub target_range: Range,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveColsMutationParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,

    #[serde(rename = "sourceRange", alias = "source_range")]
    pub source_range: Range,

    #[serde(rename = "targetRange", alias = "target_range")]
    pub target_range: Range,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveRangeMutationParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,

    #[serde(rename = "fromRange", alias = "from_range")]
    pub from_range: Range,

    #[serde(rename = "toRange", alias = "to_range")]
    pub to_range: Range,
}

// ============================================================================
// Formatting Operations
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetFrozenMutationParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,

    #[serde(rename = "startRow", alias = "start_row")]
    pub start_row: i32,

    #[serde(rename = "startColumn", alias = "start_column")]
    pub start_column: i32,

    #[serde(rename = "ySplit", alias = "y_split")]
    pub y_split: i32,

    #[serde(rename = "xSplit", alias = "x_split")]
    pub x_split: i32,
}

// ============================================================================
// Worksheet/Workbook Operations
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertSheetMutationParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,

    pub index: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sheet: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveSheetMutationParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,
}
