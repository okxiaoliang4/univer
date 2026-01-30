use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteRangeProtectionMutationParams {
    pub rule_ids: Vec<String>,
    pub unit_id: String,
    pub sub_unit_id: String,
}

pub struct DeleteRangeProtectionMutation;

impl DeleteRangeProtectionMutation {
    pub const ID: &'static str = "sheet.mutation.delete-range-protection";

    pub fn handler(_params: DeleteRangeProtectionMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
