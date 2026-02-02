use crate::mutations::sheets::SetWorksheetOrderMutation;
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::transform_helpers::lww_transform;

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

    // NOTE: No register_identity calls needed - registry falls back to identity automatically
}
