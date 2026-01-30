use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef};
use std::sync::Arc;

// Additional mutations that need transforms but don't fit existing modules
pub const REMOVE_WORKSHEET_MERGE_ID: MutationId = "sheet.mutation.remove-worksheet-merge";
pub const ADD_RANGE_PROTECTION_ID: MutationId = "sheet.mutation.add-range-protection";
pub const DELETE_RANGE_PROTECTION_ID: MutationId = "sheet.mutation.delete-range-protection";
pub const ADD_RANGE_THEME_ID: MutationId = "sheet.mutation.add-range-theme";
pub const REMOVE_RANGE_THEME_ID: MutationId = "sheet.mutation.remove-range-theme";
pub const SET_COL_DATA_ID: MutationId = "sheet.mutation.set-col-data";
pub const REMOVE_SHEET_ID: MutationId = "sheet.mutation.remove-sheet";
pub const SET_WORKSHEET_NAME_ID: MutationId = "sheet.mutation.set-worksheet-name";
pub const SET_WORKSHEET_ORDER_ID: MutationId = "sheet.mutation.set-worksheet-order";
pub const SET_WORKSHEET_HIDDEN_ID: MutationId = "sheet.mutation.set-worksheet-hidden";
pub const COPY_WORKSHEET_END_ID: MutationId = "sheet.mutation.copy-worksheet-end";

const ADDITIONAL_MUTATIONS: &[MutationId] = &[
    REMOVE_WORKSHEET_MERGE_ID,
    ADD_RANGE_PROTECTION_ID,
    DELETE_RANGE_PROTECTION_ID,
    ADD_RANGE_THEME_ID,
    REMOVE_RANGE_THEME_ID,
    SET_COL_DATA_ID,
    REMOVE_SHEET_ID,
    SET_WORKSHEET_NAME_ID,
    SET_WORKSHEET_ORDER_ID,
    SET_WORKSHEET_HIDDEN_ID,
    COPY_WORKSHEET_END_ID,
];

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Register symmetric transforms
    registry.register_symmetric_ref(REMOVE_WORKSHEET_MERGE_ID, create_identity());
    registry.register_symmetric_ref(ADD_RANGE_PROTECTION_ID, create_identity());
    registry.register_symmetric_ref(DELETE_RANGE_PROTECTION_ID, create_identity());
    registry.register_symmetric_ref(ADD_RANGE_THEME_ID, create_identity());
    registry.register_symmetric_ref(REMOVE_RANGE_THEME_ID, create_identity());
    registry.register_symmetric_ref(SET_COL_DATA_ID, create_identity());
    registry.register_symmetric_ref(REMOVE_SHEET_ID, create_identity());
    registry.register_symmetric_ref(SET_WORKSHEET_NAME_ID, create_lww());
    registry.register_symmetric_ref(SET_WORKSHEET_ORDER_ID, create_lww());
    registry.register_symmetric_ref(SET_WORKSHEET_HIDDEN_ID, create_lww());
    registry.register_symmetric_ref(COPY_WORKSHEET_END_ID, create_identity());

    // Register bidirectional within module (all are identity transforms as they don't interfere)
    let mutations = ADDITIONAL_MUTATIONS;
    for i in 0..mutations.len() {
        for j in (i + 1)..mutations.len() {
            registry.register_bidirectional_ref(mutations[i], mutations[j], create_identity());
        }
    }
}

pub fn register_cross_module_transforms(registry: &mut TransformRegistry, other_mutations: &[MutationId]) {
    for &add_id in ADDITIONAL_MUTATIONS {
        for &other_id in other_mutations {
            registry.register_identity(add_id, other_id);
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
