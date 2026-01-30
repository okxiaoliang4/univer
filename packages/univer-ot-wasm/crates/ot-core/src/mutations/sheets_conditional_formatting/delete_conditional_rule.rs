use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteConditionalRuleMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub cf_id: String,
}

pub struct DeleteConditionalRuleMutation;

impl DeleteConditionalRuleMutation {
    pub const ID: &'static str = "sheet.mutation.delete-conditional-rule";

    pub fn handler(_params: DeleteConditionalRuleMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
