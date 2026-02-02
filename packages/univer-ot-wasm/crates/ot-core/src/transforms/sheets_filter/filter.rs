use crate::mutations::sheets_filter::{
    SetSheetsFilterRangeMutation, SetSheetsFilterCriteriaMutation,
    RemoveSheetsFilterMutation, ReCalcSheetsFilterMutation,
};
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::transform_helpers::{lww_transform, identity_transform};

pub const SET_FILTER_RANGE_ID: MutationId = SetSheetsFilterRangeMutation::ID;
pub const SET_FILTER_CRITERIA_ID: MutationId = SetSheetsFilterCriteriaMutation::ID;
pub const REMOVE_FILTER_ID: MutationId = RemoveSheetsFilterMutation::ID;
pub const RECALC_FILTER_ID: MutationId = ReCalcSheetsFilterMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Register symmetric transforms for filter mutations
    registry.register_symmetric_ref(SET_FILTER_RANGE_ID, lww_transform());
    registry.register_symmetric_ref(SET_FILTER_CRITERIA_ID, identity_transform());
    registry.register_symmetric_ref(REMOVE_FILTER_ID, identity_transform());
    registry.register_symmetric_ref(RECALC_FILTER_ID, identity_transform());

    // Register bidirectional transforms within filter module
    registry.register_bidirectional_ref(SET_FILTER_RANGE_ID, SET_FILTER_CRITERIA_ID, identity_transform());
    registry.register_bidirectional_ref(SET_FILTER_RANGE_ID, REMOVE_FILTER_ID, identity_transform());
    registry.register_bidirectional_ref(SET_FILTER_RANGE_ID, RECALC_FILTER_ID, identity_transform());
    registry.register_bidirectional_ref(SET_FILTER_CRITERIA_ID, REMOVE_FILTER_ID, identity_transform());
    registry.register_bidirectional_ref(SET_FILTER_CRITERIA_ID, RECALC_FILTER_ID, identity_transform());
    registry.register_bidirectional_ref(REMOVE_FILTER_ID, RECALC_FILTER_ID, identity_transform());

    // NOTE: No register_identity calls needed - registry falls back to identity automatically
}