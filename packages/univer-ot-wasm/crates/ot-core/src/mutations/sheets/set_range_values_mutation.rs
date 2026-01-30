use serde::{Deserialize, Serialize};
use crate::types::{ICellData, IObjectMatrixPrimitiveType, ICopyToOptionsData};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetRangeValuesMutationParams {
    pub sub_unit_id: String,
    pub unit_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell_value: Option<IObjectMatrixPrimitiveType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ICopyToOptionsData>,
}

pub struct SetRangeValuesMutation;

impl SetRangeValuesMutation {
    pub const ID: &'static str = "sheet.mutation.set-range-values";

    pub fn handler(_params: SetRangeValuesMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
