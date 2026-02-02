// NOTE: Full implementations exist in remove_rows.rs and remove_col.rs
// This file provides combined registration for file-level correspondence with mutations
use crate::mutations::sheets::{RemoveRowMutation, RemoveColMutation};
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::transform_helpers::identity_transform;

pub const REMOVE_ROW_ID: MutationId = RemoveRowMutation::ID;
pub const REMOVE_COL_ID: MutationId = RemoveColMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    registry.register_symmetric_ref(REMOVE_ROW_ID, identity_transform());
    registry.register_symmetric_ref(REMOVE_COL_ID, identity_transform());

    // NOTE: No register_identity calls needed - registry falls back to identity automatically
}
