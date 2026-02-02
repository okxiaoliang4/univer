use crate::mutations::sheets::{SetRowVisibleMutation, SetRowHiddenMutation, InsertRowMutation, RemoveRowMutation};
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::generic_params::GenericRangesParams;
use crate::utils::transform_helpers::lww_transform;
use crate::utils::shared_transforms as shared;

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
///
/// Shift transforms: Row visibility ranges must be adjusted when rows are inserted/removed.
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transforms: LWW
    registry.register_symmetric_ref(SET_ROW_VISIBLE_ID, lww_transform());
    registry.register_symmetric_ref(SET_ROW_HIDDEN_ID, lww_transform());

    // Shift transforms for SetRowVisible (row-only operations)
    registry.register_bidirectional_ref(InsertRowMutation::ID, SET_ROW_VISIBLE_ID, shared::insert_row_shift::<GenericRangesParams>());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, SET_ROW_VISIBLE_ID, shared::remove_row_shift::<GenericRangesParams>());

    // Shift transforms for SetRowHidden (row-only operations)
    registry.register_bidirectional_ref(InsertRowMutation::ID, SET_ROW_HIDDEN_ID, shared::insert_row_shift::<GenericRangesParams>());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, SET_ROW_HIDDEN_ID, shared::remove_row_shift::<GenericRangesParams>());
}
