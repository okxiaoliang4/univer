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
use serde_json::Value;
use std::collections::HashMap;

// ========== Common Types ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RangeDirection {
    None,
    Backward,
    Forward,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DocRangeType {
    Rect,
    Text,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ITextRange {
    pub start_offset: i32,
    pub end_offset: i32,
    pub collapsed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<RangeDirection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ITextRangeParam {
    pub start_offset: i32,
    pub end_offset: i32,
    pub collapsed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<RangeDirection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segment_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segment_page: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range_type: Option<DocRangeType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct INodeSearch {
    pub glyph: i32,
    pub divide: i32,
    pub line: i32,
    pub column: i32,
    pub section: i32,
    pub page: i32,
    pub segment_page: i32,
    pub page_type: i32, // DocumentSkeletonPageType
    pub path: Vec<Value>, // (string | number)[]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct INodePosition {
    pub glyph: i32,
    pub divide: i32,
    pub line: i32,
    pub column: i32,
    pub section: i32,
    pub page: i32,
    pub segment_page: i32,
    pub page_type: i32,
    pub path: Vec<Value>,
    pub is_back: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ITextSelectionStyle {
    pub stroke_width: f64,
    pub stroke: String,
    pub stroke_active: String,
    pub fill: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ITextRangeWithStyle {
    pub start_offset: i32,
    pub end_offset: i32,
    pub collapsed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<RangeDirection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segment_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segment_page: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range_type: Option<DocRangeType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_node_position: Option<INodePosition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_node_position: Option<INodePosition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<ITextSelectionStyle>,
}

// ========== RichTextEditingMutation ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RichTextEditingMutationParams {
    pub unit_id: String,
    pub actions: Value, // JSONXActions - complex CRDT type, keep as Value
    pub text_ranges: Option<Vec<ITextRangeWithStyle>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segment_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_text_ranges: Option<Vec<ITextRangeWithStyle>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_need_set_text_range: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_composition_end: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_history: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debounce: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<HashMap<String, bool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_sync: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_editing: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub syncer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<String>, // From IMutationCommonParams
}

pub struct RichTextEditingMutation;

impl RichTextEditingMutation {
    pub const ID: &'static str = "doc.mutation.rich-text-editing";

    pub fn handler(
        _params: RichTextEditingMutationParams,
    ) -> Result<RichTextEditingMutationParams, String> {
        // TODO: Implement handler
        // Returns undo mutation params
        Err("Not implemented".to_string())
    }
}
