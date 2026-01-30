use serde::{Deserialize, Serialize};
use super::types::{DrawingApplyType, DrawingObjects};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetDrawingApplyMutationParams {
    pub op: serde_json::Value,  // Complex JSON1 operation type, keep as Value
    pub unit_id: String,
    pub sub_unit_id: String,
    #[serde(rename = "type")]
    pub drawing_type: DrawingApplyType,
    pub objects: DrawingObjects,
}

pub struct SetDrawingApplyMutation;

impl SetDrawingApplyMutation {
    pub const ID: &'static str = "sheet.mutation.set-drawing-apply";

    pub fn handler(_params: SetDrawingApplyMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
