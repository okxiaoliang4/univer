//! Transforms for SetRangeProtectionMutation
//!
//! This mutation has `rule: IRangeProtectionRule` which has `ranges: Vec<IRange>`.
//! These ranges need shift transforms when rows/columns are inserted/removed.

use crate::mutations::sheets::{
    SetRangeProtectionMutation, SetRangeProtectionMutationParams,
    InsertRowMutation, InsertColMutation,
    RemoveRowMutation, RemoveColMutation,
    MoveRowsMutation, MoveColsMutation, MoveRangeMutation, RemoveSheetMutation,
};
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::shared_transforms as shared;

pub const MUTATION_ID: MutationId = SetRangeProtectionMutation::ID;

/// Register transforms for SetRangeProtectionMutation
///
/// Mutation ID: sheet.mutation.set-range-protection
///
/// SetRangeProtectionMutation sets protection settings for a range.
/// Transform strategy: Identity for self-transforms (protection operations don't conflict).
///
/// Shift transforms: Protection ranges must be adjusted when rows/columns are inserted/removed.
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Shift transforms for SetRangeProtection (single rule with ranges)
    registry.register_bidirectional_ref(InsertRowMutation::ID, MUTATION_ID, shared::insert_row_shift::<SetRangeProtectionMutationParams>());
    registry.register_bidirectional_ref(InsertColMutation::ID, MUTATION_ID, shared::insert_col_shift::<SetRangeProtectionMutationParams>());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, MUTATION_ID, shared::remove_row_shift::<SetRangeProtectionMutationParams>());
    registry.register_bidirectional_ref(RemoveColMutation::ID, MUTATION_ID, shared::remove_col_shift::<SetRangeProtectionMutationParams>());
    registry.register_bidirectional_ref(MoveRowsMutation::ID, MUTATION_ID, shared::move_rows_shift::<SetRangeProtectionMutationParams>());
    registry.register_bidirectional_ref(MoveColsMutation::ID, MUTATION_ID, shared::move_cols_shift::<SetRangeProtectionMutationParams>());
    registry.register_bidirectional_ref(MoveRangeMutation::ID, MUTATION_ID, shared::move_range_shift::<SetRangeProtectionMutationParams>());
    registry.register_bidirectional_ref(RemoveSheetMutation::ID, MUTATION_ID, shared::remove_sheet_shift::<SetRangeProtectionMutationParams>());
}
