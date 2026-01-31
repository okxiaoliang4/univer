use crate::mutations::sheets::SetWorksheetOrderMutation;
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::utils::transform_helpers::lww_transform;
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef};

pub const MUTATION_ID: MutationId = SetWorksheetOrderMutation::ID;

/// Register transforms for SetWorksheetOrderMutation
///
/// Mutation ID: sheet.mutation.set-worksheet-order
///
/// SetWorksheetOrderMutation changes the order of worksheets in the workbook.
/// Transform strategy: Last-Write-Wins (LWW) at workbook level.
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transform: LWW
    registry.register_symmetric_ref(MUTATION_ID, lww_transform());
}

pub fn register_cross_module_transforms(registry: &mut TransformRegistry, other_mutations: &[MutationId]) {
    for &other_id in other_mutations {
        registry.register_identity(MUTATION_ID, other_id);
    }
}