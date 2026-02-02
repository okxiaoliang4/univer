//! Transforms for ReorderRangeMutation
//!
//! This mutation has a single `range: IRange` field that needs shift transforms.

use crate::mutations::sheets::{ReorderRangeMutation, InsertRowMutation, InsertColMutation, RemoveRowMutation, RemoveColMutation};
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::generic_params::GenericRangeParams;
use crate::utils::shared_transforms as shared;

pub const REORDER_RANGE_ID: MutationId = ReorderRangeMutation::ID;

/// Register transforms for ReorderRangeMutation
///
/// Mutation ID: sheet.mutation.reorder-range
///
/// ReorderRangeMutation reorders cells within a range.
///
/// Shift transforms: The range must be adjusted when rows/columns are inserted/removed.
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Shift transforms for ReorderRange (single range)
    registry.register_bidirectional_ref(InsertRowMutation::ID, REORDER_RANGE_ID, shared::insert_row_shift::<GenericRangeParams>());
    registry.register_bidirectional_ref(InsertColMutation::ID, REORDER_RANGE_ID, shared::insert_col_shift::<GenericRangeParams>());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, REORDER_RANGE_ID, shared::remove_row_shift::<GenericRangeParams>());
    registry.register_bidirectional_ref(RemoveColMutation::ID, REORDER_RANGE_ID, shared::remove_col_shift::<GenericRangeParams>());
}
