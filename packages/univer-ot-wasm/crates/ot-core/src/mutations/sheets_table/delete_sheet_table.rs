use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSheetTableParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub table_id: String,
}

pub struct DeleteSheetTableMutation;

impl DeleteSheetTableMutation {
    pub const ID: &'static str = "sheet.mutation.delete-table";

    pub fn handler(_params: DeleteSheetTableParams) -> Result<bool, String> {
        Ok(true)
    }
}
