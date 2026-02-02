use crate::mutations::sheets_conditional_formatting::{
    AddConditionalRuleMutation, DeleteConditionalRuleMutation, SetConditionalRuleMutation,
    MoveConditionalRuleMutation,
};
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::transform_helpers::{lww_transform, identity_transform};

pub const ADD_RULE_ID: MutationId = AddConditionalRuleMutation::ID;
pub const DELETE_RULE_ID: MutationId = DeleteConditionalRuleMutation::ID;
pub const SET_RULE_ID: MutationId = SetConditionalRuleMutation::ID;
pub const MOVE_RULE_ID: MutationId = MoveConditionalRuleMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Add rule
    registry.register_symmetric_ref(ADD_RULE_ID, identity_transform());
    registry.register_bidirectional_ref(ADD_RULE_ID, DELETE_RULE_ID, identity_transform());
    registry.register_bidirectional_ref(ADD_RULE_ID, SET_RULE_ID, identity_transform());
    registry.register_bidirectional_ref(ADD_RULE_ID, MOVE_RULE_ID, identity_transform());

    // Delete rule
    registry.register_symmetric_ref(DELETE_RULE_ID, identity_transform());
    registry.register_bidirectional_ref(DELETE_RULE_ID, SET_RULE_ID, identity_transform());
    registry.register_bidirectional_ref(DELETE_RULE_ID, MOVE_RULE_ID, identity_transform());

    // Set rule
    registry.register_symmetric_ref(SET_RULE_ID, lww_transform());
    registry.register_bidirectional_ref(SET_RULE_ID, MOVE_RULE_ID, identity_transform());

    // Move rule
    registry.register_symmetric_ref(MOVE_RULE_ID, identity_transform());

    // NOTE: No register_identity calls needed - registry falls back to identity automatically
}