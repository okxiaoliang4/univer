use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::types::{MutationInfo, TransformResultRef};
use crate::transforms::constants::*;
use std::sync::Arc;

pub const ADD_PIVOT_TABLE_ID: MutationId = "sheet.mutation.add-pivot-table";
pub const REMOVE_PIVOT_TABLE_ID: MutationId = "sheet.mutation.remove-pivot-table";
pub const SET_SOURCE_RANGE_ID: MutationId = "sheet.mutation.set-pivot-table-source-range";
pub const SET_TARGET_CELL_ID: MutationId = "sheet.mutation.set-pivot-table-target-cell";
pub const SET_FIELDS_CONFIG_ID: MutationId = "sheet.mutation.set-pivot-table-fields-config";
pub const SET_CALCULATED_DATA_ID: MutationId = "sheet.mutation.set-pivot-table-calculated-data";

const LOCAL_PIVOT_TABLE_MUTATIONS: &[MutationId] = &[
    ADD_PIVOT_TABLE_ID,
    REMOVE_PIVOT_TABLE_ID,
    SET_SOURCE_RANGE_ID,
    SET_TARGET_CELL_ID,
    SET_FIELDS_CONFIG_ID,
    SET_CALCULATED_DATA_ID,
];

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Register symmetric transforms for pivot table mutations
    registry.register_symmetric_ref(ADD_PIVOT_TABLE_ID, create_identity());
    registry.register_symmetric_ref(REMOVE_PIVOT_TABLE_ID, create_identity());
    registry.register_symmetric_ref(SET_SOURCE_RANGE_ID, create_lww());
    registry.register_symmetric_ref(SET_TARGET_CELL_ID, create_lww());
    registry.register_symmetric_ref(SET_FIELDS_CONFIG_ID, create_lww());
    registry.register_symmetric_ref(SET_CALCULATED_DATA_ID, create_lww());

    // Register bidirectional transforms within pivot table module
    registry.register_bidirectional_ref(ADD_PIVOT_TABLE_ID, REMOVE_PIVOT_TABLE_ID, create_identity());
    registry.register_bidirectional_ref(ADD_PIVOT_TABLE_ID, SET_SOURCE_RANGE_ID, create_identity());
    registry.register_bidirectional_ref(ADD_PIVOT_TABLE_ID, SET_TARGET_CELL_ID, create_identity());
    registry.register_bidirectional_ref(ADD_PIVOT_TABLE_ID, SET_FIELDS_CONFIG_ID, create_identity());
    registry.register_bidirectional_ref(ADD_PIVOT_TABLE_ID, SET_CALCULATED_DATA_ID, create_identity());
    registry.register_bidirectional_ref(REMOVE_PIVOT_TABLE_ID, SET_SOURCE_RANGE_ID, create_identity());
    registry.register_bidirectional_ref(REMOVE_PIVOT_TABLE_ID, SET_TARGET_CELL_ID, create_identity());
    registry.register_bidirectional_ref(REMOVE_PIVOT_TABLE_ID, SET_FIELDS_CONFIG_ID, create_identity());
    registry.register_bidirectional_ref(REMOVE_PIVOT_TABLE_ID, SET_CALCULATED_DATA_ID, create_identity());
    registry.register_bidirectional_ref(SET_SOURCE_RANGE_ID, SET_TARGET_CELL_ID, create_identity());
    registry.register_bidirectional_ref(SET_SOURCE_RANGE_ID, SET_FIELDS_CONFIG_ID, create_identity());
    registry.register_bidirectional_ref(SET_SOURCE_RANGE_ID, SET_CALCULATED_DATA_ID, create_identity());
    registry.register_bidirectional_ref(SET_TARGET_CELL_ID, SET_FIELDS_CONFIG_ID, create_identity());
    registry.register_bidirectional_ref(SET_TARGET_CELL_ID, SET_CALCULATED_DATA_ID, create_identity());
    registry.register_bidirectional_ref(SET_FIELDS_CONFIG_ID, SET_CALCULATED_DATA_ID, create_identity());

    // Register with all other modules
    for &pt_id in LOCAL_PIVOT_TABLE_MUTATIONS {
        for &sheet_id in ALL_SHEETS_CORE_MUTATIONS {
            registry.register_identity(pt_id, sheet_id);
        }
        for &dv_id in DATA_VALIDATION_MUTATIONS {
            registry.register_identity(pt_id, dv_id);
        }
        for &cf_id in CONDITIONAL_FORMATTING_MUTATIONS {
            registry.register_identity(pt_id, cf_id);
        }
        for &filter_id in FILTER_MUTATIONS {
            registry.register_identity(pt_id, filter_id);
        }
        for &hl_id in HYPER_LINK_MUTATIONS {
            registry.register_identity(pt_id, hl_id);
        }
        for &note_id in NOTE_MUTATIONS {
            registry.register_identity(pt_id, note_id);
        }
        for &table_id in TABLE_MUTATIONS {
            registry.register_identity(pt_id, table_id);
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
