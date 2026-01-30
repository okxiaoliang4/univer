use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::types::{MutationInfo, TransformResultRef};
use crate::transforms::constants::{
    DATA_VALIDATION_MUTATIONS,
    CONDITIONAL_FORMATTING_MUTATIONS,
    FILTER_MUTATIONS,
    HYPER_LINK_MUTATIONS,
    NOTE_MUTATIONS,
    TABLE_MUTATIONS,
    PIVOT_TABLE_MUTATIONS,
    THREAD_COMMENT_MUTATIONS,
};
use std::sync::Arc;

pub const SET_NUMFMT_ID: MutationId = "sheet.mutation.set.numfmt";
pub const REMOVE_NUMFMT_ID: MutationId = "sheet.mutation.remove.numfmt";

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Register set.numfmt transforms
    registry.register_symmetric_ref(SET_NUMFMT_ID, create_identity());
    registry.register_identity(SET_NUMFMT_ID, "sheet.mutation.insert-row");
    registry.register_identity(SET_NUMFMT_ID, "sheet.mutation.insert-col");
    registry.register_identity(SET_NUMFMT_ID, "sheet.mutation.set-range-values");
    registry.register_identity(SET_NUMFMT_ID, "sheet.mutation.remove-rows");
    registry.register_identity(SET_NUMFMT_ID, "sheet.mutation.remove-col");
    registry.register_identity(SET_NUMFMT_ID, "sheet.mutation.move-range");
    registry.register_identity(SET_NUMFMT_ID, "sheet.mutation.move-rows");
    registry.register_identity(SET_NUMFMT_ID, "sheet.mutation.move-columns");
    registry.register_identity(SET_NUMFMT_ID, "sheet.mutation.add-worksheet-merge");
    registry.register_identity(SET_NUMFMT_ID, "sheet.mutation.set-range-protection");
    registry.register_identity(SET_NUMFMT_ID, "sheet.mutation.set-range-theme");
    registry.register_identity(SET_NUMFMT_ID, "sheet.mutation.set-frozen");
    registry.register_identity(SET_NUMFMT_ID, "sheet.mutation.set-row-data");
    registry.register_identity(SET_NUMFMT_ID, "sheet.mutation.insert-sheet");
    registry.register_identity(SET_NUMFMT_ID, "sheet.mutation.set-workbook-name");

    // Register remove.numfmt transforms
    registry.register_symmetric_ref(REMOVE_NUMFMT_ID, create_identity());
    registry.register_identity(REMOVE_NUMFMT_ID, SET_NUMFMT_ID);
    registry.register_identity(REMOVE_NUMFMT_ID, "sheet.mutation.insert-row");
    registry.register_identity(REMOVE_NUMFMT_ID, "sheet.mutation.insert-col");
    registry.register_identity(REMOVE_NUMFMT_ID, "sheet.mutation.set-range-values");
    registry.register_identity(REMOVE_NUMFMT_ID, "sheet.mutation.remove-rows");
    registry.register_identity(REMOVE_NUMFMT_ID, "sheet.mutation.remove-col");
    registry.register_identity(REMOVE_NUMFMT_ID, "sheet.mutation.move-range");
    registry.register_identity(REMOVE_NUMFMT_ID, "sheet.mutation.move-rows");
    registry.register_identity(REMOVE_NUMFMT_ID, "sheet.mutation.move-columns");
    registry.register_identity(REMOVE_NUMFMT_ID, "sheet.mutation.add-worksheet-merge");
    registry.register_identity(REMOVE_NUMFMT_ID, "sheet.mutation.set-range-protection");
    registry.register_identity(REMOVE_NUMFMT_ID, "sheet.mutation.set-range-theme");
    registry.register_identity(REMOVE_NUMFMT_ID, "sheet.mutation.set-frozen");
    registry.register_identity(REMOVE_NUMFMT_ID, "sheet.mutation.set-row-data");
    registry.register_identity(REMOVE_NUMFMT_ID, "sheet.mutation.insert-sheet");
    registry.register_identity(REMOVE_NUMFMT_ID, "sheet.mutation.set-workbook-name");

    // Register remove.numfmt cross-module transforms with all feature modules
    register_cross_module_transforms(registry);
}

/// Register cross-module identity transforms for remove.numfmt with all feature modules
fn register_cross_module_transforms(registry: &mut TransformRegistry) {
    // All feature module mutations that need cross-registration
    let all_feature_modules: &[&[MutationId]] = &[
        DATA_VALIDATION_MUTATIONS,
        CONDITIONAL_FORMATTING_MUTATIONS,
        FILTER_MUTATIONS,
        HYPER_LINK_MUTATIONS,
        NOTE_MUTATIONS,
        TABLE_MUTATIONS,
        PIVOT_TABLE_MUTATIONS,
        THREAD_COMMENT_MUTATIONS,
    ];

    // Register identity transforms between remove.numfmt and all feature modules
    for &feature_module in all_feature_modules {
        for &feature_mutation in feature_module {
            registry.register_identity(REMOVE_NUMFMT_ID, feature_mutation);
        }
    }
}

fn create_identity() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        TransformResultRef::identity(m1, m2)  // Zero-copy!
    })
}
