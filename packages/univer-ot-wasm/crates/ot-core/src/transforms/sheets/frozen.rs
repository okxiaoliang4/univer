use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef};
use std::sync::Arc;

pub const MUTATION_ID: MutationId = "sheet.mutation.set-frozen";

pub fn register_transforms(registry: &mut TransformRegistry) {
    registry.register_symmetric_ref(MUTATION_ID, create_lww());
    registry.register_identity(MUTATION_ID, "sheet.mutation.insert-row");
    registry.register_identity(MUTATION_ID, "sheet.mutation.insert-col");
    registry.register_identity(MUTATION_ID, "sheet.mutation.set-range-values");

    // Identity transforms with non-interfering mutations
    registry.register_identity(MUTATION_ID, "sheet.mutation.remove-rows");
    registry.register_identity(MUTATION_ID, "sheet.mutation.remove-col");
    registry.register_identity(MUTATION_ID, "sheet.mutation.move-range");
    registry.register_identity(MUTATION_ID, "sheet.mutation.move-rows");
    registry.register_identity(MUTATION_ID, "sheet.mutation.move-columns");
    registry.register_identity(MUTATION_ID, "sheet.mutation.add-worksheet-merge");
    registry.register_identity(MUTATION_ID, "sheet.mutation.set-range-protection");
    registry.register_identity(MUTATION_ID, "sheet.mutation.set-range-theme");
    registry.register_identity(MUTATION_ID, "sheet.mutation.set.numfmt");
    registry.register_identity(MUTATION_ID, "sheet.mutation.set-row-data");
    registry.register_identity(MUTATION_ID, "sheet.mutation.insert-sheet");
    registry.register_identity(MUTATION_ID, "sheet.mutation.set-workbook-name");
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
