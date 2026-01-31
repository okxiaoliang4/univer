// NOTE: Full implementations exist in insert_row.rs and insert_col.rs
// This file provides combined registration for file-level correspondence with mutations
use crate::mutations::sheets::{InsertRowMutation, InsertColMutation};
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::utils::transform_helpers::identity_transform;
use crate::types::{MutationInfo, TransformResultRef};

pub const INSERT_ROW_ID: MutationId = InsertRowMutation::ID;
pub const INSERT_COL_ID: MutationId = InsertColMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    registry.register_symmetric_ref(INSERT_ROW_ID, identity_transform());
    registry.register_symmetric_ref(INSERT_COL_ID, identity_transform());
}

pub fn register_cross_module_transforms(registry: &mut TransformRegistry, other_mutations: &[MutationId]) {
    for &other_id in other_mutations {
        registry.register_identity(INSERT_ROW_ID, other_id);
        registry.register_identity(INSERT_COL_ID, other_id);
    }
}