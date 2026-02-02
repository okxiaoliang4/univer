use crate::mutations::sheets::SetRangeThemeMutation;
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::transform_helpers::lww_transform;

pub const MUTATION_ID: MutationId = SetRangeThemeMutation::ID;

/// Register transforms for SetRangeThemeMutation
///
/// Mutation ID: sheet.mutation.set-range-theme
///
/// SetRangeThemeMutation applies a theme to a range.
/// Transform strategy: Last-Write-Wins (LWW) at range level.
/// Identity with all other mutations (theme is independent).
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transform: LWW
    registry.register_symmetric_ref(MUTATION_ID, lww_transform());

    // NOTE: No register_identity calls needed - registry falls back to identity automatically
}
