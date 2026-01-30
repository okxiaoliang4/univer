use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyWorksheetEndMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
}

pub struct CopyWorksheetEndMutation;

impl CopyWorksheetEndMutation {
    pub const ID: &'static str = "sheet.mutation.copy-worksheet-end";

    pub fn handler(_params: CopyWorksheetEndMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
