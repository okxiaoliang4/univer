use crate::mutations::sheets::{
    SetRangeProtectionMutation, InsertRowMutation, InsertColMutation, RemoveRowMutation,
    RemoveColMutation, SetRangeValuesMutation, MoveRangeMutation, MoveRowsMutation, MoveColsMutation,
    AddWorksheetMergeMutation, SetRangeThemeMutation, SetFrozenMutation, SetRowDataMutation,
    InsertSheetMutation, SetWorkbookNameMutation,
};
use crate::mutations::sheets_numfmt::SetNumfmtMutation;
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::utils::transform_helpers::identity_transform;
use crate::types::{MutationInfo, TransformResultRef};

pub const MUTATION_ID: MutationId = SetRangeProtectionMutation::ID;

/// Register transforms for SetRangeProtectionMutation
///
/// Mutation ID: sheet.mutation.set-range-protection
///
/// SetRangeProtectionMutation sets protection settings for a range.
/// Transform strategy: Identity (protection operations don't conflict).
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transform: identity
    registry.register_symmetric_ref(MUTATION_ID, identity_transform());

    // Identity transforms with non-interfering mutations
    registry.register_identity(MUTATION_ID, InsertRowMutation::ID);
    registry.register_identity(MUTATION_ID, InsertColMutation::ID);
    registry.register_identity(MUTATION_ID, RemoveRowMutation::ID);
    registry.register_identity(MUTATION_ID, RemoveColMutation::ID);
    registry.register_identity(MUTATION_ID, SetRangeValuesMutation::ID);
    registry.register_identity(MUTATION_ID, MoveRangeMutation::ID);
    registry.register_identity(MUTATION_ID, MoveRowsMutation::ID);
    registry.register_identity(MUTATION_ID, MoveColsMutation::ID);
    registry.register_identity(MUTATION_ID, AddWorksheetMergeMutation::ID);
    registry.register_identity(MUTATION_ID, SetRangeThemeMutation::ID);
    registry.register_identity(MUTATION_ID, SetNumfmtMutation::ID);
    registry.register_identity(MUTATION_ID, SetFrozenMutation::ID);
    registry.register_identity(MUTATION_ID, SetRowDataMutation::ID);
    registry.register_identity(MUTATION_ID, InsertSheetMutation::ID);
    registry.register_identity(MUTATION_ID, SetWorkbookNameMutation::ID);
}

pub fn register_cross_module_transforms(registry: &mut TransformRegistry, other_mutations: &[MutationId]) {
    for &other_id in other_mutations {
        registry.register_identity(MUTATION_ID, other_id);
    }
}