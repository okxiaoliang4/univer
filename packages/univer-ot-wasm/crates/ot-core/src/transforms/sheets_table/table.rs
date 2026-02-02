use crate::mutations::sheets::{InsertRowMutation, InsertColMutation, RemoveRowMutation, RemoveColMutation};
use crate::mutations::sheets_table::{
    AddSheetTableMutation, AddSheetTableParams,
    SetSheetTableMutation, SetSheetTableMutationParams,
    SetSheetTableFilterMutation, DeleteSheetTableMutation,
};
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::shared_transforms as shared;
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

    // Shift transforms for AddSheetTable (has range)
    registry.register_bidirectional_ref(InsertRowMutation::ID, ADD_TABLE_ID, shared::insert_row_shift::<AddSheetTableParams>());
    registry.register_bidirectional_ref(InsertColMutation::ID, ADD_TABLE_ID, shared::insert_col_shift::<AddSheetTableParams>());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, ADD_TABLE_ID, shared::remove_row_shift::<AddSheetTableParams>());
    registry.register_bidirectional_ref(RemoveColMutation::ID, ADD_TABLE_ID, shared::remove_col_shift::<AddSheetTableParams>());

    // Shift transforms for SetSheetTable (has config.update_range.new_range)
    registry.register_bidirectional_ref(InsertRowMutation::ID, SET_TABLE_ID, shared::insert_row_shift::<SetSheetTableMutationParams>());
    registry.register_bidirectional_ref(InsertColMutation::ID, SET_TABLE_ID, shared::insert_col_shift::<SetSheetTableMutationParams>());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, SET_TABLE_ID, shared::remove_row_shift::<SetSheetTableMutationParams>());
    registry.register_bidirectional_ref(RemoveColMutation::ID, SET_TABLE_ID, shared::remove_col_shift::<SetSheetTableMutationParams>());

    // NOTE: SetSheetTableFilter and DeleteSheetTable don't have position fields
}