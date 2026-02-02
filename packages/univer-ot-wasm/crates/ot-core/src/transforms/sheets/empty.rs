use crate::mutations::sheets::EmptyMutation;
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::transform_helpers::identity_transform;

pub const EMPTY_ID: MutationId = EmptyMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Register symmetric transform - empty mutation is identity
    registry.register_symmetric_ref(EMPTY_ID, identity_transform());

    // NOTE: No register_identity calls needed - registry falls back to identity automatically
}
