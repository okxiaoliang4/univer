use serde::{Deserialize, Serialize};
use super::types::{ITableRange, ITableOptions};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddSheetTableParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub table_id: String,
    pub name: String,
    pub range: ITableRange,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ITableOptions>,
}

pub struct AddSheetTableMutation;

impl AddSheetTableMutation {
    pub const ID: &'static str = "sheet.mutation.add-table";

    pub fn handler(_params: AddSheetTableParams) -> Result<bool, String> {
        Ok(true)
    }
}
