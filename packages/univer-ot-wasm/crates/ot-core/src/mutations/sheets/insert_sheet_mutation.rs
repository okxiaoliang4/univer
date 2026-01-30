use serde::{Deserialize, Serialize};
use crate::types::{IWorksheetData, IStyleData};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsertSheetMutationParams {
    pub unit_id: String,
    pub sheet: IWorksheetData,
    pub index: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub styles: Option<HashMap<String, Option<IStyleData>>>,
}

pub struct InsertSheetMutation;

impl InsertSheetMutation {
    pub const ID: &'static str = "sheet.mutation.insert-sheet";

    pub fn handler(_params: InsertSheetMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
