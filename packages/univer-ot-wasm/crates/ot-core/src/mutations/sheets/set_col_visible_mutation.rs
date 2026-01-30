use serde::{Deserialize, Serialize};
use crate::types::IRange;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetColHiddenMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub ranges: Vec<IRange>,
}

pub struct SetColHiddenMutation;

impl SetColHiddenMutation {
    pub const ID: &'static str = "sheet.mutation.set-col-hidden";

    pub fn handler(_params: SetColHiddenMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetColVisibleMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub ranges: Vec<IRange>,
}

pub struct SetColVisibleMutation;

impl SetColVisibleMutation {
    pub const ID: &'static str = "sheet.mutation.set-col-visible";

    pub fn handler(_params: SetColVisibleMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
