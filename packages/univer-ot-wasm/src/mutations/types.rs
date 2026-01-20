use crate::types::{ObjectMatrixPrimitiveType, Range, SubUnitParams};
use serde::{Deserialize, Serialize};

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
