//! Transforms for RemoveWorksheetMergeMutation
//!
//! Merge ranges (Vec<IRange>) need to be adjusted when rows/columns are inserted or removed.
//! Uses shared Generic transforms for position shifting.

use crate::mutations::sheets::{
    RemoveWorksheetMergeMutation, InsertRowMutation, InsertColMutation,
    RemoveRowMutation, RemoveColMutation,
};
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::generic_params::GenericRangesParams;
use crate::utils::shared_transforms as shared;

pub const MUTATION_ID: MutationId = RemoveWorksheetMergeMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Bidirectional: InsertRow vs RemoveWorksheetMerge (ranges need row shift)
    registry.register_bidirectional_ref(
        InsertRowMutation::ID,
        MUTATION_ID,
        shared::insert_row_shift::<GenericRangesParams>(),
    );

    // Bidirectional: InsertCol vs RemoveWorksheetMerge (ranges need column shift)
    registry.register_bidirectional_ref(
        InsertColMutation::ID,
        MUTATION_ID,
        shared::insert_col_shift::<GenericRangesParams>(),
    );

    // Bidirectional: RemoveRow vs RemoveWorksheetMerge (ranges need row shift/removal)
    registry.register_bidirectional_ref(
        RemoveRowMutation::ID,
        MUTATION_ID,
        shared::remove_row_shift::<GenericRangesParams>(),
    );

    // Bidirectional: RemoveCol vs RemoveWorksheetMerge (ranges need column shift/removal)
    registry.register_bidirectional_ref(
        RemoveColMutation::ID,
        MUTATION_ID,
        shared::remove_col_shift::<GenericRangesParams>(),
    );
}
