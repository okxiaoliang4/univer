use crate::mutations::sheets::{
    SetColVisibleMutation, SetColHiddenMutation,
    InsertColMutation, RemoveColMutation,
    MoveColsMutation, RemoveSheetMutation,
};
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::generic_params::GenericRangesParams;
use crate::utils::transform_helpers::lww_transform;
use crate::utils::shared_transforms as shared;

pub const SET_COL_VISIBLE_ID: MutationId = SetColVisibleMutation::ID;
pub const SET_COL_HIDDEN_ID: MutationId = SetColHiddenMutation::ID;

/// Register transforms for SetColVisibleMutation and SetColHiddenMutation
///
/// Mutation IDs:
/// - sheet.mutation.set-col-visible
/// - sheet.mutation.set-col-hidden
///
/// These mutations control column visibility (show/hide).
/// Transform strategy: Last-Write-Wins (LWW) at worksheet level.
///
/// Shift transforms: Column visibility ranges must be adjusted when columns are inserted/removed.
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transforms: LWW
    registry.register_symmetric_ref(SET_COL_VISIBLE_ID, lww_transform());
    registry.register_symmetric_ref(SET_COL_HIDDEN_ID, lww_transform());

    // Shift transforms for SetColVisible (col-only operations)
    registry.register_bidirectional_ref(InsertColMutation::ID, SET_COL_VISIBLE_ID, shared::insert_col_shift::<GenericRangesParams>());
    registry.register_bidirectional_ref(RemoveColMutation::ID, SET_COL_VISIBLE_ID, shared::remove_col_shift::<GenericRangesParams>());
    registry.register_bidirectional_ref(MoveColsMutation::ID, SET_COL_VISIBLE_ID, shared::move_cols_shift::<GenericRangesParams>());
    registry.register_bidirectional_ref(RemoveSheetMutation::ID, SET_COL_VISIBLE_ID, shared::remove_sheet_shift::<GenericRangesParams>());

    // Shift transforms for SetColHidden (col-only operations)
    registry.register_bidirectional_ref(InsertColMutation::ID, SET_COL_HIDDEN_ID, shared::insert_col_shift::<GenericRangesParams>());
    registry.register_bidirectional_ref(RemoveColMutation::ID, SET_COL_HIDDEN_ID, shared::remove_col_shift::<GenericRangesParams>());
    registry.register_bidirectional_ref(MoveColsMutation::ID, SET_COL_HIDDEN_ID, shared::move_cols_shift::<GenericRangesParams>());
    registry.register_bidirectional_ref(RemoveSheetMutation::ID, SET_COL_HIDDEN_ID, shared::remove_sheet_shift::<GenericRangesParams>());
}
