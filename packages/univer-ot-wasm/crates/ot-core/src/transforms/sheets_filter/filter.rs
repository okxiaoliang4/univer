use crate::mutations::sheets::{InsertRowMutation, InsertColMutation, RemoveRowMutation, RemoveColMutation};
use crate::mutations::sheets_filter::{
    SetSheetsFilterRangeMutation, SetSheetsFilterCriteriaMutation,
    RemoveSheetsFilterMutation, ReCalcSheetsFilterMutation,
};
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::generic_params::GenericRangeParams;
use crate::utils::shared_transforms as shared;
use crate::utils::transform_helpers::lww_transform;

pub const SET_FILTER_RANGE_ID: MutationId = SetSheetsFilterRangeMutation::ID;
pub const SET_FILTER_CRITERIA_ID: MutationId = SetSheetsFilterCriteriaMutation::ID;
pub const REMOVE_FILTER_ID: MutationId = RemoveSheetsFilterMutation::ID;
pub const RECALC_FILTER_ID: MutationId = ReCalcSheetsFilterMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Register symmetric transforms for filter mutations
    registry.register_symmetric_ref(SET_FILTER_RANGE_ID, lww_transform());

    // Shift transforms for SetSheetsFilterRange (has range: IRange)
    registry.register_bidirectional_ref(InsertRowMutation::ID, SET_FILTER_RANGE_ID, shared::insert_row_shift::<GenericRangeParams>());
    registry.register_bidirectional_ref(InsertColMutation::ID, SET_FILTER_RANGE_ID, shared::insert_col_shift::<GenericRangeParams>());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, SET_FILTER_RANGE_ID, shared::remove_row_shift::<GenericRangeParams>());
    registry.register_bidirectional_ref(RemoveColMutation::ID, SET_FILTER_RANGE_ID, shared::remove_col_shift::<GenericRangeParams>());
}