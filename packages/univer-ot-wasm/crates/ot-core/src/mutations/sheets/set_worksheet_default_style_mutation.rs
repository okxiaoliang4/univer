use serde::{Deserialize, Serialize};
use crate::types::IStyleData;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DefaultStyleValue {
    String(String),
    Object(Option<IStyleData>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetWorksheetDefaultStyleMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub default_style: DefaultStyleValue,
}

pub struct SetWorksheetDefaultStyleMutation;

impl SetWorksheetDefaultStyleMutation {
    pub const ID: &'static str = "sheet.mutation.set-worksheet-default-style";

    pub fn handler(_params: SetWorksheetDefaultStyleMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
