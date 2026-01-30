use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::mutations::sheets::types::IRange;

// IPivotSortDirection
pub type IPivotSortDirection = String; // "asc" | "desc"

// AggregationType (from enum.ts)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum AggregationType {
    Sum,
    Count,
    Average,
    Max,
    Min,
    Product,
    CountNums,
    Stddev,
    Stddevp,
    Var,
    Varp,
}

// IPivotValueSortRule
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IPivotValueSortRule {
    #[serde(rename = "type")]
    pub sort_type: String, // "valueField"
    pub value_field_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<IPivotSortDirection>,
}

// IPivotFilterCriteria
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IPivotFilterCriteria {
    #[serde(rename = "type")]
    pub filter_type: String, // "value" | "condition"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator: Option<String>, // "equals" | "notEquals" | "greaterThan" | "lessThan" | "contains"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition_value: Option<serde_json::Value>, // Can be string or number
}

// IPivotField
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IPivotField {
    pub id: String,
    pub source_column_index: i32,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_rule: Option<IPivotValueSortRule>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregation: Option<AggregationType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<IPivotFilterCriteria>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_sub_totals: Option<bool>,
}

// IFieldsConfig
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IFieldsConfig {
    pub value_fields: Vec<IPivotField>,
    pub row_fields: Vec<IPivotField>,
    pub column_fields: Vec<IPivotField>,
    pub filter_fields: Vec<IPivotField>,
}

// ISourceRangeInfo
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ISourceRangeInfo {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub range: IRange,
}

// ITargetCellInfo
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ITargetCellInfo {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub row: i32,
    pub col: i32,
}

// IAxisItemType
pub type IAxisItemType = String; // "data" | "subtotal" | "grand"

// IAxisItem
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IAxisItem {
    pub headers: Vec<String>,
    pub display: Vec<String>,
    #[serde(rename = "type")]
    pub item_type: IAxisItemType,
    pub level: i32,
    pub field_index: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_field_index: Option<i32>,
    pub group_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children_keys: Option<Vec<String>>,
}

// IAxisModel
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IAxisModel {
    pub items: Vec<IAxisItem>,
    pub header_depth: i32,
    pub level_map: HashMap<String, Vec<i32>>,
    pub subtotal_map: HashMap<String, Vec<i32>>,
}

// IPivotModel
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IPivotModel {
    pub is_empty: bool,
    pub value_fields: Vec<IPivotValueField>,
    pub row_axis: IAxisModel,
    pub col_axis: IAxisModel,
    pub values: serde_json::Value, // IObjectMatrixPrimitiveType<IObjectArrayPrimitiveType<number | string | null>>
    pub dimensions: IPivotDimensions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IPivotValueField {
    pub id: String,
    pub name: String,
    pub agg: AggregationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IPivotDimensions {
    pub row_count: i32,
    pub col_count: i32,
    pub value_field_count: i32,
}
