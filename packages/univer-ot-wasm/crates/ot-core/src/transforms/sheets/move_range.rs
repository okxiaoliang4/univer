use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::types::{MutationInfo, TransformResultRef};
use std::sync::Arc;

pub const MUTATION_ID: MutationId = "sheet.mutation.move-range";

pub fn register_transforms(registry: &mut TransformRegistry) {
    registry.register_symmetric_ref(MUTATION_ID, create_identity());
    registry.register_identity(MUTATION_ID, "sheet.mutation.insert-row");
    registry.register_identity(MUTATION_ID, "sheet.mutation.insert-col");
    registry.register_identity(MUTATION_ID, "sheet.mutation.set-range-values");

    // Identity transforms with non-interfering mutations
    registry.register_identity(MUTATION_ID, "sheet.mutation.remove-rows");
    registry.register_identity(MUTATION_ID, "sheet.mutation.remove-col");
    registry.register_identity(MUTATION_ID, "sheet.mutation.move-rows");
    registry.register_identity(MUTATION_ID, "sheet.mutation.move-columns");
    registry.register_identity(MUTATION_ID, "sheet.mutation.add-worksheet-merge");
    registry.register_identity(MUTATION_ID, "sheet.mutation.set-range-protection");
    registry.register_identity(MUTATION_ID, "sheet.mutation.set-range-theme");
    registry.register_identity(MUTATION_ID, "sheet.mutation.set.numfmt");
    registry.register_identity(MUTATION_ID, "sheet.mutation.set-frozen");
    registry.register_identity(MUTATION_ID, "sheet.mutation.set-row-data");
    registry.register_identity(MUTATION_ID, "sheet.mutation.insert-sheet");
    registry.register_identity(MUTATION_ID, "sheet.mutation.set-workbook-name");
}

fn create_identity() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        TransformResultRef::identity(m1, m2)  // Zero-copy!
    })
}
