use serde::{Deserialize, Serialize};
use super::types::ISheetHyperLink;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddHyperLinkMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub link: ISheetHyperLink,
}

pub struct AddHyperLinkMutation;

impl AddHyperLinkMutation {
    pub const ID: &'static str = "sheets.mutation.add-hyper-link";

    pub fn handler(_params: AddHyperLinkMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
