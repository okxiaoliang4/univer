use serde::{Deserialize, Serialize};
use crate::types::IRange;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddWorksheetMergeMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub ranges: Vec<IRange>,
}

pub struct AddWorksheetMergeMutation;

impl AddWorksheetMergeMutation {
    pub const ID: &'static str = "sheet.mutation.add-worksheet-merge";

    pub fn handler(_params: AddWorksheetMergeMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
