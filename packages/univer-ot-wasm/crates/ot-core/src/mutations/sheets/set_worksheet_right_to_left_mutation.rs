use serde::{Deserialize, Serialize};
use crate::types::BooleanNumber;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetWorksheetRightToLeftMutationParams {
    pub right_to_left: BooleanNumber,
    pub unit_id: String,
    pub sub_unit_id: String,
}

pub struct SetWorksheetRightToLeftMutation;

impl SetWorksheetRightToLeftMutation {
    pub const ID: &'static str = "sheet.mutation.set-worksheet-right-to-left";

    pub fn handler(_params: SetWorksheetRightToLeftMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
