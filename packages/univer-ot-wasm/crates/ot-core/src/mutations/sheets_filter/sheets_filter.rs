use serde::{Deserialize, Serialize};
use super::types::IFilterColumn;
use crate::mutations::sheets::types::IRange;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetSheetsFilterRangeMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub range: IRange,
}

pub struct SetSheetsFilterRangeMutation;

impl SetSheetsFilterRangeMutation {
    pub const ID: &'static str = "sheet.mutation.set-sheets-filter-range";

    pub fn handler(_params: SetSheetsFilterRangeMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetSheetsFilterCriteriaMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub col: i32,
    pub criteria: Option<IFilterColumn>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub re_calc: Option<bool>,
}

pub struct SetSheetsFilterCriteriaMutation;

impl SetSheetsFilterCriteriaMutation {
    pub const ID: &'static str = "sheet.mutation.set-sheets-filter-criteria";

    pub fn handler(_params: SetSheetsFilterCriteriaMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveSheetsFilterMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
}

pub struct RemoveSheetsFilterMutation;

impl RemoveSheetsFilterMutation {
    pub const ID: &'static str = "sheet.mutation.remove-sheets-filter";

    pub fn handler(_params: RemoveSheetsFilterMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReCalcSheetsFilterMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
}

pub struct ReCalcSheetsFilterMutation;

impl ReCalcSheetsFilterMutation {
    pub const ID: &'static str = "sheet.mutation.re-calc-sheets-filter";

    pub fn handler(_params: ReCalcSheetsFilterMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
