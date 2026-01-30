use crate::mutations::sheets::SetTabColorMutation;
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef};
use std::sync::Arc;

pub const SET_TAB_COLOR_ID: MutationId = SetTabColorMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Register symmetric transform
    registry.register_symmetric_ref(SET_TAB_COLOR_ID, create_lww());
}

pub fn register_cross_module_transforms(registry: &mut TransformRegistry, other_mutations: &[MutationId]) {
    for &other_id in other_mutations {
        registry.register_identity(SET_TAB_COLOR_ID, other_id);
    }
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
