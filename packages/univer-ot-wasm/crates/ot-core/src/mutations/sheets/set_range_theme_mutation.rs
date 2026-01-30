use serde::{Deserialize, Serialize};
use super::types::PartialRangeThemeStyle;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetRangeThemeMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub style_name: String,
    pub style: PartialRangeThemeStyle,
}

pub struct SetRangeThemeMutation;

impl SetRangeThemeMutation {
    pub const ID: &'static str = "sheet.mutation.set-range-theme";

    pub fn handler(_params: SetRangeThemeMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
