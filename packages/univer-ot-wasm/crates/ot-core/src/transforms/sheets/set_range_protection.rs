//! Transforms for SetRangeProtectionMutation
//!
//! This mutation has `rule: IRangeProtectionRule` which has `ranges: Vec<IRange>`.
//! These ranges need shift transforms when rows/columns are inserted/removed.

use crate::mutations::sheets::{
    SetRangeProtectionMutation, SetRangeProtectionMutationParams,
    InsertRowMutation, InsertRowMutationParams,
    InsertColMutation, InsertColMutationParams,
    RemoveRowMutation, RemoveRowsMutationParams,
    RemoveColMutation, RemoveColMutationParams,
};
use crate::registry::{MutationId, TransformRegistry, TransformFnRef};
use crate::types::{MutationInfo, TransformResultRef};
use crate::utils::params::same_worksheet;
use crate::utils::shift_operations::ShiftOperation;
use crate::utils::shared_transforms::apply_shift_transform;
use std::sync::{Arc, OnceLock};

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
    // Self-transform: identity


    // Shift transforms for SetRangeProtection (single rule with ranges)
    registry.register_bidirectional_ref(InsertRowMutation::ID, MUTATION_ID, insert_row_vs_set_range_protection());
    registry.register_bidirectional_ref(InsertColMutation::ID, MUTATION_ID, insert_col_vs_set_range_protection());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, MUTATION_ID, remove_row_vs_set_range_protection());
    registry.register_bidirectional_ref(RemoveColMutation::ID, MUTATION_ID, remove_col_vs_set_range_protection());
}

// ============================================================================
// Local Transform Functions for SetRangeProtection
// ============================================================================

/// Transform: InsertRow vs SetRangeProtection (single rule with ranges)
fn insert_row_vs_set_range_protection() -> TransformFnRef {
    static TRANSFORM: OnceLock<TransformFnRef> = OnceLock::new();
    TRANSFORM.get_or_init(|| {
        Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
            if let Some(false) = same_worksheet(&m1.params, &m2.params) {
                return TransformResultRef::identity(m1, m2);
            }

            let m1_params: InsertRowMutationParams = match serde_json::from_value(m1.params.clone()) {
                Ok(p) => p,
                Err(_) => return TransformResultRef::identity(m1, m2),
            };

            let shift_op = ShiftOperation::InsertRows {
                start: m1_params.range.start_row,
                count: m1_params.range.end_row - m1_params.range.start_row + 1,
            };

            apply_shift_transform::<SetRangeProtectionMutationParams>(m1, m2, &m1_params.sub_unit_params, shift_op)
        })
    }).clone()
}

/// Transform: RemoveRow vs SetRangeProtection (single rule with ranges)
fn remove_row_vs_set_range_protection() -> TransformFnRef {
    static TRANSFORM: OnceLock<TransformFnRef> = OnceLock::new();
    TRANSFORM.get_or_init(|| {
        Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
            if let Some(false) = same_worksheet(&m1.params, &m2.params) {
                return TransformResultRef::identity(m1, m2);
            }

            let m1_params: RemoveRowsMutationParams = match serde_json::from_value(m1.params.clone()) {
                Ok(p) => p,
                Err(_) => return TransformResultRef::identity(m1, m2),
            };

            let shift_op = ShiftOperation::RemoveRows {
                start: m1_params.range.start_row,
                end: m1_params.range.end_row,
            };

            apply_shift_transform::<SetRangeProtectionMutationParams>(m1, m2, &m1_params.sub_unit_params, shift_op)
        })
    }).clone()
}

/// Transform: InsertCol vs SetRangeProtection (single rule with ranges)
fn insert_col_vs_set_range_protection() -> TransformFnRef {
    static TRANSFORM: OnceLock<TransformFnRef> = OnceLock::new();
    TRANSFORM.get_or_init(|| {
        Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
            if let Some(false) = same_worksheet(&m1.params, &m2.params) {
                return TransformResultRef::identity(m1, m2);
            }

            let m1_params: InsertColMutationParams = match serde_json::from_value(m1.params.clone()) {
                Ok(p) => p,
                Err(_) => return TransformResultRef::identity(m1, m2),
            };

            let shift_op = ShiftOperation::InsertCols {
                start: m1_params.range.start_column,
                count: m1_params.range.end_column - m1_params.range.start_column + 1,
            };

            apply_shift_transform::<SetRangeProtectionMutationParams>(m1, m2, &m1_params.sub_unit_params, shift_op)
        })
    }).clone()
}

/// Transform: RemoveCol vs SetRangeProtection (single rule with ranges)
fn remove_col_vs_set_range_protection() -> TransformFnRef {
    static TRANSFORM: OnceLock<TransformFnRef> = OnceLock::new();
    TRANSFORM.get_or_init(|| {
        Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
            if let Some(false) = same_worksheet(&m1.params, &m2.params) {
                return TransformResultRef::identity(m1, m2);
            }

            let m1_params: RemoveColMutationParams = match serde_json::from_value(m1.params.clone()) {
                Ok(p) => p,
                Err(_) => return TransformResultRef::identity(m1, m2),
            };

            let shift_op = ShiftOperation::RemoveCols {
                start: m1_params.range.start_column,
                end: m1_params.range.end_column,
            };

            apply_shift_transform::<SetRangeProtectionMutationParams>(m1, m2, &m1_params.sub_unit_params, shift_op)
        })
    }).clone()
}
