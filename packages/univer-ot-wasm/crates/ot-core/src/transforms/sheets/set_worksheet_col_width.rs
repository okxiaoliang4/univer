//! Transforms for SetWorksheetColWidthMutation
//!
//! This mutation has `ranges: Vec<IRange>` field that needs shift transforms.

use crate::mutations::sheets::{SetWorksheetColWidthMutation, InsertColMutation, RemoveColMutation};
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::generic_params::GenericRangesParams;
use crate::utils::transform_helpers::lww_transform;
use crate::utils::shared_transforms as shared;

pub const MUTATION_ID: MutationId = SetWorksheetColWidthMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transform: LWW strategy
    registry.register_symmetric_ref(MUTATION_ID, lww_transform());

    // Shift transforms for SetWorksheetColWidth (col-only operations)
    registry.register_bidirectional_ref(InsertColMutation::ID, MUTATION_ID, shared::insert_col_shift::<GenericRangesParams>());
    registry.register_bidirectional_ref(RemoveColMutation::ID, MUTATION_ID, shared::remove_col_shift::<GenericRangesParams>());
}
