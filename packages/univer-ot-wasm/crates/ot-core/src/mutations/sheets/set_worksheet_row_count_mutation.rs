use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetWorksheetRowCountMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub row_count: u32,
}

pub struct SetWorksheetRowCountMutation;

impl SetWorksheetRowCountMutation {
    pub const ID: &'static str = "sheet.mutation.set-worksheet-row-count";

    pub fn handler(_params: SetWorksheetRowCountMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
