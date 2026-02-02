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

use crate::IRange;

// ========== SetFeatureCalculationMutation ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IFeatureCalculationManagerParam {
  pub unit_id: String,
  pub sub_unit_id: String,
  pub dependency_ranges: Vec<IRange>,
  // getDirtyData
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetFeatureCalculationMutationParams {
  pub feature_id: String,
  pub calculation_param: IFeatureCalculationManagerParam,
}

pub struct SetFeatureCalculationMutation;

impl SetFeatureCalculationMutation {
    pub const ID: &'static str = "formula.mutation.set-feature-calculation";

    pub fn handler(_params: SetFeatureCalculationMutationParams) -> Result<bool, String> {
        // TODO: Implement handler
        Ok(true)
    }
}

// ========== RemoveFeatureCalculationMutation ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveFeatureCalculationMutationParams {
  pub feature_ids: Vec<String>,
  pub unit_id: String,
  pub sub_unit_id: String,
}

pub struct RemoveFeatureCalculationMutation;

impl RemoveFeatureCalculationMutation {
    pub const ID: &'static str = "formula.mutation.remove-feature-calculation";

    pub fn handler(_params: RemoveFeatureCalculationMutationParams) -> Result<bool, String> {
        // TODO: Implement handler
        Ok(true)
    }
}
