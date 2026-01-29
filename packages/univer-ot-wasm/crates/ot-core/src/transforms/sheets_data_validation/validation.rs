use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef};
use std::sync::Arc;

pub const ADD_RULE_ID: MutationId = "data-validation.mutation.addRule";
pub const REMOVE_RULE_ID: MutationId = "data-validation.mutation.removeRule";
pub const UPDATE_RULE_ID: MutationId = "data-validation.mutation.updateRule";

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Add rule transforms
    registry.register_symmetric_ref(ADD_RULE_ID, create_identity());
    registry.register_bidirectional_ref(ADD_RULE_ID, REMOVE_RULE_ID, create_identity());
    registry.register_bidirectional_ref(ADD_RULE_ID, UPDATE_RULE_ID, create_identity());

    // Remove rule transforms
    registry.register_symmetric_ref(REMOVE_RULE_ID, create_identity());
    registry.register_bidirectional_ref(REMOVE_RULE_ID, UPDATE_RULE_ID, create_identity());

    // Update rule transforms
    registry.register_symmetric_ref(UPDATE_RULE_ID, create_lww());

    // With sheet operations
    registry.register_identity(ADD_RULE_ID, "sheet.mutation.insert-row");
    registry.register_identity(ADD_RULE_ID, "sheet.mutation.insert-col");
    registry.register_identity(ADD_RULE_ID, "sheet.mutation.set-range-values");

    registry.register_identity(REMOVE_RULE_ID, "sheet.mutation.insert-row");
    registry.register_identity(REMOVE_RULE_ID, "sheet.mutation.set-range-values");

    registry.register_identity(UPDATE_RULE_ID, "sheet.mutation.insert-row");
    registry.register_identity(UPDATE_RULE_ID, "sheet.mutation.set-range-values");
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
