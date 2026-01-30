// Copyright 2023-present DreamNum Co., Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use serde::{Deserialize, Serialize};

// ========== Common Types ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IRange {
    pub start_row: i32,
    pub start_column: i32,
    pub end_row: i32,
    pub end_column: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range_type: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_absolute_ref_type: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_absolute_ref_type: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataValidationType {
    Custom,
    List,
    #[serde(rename = "listMultiple")]
    ListMultiple,
    None,
    #[serde(rename = "textLength")]
    TextLength,
    Date,
    Time,
    Whole,
    Decimal,
    Checkbox,
    Any,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DataValidationOperator {
    Between,
    Equal,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    NotBetween,
    NotEqual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataValidationErrorStyle {
    #[serde(rename = "0")]
    Info,
    #[serde(rename = "1")]
    Stop,
    #[serde(rename = "2")]
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataValidationImeMode {
    #[serde(rename = "0")]
    Disabled,
    #[serde(rename = "1")]
    FullAlpha,
    #[serde(rename = "2")]
    FullHangul,
    #[serde(rename = "3")]
    FullKatakana,
    #[serde(rename = "4")]
    HalfAlpha,
    #[serde(rename = "5")]
    HalfHangul,
    #[serde(rename = "6")]
    HalfKatakana,
    #[serde(rename = "7")]
    Hiragana,
    #[serde(rename = "8")]
    NoControl,
    #[serde(rename = "9")]
    Off,
    #[serde(rename = "10")]
    On,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataValidationRenderMode {
    #[serde(rename = "0")]
    Text,
    #[serde(rename = "1")]
    Arrow,
    #[serde(rename = "2")]
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BizInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_time: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IDataValidationRule {
    pub uid: String,
    pub ranges: Vec<IRange>,
    #[serde(rename = "type")]
    pub validation_type: String, // DataValidationType or custom string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_blank: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formula1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formula2: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator: Option<DataValidationOperator>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ime_mode: Option<DataValidationImeMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_drop_down: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_error_message: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_style: Option<DataValidationErrorStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_input_message: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub render_mode: Option<DataValidationRenderMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biz_info: Option<BizInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RuleOrRules {
    Single(IDataValidationRule),
    Multiple(Vec<IDataValidationRule>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RuleIdOrIds {
    Single(String),
    Multiple(Vec<String>),
}

// DataValidationChangeSource: 'command' | 'patched'
pub type DataValidationChangeSource = String;

// UpdateRuleType enum: SETTING = 0, RANGE = 1, OPTIONS = 2, ALL = 3
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IDataValidationRuleBase {
    #[serde(rename = "type")]
    pub validation_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_blank: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formula1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formula2: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator: Option<DataValidationOperator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IDataValidationRuleOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ime_mode: Option<DataValidationImeMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_drop_down: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_error_message: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_style: Option<DataValidationErrorStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_input_message: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub render_mode: Option<DataValidationRenderMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biz_info: Option<BizInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IDataValidationRuleWithoutUid {
    pub ranges: Vec<IRange>,
    #[serde(rename = "type")]
    pub validation_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_blank: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formula1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formula2: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator: Option<DataValidationOperator>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ime_mode: Option<DataValidationImeMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_drop_down: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_error_message: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_style: Option<DataValidationErrorStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_input_message: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub render_mode: Option<DataValidationRenderMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biz_info: Option<BizInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IUpdateRulePayload {
    Range {
        payload: Vec<IRange>,
    },
    Setting {
        payload: IDataValidationRuleBase,
    },
    Options {
        payload: IDataValidationRuleOptions,
    },
    All {
        payload: IDataValidationRuleWithoutUid,
    },
}

// ========== AddDataValidationMutation ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddDataValidationMutationParams {
    pub rule: RuleOrRules,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<DataValidationChangeSource>,
    pub unit_id: String,
    pub sub_unit_id: String,
}

pub struct AddDataValidationMutation;

impl AddDataValidationMutation {
    pub const ID: &'static str = "data-validation.mutation.addRule";

    pub fn handler(_params: AddDataValidationMutationParams) -> Result<bool, String> {
        // TODO: Implement handler
        Ok(true)
    }
}

// ========== RemoveDataValidationMutation ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveDataValidationMutationParams {
    pub rule_id: RuleIdOrIds,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<DataValidationChangeSource>,
    pub unit_id: String,
    pub sub_unit_id: String,
}

pub struct RemoveDataValidationMutation;

impl RemoveDataValidationMutation {
    pub const ID: &'static str = "data-validation.mutation.removeRule";

    pub fn handler(_params: RemoveDataValidationMutationParams) -> Result<bool, String> {
        // TODO: Implement handler
        Ok(true)
    }
}

// ========== UpdateDataValidationMutation ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDataValidationMutationParams {
    pub payload: IUpdateRulePayload,
    pub rule_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<DataValidationChangeSource>,
    pub unit_id: String,
    pub sub_unit_id: String,
}

pub struct UpdateDataValidationMutation;

impl UpdateDataValidationMutation {
    pub const ID: &'static str = "data-validation.mutation.updateRule";

    pub fn handler(_params: UpdateDataValidationMutationParams) -> Result<bool, String> {
        // TODO: Implement handler
        Ok(true)
    }
}
