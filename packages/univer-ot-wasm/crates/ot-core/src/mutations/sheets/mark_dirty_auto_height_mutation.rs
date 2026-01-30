use serde::{Deserialize, Serialize};
use crate::types::IRange;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkDirtyRowAutoHeightMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub ranges: Vec<IRange>,
    pub id: String,
}

pub struct MarkDirtyRowAutoHeightMutation;

impl MarkDirtyRowAutoHeightMutation {
    pub const ID: &'static str = "sheet.operation.mark-dirty-row-auto-height";

    pub fn handler(_params: MarkDirtyRowAutoHeightMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelMarkDirtyRowAutoHeightMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub id: String,
}

pub struct CancelMarkDirtyRowAutoHeightMutation;

impl CancelMarkDirtyRowAutoHeightMutation {
    pub const ID: &'static str = "sheet.operation.cancel-mark-dirty-row-auto-height";

    pub fn handler(_params: CancelMarkDirtyRowAutoHeightMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
