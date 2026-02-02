//! Shared transform instances for common field patterns
//!
//! This module provides generic transforms that work on any type implementing
//! `Shiftable` + `Serialize` + `Deserialize`.
//!
//! # Design
//!
//! Uses unified `ShiftOperation` enum and `Shiftable` trait for type-safe transforms.
//! Each shift operation has a single generic function - pass the params type at registration.
//!
//! # Usage
//!
//! ```ignore
//! use crate::utils::shared_transforms as shared;
//!
//! // Register with specific params type
//! registry.register_bidirectional_ref(
//!     InsertRowMutation::ID,
//!     MUTATION_ID,
//!     shared::insert_row_shift::<GenericRangesParams>(),
//! );
//! ```

use crate::mutations::sheets::{
    InsertColMutationParams, InsertRowMutationParams, MoveColsMutationParams,
    MoveRangeMutationParams, MoveRowsMutationParams, RemoveColMutationParams,
    RemoveRowsMutationParams, RemoveSheetMutationParams,
};
use crate::registry::TransformFnRef;
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef};
use crate::utils::generic_params::WorksheetParams;
use crate::utils::params::same_worksheet;
use crate::utils::shift_operations::{ShiftOperation, ShiftResult, Shiftable};
use std::sync::Arc;

// ============================================================================
// Generic Transform Function (unified implementation)
// ============================================================================

/// Generic transform function that applies a ShiftOperation to a Shiftable type
///
/// This function provides a unified way to create transforms:
/// 1. Parse m2 as the target type T
/// 2. Check worksheet location match
/// 3. Apply the shift operation
/// 4. Return appropriate result based on ShiftResult
pub fn apply_shift_transform<'a, T>(
    m1: &'a MutationInfo,
    m2: &'a MutationInfo,
    m1_sheet: &impl WorksheetParams,
    shift_op: ShiftOperation,
) -> TransformResultRef<'a>
where
    T: Shiftable + serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    // 1. Parse m2 as target type - fall back to identity if parsing fails
    // This allows the transform to handle params that don't exactly match the expected structure
    let mut m2_params: T = match serde_json::from_value(m2.params.clone()) {
        Ok(p) => p,
        Err(_) => return TransformResultRef::identity(m1, m2),
    };

    // 2. Check worksheet location match
    if !m1_sheet.same_sheet(&m2_params) {
        return TransformResultRef::identity(m1, m2);
    }

    // 3. Apply shift operation
    let result = m2_params.apply_shift(&shift_op);

    // 4. Return appropriate result
    match result {
        ShiftResult::Modified => TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(MutationInfo {
                id: m2.id.clone(),
                params: serde_json::to_value(m2_params).unwrap_or_else(|_| m2.params.clone()),
            }),
            error: None,
        },
        ShiftResult::Removed => TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Removed,
            error: None,
        },
        ShiftResult::Unchanged => TransformResultRef::identity(m1, m2),
    }
}

// ============================================================================
// Generic Shift Transforms - One function per operation type
// ============================================================================
//
// NOTE: These functions return new Arc instances each time because Rust static
// variables inside generic functions are shared across ALL type instantiations.
// This is fine since these are only called during transform registration (startup).

/// Generic InsertRow shift transform
///
/// Works with any type T that implements Shiftable + Serialize + Deserialize.
/// Pass the params type at registration time.
///
/// # Example
/// ```ignore
/// registry.register_bidirectional_ref(
///     InsertRowMutation::ID,
///     MUTATION_ID,
///     shared::insert_row_shift::<GenericRangesParams>(),
/// );
/// ```
pub fn insert_row_shift<T>() -> TransformFnRef
where
    T: Shiftable + serde::Serialize + for<'de> serde::Deserialize<'de> + 'static,
{
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return TransformResultRef::identity(m1, m2);
        }

        let m1_params: InsertRowMutationParams =
            match serde_json::from_value(m1.params.clone()) {
                Ok(p) => p,
                Err(_) => return TransformResultRef::identity(m1, m2),
            };

        let shift_op = ShiftOperation::InsertRows {
            start: m1_params.range.start_row,
            count: m1_params.range.end_row - m1_params.range.start_row + 1,
        };

        apply_shift_transform::<T>(m1, m2, &m1_params.sub_unit_params, shift_op)
    })
}

/// Generic RemoveRow shift transform
pub fn remove_row_shift<T>() -> TransformFnRef
where
    T: Shiftable + serde::Serialize + for<'de> serde::Deserialize<'de> + 'static,
{
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return TransformResultRef::identity(m1, m2);
        }

        let m1_params: RemoveRowsMutationParams =
            match serde_json::from_value(m1.params.clone()) {
                Ok(p) => p,
                Err(_) => return TransformResultRef::identity(m1, m2),
            };

        let shift_op = ShiftOperation::RemoveRows {
            start: m1_params.range.start_row,
            end: m1_params.range.end_row,
        };

        apply_shift_transform::<T>(m1, m2, &m1_params.sub_unit_params, shift_op)
    })
}

/// Generic InsertCol shift transform
pub fn insert_col_shift<T>() -> TransformFnRef
where
    T: Shiftable + serde::Serialize + for<'de> serde::Deserialize<'de> + 'static,
{
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return TransformResultRef::identity(m1, m2);
        }

        let m1_params: InsertColMutationParams =
            match serde_json::from_value(m1.params.clone()) {
                Ok(p) => p,
                Err(_) => return TransformResultRef::identity(m1, m2),
            };

        let shift_op = ShiftOperation::InsertCols {
            start: m1_params.range.start_column,
            count: m1_params.range.end_column - m1_params.range.start_column + 1,
        };

        apply_shift_transform::<T>(m1, m2, &m1_params.sub_unit_params, shift_op)
    })
}

