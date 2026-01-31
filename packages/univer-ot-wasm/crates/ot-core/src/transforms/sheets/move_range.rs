use crate::mutations::sheets::{
    MoveRangeMutation, InsertRowMutation, InsertColMutation, SetRangeValuesMutation,
    RemoveRowMutation, RemoveColMutation, MoveRowsMutation, MoveColsMutation,
    AddWorksheetMergeMutation, SetRangeProtectionMutation, SetRangeThemeMutation,
    SetFrozenMutation, SetRowDataMutation, InsertSheetMutation, SetWorkbookNameMutation,
};
use crate::mutations::sheets_numfmt::SetNumfmtMutation;
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::utils::transform_helpers::identity_transform;
use crate::types::{MutationInfo, TransformResultRef};

pub const MUTATION_ID: MutationId = MoveRangeMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    registry.register_symmetric_ref(MUTATION_ID, identity_transform());
    registry.register_identity(MUTATION_ID, InsertRowMutation::ID);
    registry.register_identity(MUTATION_ID, InsertColMutation::ID);
    registry.register_identity(MUTATION_ID, SetRangeValuesMutation::ID);

    // Identity transforms with non-interfering mutations
    registry.register_identity(MUTATION_ID, RemoveRowMutation::ID);
    registry.register_identity(MUTATION_ID, RemoveColMutation::ID);
    registry.register_identity(MUTATION_ID, MoveRowsMutation::ID);
    registry.register_identity(MUTATION_ID, MoveColsMutation::ID);
    registry.register_identity(MUTATION_ID, AddWorksheetMergeMutation::ID);
    registry.register_identity(MUTATION_ID, SetRangeProtectionMutation::ID);
    registry.register_identity(MUTATION_ID, SetRangeThemeMutation::ID);
    registry.register_identity(MUTATION_ID, SetNumfmtMutation::ID);
    registry.register_identity(MUTATION_ID, SetFrozenMutation::ID);
    registry.register_identity(MUTATION_ID, SetRowDataMutation::ID);
    registry.register_identity(MUTATION_ID, InsertSheetMutation::ID);
    registry.register_identity(MUTATION_ID, SetWorkbookNameMutation::ID);
}