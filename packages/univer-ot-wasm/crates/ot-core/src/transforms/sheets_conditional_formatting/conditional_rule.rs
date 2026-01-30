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

pub const ADD_RULE_ID: MutationId = AddConditionalRuleMutation::ID;
pub const DELETE_RULE_ID: MutationId = DeleteConditionalRuleMutation::ID;
pub const SET_RULE_ID: MutationId = SetConditionalRuleMutation::ID;
pub const MOVE_RULE_ID: MutationId = MoveConditionalRuleMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Add rule
    registry.register_symmetric_ref(ADD_RULE_ID, create_identity());
    registry.register_bidirectional_ref(ADD_RULE_ID, DELETE_RULE_ID, create_identity());
    registry.register_bidirectional_ref(ADD_RULE_ID, SET_RULE_ID, create_identity());
    registry.register_bidirectional_ref(ADD_RULE_ID, MOVE_RULE_ID, create_identity());

    // Delete rule
    registry.register_symmetric_ref(DELETE_RULE_ID, create_identity());
    registry.register_bidirectional_ref(DELETE_RULE_ID, SET_RULE_ID, create_identity());
    registry.register_bidirectional_ref(DELETE_RULE_ID, MOVE_RULE_ID, create_identity());

    // Set rule
    registry.register_symmetric_ref(SET_RULE_ID, create_lww());
    registry.register_bidirectional_ref(SET_RULE_ID, MOVE_RULE_ID, create_identity());

    // Move rule
    registry.register_symmetric_ref(MOVE_RULE_ID, create_identity());

    // With sheet operations - add-conditional-rule
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

    // With sheet operations - delete-conditional-rule
    registry.register_identity(DELETE_RULE_ID, InsertRowMutation::ID);
    registry.register_identity(DELETE_RULE_ID, InsertColMutation::ID);
    registry.register_identity(DELETE_RULE_ID, RemoveRowMutation::ID);
    registry.register_identity(DELETE_RULE_ID, RemoveColMutation::ID);
    registry.register_identity(DELETE_RULE_ID, SetRangeValuesMutation::ID);
    registry.register_identity(DELETE_RULE_ID, MoveRangeMutation::ID);
    registry.register_identity(DELETE_RULE_ID, MoveRowsMutation::ID);
    registry.register_identity(DELETE_RULE_ID, MoveColsMutation::ID);
    registry.register_identity(DELETE_RULE_ID, AddWorksheetMergeMutation::ID);
    registry.register_identity(DELETE_RULE_ID, SetRangeProtectionMutation::ID);
    registry.register_identity(DELETE_RULE_ID, SetRangeThemeMutation::ID);
    registry.register_identity(DELETE_RULE_ID, SetNumfmtMutation::ID);
    registry.register_identity(DELETE_RULE_ID, SetFrozenMutation::ID);
    registry.register_identity(DELETE_RULE_ID, SetRowDataMutation::ID);
    registry.register_identity(DELETE_RULE_ID, InsertSheetMutation::ID);
    registry.register_identity(DELETE_RULE_ID, SetWorkbookNameMutation::ID);

    // With sheet operations - set-conditional-rule
    registry.register_identity(SET_RULE_ID, InsertRowMutation::ID);
    registry.register_identity(SET_RULE_ID, InsertColMutation::ID);
    registry.register_identity(SET_RULE_ID, RemoveRowMutation::ID);
    registry.register_identity(SET_RULE_ID, RemoveColMutation::ID);
    registry.register_identity(SET_RULE_ID, SetRangeValuesMutation::ID);
    registry.register_identity(SET_RULE_ID, MoveRangeMutation::ID);
    registry.register_identity(SET_RULE_ID, MoveRowsMutation::ID);
    registry.register_identity(SET_RULE_ID, MoveColsMutation::ID);
    registry.register_identity(SET_RULE_ID, AddWorksheetMergeMutation::ID);
    registry.register_identity(SET_RULE_ID, SetRangeProtectionMutation::ID);
    registry.register_identity(SET_RULE_ID, SetRangeThemeMutation::ID);
    registry.register_identity(SET_RULE_ID, SetNumfmtMutation::ID);
    registry.register_identity(SET_RULE_ID, SetFrozenMutation::ID);
    registry.register_identity(SET_RULE_ID, SetRowDataMutation::ID);
    registry.register_identity(SET_RULE_ID, InsertSheetMutation::ID);
    registry.register_identity(SET_RULE_ID, SetWorkbookNameMutation::ID);

    // With sheet operations - move-conditional-rule
    registry.register_identity(MOVE_RULE_ID, InsertRowMutation::ID);
    registry.register_identity(MOVE_RULE_ID, InsertColMutation::ID);
    registry.register_identity(MOVE_RULE_ID, RemoveRowMutation::ID);
    registry.register_identity(MOVE_RULE_ID, RemoveColMutation::ID);
    registry.register_identity(MOVE_RULE_ID, SetRangeValuesMutation::ID);
    registry.register_identity(MOVE_RULE_ID, MoveRangeMutation::ID);
    registry.register_identity(MOVE_RULE_ID, MoveRowsMutation::ID);
    registry.register_identity(MOVE_RULE_ID, MoveColsMutation::ID);
    registry.register_identity(MOVE_RULE_ID, AddWorksheetMergeMutation::ID);
    registry.register_identity(MOVE_RULE_ID, SetRangeProtectionMutation::ID);
    registry.register_identity(MOVE_RULE_ID, SetRangeThemeMutation::ID);
    registry.register_identity(MOVE_RULE_ID, SetNumfmtMutation::ID);
    registry.register_identity(MOVE_RULE_ID, SetFrozenMutation::ID);
    registry.register_identity(MOVE_RULE_ID, SetRowDataMutation::ID);
    registry.register_identity(MOVE_RULE_ID, InsertSheetMutation::ID);
    registry.register_identity(MOVE_RULE_ID, SetWorkbookNameMutation::ID);

    // With data validation (different features, don't interfere)
    // Note: data-validation module registers these in the opposite direction
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
