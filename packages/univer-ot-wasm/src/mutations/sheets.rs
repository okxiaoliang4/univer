use crate::types::{ObjectMatrixPrimitiveType, Range, SubUnitParams};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetMutationRangeParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,
    #[serde(rename = "range")]
    pub range: Range,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetMutationRangesParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,
    #[serde(rename = "ranges")]
    pub ranges: Vec<Range>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetRowColumnDataParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,
    #[serde(rename = "rowData", alias = "row_data")]
    pub row_data: Option<serde_json::Map<String, serde_json::Value>>,
    #[serde(rename = "columnData", alias = "column_data")]
    pub column_data: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetRowColCountsParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,
    #[serde(rename = "rowCount", alias = "row_count")]
    pub row_count: Option<u32>,
    #[serde(rename = "columnCount", alias = "column_count")]
    pub column_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetMoveRangeParams {
    #[serde(rename = "unitId", alias = "unit_id")]
    pub unit_id: String,
    #[serde(rename = "fromRange", alias = "from_range")]
    pub from_range: Range,
    #[serde(rename = "toRange", alias = "to_range")]
    pub to_range: Range,
    #[serde(rename = "from")]
    pub from: SheetMoveRangeSide,
    #[serde(rename = "to")]
    pub to: SheetMoveRangeSide,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetMoveRangeSide {
    #[serde(rename = "subUnitId", alias = "sub_unit_id")]
    pub sub_unit_id: String,
    pub value: ObjectMatrixPrimitiveType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetMoveRowsColsParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,
    #[serde(rename = "sourceRange", alias = "source_range")]
    pub source_range: Range,
    #[serde(rename = "targetRange", alias = "target_range")]
    pub target_range: Range,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetReorderRangeParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,
    #[serde(rename = "range")]
    pub range: Range,
    #[serde(rename = "order")]
    pub order: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetRangeThemeStyleParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,
    #[serde(rename = "range")]
    pub range: Range,
    #[serde(rename = "themeName", alias = "theme_name")]
    pub theme_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetWorksheetMergeParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,
    #[serde(rename = "ranges")]
    pub ranges: Vec<Range>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetSetFrozenParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,
    #[serde(rename = "startRow", alias = "start_row")]
    pub start_row: u32,
    #[serde(rename = "startColumn", alias = "start_column")]
    pub start_column: u32,
    #[serde(rename = "ySplit", alias = "y_split")]
    pub y_split: u32,
    #[serde(rename = "xSplit", alias = "x_split")]
    pub x_split: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetWorksheetRangeThemeStyleParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,
    pub range: Range,
    #[serde(rename = "themeName", alias = "theme_name")]
    pub theme_name: String,
}
