// NOTE: Full implementations exist in move_rows.rs and move_columns.rs
// This file provides combined registration for file-level correspondence with mutations
use crate::mutations::sheets::{MoveRowsMutation, MoveColsMutation};
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::utils::transform_helpers::identity_transform;
use crate::types::{MutationInfo, TransformResultRef};

pub const MOVE_ROWS_ID: MutationId = MoveRowsMutation::ID;
pub const MOVE_COLS_ID: MutationId = MoveColsMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    registry.register_symmetric_ref(MOVE_ROWS_ID, identity_transform());
    registry.register_symmetric_ref(MOVE_COLS_ID, identity_transform());
}

pub fn register_cross_module_transforms(registry: &mut TransformRegistry, other_mutations: &[MutationId]) {
    for &other_id in other_mutations {
        registry.register_identity(MOVE_ROWS_ID, other_id);
        registry.register_identity(MOVE_COLS_ID, other_id);
    }
}