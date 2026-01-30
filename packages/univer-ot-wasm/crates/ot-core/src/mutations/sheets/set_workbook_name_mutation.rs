use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetWorkbookNameMutationParams {
    pub name: String,
    pub unit_id: String,
}

pub struct SetWorkbookNameMutation;

impl SetWorkbookNameMutation {
    pub const ID: &'static str = "sheet.mutation.set-workbook-name";

    pub fn handler(_params: SetWorkbookNameMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
