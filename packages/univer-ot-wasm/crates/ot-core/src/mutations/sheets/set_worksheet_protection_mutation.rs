use serde::{Deserialize, Serialize};
use crate::types::IWorksheetProtectionRule;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetWorksheetProtectionParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub rule: IWorksheetProtectionRule,
}

pub struct SetWorksheetProtectionMutation;

impl SetWorksheetProtectionMutation {
    pub const ID: &'static str = "sheet.mutation.set-worksheet-protection";

    pub fn handler(_params: SetWorksheetProtectionParams) -> Result<bool, String> {
        Ok(true)
    }
}
