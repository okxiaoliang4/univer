use crate::mutations::sheets::EmptyMutation;
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::types::{MutationInfo, TransformResultRef};
use std::sync::Arc;

pub const EMPTY_ID: MutationId = EmptyMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Register symmetric transform - empty mutation is identity
    registry.register_symmetric_ref(EMPTY_ID, create_identity());
}

pub fn register_cross_module_transforms(registry: &mut TransformRegistry, other_mutations: &[MutationId]) {
    for &other_id in other_mutations {
        registry.register_identity(EMPTY_ID, other_id);
    }
}

fn create_identity() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        TransformResultRef::identity(m1, m2)
    })
}
