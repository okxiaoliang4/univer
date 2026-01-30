use serde::{Deserialize, Serialize};
use crate::types::IRangeProtectionRule;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetRangeProtectionMutationParams {
    pub rule: IRangeProtectionRule,
    pub unit_id: String,
    pub sub_unit_id: String,
    pub rule_id: String,
}

pub struct SetRangeProtectionMutation;

impl SetRangeProtectionMutation {
    pub const ID: &'static str = "sheet.mutation.set-range-protection";

    pub fn handler(_params: SetRangeProtectionMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
