//! Transforms for AddRangeProtectionMutation
//!
//! This mutation has `rules: Vec<IRangeProtectionRule>` where each rule has `ranges: Vec<IRange>`.
//! These ranges need shift transforms when rows/columns are inserted/removed.

use crate::mutations::sheets::{
    AddRangeProtectionMutation, AddRangeProtectionMutationParams,
    InsertRowMutation, InsertColMutation, RemoveRowMutation, RemoveColMutation,
};
use crate::registry::TransformRegistry;
use crate::utils::shared_transforms::{insert_col_shift, insert_row_shift, remove_col_shift, remove_row_shift};

/// Register transforms for AddRangeProtectionMutation
///
/// Mutation ID: sheet.mutation.add-range-protection
///
/// AddRangeProtectionMutation adds protection to a range.
/// Transform strategy: Identity for self-transforms (protection operations don't conflict).
///
/// Shift transforms: Protection ranges must be adjusted when rows/columns are inserted/removed.
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Shift transforms for AddRangeProtection (rules with nested ranges)
    registry.register_bidirectional_ref(InsertRowMutation::ID, AddRangeProtectionMutation::ID, insert_row_shift::<AddRangeProtectionMutationParams>());
    registry.register_bidirectional_ref(InsertColMutation::ID, AddRangeProtectionMutation::ID, insert_col_shift::<AddRangeProtectionMutationParams>());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, AddRangeProtectionMutation::ID, remove_row_shift::<AddRangeProtectionMutationParams>());
    registry.register_bidirectional_ref(RemoveColMutation::ID, AddRangeProtectionMutation::ID, remove_col_shift::<AddRangeProtectionMutationParams>());
}
