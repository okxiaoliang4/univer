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
use std::collections::HashMap;

// ========== Common Types ==========

// IRange is already defined in data_validation module, redefine here for independence
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

// IOtherFormulaDataItem: { f: string; ranges: IRange[]; }
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IOtherFormulaDataItem {
    pub f: String,
    pub ranges: Vec<IRange>,
}

// ========== SetOtherFormulaMutation ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetOtherFormulaMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub formula_map: HashMap<String, IOtherFormulaDataItem>,
}

pub struct SetOtherFormulaMutation;

impl SetOtherFormulaMutation {
    pub const ID: &'static str = "formula.mutation.set-other-formula";

    pub fn handler(_params: SetOtherFormulaMutationParams) -> Result<bool, String> {
        // TODO: Implement handler
        Ok(true)
    }
}

// ========== RemoveOtherFormulaMutation ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveOtherFormulaMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub formula_id_list: Vec<String>,
}

pub struct RemoveOtherFormulaMutation;

impl RemoveOtherFormulaMutation {
    pub const ID: &'static str = "formula.mutation.remove-other-formula";

    pub fn handler(_params: RemoveOtherFormulaMutationParams) -> Result<bool, String> {
        // TODO: Implement handler
        Ok(true)
    }
}
