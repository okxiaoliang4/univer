use crate::mutations::sheets::SetWorksheetColumnCountMutation;
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::transform_helpers::lww_transform;

pub const MUTATION_ID: MutationId = SetWorksheetColumnCountMutation::ID;

/// Register transforms for SetWorksheetColumnCountMutation
///
/// Mutation ID: sheet.mutation.set-worksheet-column-count
///
/// SetWorksheetColumnCountMutation sets the total column count for a worksheet.
/// Transform strategy: Last-Write-Wins (LWW) at worksheet level.
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transform: LWW
    registry.register_symmetric_ref(MUTATION_ID, lww_transform());

    // NOTE: No register_identity calls needed - registry falls back to identity automatically
}
