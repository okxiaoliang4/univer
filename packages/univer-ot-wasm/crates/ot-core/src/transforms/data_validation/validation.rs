use crate::mutations::data_validation::{
    AddDataValidationMutation, RemoveDataValidationMutation, UpdateDataValidationMutation,
};
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::transform_helpers::{lww_transform, identity_transform};

pub const ADD_RULE_ID: MutationId = AddDataValidationMutation::ID;
pub const REMOVE_RULE_ID: MutationId = RemoveDataValidationMutation::ID;
pub const UPDATE_RULE_ID: MutationId = UpdateDataValidationMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Add rule transforms
    registry.register_symmetric_ref(ADD_RULE_ID, identity_transform());
    registry.register_bidirectional_ref(ADD_RULE_ID, REMOVE_RULE_ID, identity_transform());
    registry.register_bidirectional_ref(ADD_RULE_ID, UPDATE_RULE_ID, identity_transform());

    // Remove rule transforms
    registry.register_symmetric_ref(REMOVE_RULE_ID, identity_transform());
    registry.register_bidirectional_ref(REMOVE_RULE_ID, UPDATE_RULE_ID, identity_transform());

    // Update rule transforms
    registry.register_symmetric_ref(UPDATE_RULE_ID, lww_transform());

    // NOTE: No register_identity calls needed - registry falls back to identity automatically
}