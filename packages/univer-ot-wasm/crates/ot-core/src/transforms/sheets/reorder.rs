use crate::mutations::sheets::ReorderRangeMutation;
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::types::{MutationInfo, TransformResultRef};
use std::sync::Arc;

pub const REORDER_RANGE_ID: MutationId = ReorderRangeMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Register symmetric transform
    registry.register_symmetric_ref(REORDER_RANGE_ID, create_identity());
}

pub fn register_cross_module_transforms(registry: &mut TransformRegistry, other_mutations: &[MutationId]) {
    for &other_id in other_mutations {
        registry.register_identity(REORDER_RANGE_ID, other_id);
    }
}

fn create_identity() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        TransformResultRef::identity(m1, m2)
    })
}
