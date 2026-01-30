use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetTabColorMutationParams {
    pub color: String,
    pub unit_id: String,
    pub sub_unit_id: String,
}

pub struct SetTabColorMutation;

impl SetTabColorMutation {
    pub const ID: &'static str = "sheet.mutation.set-tab-color";

    pub fn handler(_params: SetTabColorMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
