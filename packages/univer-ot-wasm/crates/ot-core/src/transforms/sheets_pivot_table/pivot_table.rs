use crate::mutations::sheets_pivot_table::{
    AddPivotTableMutation, RemovePivotTableMutation, SetPivotTableSourceRangeMutation,
    SetPivotTableTargetCellMutation, SetPivotTableFieldsConfigMutation,
    SetPivotTableCalculatedDataMutation,
};
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::transform_helpers::{lww_transform, identity_transform};

pub const ADD_PIVOT_TABLE_ID: MutationId = AddPivotTableMutation::ID;
pub const REMOVE_PIVOT_TABLE_ID: MutationId = RemovePivotTableMutation::ID;
pub const SET_SOURCE_RANGE_ID: MutationId = SetPivotTableSourceRangeMutation::ID;
pub const SET_TARGET_CELL_ID: MutationId = SetPivotTableTargetCellMutation::ID;
pub const SET_FIELDS_CONFIG_ID: MutationId = SetPivotTableFieldsConfigMutation::ID;
pub const SET_CALCULATED_DATA_ID: MutationId = SetPivotTableCalculatedDataMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Register symmetric transforms for pivot table mutations
    registry.register_symmetric_ref(ADD_PIVOT_TABLE_ID, identity_transform());
    registry.register_symmetric_ref(REMOVE_PIVOT_TABLE_ID, identity_transform());
    registry.register_symmetric_ref(SET_SOURCE_RANGE_ID, lww_transform());
    registry.register_symmetric_ref(SET_TARGET_CELL_ID, lww_transform());
    registry.register_symmetric_ref(SET_FIELDS_CONFIG_ID, lww_transform());
    registry.register_symmetric_ref(SET_CALCULATED_DATA_ID, lww_transform());

    // Register bidirectional transforms within pivot table module
    registry.register_bidirectional_ref(ADD_PIVOT_TABLE_ID, REMOVE_PIVOT_TABLE_ID, identity_transform());
    registry.register_bidirectional_ref(ADD_PIVOT_TABLE_ID, SET_SOURCE_RANGE_ID, identity_transform());
    registry.register_bidirectional_ref(ADD_PIVOT_TABLE_ID, SET_TARGET_CELL_ID, identity_transform());
    registry.register_bidirectional_ref(ADD_PIVOT_TABLE_ID, SET_FIELDS_CONFIG_ID, identity_transform());
    registry.register_bidirectional_ref(ADD_PIVOT_TABLE_ID, SET_CALCULATED_DATA_ID, identity_transform());
    registry.register_bidirectional_ref(REMOVE_PIVOT_TABLE_ID, SET_SOURCE_RANGE_ID, identity_transform());
    registry.register_bidirectional_ref(REMOVE_PIVOT_TABLE_ID, SET_TARGET_CELL_ID, identity_transform());
    registry.register_bidirectional_ref(REMOVE_PIVOT_TABLE_ID, SET_FIELDS_CONFIG_ID, identity_transform());
    registry.register_bidirectional_ref(REMOVE_PIVOT_TABLE_ID, SET_CALCULATED_DATA_ID, identity_transform());
    registry.register_bidirectional_ref(SET_SOURCE_RANGE_ID, SET_TARGET_CELL_ID, identity_transform());
    registry.register_bidirectional_ref(SET_SOURCE_RANGE_ID, SET_FIELDS_CONFIG_ID, identity_transform());
    registry.register_bidirectional_ref(SET_SOURCE_RANGE_ID, SET_CALCULATED_DATA_ID, identity_transform());
    registry.register_bidirectional_ref(SET_TARGET_CELL_ID, SET_FIELDS_CONFIG_ID, identity_transform());
    registry.register_bidirectional_ref(SET_TARGET_CELL_ID, SET_CALCULATED_DATA_ID, identity_transform());
    registry.register_bidirectional_ref(SET_FIELDS_CONFIG_ID, SET_CALCULATED_DATA_ID, identity_transform());

    // NOTE: No register_identity calls needed - registry falls back to identity automatically
}