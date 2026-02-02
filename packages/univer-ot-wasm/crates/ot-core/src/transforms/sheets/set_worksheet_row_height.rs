//! Transforms for SetWorksheetRowHeightMutation
//!
//! This file handles three related mutations:
//! 1. SetWorksheetRowHeightMutation - Sets row height values (ranges: Vec<IRange>)
//! 2. SetWorksheetRowIsAutoHeightMutation - Toggles auto-height flag (ranges: Vec<IRange>)
//! 3. SetWorksheetRowAutoHeightMutation - Sets auto-calculated heights (rows_auto_height_info: Vec<IRowAutoHeightInfo>)
//!
//! All use LWW strategy for self-transforms.
//! The first two have `ranges: Vec<IRange>` field that needs shift transforms.
//! The third has a different structure and currently only needs LWW.

use crate::mutations::sheets::{SetWorksheetRowHeightMutation, SetWorksheetRowIsAutoHeightMutation, SetWorksheetRowAutoHeightMutation, InsertRowMutation, RemoveRowMutation};
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::generic_params::GenericRangesParams;
use crate::utils::transform_helpers::lww_transform;
use crate::utils::shared_transforms as shared;

pub const MUTATION_ID: MutationId = SetWorksheetRowHeightMutation::ID;
pub const MUTATION_ID_IS_AUTO_HEIGHT: MutationId = SetWorksheetRowIsAutoHeightMutation::ID;
pub const MUTATION_ID_AUTO_HEIGHT: MutationId = SetWorksheetRowAutoHeightMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    // SetWorksheetRowHeightMutation: LWW strategy
    registry.register_symmetric_ref(MUTATION_ID, lww_transform());

    // SetWorksheetRowIsAutoHeightMutation: LWW strategy
    registry.register_symmetric_ref(MUTATION_ID_IS_AUTO_HEIGHT, lww_transform());

    // SetWorksheetRowAutoHeightMutation: LWW strategy
    // Note: This mutation has rows_auto_height_info: Vec<IRowAutoHeightInfo> which is different
    // from ranges: Vec<IRange>. Shift transforms would need a custom implementation.
    registry.register_symmetric_ref(MUTATION_ID_AUTO_HEIGHT, lww_transform());

    // Shift transforms for SetWorksheetRowHeight (row-only operations)
    registry.register_bidirectional_ref(InsertRowMutation::ID, MUTATION_ID, shared::insert_row_shift::<GenericRangesParams>());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, MUTATION_ID, shared::remove_row_shift::<GenericRangesParams>());

    // Shift transforms for SetWorksheetRowIsAutoHeight (row-only operations)
    registry.register_bidirectional_ref(InsertRowMutation::ID, MUTATION_ID_IS_AUTO_HEIGHT, shared::insert_row_shift::<GenericRangesParams>());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, MUTATION_ID_IS_AUTO_HEIGHT, shared::remove_row_shift::<GenericRangesParams>());
}
