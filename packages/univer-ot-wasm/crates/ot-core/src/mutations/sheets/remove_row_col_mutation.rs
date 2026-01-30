use serde::{Deserialize, Serialize};
use crate::types::IRange;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveRowsMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub range: IRange,
}

pub struct RemoveRowMutation;

impl RemoveRowMutation {
    pub const ID: &'static str = "sheet.mutation.remove-rows";

    pub fn handler(_params: RemoveRowsMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveColMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub range: IRange,
}

pub struct RemoveColMutation;

impl RemoveColMutation {
    pub const ID: &'static str = "sheet.mutation.remove-col";

    pub fn handler(_params: RemoveColMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
