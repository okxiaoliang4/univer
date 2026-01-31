use crate::mutations::sheets::{SetColVisibleMutation, SetColHiddenMutation};
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::transform_helpers::{lww_transform, identity_transform};

pub const SET_COL_VISIBLE_ID: MutationId = SetColVisibleMutation::ID;
pub const SET_COL_HIDDEN_ID: MutationId = SetColHiddenMutation::ID;

/// Register transforms for SetColVisibleMutation and SetColHiddenMutation
///
/// Mutation IDs:
/// - sheet.mutation.set-col-visible
/// - sheet.mutation.set-col-hidden
///
/// These mutations control column visibility (show/hide).
/// Transform strategy: Last-Write-Wins (LWW) at worksheet level.
/// Identity with each other (visible vs hidden are complementary operations).
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transforms: LWW
    registry.register_symmetric_ref(SET_COL_VISIBLE_ID, lww_transform());
    registry.register_symmetric_ref(SET_COL_HIDDEN_ID, lww_transform());

    // Bidirectional: identity (complementary operations)
    registry.register_bidirectional_ref(SET_COL_VISIBLE_ID, SET_COL_HIDDEN_ID, identity_transform());
}

pub fn register_cross_module_transforms(registry: &mut TransformRegistry, other_mutations: &[MutationId]) {
    for &other_id in other_mutations {
        registry.register_identity(SET_COL_VISIBLE_ID, other_id);
        registry.register_identity(SET_COL_HIDDEN_ID, other_id);
    }
}
