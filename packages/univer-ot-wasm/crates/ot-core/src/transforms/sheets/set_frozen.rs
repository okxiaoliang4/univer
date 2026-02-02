use crate::mutations::sheets::SetFrozenMutation;
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::transform_helpers::lww_transform;

pub const MUTATION_ID: MutationId = SetFrozenMutation::ID;

/// Register transforms for SetFrozenMutation
///
/// Mutation ID: sheet.mutation.set-frozen
///
/// SetFrozenMutation sets the freeze panes configuration for a worksheet.
/// Transform strategy: Last-Write-Wins (LWW) at worksheet level.
/// Identity with all position-based mutations (frozen panes don't interfere with cell operations).
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transform: LWW
    registry.register_symmetric_ref(MUTATION_ID, lww_transform());

    // NOTE: No register_identity calls needed - registry falls back to identity automatically
}
