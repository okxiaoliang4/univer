//! Transforms for MarkDirtyRowAutoHeightMutation
//!
//! This mutation has `ranges: Vec<IRange>` field that needs shift transforms.

use crate::mutations::sheets::{MarkDirtyRowAutoHeightMutation, InsertRowMutation, RemoveRowMutation};
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::generic_params::GenericRangesParams;
use crate::utils::transform_helpers::lww_transform;
use crate::utils::shared_transforms as shared;

pub const MUTATION_ID: MutationId = MarkDirtyRowAutoHeightMutation::ID;

/// Register transforms for MarkDirtyRowAutoHeightMutation
///
/// Mutation ID: sheet.operation.mark-dirty-row-auto-height
///
/// MarkDirtyRowAutoHeightMutation marks rows as needing auto-height recalculation.
/// Transform strategy: Last-Write-Wins (LWW) at worksheet level.
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transform: LWW
    registry.register_symmetric_ref(MUTATION_ID, lww_transform());

    // Shift transforms for MarkDirtyRowAutoHeight (row-only operations)
    registry.register_bidirectional_ref(InsertRowMutation::ID, MUTATION_ID, shared::insert_row_shift::<GenericRangesParams>());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, MUTATION_ID, shared::remove_row_shift::<GenericRangesParams>());
}
