use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetWorksheetNameMutationParams {
    pub name: String,
    pub unit_id: String,
    pub sub_unit_id: String,
}

pub struct SetWorksheetNameMutation;

impl SetWorksheetNameMutation {
    pub const ID: &'static str = "sheet.mutation.set-worksheet-name";

    pub fn handler(_params: SetWorksheetNameMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
