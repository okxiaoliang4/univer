use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::types::{MutationInfo, TransformResultRef};
use crate::transforms::constants::*;
use std::sync::Arc;

pub const ADD_HYPER_LINK_ID: MutationId = "sheets.mutation.add-hyper-link";
pub const REMOVE_HYPER_LINK_ID: MutationId = "sheets.mutation.remove-hyper-link";
pub const UPDATE_HYPER_LINK_ID: MutationId = "sheets.mutation.update-hyper-link";
pub const UPDATE_HYPER_LINK_REF_ID: MutationId = "sheets.mutation.update-hyper-link-ref";
pub const UPDATE_RICH_HYPER_LINK_ID: MutationId = "sheets.mutation.update-rich-hyper-link";

const LOCAL_HYPER_LINK_MUTATIONS: &[MutationId] = &[
    ADD_HYPER_LINK_ID,
    REMOVE_HYPER_LINK_ID,
    UPDATE_HYPER_LINK_ID,
    UPDATE_HYPER_LINK_REF_ID,
    UPDATE_RICH_HYPER_LINK_ID,
];

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Register symmetric transforms for hyper link mutations
    registry.register_symmetric_ref(ADD_HYPER_LINK_ID, create_identity());
    registry.register_symmetric_ref(REMOVE_HYPER_LINK_ID, create_identity());
    registry.register_symmetric_ref(UPDATE_HYPER_LINK_ID, create_lww());
    registry.register_symmetric_ref(UPDATE_HYPER_LINK_REF_ID, create_lww());
    registry.register_symmetric_ref(UPDATE_RICH_HYPER_LINK_ID, create_lww());

    // Register bidirectional transforms within hyper link module
    registry.register_bidirectional_ref(ADD_HYPER_LINK_ID, REMOVE_HYPER_LINK_ID, create_identity());
    registry.register_bidirectional_ref(ADD_HYPER_LINK_ID, UPDATE_HYPER_LINK_ID, create_identity());
    registry.register_bidirectional_ref(ADD_HYPER_LINK_ID, UPDATE_HYPER_LINK_REF_ID, create_identity());
    registry.register_bidirectional_ref(ADD_HYPER_LINK_ID, UPDATE_RICH_HYPER_LINK_ID, create_identity());
    registry.register_bidirectional_ref(REMOVE_HYPER_LINK_ID, UPDATE_HYPER_LINK_ID, create_identity());
    registry.register_bidirectional_ref(REMOVE_HYPER_LINK_ID, UPDATE_HYPER_LINK_REF_ID, create_identity());
    registry.register_bidirectional_ref(REMOVE_HYPER_LINK_ID, UPDATE_RICH_HYPER_LINK_ID, create_identity());
    registry.register_bidirectional_ref(UPDATE_HYPER_LINK_ID, UPDATE_HYPER_LINK_REF_ID, create_identity());
    registry.register_bidirectional_ref(UPDATE_HYPER_LINK_ID, UPDATE_RICH_HYPER_LINK_ID, create_identity());
    registry.register_bidirectional_ref(UPDATE_HYPER_LINK_REF_ID, UPDATE_RICH_HYPER_LINK_ID, create_identity());

    // Register with all other modules (using constants)
    for &hl_id in LOCAL_HYPER_LINK_MUTATIONS {
        for &sheet_id in ALL_SHEETS_CORE_MUTATIONS {
            registry.register_identity(hl_id, sheet_id);
        }
        for &dv_id in DATA_VALIDATION_MUTATIONS {
            registry.register_identity(hl_id, dv_id);
        }
        for &cf_id in CONDITIONAL_FORMATTING_MUTATIONS {
            registry.register_identity(hl_id, cf_id);
        }
        for &filter_id in FILTER_MUTATIONS {
            registry.register_identity(hl_id, filter_id);
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
