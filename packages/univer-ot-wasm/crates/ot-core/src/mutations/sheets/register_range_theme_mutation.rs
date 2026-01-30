use serde::{Deserialize, Serialize};
use super::types::IRangeThemeStyleJSON;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterWorksheetRangeThemeStyleMutationParams {
    pub unit_id: String,
    pub theme_name: String,
    pub range_theme_style_json: IRangeThemeStyleJSON,
}

pub struct RegisterWorksheetRangeThemeStyleMutation;

impl RegisterWorksheetRangeThemeStyleMutation {
    pub const ID: &'static str = "sheet.mutation.register-worksheet-range-theme-style";

    pub fn handler(_params: RegisterWorksheetRangeThemeStyleMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
