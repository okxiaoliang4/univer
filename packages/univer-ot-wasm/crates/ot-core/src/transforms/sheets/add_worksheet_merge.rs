//! Transforms for AddWorksheetMergeMutation
//!
//! Merge ranges (Vec<IRange>) need to be adjusted when rows/columns are inserted or removed.

use crate::mutations::sheets::{
    AddWorksheetMergeMutation, InsertRowMutation, InsertColMutation, RemoveRowMutation,
    RemoveColMutation, SetRangeValuesMutation, MoveRangeMutation, MoveRowsMutation, MoveColsMutation,
    SetRangeProtectionMutation, SetRangeThemeMutation, SetFrozenMutation, SetRowDataMutation,
    InsertSheetMutation, SetWorkbookNameMutation,
    AddWorksheetMergeMutationParams, InsertRowMutationParams, InsertColMutationParams,
    RemoveRowsMutationParams, RemoveColMutationParams,
};
use crate::mutations::sheets_numfmt::SetNumfmtMutation;
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::utils::transform_helpers::identity_transform;
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef};
use crate::utils::params::same_worksheet;
use crate::utils::shift::{
    shift_range_rows_for_insert, shift_range_rows_for_remove,
    shift_range_cols_for_insert, shift_range_cols_for_remove,
};
use std::sync::Arc;

pub const MUTATION_ID: MutationId = AddWorksheetMergeMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transform (AddWorksheetMerge vs AddWorksheetMerge)
    registry.register_symmetric_ref(MUTATION_ID, identity_transform());

    // Bidirectional: InsertRow vs AddWorksheetMerge (ranges need row shift)
    registry.register_bidirectional_ref(
        InsertRowMutation::ID,
        MUTATION_ID,
        create_insert_row_vs_add_merge(),
    );

    // Bidirectional: InsertCol vs AddWorksheetMerge (ranges need column shift)
    registry.register_bidirectional_ref(
        InsertColMutation::ID,
        MUTATION_ID,
        create_insert_col_vs_add_merge(),
    );

    // Bidirectional: RemoveRow vs AddWorksheetMerge (ranges need row shift/removal)
    registry.register_bidirectional_ref(
        RemoveRowMutation::ID,
        MUTATION_ID,
        create_remove_row_vs_add_merge(),
    );

    // Bidirectional: RemoveCol vs AddWorksheetMerge (ranges need column shift/removal)
    registry.register_bidirectional_ref(
        RemoveColMutation::ID,
        MUTATION_ID,
        create_remove_col_vs_add_merge(),
    );

    // Identity transforms with non-interfering mutations
    registry.register_identity(MUTATION_ID, SetRangeValuesMutation::ID);
    registry.register_identity(MUTATION_ID, MoveRangeMutation::ID);
    registry.register_identity(MUTATION_ID, MoveRowsMutation::ID);
    registry.register_identity(MUTATION_ID, MoveColsMutation::ID);
    registry.register_identity(MUTATION_ID, SetRangeProtectionMutation::ID);
    registry.register_identity(MUTATION_ID, SetRangeThemeMutation::ID);
    registry.register_identity(MUTATION_ID, SetNumfmtMutation::ID);
    registry.register_identity(MUTATION_ID, SetFrozenMutation::ID);
    registry.register_identity(MUTATION_ID, SetRowDataMutation::ID);
    registry.register_identity(MUTATION_ID, InsertSheetMutation::ID);
    registry.register_identity(MUTATION_ID, SetWorkbookNameMutation::ID);
}

/// Helper to create identity transform result (zero-copy!)
#[inline]
fn identity<'a>(m1: &'a MutationInfo, m2: &'a MutationInfo) -> TransformResultRef<'a> {
    TransformResultRef::identity(m1, m2)
}

/// Helper to create error result (zero-copy for mutations)
#[inline]
fn parse_error<'a>(m1: &'a MutationInfo, m2: &'a MutationInfo, msg: &str) -> TransformResultRef<'a> {
    TransformResultRef::parse_error(m1, m2, msg)
}

/// InsertRow vs AddWorksheetMerge: shift merge range rows >= insert position
fn create_insert_row_vs_add_merge() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        // Quick worksheet check BEFORE parsing
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        // Parse InsertRow params (m1)
        let m1_params: InsertRowMutationParams = match serde_json::from_value(m1.params.clone()) {
            Ok(p) => p,
            Err(_) => return parse_error(m1, m2, "Failed to parse m1 (InsertRow) params"),
        };

        // Parse AddWorksheetMerge params (m2)
        let mut m2_params: AddWorksheetMergeMutationParams = match serde_json::from_value(m2.params.clone()) {
            Ok(p) => p,
            Err(_) => return parse_error(m1, m2, "Failed to parse m2 (AddWorksheetMerge) params"),
        };

        // Double-check worksheet match
        if m1_params.sub_unit_params.unit_id != m2_params.unit_id
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_id
        {
            return identity(m1, m2);
        }

        let insert_start = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - m1_params.range.start_row + 1;

        // Shift all merge ranges
        for range in &mut m2_params.ranges {
            shift_range_rows_for_insert(range, insert_start, insert_count);
        }

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(MutationInfo {
                id: m2.id.clone(),
                params: serde_json::to_value(m2_params).unwrap_or_else(|_| m2.params.clone()),
            }),
            error: None,
        }
    })
}

