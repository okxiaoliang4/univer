use crate::mutations::sheets::ToggleGridlinesMutation;
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::transform_helpers::lww_transform;

pub const TOGGLE_GRIDLINES_ID: MutationId = ToggleGridlinesMutation::ID;

/// Register transforms for ToggleGridlinesMutation
///
/// Mutation ID: sheet.mutation.toggle-gridlines
///
/// ToggleGridlinesMutation shows/hides gridlines for a worksheet.
/// Transform strategy: Last-Write-Wins (LWW) at worksheet level.
/// Identity with all other mutations (gridlines display is independent).
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transform: LWW
    registry.register_symmetric_ref(TOGGLE_GRIDLINES_ID, lww_transform());

    // NOTE: No register_identity calls needed - registry falls back to identity automatically
}
