use serde::{Deserialize, Serialize};
use crate::types::IRange;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteWorksheetRangeThemeStyleMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub range: IRange,
    pub theme_name: String,
}

pub struct DeleteWorksheetRangeThemeStyleMutation;

impl DeleteWorksheetRangeThemeStyleMutation {
    pub const ID: &'static str = "sheet.mutation.remove-worksheet-range-theme-style";

    pub fn handler(_params: DeleteWorksheetRangeThemeStyleMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
