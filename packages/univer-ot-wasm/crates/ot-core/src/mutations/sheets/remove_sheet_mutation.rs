use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveSheetMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_unit_name: Option<String>,
}

pub struct RemoveSheetMutation;

impl RemoveSheetMutation {
    pub const ID: &'static str = "sheet.mutation.remove-sheet";

    pub fn handler(_params: RemoveSheetMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
