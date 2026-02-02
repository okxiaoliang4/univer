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

// ========== SetFormulaDataMutation ==========

// IFormulaData: [unitId: string]: Nullable<{ [sheetId: string]: Nullable<IObjectMatrixPrimitiveType<Nullable<IFormulaDataItem>>> }>;
// Due to the complex nested structure with IObjectMatrixPrimitiveType and ICellData, using Value
// IFormulaDataItem: { f: string; x?: number; y?: number; si?: string; }
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetFormulaDataMutationParams {
    #[serde(flatten)]
    pub params: Value, // ISetFormulaDataMutationParams
}

pub struct SetFormulaDataMutation;

// NOTE: onlyLocal 所以这个不需要做转换处理，本身协同就不会遇到这个mutation
impl SetFormulaDataMutation {
    pub const ID: &'static str = "formula.mutation.set-formula-data";

    pub fn handler(_params: SetFormulaDataMutationParams) -> Result<bool, String> {
        // TODO: Implement handler
        Ok(true)
    }
}
