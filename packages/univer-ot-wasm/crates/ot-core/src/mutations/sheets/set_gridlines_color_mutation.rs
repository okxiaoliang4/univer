use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetGridlinesColorMutationParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    pub unit_id: String,
    pub sub_unit_id: String,
}

pub struct SetGridlinesColorMutation;

impl SetGridlinesColorMutation {
    pub const ID: &'static str = "sheet.mutation.set-gridlines-color";

    pub fn handler(_params: SetGridlinesColorMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
