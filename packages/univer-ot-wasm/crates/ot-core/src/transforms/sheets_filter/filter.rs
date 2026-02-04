use crate::mutations::sheets::{
    InsertRowMutation, InsertColMutation, RemoveRowMutation, RemoveColMutation,
    MoveRowsMutation, MoveColsMutation, MoveRangeMutation, RemoveSheetMutation,
};
use crate::mutations::sheets_filter::{
    SetSheetsFilterRangeMutation, SetSheetsFilterCriteriaMutation,
    RemoveSheetsFilterMutation, ReCalcSheetsFilterMutation,
};
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::generic_params::{GenericRangeParams, GenericColParams};
use crate::utils::shared_transforms as shared;
use crate::utils::transform_helpers::lww_transform;

pub const SET_FILTER_RANGE_ID: MutationId = SetSheetsFilterRangeMutation::ID;
pub const SET_FILTER_CRITERIA_ID: MutationId = SetSheetsFilterCriteriaMutation::ID;
pub const REMOVE_FILTER_ID: MutationId = RemoveSheetsFilterMutation::ID;
pub const RECALC_FILTER_ID: MutationId = ReCalcSheetsFilterMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Register symmetric transforms for filter mutations
    registry.register_symmetric_ref(SET_FILTER_RANGE_ID, lww_transform());
    registry.register_symmetric_ref(SET_FILTER_CRITERIA_ID, lww_transform());

    // Shift transforms for SetSheetsFilterRange (has range: IRange)
    registry.register_bidirectional_ref(InsertRowMutation::ID, SET_FILTER_RANGE_ID, shared::insert_row_shift::<GenericRangeParams>());
    registry.register_bidirectional_ref(InsertColMutation::ID, SET_FILTER_RANGE_ID, shared::insert_col_shift::<GenericRangeParams>());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, SET_FILTER_RANGE_ID, shared::remove_row_shift::<GenericRangeParams>());
    registry.register_bidirectional_ref(RemoveColMutation::ID, SET_FILTER_RANGE_ID, shared::remove_col_shift::<GenericRangeParams>());
    registry.register_bidirectional_ref(MoveRowsMutation::ID, SET_FILTER_RANGE_ID, shared::move_rows_shift::<GenericRangeParams>());
    registry.register_bidirectional_ref(MoveColsMutation::ID, SET_FILTER_RANGE_ID, shared::move_cols_shift::<GenericRangeParams>());
    registry.register_bidirectional_ref(MoveRangeMutation::ID, SET_FILTER_RANGE_ID, shared::move_range_shift::<GenericRangeParams>());
    registry.register_bidirectional_ref(RemoveSheetMutation::ID, SET_FILTER_RANGE_ID, shared::remove_sheet_shift::<GenericRangeParams>());

    // Shift transforms for SetSheetsFilterCriteria (has col: i32)
    // Only affected by column operations
    registry.register_bidirectional_ref(InsertColMutation::ID, SET_FILTER_CRITERIA_ID, shared::insert_col_shift::<GenericColParams>());
    registry.register_bidirectional_ref(RemoveColMutation::ID, SET_FILTER_CRITERIA_ID, shared::remove_col_shift::<GenericColParams>());
    registry.register_bidirectional_ref(MoveColsMutation::ID, SET_FILTER_CRITERIA_ID, shared::move_cols_shift::<GenericColParams>());
    registry.register_bidirectional_ref(RemoveSheetMutation::ID, SET_FILTER_CRITERIA_ID, shared::remove_sheet_shift::<GenericColParams>());
}