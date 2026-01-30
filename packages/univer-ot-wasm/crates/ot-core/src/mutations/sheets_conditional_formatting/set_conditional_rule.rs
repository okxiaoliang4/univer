use serde::{Deserialize, Serialize};
use super::types::IConditionFormattingRule;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetConditionalRuleMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cf_id: Option<String>,
    pub rule: IConditionFormattingRule,
}

pub struct SetConditionalRuleMutation;

impl SetConditionalRuleMutation {
    pub const ID: &'static str = "sheet.mutation.set-conditional-rule";

    pub fn handler(_params: SetConditionalRuleMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
