use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetFrozenMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub start_row: i32,
    pub start_column: i32,
    pub y_split: i32,
    pub x_split: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reset_scroll: Option<bool>,
}

pub struct SetFrozenMutation;

impl SetFrozenMutation {
    pub const ID: &'static str = "sheet.mutation.set-frozen";

    pub fn handler(_params: SetFrozenMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
