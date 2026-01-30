use serde::{Deserialize, Serialize};
use crate::types::BooleanNumber;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToggleGridlinesMutationParams {
    pub show_gridlines: BooleanNumber,
    pub unit_id: String,
    pub sub_unit_id: String,
}

pub struct ToggleGridlinesMutation;

impl ToggleGridlinesMutation {
    pub const ID: &'static str = "sheet.mutation.toggle-gridlines";

    pub fn handler(_params: ToggleGridlinesMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
