use serde::{Deserialize, Serialize};
use crate::types::IRange;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetRowVisibleMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub ranges: Vec<IRange>,
}

pub struct SetRowVisibleMutation;

impl SetRowVisibleMutation {
    pub const ID: &'static str = "sheet.mutation.set-row-visible";

    pub fn handler(_params: SetRowVisibleMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetRowHiddenMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub ranges: Vec<IRange>,
}

pub struct SetRowHiddenMutation;

impl SetRowHiddenMutation {
    pub const ID: &'static str = "sheet.mutation.set-row-hidden";

    pub fn handler(_params: SetRowHiddenMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
