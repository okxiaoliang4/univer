use crate::mutations::sheets::{
    SetWorksheetDefaultStyleMutation, SetWorksheetRightToLeftMutation,
};
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef};
use std::sync::Arc;

pub const SET_DEFAULT_STYLE_ID: MutationId = SetWorksheetDefaultStyleMutation::ID;
pub const SET_RIGHT_TO_LEFT_ID: MutationId = SetWorksheetRightToLeftMutation::ID;

const WORKSHEET_STYLE_MUTATIONS: &[MutationId] = &[
    SET_DEFAULT_STYLE_ID,
    SET_RIGHT_TO_LEFT_ID,
];

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Register symmetric transforms
    registry.register_symmetric_ref(SET_DEFAULT_STYLE_ID, create_lww());
    registry.register_symmetric_ref(SET_RIGHT_TO_LEFT_ID, create_lww());

    // Register bidirectional within module
    registry.register_bidirectional_ref(SET_DEFAULT_STYLE_ID, SET_RIGHT_TO_LEFT_ID, create_identity());
}

pub fn register_cross_module_transforms(registry: &mut TransformRegistry, other_mutations: &[MutationId]) {
    for &ws_style_id in WORKSHEET_STYLE_MUTATIONS {
        for &other_id in other_mutations {
            registry.register_identity(ws_style_id, other_id);
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
        TransformResultRef {
            m1_prime: MutationOutcome::Removed,
            m2_prime: MutationOutcome::Unchanged(m2),
            error: None,
        }
    })
}
