use crate::mutations::sheets::ToggleGridlinesMutation;
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::utils::transform_helpers::lww_transform;
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef};

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
}

pub fn register_cross_module_transforms(registry: &mut TransformRegistry, other_mutations: &[MutationId]) {
    for &other_id in other_mutations {
        registry.register_identity(TOGGLE_GRIDLINES_ID, other_id);
    }
}