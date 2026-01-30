use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnregisterWorksheetRangeThemeStyleMutationParams {
    pub unit_id: String,
    pub theme_name: String,
}

pub struct UnregisterWorksheetRangeThemeStyleMutation;

impl UnregisterWorksheetRangeThemeStyleMutation {
    pub const ID: &'static str = "sheet.mutation.unregister-worksheet-range-theme-style";

    pub fn handler(_params: UnregisterWorksheetRangeThemeStyleMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
