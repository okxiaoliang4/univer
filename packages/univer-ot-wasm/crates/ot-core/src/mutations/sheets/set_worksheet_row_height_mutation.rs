use serde::{Deserialize, Serialize};
use crate::types::{IRange, IObjectArrayPrimitiveType, BooleanNumber};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RowHeightValue {
    Number(f64),
    Object(IObjectArrayPrimitiveType<Option<f64>>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AutoHeightInfoValue {
    Number(BooleanNumber),
    Object(IObjectArrayPrimitiveType<Option<BooleanNumber>>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IRowAutoHeightInfo {
    pub row: i32,
    pub auto_height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetWorksheetRowHeightMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub ranges: Vec<IRange>,
    pub row_height: RowHeightValue,
}

pub struct SetWorksheetRowHeightMutation;

impl SetWorksheetRowHeightMutation {
    pub const ID: &'static str = "sheet.mutation.set-worksheet-row-height";

    pub fn handler(_params: SetWorksheetRowHeightMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetWorksheetRowIsAutoHeightMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub ranges: Vec<IRange>,
    pub auto_height_info: AutoHeightInfoValue,
}

pub struct SetWorksheetRowIsAutoHeightMutation;

impl SetWorksheetRowIsAutoHeightMutation {
    pub const ID: &'static str = "sheet.mutation.set-worksheet-row-is-auto-height";

    pub fn handler(_params: SetWorksheetRowIsAutoHeightMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetWorksheetRowAutoHeightMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub rows_auto_height_info: Vec<IRowAutoHeightInfo>,
}

pub struct SetWorksheetRowAutoHeightMutation;

impl SetWorksheetRowAutoHeightMutation {
    pub const ID: &'static str = "sheet.mutation.set-worksheet-row-auto-height";

    pub fn handler(_params: SetWorksheetRowAutoHeightMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
