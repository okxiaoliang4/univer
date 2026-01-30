use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::types::{MutationInfo, TransformResultRef};
use crate::transforms::constants::*;
use std::sync::Arc;

pub const UPDATE_NOTE_ID: MutationId = "sheet.mutation.update-note";
pub const REMOVE_NOTE_ID: MutationId = "sheet.mutation.remove-note";
pub const TOGGLE_NOTE_POPUP_ID: MutationId = "sheet.mutation.toggle-note-popup";
pub const UPDATE_NOTE_POSITION_ID: MutationId = "sheet.mutation.update-note-position";

const LOCAL_NOTE_MUTATIONS: &[MutationId] = &[
    UPDATE_NOTE_ID,
    REMOVE_NOTE_ID,
    TOGGLE_NOTE_POPUP_ID,
    UPDATE_NOTE_POSITION_ID,
];

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Register symmetric transforms for note mutations
    registry.register_symmetric_ref(UPDATE_NOTE_ID, create_lww());
    registry.register_symmetric_ref(REMOVE_NOTE_ID, create_identity());
    registry.register_symmetric_ref(TOGGLE_NOTE_POPUP_ID, create_lww());
    registry.register_symmetric_ref(UPDATE_NOTE_POSITION_ID, create_lww());

    // Register bidirectional transforms within note module
    registry.register_bidirectional_ref(UPDATE_NOTE_ID, REMOVE_NOTE_ID, create_identity());
    registry.register_bidirectional_ref(UPDATE_NOTE_ID, TOGGLE_NOTE_POPUP_ID, create_identity());
    registry.register_bidirectional_ref(UPDATE_NOTE_ID, UPDATE_NOTE_POSITION_ID, create_identity());
    registry.register_bidirectional_ref(REMOVE_NOTE_ID, TOGGLE_NOTE_POPUP_ID, create_identity());
    registry.register_bidirectional_ref(REMOVE_NOTE_ID, UPDATE_NOTE_POSITION_ID, create_identity());
    registry.register_bidirectional_ref(TOGGLE_NOTE_POPUP_ID, UPDATE_NOTE_POSITION_ID, create_identity());

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

fn create_identity() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        TransformResultRef::identity(m1, m2)
    })
}

fn create_lww() -> TransformFnRef {
    Arc::new(|_m1: &MutationInfo, m2: &MutationInfo| {
        use crate::types::MutationOutcome;
        TransformResultRef {
            m1_prime: MutationOutcome::Removed,
            m2_prime: MutationOutcome::Unchanged(m2),
            error: None,
        }
    })
}
