use crate::mutations::sheets::{SetRowVisibleMutation, SetRowHiddenMutation};
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::utils::transform_helpers::{lww_transform, identity_transform};
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef};

pub const SET_ROW_VISIBLE_ID: MutationId = SetRowVisibleMutation::ID;
pub const SET_ROW_HIDDEN_ID: MutationId = SetRowHiddenMutation::ID;

/// Register transforms for SetRowVisibleMutation and SetRowHiddenMutation
///
/// Mutation IDs:
/// - sheet.mutation.set-row-visible
/// - sheet.mutation.set-row-hidden
///
/// These mutations control row visibility (show/hide).
/// Transform strategy: Last-Write-Wins (LWW) at worksheet level.
/// Identity with each other (visible vs hidden are complementary operations).
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transforms: LWW
    registry.register_symmetric_ref(SET_ROW_VISIBLE_ID, lww_transform());
    registry.register_symmetric_ref(SET_ROW_HIDDEN_ID, lww_transform());

    // Bidirectional: identity (complementary operations)
    registry.register_bidirectional_ref(SET_ROW_VISIBLE_ID, SET_ROW_HIDDEN_ID, identity_transform());
}

pub fn register_cross_module_transforms(registry: &mut TransformRegistry, other_mutations: &[MutationId]) {
    for &other_id in other_mutations {
        registry.register_identity(SET_ROW_VISIBLE_ID, other_id);
        registry.register_identity(SET_ROW_HIDDEN_ID, other_id);
    }
}