use serde::{Deserialize, Serialize};
use crate::types::BooleanNumber;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetWorksheetHideMutationParams {
    pub hidden: BooleanNumber,
    pub unit_id: String,
    pub sub_unit_id: String,
}

pub struct SetWorksheetHideMutation;

impl SetWorksheetHideMutation {
    pub const ID: &'static str = "sheet.mutation.set-worksheet-hidden";

    pub fn handler(_params: SetWorksheetHideMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
