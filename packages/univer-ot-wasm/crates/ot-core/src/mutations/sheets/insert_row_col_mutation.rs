use serde::{Deserialize, Serialize};
use crate::types::{IRange, IRowData, IColumnData, IObjectArrayPrimitiveType};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsertRowMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub range: IRange,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_info: Option<IObjectArrayPrimitiveType<IRowData>>,
}

pub struct InsertRowMutation;

impl InsertRowMutation {
    pub const ID: &'static str = "sheet.mutation.insert-row";

    pub fn handler(_params: InsertRowMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsertColMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub range: IRange,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub col_info: Option<IObjectArrayPrimitiveType<IColumnData>>,
}

pub struct InsertColMutation;

impl InsertColMutation {
    pub const ID: &'static str = "sheet.mutation.insert-col";

    pub fn handler(_params: InsertColMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
