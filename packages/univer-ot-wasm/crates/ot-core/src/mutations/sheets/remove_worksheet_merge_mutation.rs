use serde::{Deserialize, Serialize};
use crate::types::IRange;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveWorksheetMergeMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub ranges: Vec<IRange>,
}

pub struct RemoveWorksheetMergeMutation;

impl RemoveWorksheetMergeMutation {
    pub const ID: &'static str = "sheet.mutation.remove-worksheet-merge";

    pub fn handler(_params: RemoveWorksheetMergeMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
