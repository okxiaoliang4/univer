use serde::{Deserialize, Serialize};
use super::types::IRangeThemeStyleJSON;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddRangeThemeMutationParams {
    pub style_json: IRangeThemeStyleJSON,
    pub unit_id: String,
    pub sub_unit_id: String,
}

pub struct AddRangeThemeMutation;

impl AddRangeThemeMutation {
    pub const ID: &'static str = "sheet.mutation.add-range-theme";

    pub fn handler(_params: AddRangeThemeMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
