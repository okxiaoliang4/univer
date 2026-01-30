use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveRangeThemeMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub style_name: String,
}

pub struct RemoveRangeThemeMutation;

impl RemoveRangeThemeMutation {
    pub const ID: &'static str = "sheet.mutation.remove-range-theme";

    pub fn handler(_params: RemoveRangeThemeMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
