use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::types::{MutationInfo, TransformResultRef};
use crate::transforms::constants::*;
use std::sync::Arc;

pub const ADD_TABLE_ID: MutationId = "sheet.mutation.add-table";
pub const SET_TABLE_ID: MutationId = "sheet.mutation.set-sheet-table";
pub const SET_TABLE_FILTER_ID: MutationId = "sheet.mutation.set-table-filter";
pub const DELETE_TABLE_ID: MutationId = "sheet.mutation.delete-table";

const LOCAL_TABLE_MUTATIONS: &[MutationId] = &[
    ADD_TABLE_ID,
    SET_TABLE_ID,
    SET_TABLE_FILTER_ID,
    DELETE_TABLE_ID,
];

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Register symmetric transforms for table mutations
    registry.register_symmetric_ref(ADD_TABLE_ID, create_identity());
    registry.register_symmetric_ref(SET_TABLE_ID, create_lww());
    registry.register_symmetric_ref(SET_TABLE_FILTER_ID, create_lww());
    registry.register_symmetric_ref(DELETE_TABLE_ID, create_identity());

    // Register bidirectional transforms within table module
    registry.register_bidirectional_ref(ADD_TABLE_ID, SET_TABLE_ID, create_identity());
    registry.register_bidirectional_ref(ADD_TABLE_ID, SET_TABLE_FILTER_ID, create_identity());
    registry.register_bidirectional_ref(ADD_TABLE_ID, DELETE_TABLE_ID, create_identity());
    registry.register_bidirectional_ref(SET_TABLE_ID, SET_TABLE_FILTER_ID, create_identity());
    registry.register_bidirectional_ref(SET_TABLE_ID, DELETE_TABLE_ID, create_identity());
    registry.register_bidirectional_ref(SET_TABLE_FILTER_ID, DELETE_TABLE_ID, create_identity());

    // Register with all other modules (using constants)
    for &table_id in LOCAL_TABLE_MUTATIONS {
        for &sheet_id in ALL_SHEETS_CORE_MUTATIONS {
            registry.register_identity(table_id, sheet_id);
        }
        for &dv_id in DATA_VALIDATION_MUTATIONS {
            registry.register_identity(table_id, dv_id);
        }
        for &cf_id in CONDITIONAL_FORMATTING_MUTATIONS {
            registry.register_identity(table_id, cf_id);
        }
        for &filter_id in FILTER_MUTATIONS {
            registry.register_identity(table_id, filter_id);
        }
        for &hl_id in HYPER_LINK_MUTATIONS {
            registry.register_identity(table_id, hl_id);
        }
        for &note_id in NOTE_MUTATIONS {
            registry.register_identity(table_id, note_id);
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
