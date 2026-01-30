use serde::{Deserialize, Serialize};
use crate::types::IRange;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveRowsMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub source_range: IRange,
    pub target_range: IRange,
}

pub struct MoveRowsMutation;

impl MoveRowsMutation {
    pub const ID: &'static str = "sheet.mutation.move-rows";

    pub fn handler(_params: MoveRowsMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveColumnsMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub source_range: IRange,
    pub target_range: IRange,
}

pub struct MoveColsMutation;

impl MoveColsMutation {
    pub const ID: &'static str = "sheet.mutation.move-columns";

    pub fn handler(_params: MoveColumnsMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
