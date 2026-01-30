use serde::{Deserialize, Serialize};
use crate::types::IWorksheetProtectionRule;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddWorksheetProtectionParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub rule: IWorksheetProtectionRule,
}

pub struct AddWorksheetProtectionMutation;

impl AddWorksheetProtectionMutation {
    pub const ID: &'static str = "sheet.mutation.add-worksheet-protection";

    pub fn handler(_params: AddWorksheetProtectionParams) -> Result<bool, String> {
        Ok(true)
    }
}
