use serde::{Deserialize, Serialize};
use super::types::ICellLinkContent;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateHyperLinkMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub id: String,
    pub payload: ICellLinkContent,
}

pub struct UpdateHyperLinkMutation;

impl UpdateHyperLinkMutation {
    pub const ID: &'static str = "sheets.mutation.update-hyper-link";

    pub fn handler(_params: UpdateHyperLinkMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateHyperLinkRefMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub id: String,
    pub row: i32,
    pub column: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub silent: Option<bool>,
}

pub struct UpdateHyperLinkRefMutation;

impl UpdateHyperLinkRefMutation {
    pub const ID: &'static str = "sheets.mutation.update-hyper-link-ref";

    pub fn handler(_params: UpdateHyperLinkRefMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRichHyperLinkMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub row: i32,
    pub col: i32,
    pub id: String,
    pub url: String,
}

pub struct UpdateRichHyperLinkMutation;

impl UpdateRichHyperLinkMutation {
    pub const ID: &'static str = "sheets.mutation.update-rich-hyper-link";

    pub fn handler(_params: UpdateRichHyperLinkMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
