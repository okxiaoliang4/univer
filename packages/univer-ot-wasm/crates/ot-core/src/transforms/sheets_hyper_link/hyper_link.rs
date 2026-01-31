use crate::mutations::sheets_hyper_link::{
    AddHyperLinkMutation, RemoveHyperLinkMutation, UpdateHyperLinkMutation,
    UpdateHyperLinkRefMutation, UpdateRichHyperLinkMutation,
};
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::utils::transform_helpers::{lww_transform, identity_transform};
use crate::types::{MutationInfo, TransformResultRef};
use crate::transforms::constants::*;

pub const ADD_HYPER_LINK_ID: MutationId = AddHyperLinkMutation::ID;
pub const REMOVE_HYPER_LINK_ID: MutationId = RemoveHyperLinkMutation::ID;
pub const UPDATE_HYPER_LINK_ID: MutationId = UpdateHyperLinkMutation::ID;
pub const UPDATE_HYPER_LINK_REF_ID: MutationId = UpdateHyperLinkRefMutation::ID;
pub const UPDATE_RICH_HYPER_LINK_ID: MutationId = UpdateRichHyperLinkMutation::ID;

const LOCAL_HYPER_LINK_MUTATIONS: &[MutationId] = &[
    ADD_HYPER_LINK_ID,
    REMOVE_HYPER_LINK_ID,
    UPDATE_HYPER_LINK_ID,
    UPDATE_HYPER_LINK_REF_ID,
    UPDATE_RICH_HYPER_LINK_ID,
];

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Register symmetric transforms for hyper link mutations
    registry.register_symmetric_ref(ADD_HYPER_LINK_ID, identity_transform());
    registry.register_symmetric_ref(REMOVE_HYPER_LINK_ID, identity_transform());
    registry.register_symmetric_ref(UPDATE_HYPER_LINK_ID, lww_transform());
    registry.register_symmetric_ref(UPDATE_HYPER_LINK_REF_ID, lww_transform());
    registry.register_symmetric_ref(UPDATE_RICH_HYPER_LINK_ID, lww_transform());

    // Register bidirectional transforms within hyper link module
    registry.register_bidirectional_ref(ADD_HYPER_LINK_ID, REMOVE_HYPER_LINK_ID, identity_transform());
    registry.register_bidirectional_ref(ADD_HYPER_LINK_ID, UPDATE_HYPER_LINK_ID, identity_transform());
    registry.register_bidirectional_ref(ADD_HYPER_LINK_ID, UPDATE_HYPER_LINK_REF_ID, identity_transform());
    registry.register_bidirectional_ref(ADD_HYPER_LINK_ID, UPDATE_RICH_HYPER_LINK_ID, identity_transform());
    registry.register_bidirectional_ref(REMOVE_HYPER_LINK_ID, UPDATE_HYPER_LINK_ID, identity_transform());
    registry.register_bidirectional_ref(REMOVE_HYPER_LINK_ID, UPDATE_HYPER_LINK_REF_ID, identity_transform());
    registry.register_bidirectional_ref(REMOVE_HYPER_LINK_ID, UPDATE_RICH_HYPER_LINK_ID, identity_transform());
    registry.register_bidirectional_ref(UPDATE_HYPER_LINK_ID, UPDATE_HYPER_LINK_REF_ID, identity_transform());
    registry.register_bidirectional_ref(UPDATE_HYPER_LINK_ID, UPDATE_RICH_HYPER_LINK_ID, identity_transform());
    registry.register_bidirectional_ref(UPDATE_HYPER_LINK_REF_ID, UPDATE_RICH_HYPER_LINK_ID, identity_transform());

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