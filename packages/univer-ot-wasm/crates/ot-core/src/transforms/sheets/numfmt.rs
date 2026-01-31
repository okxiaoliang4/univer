use crate::mutations::sheets::{
    InsertRowMutation, InsertColMutation, SetRangeValuesMutation, RemoveRowMutation,
    RemoveColMutation, MoveRangeMutation, MoveRowsMutation, MoveColsMutation,
    AddWorksheetMergeMutation, SetRangeProtectionMutation, SetRangeThemeMutation,
    SetFrozenMutation, SetRowDataMutation, InsertSheetMutation, SetWorkbookNameMutation,
};
use crate::mutations::sheets_numfmt::{SetNumfmtMutation, RemoveNumfmtMutation};
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::utils::transform_helpers::identity_transform;
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

pub const SET_NUMFMT_ID: MutationId = SetNumfmtMutation::ID;
pub const REMOVE_NUMFMT_ID: MutationId = RemoveNumfmtMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Register set.numfmt transforms
    registry.register_symmetric_ref(SET_NUMFMT_ID, identity_transform());
    registry.register_identity(SET_NUMFMT_ID, InsertRowMutation::ID);
    registry.register_identity(SET_NUMFMT_ID, InsertColMutation::ID);
    registry.register_identity(SET_NUMFMT_ID, SetRangeValuesMutation::ID);
    registry.register_identity(SET_NUMFMT_ID, RemoveRowMutation::ID);
    registry.register_identity(SET_NUMFMT_ID, RemoveColMutation::ID);
    registry.register_identity(SET_NUMFMT_ID, MoveRangeMutation::ID);
    registry.register_identity(SET_NUMFMT_ID, MoveRowsMutation::ID);
    registry.register_identity(SET_NUMFMT_ID, MoveColsMutation::ID);
    registry.register_identity(SET_NUMFMT_ID, AddWorksheetMergeMutation::ID);
    registry.register_identity(SET_NUMFMT_ID, SetRangeProtectionMutation::ID);
    registry.register_identity(SET_NUMFMT_ID, SetRangeThemeMutation::ID);
    registry.register_identity(SET_NUMFMT_ID, SetFrozenMutation::ID);
    registry.register_identity(SET_NUMFMT_ID, SetRowDataMutation::ID);
    registry.register_identity(SET_NUMFMT_ID, InsertSheetMutation::ID);
    registry.register_identity(SET_NUMFMT_ID, SetWorkbookNameMutation::ID);

    // Register remove.numfmt transforms
    registry.register_symmetric_ref(REMOVE_NUMFMT_ID, identity_transform());
    registry.register_identity(REMOVE_NUMFMT_ID, SET_NUMFMT_ID);
    registry.register_identity(REMOVE_NUMFMT_ID, InsertRowMutation::ID);
    registry.register_identity(REMOVE_NUMFMT_ID, InsertColMutation::ID);
    registry.register_identity(REMOVE_NUMFMT_ID, SetRangeValuesMutation::ID);
    registry.register_identity(REMOVE_NUMFMT_ID, RemoveRowMutation::ID);
    registry.register_identity(REMOVE_NUMFMT_ID, RemoveColMutation::ID);
    registry.register_identity(REMOVE_NUMFMT_ID, MoveRangeMutation::ID);
    registry.register_identity(REMOVE_NUMFMT_ID, MoveRowsMutation::ID);
    registry.register_identity(REMOVE_NUMFMT_ID, MoveColsMutation::ID);
    registry.register_identity(REMOVE_NUMFMT_ID, AddWorksheetMergeMutation::ID);
    registry.register_identity(REMOVE_NUMFMT_ID, SetRangeProtectionMutation::ID);
    registry.register_identity(REMOVE_NUMFMT_ID, SetRangeThemeMutation::ID);
    registry.register_identity(REMOVE_NUMFMT_ID, SetFrozenMutation::ID);
    registry.register_identity(REMOVE_NUMFMT_ID, SetRowDataMutation::ID);
    registry.register_identity(REMOVE_NUMFMT_ID, InsertSheetMutation::ID);
    registry.register_identity(REMOVE_NUMFMT_ID, SetWorkbookNameMutation::ID);

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