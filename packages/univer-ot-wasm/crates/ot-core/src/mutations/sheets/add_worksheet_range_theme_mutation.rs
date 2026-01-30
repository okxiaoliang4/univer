use serde::{Deserialize, Serialize};
use crate::types::IRange;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorksheetRangeThemeStyleMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub range: IRange,
    pub theme_name: String,
}

pub struct SetWorksheetRangeThemeStyleMutation;

impl SetWorksheetRangeThemeStyleMutation {
    pub const ID: &'static str = "sheet.mutation.set-worksheet-range-theme-style";

    pub fn handler(_params: WorksheetRangeThemeStyleMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
