use serde::{Deserialize, Serialize};
use crate::types::{IColumnData, IObjectArrayPrimitiveType};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetColDataMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub column_data: IObjectArrayPrimitiveType<Option<IColumnData>>,
}

pub struct SetColDataMutation;

impl SetColDataMutation {
    pub const ID: &'static str = "sheet.mutation.set-col-data";

    pub fn handler(_params: SetColDataMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
