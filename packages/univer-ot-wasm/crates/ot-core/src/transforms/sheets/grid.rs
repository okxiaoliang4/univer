use crate::mutations::sheets::{ToggleGridlinesMutation, SetGridlinesColorMutation};
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef};
use std::sync::Arc;

pub const TOGGLE_GRIDLINES_ID: MutationId = ToggleGridlinesMutation::ID;
pub const SET_GRIDLINES_COLOR_ID: MutationId = SetGridlinesColorMutation::ID;

const GRID_MUTATIONS: &[MutationId] = &[
    TOGGLE_GRIDLINES_ID,
    SET_GRIDLINES_COLOR_ID,
];

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Register symmetric transforms
    registry.register_symmetric_ref(TOGGLE_GRIDLINES_ID, create_lww());
    registry.register_symmetric_ref(SET_GRIDLINES_COLOR_ID, create_lww());

    // Register bidirectional within module
    registry.register_bidirectional_ref(TOGGLE_GRIDLINES_ID, SET_GRIDLINES_COLOR_ID, create_identity());
}

pub fn register_cross_module_transforms(registry: &mut TransformRegistry, other_mutations: &[MutationId]) {
    for &grid_id in GRID_MUTATIONS {
        for &other_id in other_mutations {
            registry.register_identity(grid_id, other_id);
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
