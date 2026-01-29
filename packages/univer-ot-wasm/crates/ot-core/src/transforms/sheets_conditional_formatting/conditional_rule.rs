use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef};
use std::sync::Arc;

pub const ADD_RULE_ID: MutationId = "sheet.mutation.add-conditional-rule";
pub const DELETE_RULE_ID: MutationId = "sheet.mutation.delete-conditional-rule";
pub const SET_RULE_ID: MutationId = "sheet.mutation.set-conditional-rule";
pub const MOVE_RULE_ID: MutationId = "sheet.mutation.move-conditional-rule";

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Add rule
    registry.register_symmetric_ref(ADD_RULE_ID, create_identity());
    registry.register_bidirectional_ref(ADD_RULE_ID, DELETE_RULE_ID, create_identity());
    registry.register_bidirectional_ref(ADD_RULE_ID, SET_RULE_ID, create_identity());
    registry.register_bidirectional_ref(ADD_RULE_ID, MOVE_RULE_ID, create_identity());

    // Delete rule
    registry.register_symmetric_ref(DELETE_RULE_ID, create_identity());
    registry.register_bidirectional_ref(DELETE_RULE_ID, SET_RULE_ID, create_identity());
    registry.register_bidirectional_ref(DELETE_RULE_ID, MOVE_RULE_ID, create_identity());

    // Set rule
    registry.register_symmetric_ref(SET_RULE_ID, create_lww());
    registry.register_bidirectional_ref(SET_RULE_ID, MOVE_RULE_ID, create_identity());

    // Move rule
    registry.register_symmetric_ref(MOVE_RULE_ID, create_identity());

    // With sheet operations - add-conditional-rule
    registry.register_identity(ADD_RULE_ID, "sheet.mutation.insert-row");
    registry.register_identity(ADD_RULE_ID, "sheet.mutation.insert-col");
    registry.register_identity(ADD_RULE_ID, "sheet.mutation.remove-rows");
    registry.register_identity(ADD_RULE_ID, "sheet.mutation.remove-col");
    registry.register_identity(ADD_RULE_ID, "sheet.mutation.set-range-values");
    registry.register_identity(ADD_RULE_ID, "sheet.mutation.move-range");
    registry.register_identity(ADD_RULE_ID, "sheet.mutation.move-rows");
    registry.register_identity(ADD_RULE_ID, "sheet.mutation.move-columns");
    registry.register_identity(ADD_RULE_ID, "sheet.mutation.add-worksheet-merge");
    registry.register_identity(ADD_RULE_ID, "sheet.mutation.set-range-protection");
    registry.register_identity(ADD_RULE_ID, "sheet.mutation.set-range-theme");
    registry.register_identity(ADD_RULE_ID, "sheet.mutation.set.numfmt");
    registry.register_identity(ADD_RULE_ID, "sheet.mutation.set-frozen");
    registry.register_identity(ADD_RULE_ID, "sheet.mutation.set-row-data");
    registry.register_identity(ADD_RULE_ID, "sheet.mutation.insert-sheet");
    registry.register_identity(ADD_RULE_ID, "sheet.mutation.set-workbook-name");

    // With sheet operations - delete-conditional-rule
    registry.register_identity(DELETE_RULE_ID, "sheet.mutation.insert-row");
    registry.register_identity(DELETE_RULE_ID, "sheet.mutation.insert-col");
    registry.register_identity(DELETE_RULE_ID, "sheet.mutation.remove-rows");
    registry.register_identity(DELETE_RULE_ID, "sheet.mutation.remove-col");
    registry.register_identity(DELETE_RULE_ID, "sheet.mutation.set-range-values");
    registry.register_identity(DELETE_RULE_ID, "sheet.mutation.move-range");
    registry.register_identity(DELETE_RULE_ID, "sheet.mutation.move-rows");
    registry.register_identity(DELETE_RULE_ID, "sheet.mutation.move-columns");
    registry.register_identity(DELETE_RULE_ID, "sheet.mutation.add-worksheet-merge");
    registry.register_identity(DELETE_RULE_ID, "sheet.mutation.set-range-protection");
    registry.register_identity(DELETE_RULE_ID, "sheet.mutation.set-range-theme");
    registry.register_identity(DELETE_RULE_ID, "sheet.mutation.set.numfmt");
    registry.register_identity(DELETE_RULE_ID, "sheet.mutation.set-frozen");
    registry.register_identity(DELETE_RULE_ID, "sheet.mutation.set-row-data");
    registry.register_identity(DELETE_RULE_ID, "sheet.mutation.insert-sheet");
    registry.register_identity(DELETE_RULE_ID, "sheet.mutation.set-workbook-name");

    // With sheet operations - set-conditional-rule
    registry.register_identity(SET_RULE_ID, "sheet.mutation.insert-row");
    registry.register_identity(SET_RULE_ID, "sheet.mutation.insert-col");
    registry.register_identity(SET_RULE_ID, "sheet.mutation.remove-rows");
    registry.register_identity(SET_RULE_ID, "sheet.mutation.remove-col");
    registry.register_identity(SET_RULE_ID, "sheet.mutation.set-range-values");
    registry.register_identity(SET_RULE_ID, "sheet.mutation.move-range");
    registry.register_identity(SET_RULE_ID, "sheet.mutation.move-rows");
    registry.register_identity(SET_RULE_ID, "sheet.mutation.move-columns");
    registry.register_identity(SET_RULE_ID, "sheet.mutation.add-worksheet-merge");
    registry.register_identity(SET_RULE_ID, "sheet.mutation.set-range-protection");
    registry.register_identity(SET_RULE_ID, "sheet.mutation.set-range-theme");
    registry.register_identity(SET_RULE_ID, "sheet.mutation.set.numfmt");
    registry.register_identity(SET_RULE_ID, "sheet.mutation.set-frozen");
    registry.register_identity(SET_RULE_ID, "sheet.mutation.set-row-data");
    registry.register_identity(SET_RULE_ID, "sheet.mutation.insert-sheet");
    registry.register_identity(SET_RULE_ID, "sheet.mutation.set-workbook-name");

    // With sheet operations - move-conditional-rule
    registry.register_identity(MOVE_RULE_ID, "sheet.mutation.insert-row");
    registry.register_identity(MOVE_RULE_ID, "sheet.mutation.insert-col");
    registry.register_identity(MOVE_RULE_ID, "sheet.mutation.remove-rows");
    registry.register_identity(MOVE_RULE_ID, "sheet.mutation.remove-col");
    registry.register_identity(MOVE_RULE_ID, "sheet.mutation.set-range-values");
    registry.register_identity(MOVE_RULE_ID, "sheet.mutation.move-range");
    registry.register_identity(MOVE_RULE_ID, "sheet.mutation.move-rows");
    registry.register_identity(MOVE_RULE_ID, "sheet.mutation.move-columns");
    registry.register_identity(MOVE_RULE_ID, "sheet.mutation.add-worksheet-merge");
    registry.register_identity(MOVE_RULE_ID, "sheet.mutation.set-range-protection");
    registry.register_identity(MOVE_RULE_ID, "sheet.mutation.set-range-theme");
    registry.register_identity(MOVE_RULE_ID, "sheet.mutation.set.numfmt");
    registry.register_identity(MOVE_RULE_ID, "sheet.mutation.set-frozen");
    registry.register_identity(MOVE_RULE_ID, "sheet.mutation.set-row-data");
    registry.register_identity(MOVE_RULE_ID, "sheet.mutation.insert-sheet");
    registry.register_identity(MOVE_RULE_ID, "sheet.mutation.set-workbook-name");

    // With data validation (different features, don't interfere)
    // Note: data-validation module registers these in the opposite direction
}

fn create_identity() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        TransformResultRef::identity(m1, m2)  // Zero-copy!
    })
}

fn create_lww() -> TransformFnRef {
    Arc::new(|_m1: &MutationInfo, m2: &MutationInfo| {
        // LWW: m2 wins
        TransformResultRef {
            m1_prime: MutationOutcome::Removed,
            m2_prime: MutationOutcome::Unchanged(m2),  // Zero-copy!
            error: None,
        }
    })
}
