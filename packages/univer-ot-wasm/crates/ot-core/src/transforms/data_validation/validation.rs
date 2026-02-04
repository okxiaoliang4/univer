use crate::mutations::sheets::{
    InsertRowMutation, InsertColMutation, RemoveRowMutation, RemoveColMutation,
    MoveRowsMutation, MoveColsMutation, MoveRangeMutation, RemoveSheetMutation,
};
use crate::mutations::data_validation::{
    AddDataValidationMutation, AddDataValidationMutationParams,
    RemoveDataValidationMutation,
    UpdateDataValidationMutation, UpdateDataValidationMutationParams,
};
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::shared_transforms as shared;
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

    // Shift transforms for AddDataValidation (has rule.ranges)
    registry.register_bidirectional_ref(InsertRowMutation::ID, ADD_RULE_ID, shared::insert_row_shift::<AddDataValidationMutationParams>());
    registry.register_bidirectional_ref(InsertColMutation::ID, ADD_RULE_ID, shared::insert_col_shift::<AddDataValidationMutationParams>());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, ADD_RULE_ID, shared::remove_row_shift::<AddDataValidationMutationParams>());
    registry.register_bidirectional_ref(RemoveColMutation::ID, ADD_RULE_ID, shared::remove_col_shift::<AddDataValidationMutationParams>());
    registry.register_bidirectional_ref(MoveRowsMutation::ID, ADD_RULE_ID, shared::move_rows_shift::<AddDataValidationMutationParams>());
    registry.register_bidirectional_ref(MoveColsMutation::ID, ADD_RULE_ID, shared::move_cols_shift::<AddDataValidationMutationParams>());
    registry.register_bidirectional_ref(MoveRangeMutation::ID, ADD_RULE_ID, shared::move_range_shift::<AddDataValidationMutationParams>());
    registry.register_bidirectional_ref(RemoveSheetMutation::ID, ADD_RULE_ID, shared::remove_sheet_shift::<AddDataValidationMutationParams>());

    // Shift transforms for UpdateDataValidation (has payload.ranges in Range/All variants)
    registry.register_bidirectional_ref(InsertRowMutation::ID, UPDATE_RULE_ID, shared::insert_row_shift::<UpdateDataValidationMutationParams>());
    registry.register_bidirectional_ref(InsertColMutation::ID, UPDATE_RULE_ID, shared::insert_col_shift::<UpdateDataValidationMutationParams>());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, UPDATE_RULE_ID, shared::remove_row_shift::<UpdateDataValidationMutationParams>());
    registry.register_bidirectional_ref(RemoveColMutation::ID, UPDATE_RULE_ID, shared::remove_col_shift::<UpdateDataValidationMutationParams>());
    registry.register_bidirectional_ref(MoveRowsMutation::ID, UPDATE_RULE_ID, shared::move_rows_shift::<UpdateDataValidationMutationParams>());
    registry.register_bidirectional_ref(MoveColsMutation::ID, UPDATE_RULE_ID, shared::move_cols_shift::<UpdateDataValidationMutationParams>());
    registry.register_bidirectional_ref(MoveRangeMutation::ID, UPDATE_RULE_ID, shared::move_range_shift::<UpdateDataValidationMutationParams>());
    registry.register_bidirectional_ref(RemoveSheetMutation::ID, UPDATE_RULE_ID, shared::remove_sheet_shift::<UpdateDataValidationMutationParams>());

    // NOTE: RemoveDataValidation only has rule_id, no position fields
}