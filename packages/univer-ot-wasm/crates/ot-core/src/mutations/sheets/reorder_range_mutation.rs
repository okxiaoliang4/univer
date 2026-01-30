use serde::{Deserialize, Serialize};
use crate::types::IRange;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReorderRangeMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub range: IRange,
    pub order: HashMap<u32, u32>,
}

pub struct ReorderRangeMutation;

impl ReorderRangeMutation {
    pub const ID: &'static str = "sheet.mutation.reorder-range";

    pub fn handler(_params: ReorderRangeMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
