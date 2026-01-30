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

// ========== SetArrayFormulaDataMutation ==========

// IArrayFormulaRangeType: [unitId: string]: Nullable<{ [sheetId: string]: IObjectMatrixPrimitiveType<IRange> }>;
// IArrayFormulaUnitCellType: [unitId: string]: Nullable<{ [sheetId: string]: IObjectMatrixPrimitiveType<Nullable<ICellData>> }>;
// IArrayFormulaEmbeddedMap: [unitId: string]: Nullable<{ [sheetId: string]: IObjectMatrixPrimitiveType<boolean> }>;
// Due to complex nested structures, using Value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetArrayFormulaDataMutationParams {
    pub array_formula_range: Value,      // IArrayFormulaRangeType
    pub array_formula_cell_data: Value,   // IArrayFormulaUnitCellType
    pub array_formula_embedded: Value,    // IArrayFormulaEmbeddedMap
}

pub struct SetArrayFormulaDataMutation;

impl SetArrayFormulaDataMutation {
    pub const ID: &'static str = "formula.mutation.set-array-formula-data";

    pub fn handler(_params: SetArrayFormulaDataMutationParams) -> Result<bool, String> {
        // TODO: Implement handler
        Ok(true)
    }
}
