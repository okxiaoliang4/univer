use crate::mutations::sheets_note::{
    UpdateNoteMutation, RemoveNoteMutation, ToggleNotePopupMutation, UpdateNotePositionMutation,
};
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::transform_helpers::{lww_transform, identity_transform};

pub const UPDATE_NOTE_ID: MutationId = UpdateNoteMutation::ID;
pub const REMOVE_NOTE_ID: MutationId = RemoveNoteMutation::ID;
pub const TOGGLE_NOTE_POPUP_ID: MutationId = ToggleNotePopupMutation::ID;
pub const UPDATE_NOTE_POSITION_ID: MutationId = UpdateNotePositionMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Register symmetric transforms for note mutations
    registry.register_symmetric_ref(UPDATE_NOTE_ID, lww_transform());
    registry.register_symmetric_ref(REMOVE_NOTE_ID, identity_transform());
    registry.register_symmetric_ref(TOGGLE_NOTE_POPUP_ID, lww_transform());
    registry.register_symmetric_ref(UPDATE_NOTE_POSITION_ID, lww_transform());

    // Register bidirectional transforms within note module
    registry.register_bidirectional_ref(UPDATE_NOTE_ID, REMOVE_NOTE_ID, identity_transform());
    registry.register_bidirectional_ref(UPDATE_NOTE_ID, TOGGLE_NOTE_POPUP_ID, identity_transform());
    registry.register_bidirectional_ref(UPDATE_NOTE_ID, UPDATE_NOTE_POSITION_ID, identity_transform());
    registry.register_bidirectional_ref(REMOVE_NOTE_ID, TOGGLE_NOTE_POPUP_ID, identity_transform());
    registry.register_bidirectional_ref(REMOVE_NOTE_ID, UPDATE_NOTE_POSITION_ID, identity_transform());
    registry.register_bidirectional_ref(TOGGLE_NOTE_POPUP_ID, UPDATE_NOTE_POSITION_ID, identity_transform());

    // NOTE: No register_identity calls needed - registry falls back to identity automatically
}