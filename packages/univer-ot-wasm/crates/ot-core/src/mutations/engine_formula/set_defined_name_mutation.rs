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

// ========== SetDefinedNameMutation ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetDefinedNameMutationSearchParam {
    pub unit_id: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetDefinedNameMutationParam {
    pub unit_id: String,
    pub id: String,
    pub name: String,
    pub formula_or_ref_string: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_sheet_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formula_or_ref_string_with_prefix: Option<String>,
}

pub struct SetDefinedNameMutation;

impl SetDefinedNameMutation {
    pub const ID: &'static str = "formula.mutation.set-defined-name";

    pub fn handler(_params: SetDefinedNameMutationParam) -> Result<bool, String> {
        // TODO: Implement handler
        Ok(true)
    }
}

// ========== RemoveDefinedNameMutation ==========

pub struct RemoveDefinedNameMutation;

impl RemoveDefinedNameMutation {
    pub const ID: &'static str = "formula.mutation.remove-defined-name";

    pub fn handler(_params: SetDefinedNameMutationSearchParam) -> Result<bool, String> {
        // TODO: Implement handler
        Ok(true)
    }
}
