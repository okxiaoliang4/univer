use serde::{Deserialize, Serialize};
use super::types::IAnchor;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveConditionalRuleMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub start: IAnchor,
    pub end: IAnchor,
}

pub struct MoveConditionalRuleMutation;

impl MoveConditionalRuleMutation {
    pub const ID: &'static str = "sheet.mutation.move-conditional-rule";

    pub fn handler(_params: MoveConditionalRuleMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
