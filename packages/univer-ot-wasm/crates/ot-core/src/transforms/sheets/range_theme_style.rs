use crate::mutations::sheets::{
    SetWorksheetRangeThemeStyleMutation, DeleteWorksheetRangeThemeStyleMutation,
    RegisterWorksheetRangeThemeStyleMutation, UnregisterWorksheetRangeThemeStyleMutation,
};
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef};
use std::sync::Arc;

pub const SET_RANGE_THEME_STYLE_ID: MutationId = SetWorksheetRangeThemeStyleMutation::ID;
pub const REMOVE_RANGE_THEME_STYLE_ID: MutationId = DeleteWorksheetRangeThemeStyleMutation::ID;
pub const REGISTER_RANGE_THEME_STYLE_ID: MutationId = RegisterWorksheetRangeThemeStyleMutation::ID;
pub const UNREGISTER_RANGE_THEME_STYLE_ID: MutationId = UnregisterWorksheetRangeThemeStyleMutation::ID;

const RANGE_THEME_STYLE_MUTATIONS: &[MutationId] = &[
    SET_RANGE_THEME_STYLE_ID,
    REMOVE_RANGE_THEME_STYLE_ID,
    REGISTER_RANGE_THEME_STYLE_ID,
    UNREGISTER_RANGE_THEME_STYLE_ID,
];

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Register symmetric transforms
    registry.register_symmetric_ref(SET_RANGE_THEME_STYLE_ID, create_lww());
    registry.register_symmetric_ref(REMOVE_RANGE_THEME_STYLE_ID, create_identity());
    registry.register_symmetric_ref(REGISTER_RANGE_THEME_STYLE_ID, create_identity());
    registry.register_symmetric_ref(UNREGISTER_RANGE_THEME_STYLE_ID, create_identity());

    // Register bidirectional within module
    registry.register_bidirectional_ref(SET_RANGE_THEME_STYLE_ID, REMOVE_RANGE_THEME_STYLE_ID, create_identity());
    registry.register_bidirectional_ref(SET_RANGE_THEME_STYLE_ID, REGISTER_RANGE_THEME_STYLE_ID, create_identity());
    registry.register_bidirectional_ref(SET_RANGE_THEME_STYLE_ID, UNREGISTER_RANGE_THEME_STYLE_ID, create_identity());
    registry.register_bidirectional_ref(REMOVE_RANGE_THEME_STYLE_ID, REGISTER_RANGE_THEME_STYLE_ID, create_identity());
    registry.register_bidirectional_ref(REMOVE_RANGE_THEME_STYLE_ID, UNREGISTER_RANGE_THEME_STYLE_ID, create_identity());
    registry.register_bidirectional_ref(REGISTER_RANGE_THEME_STYLE_ID, UNREGISTER_RANGE_THEME_STYLE_ID, create_identity());
}

pub fn register_cross_module_transforms(registry: &mut TransformRegistry, other_mutations: &[MutationId]) {
    for &rt_style_id in RANGE_THEME_STYLE_MUTATIONS {
        for &other_id in other_mutations {
            registry.register_identity(rt_style_id, other_id);
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
