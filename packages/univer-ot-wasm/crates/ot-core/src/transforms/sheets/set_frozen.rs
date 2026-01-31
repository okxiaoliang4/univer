use crate::mutations::sheets::{
    SetFrozenMutation, InsertRowMutation, InsertColMutation, SetRangeValuesMutation,
    RemoveRowMutation, RemoveColMutation, MoveRangeMutation, MoveRowsMutation, MoveColsMutation,
    AddWorksheetMergeMutation, SetRangeProtectionMutation, SetRangeThemeMutation, SetRowDataMutation,
    InsertSheetMutation, SetWorkbookNameMutation,
};
use crate::mutations::sheets_numfmt::SetNumfmtMutation;
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::utils::transform_helpers::lww_transform;
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef};

pub const MUTATION_ID: MutationId = SetFrozenMutation::ID;

/// Register transforms for SetFrozenMutation
///
/// Mutation ID: sheet.mutation.set-frozen
///
/// SetFrozenMutation sets the freeze panes configuration for a worksheet.
/// Transform strategy: Last-Write-Wins (LWW) at worksheet level.
/// Identity with all position-based mutations (frozen panes don't interfere with cell operations).
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transform: LWW
    registry.register_symmetric_ref(MUTATION_ID, lww_transform());

    // Identity transforms with structural mutations
    registry.register_identity(MUTATION_ID, InsertRowMutation::ID);
    registry.register_identity(MUTATION_ID, InsertColMutation::ID);
    registry.register_identity(MUTATION_ID, RemoveRowMutation::ID);
    registry.register_identity(MUTATION_ID, RemoveColMutation::ID);
    registry.register_identity(MUTATION_ID, MoveRangeMutation::ID);
    registry.register_identity(MUTATION_ID, MoveRowsMutation::ID);
    registry.register_identity(MUTATION_ID, MoveColsMutation::ID);

    // Identity transforms with data mutations
    registry.register_identity(MUTATION_ID, SetRangeValuesMutation::ID);
    registry.register_identity(MUTATION_ID, SetRowDataMutation::ID);

    // Identity transforms with other feature mutations
    registry.register_identity(MUTATION_ID, AddWorksheetMergeMutation::ID);
    registry.register_identity(MUTATION_ID, SetRangeProtectionMutation::ID);
    registry.register_identity(MUTATION_ID, SetRangeThemeMutation::ID);
    registry.register_identity(MUTATION_ID, SetNumfmtMutation::ID);
    registry.register_identity(MUTATION_ID, InsertSheetMutation::ID);
    registry.register_identity(MUTATION_ID, SetWorkbookNameMutation::ID);
}