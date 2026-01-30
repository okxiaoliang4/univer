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

// ========== OtherFormulaMarkDirtyMutation ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtherFormulaMarkDirtyParams {
    #[serde(flatten)]
    pub params: Value, // IOtherFormulaMarkDirtyParams
}

pub struct OtherFormulaMarkDirtyMutation;

impl OtherFormulaMarkDirtyMutation {
    pub const ID: &'static str = "sheet.mutation.data-validation-formula-mark-dirty";

    pub fn handler(_params: OtherFormulaMarkDirtyParams) -> Result<bool, String> {
        // TODO: Implement handler
        Ok(true)
    }
}
