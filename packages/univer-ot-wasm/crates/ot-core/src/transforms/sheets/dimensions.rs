use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef};
use std::sync::Arc;

pub const SET_ROW_HEIGHT_ID: MutationId = "sheet.mutation.set-worksheet-row-height";
pub const SET_ROW_IS_AUTO_HEIGHT_ID: MutationId = "sheet.mutation.set-worksheet-row-is-auto-height";
pub const SET_ROW_AUTO_HEIGHT_ID: MutationId = "sheet.mutation.set-worksheet-row-auto-height";
pub const SET_COL_WIDTH_ID: MutationId = "sheet.mutation.set-worksheet-col-width";
pub const SET_ROW_COUNT_ID: MutationId = "sheet.mutation.set-worksheet-row-count";
pub const SET_COL_COUNT_ID: MutationId = "sheet.mutation.set-worksheet-column-count";

const DIMENSION_MUTATIONS: &[MutationId] = &[
    SET_ROW_HEIGHT_ID,
    SET_ROW_IS_AUTO_HEIGHT_ID,
    SET_ROW_AUTO_HEIGHT_ID,
    SET_COL_WIDTH_ID,
    SET_ROW_COUNT_ID,
    SET_COL_COUNT_ID,
];

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Register symmetric transforms
    registry.register_symmetric_ref(SET_ROW_HEIGHT_ID, create_lww());
    registry.register_symmetric_ref(SET_ROW_IS_AUTO_HEIGHT_ID, create_lww());
    registry.register_symmetric_ref(SET_ROW_AUTO_HEIGHT_ID, create_lww());
    registry.register_symmetric_ref(SET_COL_WIDTH_ID, create_lww());
    registry.register_symmetric_ref(SET_ROW_COUNT_ID, create_lww());
    registry.register_symmetric_ref(SET_COL_COUNT_ID, create_lww());

    // Register bidirectional within module - all are identity (different dimensions)
    let mutations = DIMENSION_MUTATIONS;
    for i in 0..mutations.len() {
        for j in (i + 1)..mutations.len() {
            registry.register_bidirectional_ref(mutations[i], mutations[j], create_identity());
        }
    }
}

pub fn register_cross_module_transforms(registry: &mut TransformRegistry, other_mutations: &[MutationId]) {
    for &dim_id in DIMENSION_MUTATIONS {
        for &other_id in other_mutations {
            registry.register_identity(dim_id, other_id);
        }
    }
}

fn create_identity() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        TransformResultRef::identity(m1, m2)
    })
}

fn create_lww() -> TransformFnRef {
    Arc::new(|_m1: &MutationInfo, m2: &MutationInfo| {
        TransformResultRef {
            m1_prime: MutationOutcome::Removed,
            m2_prime: MutationOutcome::Unchanged(m2),
            error: None,
        }
    })
}
