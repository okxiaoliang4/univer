use serde::{Deserialize, Serialize};
use super::types::{ISourceRangeInfo, ITargetCellInfo, IFieldsConfig, IPivotModel};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PivotTableConfig {
    pub name: String,
    pub source_range_info: ISourceRangeInfo,
    pub target_cell_info: ITargetCellInfo,
    pub fields_config: IFieldsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddPivotTableMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub pivot_table_id: String,
    pub config: PivotTableConfig,
}

pub struct AddPivotTableMutation;

impl AddPivotTableMutation {
    pub const ID: &'static str = "sheet.mutation.add-pivot-table";

    pub fn handler(_params: AddPivotTableMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemovePivotTableMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub pivot_table_id: String,
}

pub struct RemovePivotTableMutation;

impl RemovePivotTableMutation {
    pub const ID: &'static str = "sheet.mutation.remove-pivot-table";

    pub fn handler(_params: RemovePivotTableMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetPivotTableSourceRangeMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub pivot_table_id: String,
    pub source_range_info: ISourceRangeInfo,
}

pub struct SetPivotTableSourceRangeMutation;

impl SetPivotTableSourceRangeMutation {
    pub const ID: &'static str = "sheet.mutation.set-pivot-table-source-range";

    pub fn handler(_params: SetPivotTableSourceRangeMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetPivotTableTargetCellMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub pivot_table_id: String,
    pub target_cell_info: ITargetCellInfo,
}

pub struct SetPivotTableTargetCellMutation;

impl SetPivotTableTargetCellMutation {
    pub const ID: &'static str = "sheet.mutation.set-pivot-table-target-cell";

    pub fn handler(_params: SetPivotTableTargetCellMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetPivotTableFieldsConfigMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub pivot_table_id: String,
    pub fields_config: IFieldsConfig,
}

pub struct SetPivotTableFieldsConfigMutation;

impl SetPivotTableFieldsConfigMutation {
    pub const ID: &'static str = "sheet.mutation.set-pivot-table-fields-config";

    pub fn handler(_params: SetPivotTableFieldsConfigMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetPivotTableCalculatedDataMutationParams {
    pub unit_id: String,
    pub sub_unit_id: String,
    pub pivot_table_id: String,
    pub pivot_model: IPivotModel,
}

pub struct SetPivotTableCalculatedDataMutation;

impl SetPivotTableCalculatedDataMutation {
    pub const ID: &'static str = "sheet.mutation.set-pivot-table-calculated-data";

    pub fn handler(_params: SetPivotTableCalculatedDataMutationParams) -> Result<bool, String> {
        Ok(true)
    }
}
