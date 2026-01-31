use crate::mutations::sheets::EmptyMutation;
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::utils::transform_helpers::identity_transform;
use crate::types::{MutationInfo, TransformResultRef};

pub const EMPTY_ID: MutationId = EmptyMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Register symmetric transform - empty mutation is identity
    registry.register_symmetric_ref(EMPTY_ID, identity_transform());
}

pub fn register_cross_module_transforms(registry: &mut TransformRegistry, other_mutations: &[MutationId]) {
    for &other_id in other_mutations {
        registry.register_identity(EMPTY_ID, other_id);
    }
}