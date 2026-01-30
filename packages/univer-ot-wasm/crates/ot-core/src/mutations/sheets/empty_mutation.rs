use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmptyMutationParams {}

pub struct EmptyMutation;

impl EmptyMutation {
    pub const ID: &'static str = "sheet.mutation.empty";

    pub fn handler(_params: EmptyMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
