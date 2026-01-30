use crate::mutations::sheets_filter::{
    SetSheetsFilterRangeMutation, SetSheetsFilterCriteriaMutation,
    RemoveSheetsFilterMutation, ReCalcSheetsFilterMutation,
};
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::types::{MutationInfo, TransformResultRef};
use crate::transforms::constants::*;
use std::sync::Arc;

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
    registry.register_symmetric_ref(SET_FILTER_RANGE_ID, create_lww());
    registry.register_symmetric_ref(SET_FILTER_CRITERIA_ID, create_identity());
    registry.register_symmetric_ref(REMOVE_FILTER_ID, create_identity());
    registry.register_symmetric_ref(RECALC_FILTER_ID, create_identity());

    // Register bidirectional transforms within filter module
    registry.register_bidirectional_ref(SET_FILTER_RANGE_ID, SET_FILTER_CRITERIA_ID, create_identity());
    registry.register_bidirectional_ref(SET_FILTER_RANGE_ID, REMOVE_FILTER_ID, create_identity());
    registry.register_bidirectional_ref(SET_FILTER_RANGE_ID, RECALC_FILTER_ID, create_identity());
    registry.register_bidirectional_ref(SET_FILTER_CRITERIA_ID, REMOVE_FILTER_ID, create_identity());
    registry.register_bidirectional_ref(SET_FILTER_CRITERIA_ID, RECALC_FILTER_ID, create_identity());
    registry.register_bidirectional_ref(REMOVE_FILTER_ID, RECALC_FILTER_ID, create_identity());

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

fn create_identity() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        TransformResultRef::identity(m1, m2)
    })
}

fn create_lww() -> TransformFnRef {
    Arc::new(|_m1: &MutationInfo, m2: &MutationInfo| {
        use crate::types::MutationOutcome;
        TransformResultRef {
            m1_prime: MutationOutcome::Removed,
            m2_prime: MutationOutcome::Unchanged(m2),
            error: None,
        }
    })
}
