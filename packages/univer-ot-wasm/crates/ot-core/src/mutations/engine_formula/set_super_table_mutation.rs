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

// IRange is already defined in other modules, redefine here for independence
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

// TableOptionType enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TableOptionType {
    #[serde(rename = "#All")]
    All,
    #[serde(rename = "#Data")]
    Data,
    #[serde(rename = "#Headers")]
    Headers,
    #[serde(rename = "#Totals")]
    Totals,
    #[serde(rename = "#This Row")]
    ThisRow,
}

// ISuperTable: { sheetId: string; titleMap: Map<string, number>; range: IRange; }
// Note: TypeScript Map<string, number> serializes as object/HashMap
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ISuperTable {
    pub sheet_id: String,
    pub title_map: HashMap<String, i32>,
    pub range: IRange,
}

// ========== SetSuperTableMutation ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetSuperTableMutationSearchParam {
    pub unit_id: String,
    pub table_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetSuperTableMutationParam {
    pub unit_id: String,
    pub table_name: String,
    pub reference: ISuperTable,
}

pub struct SetSuperTableMutation;

impl SetSuperTableMutation {
    pub const ID: &'static str = "formula.mutation.set-super-table";

    pub fn handler(_params: SetSuperTableMutationParam) -> Result<bool, String> {
        // TODO: Implement handler
        Ok(true)
    }
}

// ========== RemoveSuperTableMutation ==========

pub struct RemoveSuperTableMutation;

impl RemoveSuperTableMutation {
    pub const ID: &'static str = "formula.mutation.remove-super-table";

    pub fn handler(_params: SetSuperTableMutationSearchParam) -> Result<bool, String> {
        // TODO: Implement handler
        Ok(true)
    }
}

// ========== SetSuperTableOptionMutation ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ISuperTableOptionParam {
    pub table_option: String,
    pub table_option_type: TableOptionType,
}

pub struct SetSuperTableOptionMutation;

impl SetSuperTableOptionMutation {
    pub const ID: &'static str = "formula.mutation.set-super-table-option";

    pub fn handler(_params: ISuperTableOptionParam) -> Result<bool, String> {
        // TODO: Implement handler
        Ok(true)
    }
}
