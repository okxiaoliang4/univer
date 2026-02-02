//! Transforms for AddWorksheetMergeMutation
//!
//! Merge ranges (Vec<IRange>) need to be adjusted when rows/columns are inserted or removed.
//! Uses shared Generic transforms for position shifting.

use crate::mutations::sheets::{
    AddWorksheetMergeMutation, InsertRowMutation, InsertColMutation,
    RemoveRowMutation, RemoveColMutation,
    MoveColsMutation, MoveRangeMutation, MoveRowsMutation, RemoveSheetMutation
};
use crate::registry::{TransformRegistry};
use crate::utils::GenericRangesParams;
use crate::utils::shared_transforms::{insert_col_shift, insert_row_shift, remove_col_shift, remove_row_shift, move_cols_shift, move_rows_shift, move_range_shift, remove_sheet_shift};


pub fn register_transforms(registry: &mut TransformRegistry) {
  registry.register_bidirectional_ref(InsertRowMutation::ID, AddWorksheetMergeMutation::ID, insert_row_shift::<GenericRangesParams>());
  registry.register_bidirectional_ref(InsertColMutation::ID, AddWorksheetMergeMutation::ID, insert_col_shift::<GenericRangesParams>());
  registry.register_bidirectional_ref(RemoveRowMutation::ID, AddWorksheetMergeMutation::ID, remove_row_shift::<GenericRangesParams>());
  registry.register_bidirectional_ref(RemoveColMutation::ID, AddWorksheetMergeMutation::ID, remove_col_shift::<GenericRangesParams>());
  registry.register_bidirectional_ref(MoveColsMutation::ID, AddWorksheetMergeMutation::ID, move_cols_shift::<GenericRangesParams>());
  registry.register_bidirectional_ref(MoveRowsMutation::ID, AddWorksheetMergeMutation::ID, move_rows_shift::<GenericRangesParams>());
  registry.register_bidirectional_ref(MoveRangeMutation::ID, AddWorksheetMergeMutation::ID, move_range_shift::<GenericRangesParams>());
  registry.register_bidirectional_ref(RemoveSheetMutation::ID, AddWorksheetMergeMutation::ID, remove_sheet_shift::<GenericRangesParams>());
}
