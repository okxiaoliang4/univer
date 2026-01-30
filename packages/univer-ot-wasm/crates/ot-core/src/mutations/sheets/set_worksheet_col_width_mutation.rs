use serde::{Deserialize, Serialize};
use crate::types::{IRange, IObjectArrayPrimitiveType};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ColWidthValue {
    Number(f64),
    Object(IObjectArrayPrimitiveType<Option<f64>>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetWorksheetColWidthMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub ranges: Vec<IRange>,
    pub col_width: ColWidthValue,
}

pub struct SetWorksheetColWidthMutation;

impl SetWorksheetColWidthMutation {
    pub const ID: &'static str = "sheet.mutation.set-worksheet-col-width";

    pub fn handler(_params: SetWorksheetColWidthMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
