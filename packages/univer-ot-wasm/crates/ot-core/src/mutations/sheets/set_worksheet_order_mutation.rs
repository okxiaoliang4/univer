use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetWorksheetOrderMutationParams {
    pub from_order: i32,
    pub to_order: i32,
    pub unit_id: String,
    pub sub_unit_id: String,
}

pub struct SetWorksheetOrderMutation;

impl SetWorksheetOrderMutation {
    pub const ID: &'static str = "sheet.mutation.set-worksheet-order";

    pub fn handler(_params: SetWorksheetOrderMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
