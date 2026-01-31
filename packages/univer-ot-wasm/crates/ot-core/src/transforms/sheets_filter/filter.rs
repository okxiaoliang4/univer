use crate::mutations::sheets_filter::{
    SetSheetsFilterRangeMutation, SetSheetsFilterCriteriaMutation,
    RemoveSheetsFilterMutation, ReCalcSheetsFilterMutation,
};
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::utils::transform_helpers::{lww_transform, identity_transform};
use crate::types::{MutationInfo, TransformResultRef};
use crate::transforms::constants::*;

pub const SET_FILTER_RANGE_ID: MutationId = SetSheetsFilterRangeMutation::ID;
pub const SET_FILTER_CRITERIA_ID: MutationId = SetSheetsFilterCriteriaMutation::ID;
pub const REMOVE_FILTER_ID: MutationId = RemoveSheetsFilterMutation::ID;
pub const RECALC_FILTER_ID: MutationId = ReCalcSheetsFilterMutation::ID;

const LOCAL_FILTER_MUTATIONS: &[MutationId] = &[
    SET_FILTER_RANGE_ID,
    SET_FILTER_CRITERIA_ID,
    REMOVE_FILTER_ID,
    RECALC_FILTER_ID,
];

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

    // Register identity transforms with ALL sheets core mutations (using constants)
    for &filter_id in LOCAL_FILTER_MUTATIONS {
        for &sheet_id in ALL_SHEETS_CORE_MUTATIONS {
            registry.register_identity(filter_id, sheet_id);
        }
        for &dv_id in DATA_VALIDATION_MUTATIONS {
            registry.register_identity(filter_id, dv_id);
        }
        for &cf_id in CONDITIONAL_FORMATTING_MUTATIONS {
            registry.register_identity(filter_id, cf_id);
        }
    }
}