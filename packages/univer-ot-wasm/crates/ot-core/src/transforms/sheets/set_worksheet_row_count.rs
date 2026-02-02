use crate::mutations::sheets::SetWorksheetRowCountMutation;
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::transform_helpers::lww_transform;

pub const MUTATION_ID: MutationId = SetWorksheetRowCountMutation::ID;

/// Register transforms for SetWorksheetRowCountMutation
///
/// Mutation ID: sheet.mutation.set-worksheet-row-count
///
/// SetWorksheetRowCountMutation sets the total row count for a worksheet.
/// Transform strategy: Last-Write-Wins (LWW) at worksheet level.
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transform: LWW
    registry.register_symmetric_ref(MUTATION_ID, lww_transform());

    // NOTE: No register_identity calls needed - registry falls back to identity automatically
}
