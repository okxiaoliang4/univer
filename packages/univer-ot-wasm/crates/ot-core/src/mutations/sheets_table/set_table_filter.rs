use serde::{Deserialize, Serialize};
use super::types::ITableFilterItem;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetSheetTableFilterParams {
    pub unit_id: String,
    pub table_id: String,
    pub column: i32,
    pub table_filter: Option<ITableFilterItem>,
}

pub struct SetSheetTableFilterMutation;

impl SetSheetTableFilterMutation {
    pub const ID: &'static str = "sheet.mutation.set-table-filter";

    pub fn handler(_params: SetSheetTableFilterParams) -> Result<bool, String> {
        Ok(true)
    }
}
