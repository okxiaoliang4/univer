use serde::{Deserialize, Serialize};
use crate::types::{IRowData, IObjectArrayPrimitiveType};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetRowDataMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub row_data: IObjectArrayPrimitiveType<Option<IRowData>>,
}

pub struct SetRowDataMutation;

impl SetRowDataMutation {
    pub const ID: &'static str = "sheet.mutation.set-row-data";

    pub fn handler(_params: SetRowDataMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
