use crate::mutations::sheets::{
    AddWorksheetProtectionMutation, SetWorksheetProtectionMutation,
    DeleteWorksheetProtectionMutation, SetWorksheetPermissionPointsMutation,
};
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef};
use std::sync::Arc;

pub const ADD_WORKSHEET_PROTECTION_ID: MutationId = AddWorksheetProtectionMutation::ID;
pub const SET_WORKSHEET_PROTECTION_ID: MutationId = SetWorksheetProtectionMutation::ID;
pub const DELETE_WORKSHEET_PROTECTION_ID: MutationId = DeleteWorksheetProtectionMutation::ID;
pub const SET_PERMISSION_POINTS_ID: MutationId = SetWorksheetPermissionPointsMutation::ID;

const WORKSHEET_PROTECTION_MUTATIONS: &[MutationId] = &[
    ADD_WORKSHEET_PROTECTION_ID,
    SET_WORKSHEET_PROTECTION_ID,
    DELETE_WORKSHEET_PROTECTION_ID,
    SET_PERMISSION_POINTS_ID,
];

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Register symmetric transforms
    registry.register_symmetric_ref(ADD_WORKSHEET_PROTECTION_ID, create_identity());
    registry.register_symmetric_ref(SET_WORKSHEET_PROTECTION_ID, create_lww());
    registry.register_symmetric_ref(DELETE_WORKSHEET_PROTECTION_ID, create_identity());
    registry.register_symmetric_ref(SET_PERMISSION_POINTS_ID, create_lww());

    // Register bidirectional within module
    registry.register_bidirectional_ref(ADD_WORKSHEET_PROTECTION_ID, SET_WORKSHEET_PROTECTION_ID, create_identity());
    registry.register_bidirectional_ref(ADD_WORKSHEET_PROTECTION_ID, DELETE_WORKSHEET_PROTECTION_ID, create_identity());
    registry.register_bidirectional_ref(ADD_WORKSHEET_PROTECTION_ID, SET_PERMISSION_POINTS_ID, create_identity());
    registry.register_bidirectional_ref(SET_WORKSHEET_PROTECTION_ID, DELETE_WORKSHEET_PROTECTION_ID, create_identity());
    registry.register_bidirectional_ref(SET_WORKSHEET_PROTECTION_ID, SET_PERMISSION_POINTS_ID, create_identity());
    registry.register_bidirectional_ref(DELETE_WORKSHEET_PROTECTION_ID, SET_PERMISSION_POINTS_ID, create_identity());
}

pub fn register_cross_module_transforms(registry: &mut TransformRegistry, other_mutations: &[MutationId]) {
    for &ws_prot_id in WORKSHEET_PROTECTION_MUTATIONS {
        for &other_id in other_mutations {
            registry.register_identity(ws_prot_id, other_id);
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
        TransformResultRef {
            m1_prime: MutationOutcome::Removed,
            m2_prime: MutationOutcome::Unchanged(m2),
            error: None,
        }
    })
}
