use serde::{Deserialize, Serialize};
use crate::types::{IRange, ICellData, IObjectMatrixPrimitiveType};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveRangeFromTo {
    pub sub_unit_id: String,
    pub value: IObjectMatrixPrimitiveType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveRangeMutationParams {
    pub unit_id: String,
    pub from_range: IRange,
    pub to_range: IRange,
    pub from: MoveRangeFromTo,
    pub to: MoveRangeFromTo,
}

pub struct MoveRangeMutation;

impl MoveRangeMutation {
    pub const ID: &'static str = "sheet.mutation.move-range";

    pub fn handler(_params: MoveRangeMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
