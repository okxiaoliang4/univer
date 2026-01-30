use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteWorksheetProtectionParams {
    pub unit_id: String,
    pub sub_unit_id: String,
}

pub struct DeleteWorksheetProtectionMutation;

impl DeleteWorksheetProtectionMutation {
    pub const ID: &'static str = "sheet.mutation.delete-worksheet-protection";

    pub fn handler(_params: DeleteWorksheetProtectionParams) -> Result<bool, String> {
        Ok(true)
    }
}
