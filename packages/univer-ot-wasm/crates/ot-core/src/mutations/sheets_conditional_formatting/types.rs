use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export IRange from sheets module
use crate::mutations::sheets::types::IRange;

// CFRuleType enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum CFRuleType {
    HighlightCell,
    DataBar,
    ColorScale,
    IconSet,
}

// CFSubRuleType enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum CFSubRuleType {
    UniqueValues,
    DuplicateValues,
    Rank,
    Text,
    TimePeriod,
    Number,
    Average,
    Formula,
}

// CFValueType enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum CFValueType {
    Num,
    Percent,
    Percentile,
    Formula,
    Min,
    Max,
}

// CFNumberOperator enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum CFNumberOperator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Between,
    NotBetween,
}

// CFTextOperator enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum CFTextOperator {
    ContainsText,
    NotContainsText,
    BeginsWith,
    EndsWith,
}

// CFTimePeriodOperator enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum CFTimePeriodOperator {
    Yesterday,
    Today,
    Tomorrow,
    Last7Days,
    LastWeek,
    ThisWeek,
    NextWeek,
    LastMonth,
    ThisMonth,
    NextMonth,
}

// IValueConfig
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IValueConfig {
    #[serde(rename = "type")]
    pub value_type: CFValueType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>, // Can be number or string
}

// IStyleBase
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IStyleBase {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bg: Option<IColorStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cl: Option<IColorStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bl: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub it: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ul: Option<ITextDecoration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub st: Option<ITextDecoration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IColorStyle {
    pub rgb: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ITextDecoration {
    pub s: i32,
}

// IconType
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IIconType {
    pub name: String,
    pub index: i32,
}

// Conditional formatting rule configs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IHighlightCell {
    #[serde(rename = "type")]
    pub rule_type: CFRuleType,
    pub sub_type: CFSubRuleType,
    pub style: IStyleBase,
    // Extended fields for specific subtypes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator: Option<serde_json::Value>, // Can be CFNumberOperator, CFTextOperator, or CFTimePeriodOperator
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>, // Can be string, number, or [number, number]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_bottom: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_percent: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IDataBar {
    #[serde(rename = "type")]
    pub rule_type: CFRuleType,
    pub is_show_value: bool,
    pub config: IDataBarConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IDataBarConfig {
    pub min: IValueConfig,
    pub max: IValueConfig,
    pub is_gradient: bool,
    pub positive_color: String,
    pub native_color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IColorScale {
    #[serde(rename = "type")]
    pub rule_type: CFRuleType,
    pub config: Vec<IColorScaleConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IColorScaleConfig {
    pub index: i32,
    pub color: String,
    pub value: IValueConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IIconSet {
    #[serde(rename = "type")]
    pub rule_type: CFRuleType,
    pub is_show_value: bool,
    pub config: Vec<IIconSetConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IIconSetConfig {
    pub operator: CFNumberOperator,
    pub value: IValueConfig,
    pub icon_type: IIconType,
    pub icon_id: String,
}

// Union type for all conditional formatting rules
// In Rust, we use an enum to represent the union type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IConditionalFormattingRuleConfig {
    HighlightCell(IHighlightCell),
    DataBar(IDataBar),
    ColorScale(IColorScale),
    IconSet(IIconSet),
}

// IConditionFormattingRule
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IConditionFormattingRule {
    pub ranges: Vec<IRange>,
    pub cf_id: String,
    pub stop_if_true: bool,
    pub rule: IConditionalFormattingRuleConfig,
}

// IAnchor for move operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IAnchor {
    pub id: String,
    #[serde(rename = "type")]
    pub anchor_type: AnchorType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AnchorType {
    Before,
    After,
    #[serde(rename = "self")]
    SelfAnchor,
}
