use crate::mutations::sheets::{
    SetRangeThemeMutation, InsertRowMutation, InsertColMutation, RemoveRowMutation,
    RemoveColMutation, SetRangeValuesMutation, MoveRangeMutation, MoveRowsMutation, MoveColsMutation,
    AddWorksheetMergeMutation, SetRangeProtectionMutation, SetFrozenMutation, SetRowDataMutation,
    InsertSheetMutation, SetWorkbookNameMutation,
};
use crate::mutations::sheets_numfmt::SetNumfmtMutation;
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef};
use std::sync::Arc;

pub const MUTATION_ID: MutationId = SetRangeThemeMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    registry.register_symmetric_ref(MUTATION_ID, create_lww());

    // Identity transforms with all other mutations
    // Theme is a separate concern that doesn't interfere with other operations
    registry.register_identity(MUTATION_ID, InsertRowMutation::ID);
    registry.register_identity(MUTATION_ID, InsertColMutation::ID);
    registry.register_identity(MUTATION_ID, RemoveRowMutation::ID);
    registry.register_identity(MUTATION_ID, RemoveColMutation::ID);
    registry.register_identity(MUTATION_ID, SetRangeValuesMutation::ID);
    registry.register_identity(MUTATION_ID, MoveRangeMutation::ID);
    registry.register_identity(MUTATION_ID, MoveRowsMutation::ID);
    registry.register_identity(MUTATION_ID, MoveColsMutation::ID);
    registry.register_identity(MUTATION_ID, AddWorksheetMergeMutation::ID);
    registry.register_identity(MUTATION_ID, SetRangeProtectionMutation::ID);
    registry.register_identity(MUTATION_ID, SetNumfmtMutation::ID);
    registry.register_identity(MUTATION_ID, SetFrozenMutation::ID);
    registry.register_identity(MUTATION_ID, SetRowDataMutation::ID);
    registry.register_identity(MUTATION_ID, InsertSheetMutation::ID);
    registry.register_identity(MUTATION_ID, SetWorkbookNameMutation::ID);
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