/// Generic RemoveCol shift transform
pub fn remove_col_shift<T>() -> TransformFnRef
where
    T: Shiftable + serde::Serialize + for<'de> serde::Deserialize<'de> + 'static,
{
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return TransformResultRef::identity(m1, m2);
        }

        let m1_params: RemoveColMutationParams =
            match serde_json::from_value(m1.params.clone()) {
                Ok(p) => p,
                Err(_) => return TransformResultRef::identity(m1, m2),
            };

        let shift_op = ShiftOperation::RemoveCols {
            start: m1_params.range.start_column,
            end: m1_params.range.end_column,
        };

        apply_shift_transform::<T>(m1, m2, &m1_params.sub_unit_params, shift_op)
    })
}

/// Generic MoveRows shift transform
///
/// When rows are moved from one position to another, affected data needs to be shifted.
/// Data in the moved rows follows them; data between source and target shifts accordingly.
pub fn move_rows_shift<T>() -> TransformFnRef
where
    T: Shiftable + serde::Serialize + for<'de> serde::Deserialize<'de> + 'static,
{
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return TransformResultRef::identity(m1, m2);
        }

        let m1_params: MoveRowsMutationParams =
            match serde_json::from_value(m1.params.clone()) {
                Ok(p) => p,
                Err(_) => return TransformResultRef::identity(m1, m2),
            };

        let shift_op = ShiftOperation::MoveRows {
            from_start: m1_params.source_range.start_row,
            from_end: m1_params.source_range.end_row,
            to: m1_params.target_range.start_row,
        };

        apply_shift_transform::<T>(m1, m2, &m1_params.sub_unit_params, shift_op)
    })
}

/// Generic MoveCols shift transform
///
/// When columns are moved from one position to another, affected data needs to be shifted.
/// Data in the moved columns follows them; data between source and target shifts accordingly.
pub fn move_cols_shift<T>() -> TransformFnRef
where
    T: Shiftable + serde::Serialize + for<'de> serde::Deserialize<'de> + 'static,
{
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return TransformResultRef::identity(m1, m2);
        }

        let m1_params: MoveColsMutationParams =
            match serde_json::from_value(m1.params.clone()) {
                Ok(p) => p,
                Err(_) => return TransformResultRef::identity(m1, m2),
            };

        let shift_op = ShiftOperation::MoveCols {
            from_start: m1_params.source_range.start_column,
            from_end: m1_params.source_range.end_column,
            to: m1_params.target_range.start_column,
        };

        apply_shift_transform::<T>(m1, m2, &m1_params.sub_unit_params, shift_op)
    })
}

/// Generic MoveRange shift transform
///
/// When a range is moved from one location to another, affected data needs to be shifted.
/// This handles within-sheet moves (cross-sheet moves would need additional handling).
pub fn move_range_shift<T>() -> TransformFnRef
where
    T: Shiftable + serde::Serialize + for<'de> serde::Deserialize<'de> + 'static,
{
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return TransformResultRef::identity(m1, m2);
        }

        let m1_params: MoveRangeMutationParams =
            match serde_json::from_value(m1.params.clone()) {
                Ok(p) => p,
                Err(_) => return TransformResultRef::identity(m1, m2),
            };

        let shift_op = ShiftOperation::MoveRange {
            from: m1_params.from_range.clone(),
            to: m1_params.to_range.clone(),
        };

        apply_shift_transform::<T>(m1, m2, &m1_params.sub_unit_params, shift_op)
    })
}

/// Generic RemoveSheet shift transform
///
/// When a sheet is removed, all mutations targeting that sheet should be removed.
/// Unlike other shift transforms, this checks if m2 is on the *removed* sheet.
pub fn remove_sheet_shift<T>() -> TransformFnRef
where
    T: Shiftable + serde::Serialize + for<'de> serde::Deserialize<'de> + 'static,
{
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        let m1_params: RemoveSheetMutationParams =
            match serde_json::from_value(m1.params.clone()) {
                Ok(p) => p,
                Err(_) => return TransformResultRef::identity(m1, m2),
            };

        // Check if m2 is on the same unit (workbook)
        let m2_unit_id = m2.params.get("unitId").and_then(|v| v.as_str());
        if m2_unit_id != Some(&m1_params.sub_unit_params.unit_id) {
            return TransformResultRef::identity(m1, m2);
        }

        // Check if m2 is targeting the removed sheet
        let m2_sub_unit_id = m2.params.get("subUnitId").and_then(|v| v.as_str());
        if m2_sub_unit_id != Some(&m1_params.sub_unit_params.sub_unit_id) {
            return TransformResultRef::identity(m1, m2);
        }

        // m2 is on the removed sheet - it should be removed
        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Removed,
            error: None,
        }
    })
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::generic_params::GenericRangesParams;

    #[test]
    fn test_generic_transform_creates_new_arc() {
        // Generic functions create new Arc each time (no singleton for generics)
        // This is intentional - Rust static variables in generic functions are
        // shared across all type instantiations, so OnceLock would be incorrect.
        let t1 = insert_row_shift::<GenericRangesParams>();
        let t2 = insert_row_shift::<GenericRangesParams>();

        // They are different Arc instances (correct behavior for generics)
        assert!(!Arc::ptr_eq(&t1, &t2));
    }
}
