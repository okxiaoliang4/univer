//! Transforms for DeleteWorksheetRangeThemeStyleMutation
//!
//! This mutation has a single `range: IRange` field that needs shift transforms.

use crate::mutations::sheets::{DeleteWorksheetRangeThemeStyleMutation, InsertRowMutation, InsertColMutation, RemoveRowMutation, RemoveColMutation};
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::generic_params::GenericRangeParams;
use crate::utils::shared_transforms as shared;

pub const MUTATION_ID: MutationId = DeleteWorksheetRangeThemeStyleMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Shift transforms for DeleteWorksheetRangeThemeStyle (single range)
    registry.register_bidirectional_ref(InsertRowMutation::ID, MUTATION_ID, shared::insert_row_shift::<GenericRangeParams>());
    registry.register_bidirectional_ref(InsertColMutation::ID, MUTATION_ID, shared::insert_col_shift::<GenericRangeParams>());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, MUTATION_ID, shared::remove_row_shift::<GenericRangeParams>());
    registry.register_bidirectional_ref(RemoveColMutation::ID, MUTATION_ID, shared::remove_col_shift::<GenericRangeParams>());
}
