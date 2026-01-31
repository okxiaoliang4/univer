use crate::mutations::sheets::SetGridlinesColorMutation;
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::utils::transform_helpers::lww_transform;
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef};

pub const SET_GRIDLINES_COLOR_ID: MutationId = SetGridlinesColorMutation::ID;

/// Register transforms for SetGridlinesColorMutation
///
/// Mutation ID: sheet.mutation.set-gridlines-color
///
/// SetGridlinesColorMutation sets the gridlines color for a worksheet.
/// Transform strategy: Last-Write-Wins (LWW) at worksheet level.
/// Identity with all other mutations (gridlines color is independent).
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transform: LWW
    registry.register_symmetric_ref(SET_GRIDLINES_COLOR_ID, lww_transform());
}

pub fn register_cross_module_transforms(registry: &mut TransformRegistry, other_mutations: &[MutationId]) {
    for &other_id in other_mutations {
        registry.register_identity(SET_GRIDLINES_COLOR_ID, other_id);
    }
}