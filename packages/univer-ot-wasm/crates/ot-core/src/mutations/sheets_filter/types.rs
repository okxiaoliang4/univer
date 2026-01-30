use serde::{Deserialize, Serialize};

// Re-use IRange from sheets module
use crate::mutations::sheets::types::IRange;

// CustomFilterOperator enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum CustomFilterOperator {
    Equal,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    NotEqual,
}

// IFilters - basic filters (filter by value)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blank: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<Vec<String>>,
}

// IColorFilters - color filters
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IColorFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell_fill_colors: Option<Vec<Option<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell_text_colors: Option<Vec<String>>,
}

// ICustomFilter - a single custom filter
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ICustomFilter {
    pub val: serde_json::Value, // Can be string or number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator: Option<CustomFilterOperator>,
}

// ICustomFilters - custom filters with logical operator
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ICustomFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub and: Option<i32>, // BooleanNumber.TRUE = 1
    pub custom_filters: Vec<ICustomFilter>, // Can have 1 or 2 elements
}

// IFilterColumn - the filter criteria of a column
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IFilterColumn {
    pub col_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<IFilters>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_filters: Option<IColorFilters>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_filters: Option<ICustomFilters>,
}
