use crate::params::MoveRowsMutationParams;
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::types::{MutationInfo, TransformResultRef};
use crate::utils::params::same_worksheet;
use std::sync::Arc;

pub const MUTATION_ID: MutationId = "sheet.mutation.move-rows";

/// Register all transforms for move-rows mutation
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transform
    registry.register_symmetric_ref(MUTATION_ID, create_self_transform());

    // Bidirectional with structural operations
    registry.register_bidirectional_ref(
        MUTATION_ID,
        "sheet.mutation.insert-row",
        create_identity(),
    );
    registry.register_bidirectional_ref(
        MUTATION_ID,
        "sheet.mutation.remove-rows",
        create_identity(),
    );
    registry.register_bidirectional_ref(
        MUTATION_ID,
        "sheet.mutation.set-range-values",
        create_identity(),
    );

    // Identity with column operations
    registry.register_identity(MUTATION_ID, "sheet.mutation.insert-col");
    registry.register_identity(MUTATION_ID, "sheet.mutation.remove-col");
    registry.register_identity(MUTATION_ID, "sheet.mutation.move-columns");

    // Identity transforms with non-interfering mutations
    registry.register_identity(MUTATION_ID, "sheet.mutation.move-range");
    registry.register_identity(MUTATION_ID, "sheet.mutation.add-worksheet-merge");
    registry.register_identity(MUTATION_ID, "sheet.mutation.set-range-protection");
    registry.register_identity(MUTATION_ID, "sheet.mutation.set-range-theme");
    registry.register_identity(MUTATION_ID, "sheet.mutation.set.numfmt");
    registry.register_identity(MUTATION_ID, "sheet.mutation.set-frozen");
    registry.register_identity(MUTATION_ID, "sheet.mutation.set-row-data");
    registry.register_identity(MUTATION_ID, "sheet.mutation.insert-sheet");
    registry.register_identity(MUTATION_ID, "sheet.mutation.set-workbook-name");
}

/// Helper to create identity transform result (zero-copy!)
#[inline]
fn identity<'a>(m1: &'a MutationInfo, m2: &'a MutationInfo) -> TransformResultRef<'a> {
    TransformResultRef::identity(m1, m2)
}

/// Helper to create error result (zero-copy for mutations)
#[inline]
fn parse_error<'a>(m1: &'a MutationInfo, m2: &'a MutationInfo, msg: &str) -> TransformResultRef<'a> {
    TransformResultRef::parse_error(m1, m2, msg)
}

fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        // Quick worksheet check BEFORE parsing (avoids clone for different worksheets)
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);  // Zero-copy!
        }

        let m1_params: MoveRowsMutationParams = match serde_json::from_value(m1.params.clone()) {
            Ok(p) => p,
            Err(_) => return parse_error(m1, m2, "Failed to parse m1 params"),
        };

        let m2_params: MoveRowsMutationParams = match serde_json::from_value(m2.params.clone()) {
            Ok(p) => p,
            Err(_) => return parse_error(m1, m2, "Failed to parse m2 params"),
        };

        // Double-check worksheet match (in case quick check returned None)
        if m1_params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
        {
            return identity(m1, m2);  // Zero-copy!
        }

        // Move operations are complex: they remove from source and insert at target
        // For simplicity, apply identity transform (both succeed independently)
        // In real-world, this would need more sophisticated handling
        identity(m1, m2)  // Zero-copy!
    })
}

fn create_identity() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        TransformResultRef::identity(m1, m2)  // Zero-copy!
    })
}