/// InsertCol vs AddWorksheetMerge: shift merge range columns >= insert position
fn create_insert_col_vs_add_merge() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        // Quick worksheet check BEFORE parsing
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        // Parse InsertCol params (m1)
        let m1_params: InsertColMutationParams = match serde_json::from_value(m1.params.clone()) {
            Ok(p) => p,
            Err(_) => return parse_error(m1, m2, "Failed to parse m1 (InsertCol) params"),
        };

        // Parse AddWorksheetMerge params (m2)
        let mut m2_params: AddWorksheetMergeMutationParams = match serde_json::from_value(m2.params.clone()) {
            Ok(p) => p,
            Err(_) => return parse_error(m1, m2, "Failed to parse m2 (AddWorksheetMerge) params"),
        };

        // Double-check worksheet match
        if m1_params.sub_unit_params.unit_id != m2_params.unit_id
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_id
        {
            return identity(m1, m2);
        }

        let insert_start = m1_params.range.start_column;
        let insert_count = m1_params.range.end_column - m1_params.range.start_column + 1;

        // Shift all merge ranges
        for range in &mut m2_params.ranges {
            shift_range_cols_for_insert(range, insert_start, insert_count);
        }

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(MutationInfo {
                id: m2.id.clone(),
                params: serde_json::to_value(m2_params).unwrap_or_else(|_| m2.params.clone()),
            }),
            error: None,
        }
    })
}

/// RemoveRow vs AddWorksheetMerge: shift/remove merge ranges based on removed rows
fn create_remove_row_vs_add_merge() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        // Quick worksheet check BEFORE parsing
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        // Parse RemoveRow params (m1)
        let m1_params: RemoveRowsMutationParams = match serde_json::from_value(m1.params.clone()) {
            Ok(p) => p,
            Err(_) => return parse_error(m1, m2, "Failed to parse m1 (RemoveRow) params"),
        };

        // Parse AddWorksheetMerge params (m2)
        let mut m2_params: AddWorksheetMergeMutationParams = match serde_json::from_value(m2.params.clone()) {
            Ok(p) => p,
            Err(_) => return parse_error(m1, m2, "Failed to parse m2 (AddWorksheetMerge) params"),
        };

        // Double-check worksheet match
        if m1_params.sub_unit_params.unit_id != m2_params.unit_id
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_id
        {
            return identity(m1, m2);
        }

        let remove_start = m1_params.range.start_row;
        let remove_end = m1_params.range.end_row;

        // Shift/remove merge ranges, keeping only valid ones
        m2_params.ranges.retain_mut(|range| {
            shift_range_rows_for_remove(range, remove_start, remove_end)
        });

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(MutationInfo {
                id: m2.id.clone(),
                params: serde_json::to_value(m2_params).unwrap_or_else(|_| m2.params.clone()),
            }),
            error: None,
        }
    })
}

/// RemoveCol vs AddWorksheetMerge: shift/remove merge ranges based on removed columns
fn create_remove_col_vs_add_merge() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        // Quick worksheet check BEFORE parsing
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        // Parse RemoveCol params (m1)
        let m1_params: RemoveColMutationParams = match serde_json::from_value(m1.params.clone()) {
            Ok(p) => p,
            Err(_) => return parse_error(m1, m2, "Failed to parse m1 (RemoveCol) params"),
        };

        // Parse AddWorksheetMerge params (m2)
        let mut m2_params: AddWorksheetMergeMutationParams = match serde_json::from_value(m2.params.clone()) {
            Ok(p) => p,
            Err(_) => return parse_error(m1, m2, "Failed to parse m2 (AddWorksheetMerge) params"),
        };

        // Double-check worksheet match
        if m1_params.sub_unit_params.unit_id != m2_params.unit_id
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_id
        {
            return identity(m1, m2);
        }

        let remove_start = m1_params.range.start_column;
        let remove_end = m1_params.range.end_column;

        // Shift/remove merge ranges, keeping only valid ones
        m2_params.ranges.retain_mut(|range| {
            shift_range_cols_for_remove(range, remove_start, remove_end)
        });

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(MutationInfo {
                id: m2.id.clone(),
                params: serde_json::to_value(m2_params).unwrap_or_else(|_| m2.params.clone()),
            }),
            error: None,
        }
    })
}
