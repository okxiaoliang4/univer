use crate::mutations::sheets::{InsertRowMutation, InsertColMutation, RemoveRowMutation, RemoveColMutation};
use crate::mutations::sheets_conditional_formatting::{
    AddConditionalRuleMutation, AddConditionalRuleMutationParams,
    DeleteConditionalRuleMutation, SetConditionalRuleMutation, SetConditionalRuleMutationParams,
    MoveConditionalRuleMutation,
};
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::shared_transforms as shared;
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

    // Shift transforms for AddConditionalRule (has rule.ranges)
    registry.register_bidirectional_ref(InsertRowMutation::ID, ADD_RULE_ID, shared::insert_row_shift::<AddConditionalRuleMutationParams>());
    registry.register_bidirectional_ref(InsertColMutation::ID, ADD_RULE_ID, shared::insert_col_shift::<AddConditionalRuleMutationParams>());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, ADD_RULE_ID, shared::remove_row_shift::<AddConditionalRuleMutationParams>());
    registry.register_bidirectional_ref(RemoveColMutation::ID, ADD_RULE_ID, shared::remove_col_shift::<AddConditionalRuleMutationParams>());

    // Shift transforms for SetConditionalRule (has rule.ranges)
    registry.register_bidirectional_ref(InsertRowMutation::ID, SET_RULE_ID, shared::insert_row_shift::<SetConditionalRuleMutationParams>());
    registry.register_bidirectional_ref(InsertColMutation::ID, SET_RULE_ID, shared::insert_col_shift::<SetConditionalRuleMutationParams>());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, SET_RULE_ID, shared::remove_row_shift::<SetConditionalRuleMutationParams>());
    registry.register_bidirectional_ref(RemoveColMutation::ID, SET_RULE_ID, shared::remove_col_shift::<SetConditionalRuleMutationParams>());

    // NOTE: DeleteConditionalRule and MoveConditionalRule don't have position fields, no shift needed
}