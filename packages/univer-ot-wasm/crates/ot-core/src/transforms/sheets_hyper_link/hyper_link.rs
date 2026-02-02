use crate::mutations::sheets_hyper_link::{
    AddHyperLinkMutation, RemoveHyperLinkMutation, UpdateHyperLinkMutation,
    UpdateHyperLinkRefMutation, UpdateRichHyperLinkMutation,
};
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::transform_helpers::{lww_transform, identity_transform};

pub const ADD_HYPER_LINK_ID: MutationId = AddHyperLinkMutation::ID;
pub const REMOVE_HYPER_LINK_ID: MutationId = RemoveHyperLinkMutation::ID;
pub const UPDATE_HYPER_LINK_ID: MutationId = UpdateHyperLinkMutation::ID;
pub const UPDATE_HYPER_LINK_REF_ID: MutationId = UpdateHyperLinkRefMutation::ID;
pub const UPDATE_RICH_HYPER_LINK_ID: MutationId = UpdateRichHyperLinkMutation::ID;

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

    // NOTE: No register_identity calls needed - registry falls back to identity automatically
}