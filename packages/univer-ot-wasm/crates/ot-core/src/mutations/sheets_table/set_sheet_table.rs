use serde::{Deserialize, Serialize};
use super::types::ITableSetConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetSheetTableMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub table_id: String,
    pub config: ITableSetConfig,
}

pub struct SetSheetTableMutation;

impl SetSheetTableMutation {
    pub const ID: &'static str = "sheet.mutation.set-sheet-table";

    pub fn handler(_params: SetSheetTableMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
