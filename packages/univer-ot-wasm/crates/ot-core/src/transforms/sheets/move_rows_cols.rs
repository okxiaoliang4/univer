// NOTE: Full implementations exist in move_rows.rs and move_columns.rs
// This file provides combined registration for file-level correspondence with mutations
use crate::mutations::sheets::{MoveRowsMutation, MoveColsMutation};
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::transform_helpers::identity_transform;

pub const MOVE_ROWS_ID: MutationId = MoveRowsMutation::ID;
pub const MOVE_COLS_ID: MutationId = MoveColsMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    registry.register_symmetric_ref(MOVE_ROWS_ID, identity_transform());
    registry.register_symmetric_ref(MOVE_COLS_ID, identity_transform());

    // NOTE: No register_identity calls needed - registry falls back to identity automatically
}
