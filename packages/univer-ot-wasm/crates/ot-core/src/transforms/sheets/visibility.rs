use crate::mutations::sheets::{
    SetRowVisibleMutation, SetRowHiddenMutation, SetColVisibleMutation, SetColHiddenMutation,
};
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef};
use std::sync::Arc;

pub const SET_ROW_VISIBLE_ID: MutationId = SetRowVisibleMutation::ID;
pub const SET_ROW_HIDDEN_ID: MutationId = SetRowHiddenMutation::ID;
pub const SET_COL_VISIBLE_ID: MutationId = SetColVisibleMutation::ID;
pub const SET_COL_HIDDEN_ID: MutationId = SetColHiddenMutation::ID;

const VISIBILITY_MUTATIONS: &[MutationId] = &[
    SET_ROW_VISIBLE_ID,
    SET_ROW_HIDDEN_ID,
    SET_COL_VISIBLE_ID,
    SET_COL_HIDDEN_ID,
];

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Register symmetric transforms
    registry.register_symmetric_ref(SET_ROW_VISIBLE_ID, create_lww());
    registry.register_symmetric_ref(SET_ROW_HIDDEN_ID, create_lww());
    registry.register_symmetric_ref(SET_COL_VISIBLE_ID, create_lww());
    registry.register_symmetric_ref(SET_COL_HIDDEN_ID, create_lww());

    // Register bidirectional within module
    registry.register_bidirectional_ref(SET_ROW_VISIBLE_ID, SET_ROW_HIDDEN_ID, create_identity());
    registry.register_bidirectional_ref(SET_ROW_VISIBLE_ID, SET_COL_VISIBLE_ID, create_identity());
    registry.register_bidirectional_ref(SET_ROW_VISIBLE_ID, SET_COL_HIDDEN_ID, create_identity());
    registry.register_bidirectional_ref(SET_ROW_HIDDEN_ID, SET_COL_VISIBLE_ID, create_identity());
    registry.register_bidirectional_ref(SET_ROW_HIDDEN_ID, SET_COL_HIDDEN_ID, create_identity());
    registry.register_bidirectional_ref(SET_COL_VISIBLE_ID, SET_COL_HIDDEN_ID, create_identity());
}

pub fn register_cross_module_transforms(registry: &mut TransformRegistry, other_mutations: &[MutationId]) {
    for &vis_id in VISIBILITY_MUTATIONS {
        for &other_id in other_mutations {
            registry.register_identity(vis_id, other_id);
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
