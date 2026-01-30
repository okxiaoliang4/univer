use crate::mutations::data_validation::{
    AddDataValidationMutation, RemoveDataValidationMutation, UpdateDataValidationMutation,
};
use crate::mutations::sheets::{
    InsertRowMutation, InsertColMutation, RemoveRowMutation, RemoveColMutation,
    SetRangeValuesMutation, MoveRangeMutation, MoveRowsMutation, MoveColsMutation,
    AddWorksheetMergeMutation, SetRangeProtectionMutation, SetRangeThemeMutation,
    SetFrozenMutation, SetRowDataMutation, InsertSheetMutation, SetWorkbookNameMutation,
};
use crate::mutations::sheets_numfmt::SetNumfmtMutation;
use crate::mutations::sheets_conditional_formatting::{
    AddConditionalRuleMutation, DeleteConditionalRuleMutation, SetConditionalRuleMutation,
    MoveConditionalRuleMutation,
};
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef};
use std::sync::Arc;

pub const ADD_RULE_ID: MutationId = AddDataValidationMutation::ID;
pub const REMOVE_RULE_ID: MutationId = RemoveDataValidationMutation::ID;
pub const UPDATE_RULE_ID: MutationId = UpdateDataValidationMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Add rule transforms
    registry.register_symmetric_ref(ADD_RULE_ID, create_identity());
    registry.register_bidirectional_ref(ADD_RULE_ID, REMOVE_RULE_ID, create_identity());
    registry.register_bidirectional_ref(ADD_RULE_ID, UPDATE_RULE_ID, create_identity());

    // Remove rule transforms
    registry.register_symmetric_ref(REMOVE_RULE_ID, create_identity());
    registry.register_bidirectional_ref(REMOVE_RULE_ID, UPDATE_RULE_ID, create_identity());

    // Update rule transforms
    registry.register_symmetric_ref(UPDATE_RULE_ID, create_lww());

    // With sheet operations - addRule
    registry.register_identity(ADD_RULE_ID, InsertRowMutation::ID);
    registry.register_identity(ADD_RULE_ID, InsertColMutation::ID);
    registry.register_identity(ADD_RULE_ID, RemoveRowMutation::ID);
    registry.register_identity(ADD_RULE_ID, RemoveColMutation::ID);
    registry.register_identity(ADD_RULE_ID, SetRangeValuesMutation::ID);
    registry.register_identity(ADD_RULE_ID, MoveRangeMutation::ID);
    registry.register_identity(ADD_RULE_ID, MoveRowsMutation::ID);
    registry.register_identity(ADD_RULE_ID, MoveColsMutation::ID);
    registry.register_identity(ADD_RULE_ID, AddWorksheetMergeMutation::ID);
    registry.register_identity(ADD_RULE_ID, SetRangeProtectionMutation::ID);
    registry.register_identity(ADD_RULE_ID, SetRangeThemeMutation::ID);
    registry.register_identity(ADD_RULE_ID, SetNumfmtMutation::ID);
    registry.register_identity(ADD_RULE_ID, SetFrozenMutation::ID);
    registry.register_identity(ADD_RULE_ID, SetRowDataMutation::ID);
    registry.register_identity(ADD_RULE_ID, InsertSheetMutation::ID);
    registry.register_identity(ADD_RULE_ID, SetWorkbookNameMutation::ID);

    // With sheet operations - removeRule
    registry.register_identity(REMOVE_RULE_ID, InsertRowMutation::ID);
    registry.register_identity(REMOVE_RULE_ID, InsertColMutation::ID);
    registry.register_identity(REMOVE_RULE_ID, RemoveRowMutation::ID);
    registry.register_identity(REMOVE_RULE_ID, RemoveColMutation::ID);
    registry.register_identity(REMOVE_RULE_ID, SetRangeValuesMutation::ID);
    registry.register_identity(REMOVE_RULE_ID, MoveRangeMutation::ID);
    registry.register_identity(REMOVE_RULE_ID, MoveRowsMutation::ID);
    registry.register_identity(REMOVE_RULE_ID, MoveColsMutation::ID);
    registry.register_identity(REMOVE_RULE_ID, AddWorksheetMergeMutation::ID);
    registry.register_identity(REMOVE_RULE_ID, SetRangeProtectionMutation::ID);
    registry.register_identity(REMOVE_RULE_ID, SetRangeThemeMutation::ID);
    registry.register_identity(REMOVE_RULE_ID, SetNumfmtMutation::ID);
    registry.register_identity(REMOVE_RULE_ID, SetFrozenMutation::ID);
    registry.register_identity(REMOVE_RULE_ID, SetRowDataMutation::ID);
    registry.register_identity(REMOVE_RULE_ID, InsertSheetMutation::ID);
    registry.register_identity(REMOVE_RULE_ID, SetWorkbookNameMutation::ID);

    // With sheet operations - updateRule
    registry.register_identity(UPDATE_RULE_ID, InsertRowMutation::ID);
    registry.register_identity(UPDATE_RULE_ID, InsertColMutation::ID);
    registry.register_identity(UPDATE_RULE_ID, RemoveRowMutation::ID);
    registry.register_identity(UPDATE_RULE_ID, RemoveColMutation::ID);
    registry.register_identity(UPDATE_RULE_ID, SetRangeValuesMutation::ID);
    registry.register_identity(UPDATE_RULE_ID, MoveRangeMutation::ID);
    registry.register_identity(UPDATE_RULE_ID, MoveRowsMutation::ID);
    registry.register_identity(UPDATE_RULE_ID, MoveColsMutation::ID);
    registry.register_identity(UPDATE_RULE_ID, AddWorksheetMergeMutation::ID);
    registry.register_identity(UPDATE_RULE_ID, SetRangeProtectionMutation::ID);
    registry.register_identity(UPDATE_RULE_ID, SetRangeThemeMutation::ID);
    registry.register_identity(UPDATE_RULE_ID, SetNumfmtMutation::ID);
    registry.register_identity(UPDATE_RULE_ID, SetFrozenMutation::ID);
    registry.register_identity(UPDATE_RULE_ID, SetRowDataMutation::ID);
    registry.register_identity(UPDATE_RULE_ID, InsertSheetMutation::ID);
    registry.register_identity(UPDATE_RULE_ID, SetWorkbookNameMutation::ID);

    // With conditional formatting (different features, don't interfere)
    registry.register_identity(ADD_RULE_ID, AddConditionalRuleMutation::ID);
    registry.register_identity(ADD_RULE_ID, DeleteConditionalRuleMutation::ID);
    registry.register_identity(ADD_RULE_ID, SetConditionalRuleMutation::ID);
    registry.register_identity(ADD_RULE_ID, MoveConditionalRuleMutation::ID);

    registry.register_identity(REMOVE_RULE_ID, AddConditionalRuleMutation::ID);
    registry.register_identity(REMOVE_RULE_ID, DeleteConditionalRuleMutation::ID);
    registry.register_identity(REMOVE_RULE_ID, SetConditionalRuleMutation::ID);
    registry.register_identity(REMOVE_RULE_ID, MoveConditionalRuleMutation::ID);

    registry.register_identity(UPDATE_RULE_ID, AddConditionalRuleMutation::ID);
    registry.register_identity(UPDATE_RULE_ID, DeleteConditionalRuleMutation::ID);
    registry.register_identity(UPDATE_RULE_ID, SetConditionalRuleMutation::ID);
    registry.register_identity(UPDATE_RULE_ID, MoveConditionalRuleMutation::ID);
}

fn create_identity() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        TransformResultRef::identity(m1, m2)  // Zero-copy!
    })
}

fn create_lww() -> TransformFnRef {
    Arc::new(|_m1: &MutationInfo, m2: &MutationInfo| {
        // LWW: m2 wins
        TransformResultRef {
            m1_prime: MutationOutcome::Removed,
            m2_prime: MutationOutcome::Unchanged(m2),  // Zero-copy!
            error: None,
        }
    })
}
