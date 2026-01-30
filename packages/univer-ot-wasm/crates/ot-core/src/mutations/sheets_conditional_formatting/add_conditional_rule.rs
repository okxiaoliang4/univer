use serde::{Deserialize, Serialize};
use super::types::IConditionFormattingRule;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddConditionalRuleMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub rule: IConditionFormattingRule,
}

pub struct AddConditionalRuleMutation;

impl AddConditionalRuleMutation {
    pub const ID: &'static str = "sheet.mutation.add-conditional-rule";

    pub fn handler(_params: AddConditionalRuleMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
