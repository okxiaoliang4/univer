use crate::{IObjectMatrixPrimitiveType, SubUnitParams};
use serde::{Deserialize, Serialize};

/// Set range values mutation parameters (transform version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetRangeValuesMutationParams {
    #[serde(flatten)]
    pub sub_unit_params: SubUnitParams,

    #[serde(rename = "cellValue", alias = "cell_value")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell_value: Option<IObjectMatrixPrimitiveType>,
}

/// SetRangeValues mutation definition
pub struct SetRangeValuesMutation;

impl SetRangeValuesMutation {
    pub const ID: &'static str = "sheet.mutation.set-range-values";

    pub fn handler(_params: SetRangeValuesMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
