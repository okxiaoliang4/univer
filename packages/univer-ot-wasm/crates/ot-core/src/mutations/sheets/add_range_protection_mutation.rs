use serde::{Deserialize, Serialize};
use crate::types::IRangeProtectionRule;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddRangeProtectionMutationParams {
    pub rules: Vec<IRangeProtectionRule>,
    pub unit_id: String,
    pub sub_unit_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

pub struct AddRangeProtectionMutation;

impl AddRangeProtectionMutation {
    pub const ID: &'static str = "sheet.mutation.add-range-protection";

    pub fn handler(_params: AddRangeProtectionMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
