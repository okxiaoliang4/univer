use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ITableRange
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ITableRange {
    pub start_row: i32,
    pub start_column: i32,
    pub end_row: i32,
    pub end_column: i32,
}

// TableColumnDataTypeEnum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum TableColumnDataTypeEnum {
    String,
    Number,
    Boolean,
    Date,
}

// IStyleData - simplified for table columns
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IStyleData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bg: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cl: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bl: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub it: Option<i32>,
}

// ITableColumnJson
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ITableColumnJson {
    pub data_type: TableColumnDataTypeEnum,
    pub id: String,
    pub display_name: String,
    pub formula: String,
    pub meta: HashMap<String, serde_json::Value>,
    pub style: IStyleData,
}

// SheetsTableSortStateEnum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum SheetsTableSortStateEnum {
    Asc,
    Desc,
    None,
}

// TableColumnFilterTypeEnum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TableColumnFilterTypeEnum {
    Manual,
    Condition,
}

// TableConditionTypeEnum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum TableConditionTypeEnum {
    Date,
    String,
    Number,
    Logic,
}

// TableDateCompareTypeEnum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TableDateCompareTypeEnum {
    Equals,
    Before,
    After,
    Between,
}

// TableStringCompareTypeEnum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TableStringCompareTypeEnum {
    Contains,
    NotContains,
    Equals,
    NotEquals,
    BeginsWith,
    EndsWith,
}

// TableNumberCompareTypeEnum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TableNumberCompareTypeEnum {
    Equals,
    NotEquals,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Between,
}

// Filter info structures
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ITableDateFilterInfo {
    pub condition_type: TableConditionTypeEnum,
    pub compare_type: TableDateCompareTypeEnum,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_value: Option<serde_json::Value>, // Can be string, [string, string], Date, or [Date, Date]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor_time: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ITableStringFilterInfo {
    pub condition_type: TableConditionTypeEnum,
    pub compare_type: TableStringCompareTypeEnum,
    pub expected_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ITableNumberFilterInfo {
    pub condition_type: TableConditionTypeEnum,
    pub compare_type: TableNumberCompareTypeEnum,
    pub expected_value: serde_json::Value, // Can be number or [number, number]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ITableLogicFilterInfo {
    pub condition_type: TableConditionTypeEnum,
    pub compare_type: TableNumberCompareTypeEnum,
    pub expected_value: Vec<serde_json::Value>, // Array of filter infos
}

// ITableFilterItem (union type)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ITableManualFilterItem {
    pub filter_type: TableColumnFilterTypeEnum,
    pub values: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_all_selected: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ITableConditionFilterItem {
    pub filter_type: TableColumnFilterTypeEnum,
    pub filter_info: serde_json::Value, // Can be ITableDateFilterInfo, ITableStringFilterInfo, ITableNumberFilterInfo, or ITableLogicFilterInfo
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ITableFilterItem {
    Manual(ITableManualFilterItem),
    Condition(ITableConditionFilterItem),
}

// ITableFilterJSON
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ITableFilterJSON {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_column_filter_list: Option<Vec<Option<ITableFilterItem>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_sort_info: Option<ITableSortInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ITableSortInfo {
    pub column_index: i32,
    pub sort_state: SheetsTableSortStateEnum,
}

// ITableOptions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ITableOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_header: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_footer: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_style_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_total_row: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns: Option<Vec<ITableColumnJson>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<Vec<ITableFilterItem>>,
}

// IRangeOperationTypeEnum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum IRangeOperationTypeEnum {
    Insert,
    Delete,
}

// IRowColTypeEnum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum IRowColTypeEnum {
    Row,
    Column,
}

// ITableRangeRowColOperation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ITableRangeRowColOperation {
    pub operation_type: IRangeOperationTypeEnum,
    pub row_col_type: IRowColTypeEnum,
    pub index: i32,
    pub count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns_json: Option<Vec<ITableColumnJson>>,
}

// ITableRangeUpdate
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ITableRangeUpdate {
    pub new_range: ITableRange,
}

// ITableSetConfig
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ITableSetConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_range: Option<ITableRangeUpdate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_col_operation: Option<ITableRangeRowColOperation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ITableOptionsUpdate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ITableOptionsUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_header: Option<bool>,
}
