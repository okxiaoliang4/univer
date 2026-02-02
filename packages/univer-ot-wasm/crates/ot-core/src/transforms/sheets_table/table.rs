use crate::mutations::sheets_table::{
    AddSheetTableMutation, SetSheetTableMutation, SetSheetTableFilterMutation, DeleteSheetTableMutation,
};
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::transform_helpers::{lww_transform, identity_transform};

pub const ADD_TABLE_ID: MutationId = AddSheetTableMutation::ID;
pub const SET_TABLE_ID: MutationId = SetSheetTableMutation::ID;
pub const SET_TABLE_FILTER_ID: MutationId = SetSheetTableFilterMutation::ID;
pub const DELETE_TABLE_ID: MutationId = DeleteSheetTableMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Register symmetric transforms for table mutations
    registry.register_symmetric_ref(ADD_TABLE_ID, identity_transform());
    registry.register_symmetric_ref(SET_TABLE_ID, lww_transform());
    registry.register_symmetric_ref(SET_TABLE_FILTER_ID, lww_transform());
    registry.register_symmetric_ref(DELETE_TABLE_ID, identity_transform());

    // Register bidirectional transforms within table module
    registry.register_bidirectional_ref(ADD_TABLE_ID, SET_TABLE_ID, identity_transform());
    registry.register_bidirectional_ref(ADD_TABLE_ID, SET_TABLE_FILTER_ID, identity_transform());
    registry.register_bidirectional_ref(ADD_TABLE_ID, DELETE_TABLE_ID, identity_transform());
    registry.register_bidirectional_ref(SET_TABLE_ID, SET_TABLE_FILTER_ID, identity_transform());
    registry.register_bidirectional_ref(SET_TABLE_ID, DELETE_TABLE_ID, identity_transform());
    registry.register_bidirectional_ref(SET_TABLE_FILTER_ID, DELETE_TABLE_ID, identity_transform());

    // NOTE: No register_identity calls needed - registry falls back to identity automatically
}