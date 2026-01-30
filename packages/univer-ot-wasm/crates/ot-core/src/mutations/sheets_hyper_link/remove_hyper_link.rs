use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveHyperLinkMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub id: String,
}

pub struct RemoveHyperLinkMutation;

impl RemoveHyperLinkMutation {
    pub const ID: &'static str = "sheets.mutation.remove-hyper-link";

    pub fn handler(_params: RemoveHyperLinkMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
