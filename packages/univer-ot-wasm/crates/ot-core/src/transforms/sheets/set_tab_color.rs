use crate::mutations::sheets::SetTabColorMutation;
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::transform_helpers::lww_transform;

pub const SET_TAB_COLOR_ID: MutationId = SetTabColorMutation::ID;

/// Register transforms for SetTabColorMutation
///
/// Mutation ID: sheet.mutation.set-tab-color
///
/// SetTabColorMutation sets the tab color for a worksheet.
/// Transform strategy: Last-Write-Wins (LWW) at worksheet level.
/// Identity with all other mutations (tab color is independent).
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transform: LWW
    registry.register_symmetric_ref(SET_TAB_COLOR_ID, lww_transform());

    // NOTE: No register_identity calls needed - registry falls back to identity automatically
}
