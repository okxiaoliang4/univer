use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetWorksheetColumnCountMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub column_count: u32,
}

pub struct SetWorksheetColumnCountMutation;

impl SetWorksheetColumnCountMutation {
    pub const ID: &'static str = "sheet.mutation.set-worksheet-column-count";

    pub fn handler(_params: SetWorksheetColumnCountMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
