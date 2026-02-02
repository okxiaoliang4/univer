use crate::mutations::{MoveColsMutation, MoveRangeMutation, MoveRowsMutation, RemoveSheetMutation};
use crate::mutations::sheets::{InsertRowMutation, InsertColMutation, RemoveRowMutation, RemoveColMutation};
use crate::mutations::sheets_note::{
    UpdateNoteMutation, RemoveNoteMutation, ToggleNotePopupMutation, UpdateNotePositionMutation,
};
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::generic_params::GenericRowColParams;
use crate::utils::shared_transforms as shared;
use crate::utils::transform_helpers::lww_transform;

pub const UPDATE_NOTE_ID: MutationId = UpdateNoteMutation::ID;
pub const REMOVE_NOTE_ID: MutationId = RemoveNoteMutation::ID;
pub const TOGGLE_NOTE_POPUP_ID: MutationId = ToggleNotePopupMutation::ID;
pub const UPDATE_NOTE_POSITION_ID: MutationId = UpdateNotePositionMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Register symmetric transforms for note mutations
    registry.register_symmetric_ref(UPDATE_NOTE_ID, lww_transform());
    registry.register_symmetric_ref(TOGGLE_NOTE_POPUP_ID, lww_transform());
    registry.register_symmetric_ref(UPDATE_NOTE_POSITION_ID, lww_transform());

    // Shift transforms for UpdateNote (has row, col)
    registry.register_bidirectional_ref(InsertRowMutation::ID, UPDATE_NOTE_ID, shared::insert_row_shift::<GenericRowColParams>());
    registry.register_bidirectional_ref(InsertColMutation::ID, UPDATE_NOTE_ID, shared::insert_col_shift::<GenericRowColParams>());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, UPDATE_NOTE_ID, shared::remove_row_shift::<GenericRowColParams>());
    registry.register_bidirectional_ref(RemoveColMutation::ID, UPDATE_NOTE_ID, shared::remove_col_shift::<GenericRowColParams>());
    registry.register_bidirectional_ref(MoveRowsMutation::ID, UPDATE_NOTE_ID, shared::move_rows_shift::<GenericRowColParams>());
    registry.register_bidirectional_ref(MoveColsMutation::ID, UPDATE_NOTE_ID, shared::move_cols_shift::<GenericRowColParams>());
    registry.register_bidirectional_ref(MoveRangeMutation::ID, UPDATE_NOTE_ID, shared::move_range_shift::<GenericRowColParams>());
    registry.register_bidirectional_ref(RemoveSheetMutation::ID, UPDATE_NOTE_ID, shared::remove_sheet_shift::<GenericRowColParams>());

    // Shift transforms for RemoveNote (has row, col)
    registry.register_bidirectional_ref(InsertRowMutation::ID, REMOVE_NOTE_ID, shared::insert_row_shift::<GenericRowColParams>());
    registry.register_bidirectional_ref(InsertColMutation::ID, REMOVE_NOTE_ID, shared::insert_col_shift::<GenericRowColParams>());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, REMOVE_NOTE_ID, shared::remove_row_shift::<GenericRowColParams>());
    registry.register_bidirectional_ref(RemoveColMutation::ID, REMOVE_NOTE_ID, shared::remove_col_shift::<GenericRowColParams>());
    registry.register_bidirectional_ref(MoveRowsMutation::ID, REMOVE_NOTE_ID, shared::move_rows_shift::<GenericRowColParams>());
    registry.register_bidirectional_ref(MoveColsMutation::ID, REMOVE_NOTE_ID, shared::move_cols_shift::<GenericRowColParams>());
    registry.register_bidirectional_ref(MoveRangeMutation::ID, REMOVE_NOTE_ID, shared::move_range_shift::<GenericRowColParams>());
    registry.register_bidirectional_ref(RemoveSheetMutation::ID, REMOVE_NOTE_ID, shared::remove_sheet_shift::<GenericRowColParams>());

    // Shift transforms for ToggleNotePopup (has row, col)
    registry.register_bidirectional_ref(InsertRowMutation::ID, TOGGLE_NOTE_POPUP_ID, shared::insert_row_shift::<GenericRowColParams>());
    registry.register_bidirectional_ref(InsertColMutation::ID, TOGGLE_NOTE_POPUP_ID, shared::insert_col_shift::<GenericRowColParams>());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, TOGGLE_NOTE_POPUP_ID, shared::remove_row_shift::<GenericRowColParams>());
    registry.register_bidirectional_ref(RemoveColMutation::ID, TOGGLE_NOTE_POPUP_ID, shared::remove_col_shift::<GenericRowColParams>());
    registry.register_bidirectional_ref(MoveRowsMutation::ID, TOGGLE_NOTE_POPUP_ID, shared::move_rows_shift::<GenericRowColParams>());
    registry.register_bidirectional_ref(MoveColsMutation::ID, TOGGLE_NOTE_POPUP_ID, shared::move_cols_shift::<GenericRowColParams>());
    registry.register_bidirectional_ref(MoveRangeMutation::ID, TOGGLE_NOTE_POPUP_ID, shared::move_range_shift::<GenericRowColParams>());
    registry.register_bidirectional_ref(RemoveSheetMutation::ID, TOGGLE_NOTE_POPUP_ID, shared::remove_sheet_shift::<GenericRowColParams>());

    // Shift transforms for UpdateNotePosition (has row, col)
    registry.register_bidirectional_ref(InsertRowMutation::ID, UPDATE_NOTE_POSITION_ID, shared::insert_row_shift::<GenericRowColParams>());
    registry.register_bidirectional_ref(InsertColMutation::ID, UPDATE_NOTE_POSITION_ID, shared::insert_col_shift::<GenericRowColParams>());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, UPDATE_NOTE_POSITION_ID, shared::remove_row_shift::<GenericRowColParams>());
    registry.register_bidirectional_ref(RemoveColMutation::ID, UPDATE_NOTE_POSITION_ID, shared::remove_col_shift::<GenericRowColParams>());
    registry.register_bidirectional_ref(MoveRowsMutation::ID, UPDATE_NOTE_POSITION_ID, shared::move_rows_shift::<GenericRowColParams>());
    registry.register_bidirectional_ref(MoveColsMutation::ID, UPDATE_NOTE_POSITION_ID, shared::move_cols_shift::<GenericRowColParams>());
    registry.register_bidirectional_ref(MoveRangeMutation::ID, UPDATE_NOTE_POSITION_ID, shared::move_range_shift::<GenericRowColParams>());
    registry.register_bidirectional_ref(RemoveSheetMutation::ID, UPDATE_NOTE_POSITION_ID, shared::remove_sheet_shift::<GenericRowColParams>());
}