use crate::mutations::sheets::{
  SetWorksheetRangeThemeStyleMutation, InsertRowMutation, InsertColMutation,
  RemoveRowMutation, RemoveColMutation,
  MoveColsMutation, MoveRangeMutation, MoveRowsMutation, RemoveSheetMutation
};
use crate::registry::{TransformRegistry};
use crate::utils::GenericRangesParams;
use crate::utils::shared_transforms::{insert_col_shift, insert_row_shift, remove_col_shift, remove_row_shift, move_cols_shift, move_rows_shift, move_range_shift, remove_sheet_shift};
use crate::utils::transform_helpers::lww_transform;


pub fn register_transforms(registry: &mut TransformRegistry) {
// Register symmetric LWW transform for SetWorksheetRangeThemeStyle
registry.register_symmetric_ref(SetWorksheetRangeThemeStyleMutation::ID, lww_transform());

registry.register_bidirectional_ref(InsertRowMutation::ID, SetWorksheetRangeThemeStyleMutation::ID, insert_row_shift::<GenericRangesParams>());
registry.register_bidirectional_ref(InsertColMutation::ID, SetWorksheetRangeThemeStyleMutation::ID, insert_col_shift::<GenericRangesParams>());
registry.register_bidirectional_ref(RemoveRowMutation::ID, SetWorksheetRangeThemeStyleMutation::ID, remove_row_shift::<GenericRangesParams>());
registry.register_bidirectional_ref(RemoveColMutation::ID, SetWorksheetRangeThemeStyleMutation::ID, remove_col_shift::<GenericRangesParams>());
registry.register_bidirectional_ref(MoveColsMutation::ID, SetWorksheetRangeThemeStyleMutation::ID, move_cols_shift::<GenericRangesParams>());
registry.register_bidirectional_ref(MoveRowsMutation::ID, SetWorksheetRangeThemeStyleMutation::ID, move_rows_shift::<GenericRangesParams>());
registry.register_bidirectional_ref(MoveRangeMutation::ID, SetWorksheetRangeThemeStyleMutation::ID, move_range_shift::<GenericRangesParams>());
registry.register_bidirectional_ref(RemoveSheetMutation::ID, SetWorksheetRangeThemeStyleMutation::ID, remove_sheet_shift::<GenericRangesParams>());
}
