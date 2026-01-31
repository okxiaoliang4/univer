use crate::mutations::sheets_note::{
    UpdateNoteMutation, RemoveNoteMutation, ToggleNotePopupMutation, UpdateNotePositionMutation,
};
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::utils::transform_helpers::{lww_transform, identity_transform};
use crate::types::{MutationInfo, TransformResultRef};
use crate::transforms::constants::*;

pub const UPDATE_NOTE_ID: MutationId = UpdateNoteMutation::ID;
pub const REMOVE_NOTE_ID: MutationId = RemoveNoteMutation::ID;
pub const TOGGLE_NOTE_POPUP_ID: MutationId = ToggleNotePopupMutation::ID;
pub const UPDATE_NOTE_POSITION_ID: MutationId = UpdateNotePositionMutation::ID;

const LOCAL_NOTE_MUTATIONS: &[MutationId] = &[
    UPDATE_NOTE_ID,
    REMOVE_NOTE_ID,
    TOGGLE_NOTE_POPUP_ID,
    UPDATE_NOTE_POSITION_ID,
];

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

    // Register with all other modules (using constants)
    for &note_id in LOCAL_NOTE_MUTATIONS {
        for &sheet_id in ALL_SHEETS_CORE_MUTATIONS {
            registry.register_identity(note_id, sheet_id);
        }
        for &dv_id in DATA_VALIDATION_MUTATIONS {
            registry.register_identity(note_id, dv_id);
        }
        for &cf_id in CONDITIONAL_FORMATTING_MUTATIONS {
            registry.register_identity(note_id, cf_id);
        }
        for &filter_id in FILTER_MUTATIONS {
            registry.register_identity(note_id, filter_id);
        }
        for &hl_id in HYPER_LINK_MUTATIONS {
            registry.register_identity(note_id, hl_id);
        }
    }
}