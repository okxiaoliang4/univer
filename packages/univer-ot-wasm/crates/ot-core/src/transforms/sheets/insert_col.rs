use crate::mutations::sheets::{
    InsertColMutation, InsertColMutationParams,
    SetRangeValuesMutation, SetRangeValuesMutationParams,
    InsertRowMutation, MoveColsMutation, SetRangeThemeMutation,
    SetRowDataMutation, InsertSheetMutation, SetWorkbookNameMutation,
};
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef};
use crate::utils::shift::shift_col_keys_for_insert;
use crate::utils::params::same_worksheet;
use std::sync::Arc;

pub const MUTATION_ID: MutationId = InsertColMutation::ID;

/// Register all transforms for insert-col mutation
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transform (insert-col vs insert-col)
    registry.register_symmetric_ref(MUTATION_ID, create_self_transform());

    // Bidirectional: insert-col vs set-range-values
    registry.register_bidirectional_ref(
        MUTATION_ID,
        SetRangeValuesMutation::ID,
        create_transform_with_set_range_values(),
    );

    // Identity: insert-col vs insert-row (different dimensions, don't interfere)
    registry.register_identity(MUTATION_ID, InsertRowMutation::ID);

    // Identity transforms with non-interfering mutations
    registry.register_identity(MUTATION_ID, MoveColsMutation::ID);
    registry.register_identity(MUTATION_ID, SetRangeThemeMutation::ID);
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

/// Create the symmetric transform for insert-col vs insert-col
fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        // Quick worksheet check BEFORE parsing (avoids clone for different worksheets)
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);  // Zero-copy!
        }

        let m1_params: InsertColMutationParams = match serde_json::from_value(m1.params.clone()) {
            Ok(p) => p,
            Err(_) => return parse_error(m1, m2, "Failed to parse m1 params"),
        };

        let mut m2_params: InsertColMutationParams = match serde_json::from_value(m2.params.clone()) {
            Ok(p) => p,
            Err(_) => return parse_error(m1, m2, "Failed to parse m2 params"),
        };

        // Double-check worksheet match (in case quick check returned None)
        if m1_params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
        {
            return identity(m1, m2);  // Zero-copy!
        }

        let m1_start = m1_params.range.start_column;
        let m1_count = m1_params.range.end_column - m1_params.range.start_column + 1;

        // Adjust m2's insert position based on m1's insert
        if m2_params.range.start_column >= m1_start {
            m2_params.range.start_column += m1_count;
            m2_params.range.end_column += m1_count;
        }

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),  // Zero-copy for m1!
            m2_prime: MutationOutcome::Modified(MutationInfo {
                id: m2.id.clone(),
                params: serde_json::to_value(m2_params).unwrap_or_else(|_| m2.params.clone()),
            }),
            error: None,
        }
    })
}

/// Create the bidirectional transform for insert-col vs set-range-values
fn create_transform_with_set_range_values() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        // Quick worksheet check BEFORE parsing
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);  // Zero-copy!
        }

        let m1_params: InsertColMutationParams = match serde_json::from_value(m1.params.clone()) {
            Ok(p) => p,
            Err(_) => return parse_error(m1, m2, "Failed to parse m1 params"),
        };

        let mut m2_params: SetRangeValuesMutationParams =
            match serde_json::from_value(m2.params.clone()) {
                Ok(p) => p,
                Err(_) => return parse_error(m1, m2, "Failed to parse m2 params"),
            };

        // Double-check worksheet match
        if m1_params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
        {
            return identity(m1, m2);  // Zero-copy!
        }

        let insert_col = m1_params.range.start_column;
        let insert_count = m1_params.range.end_column - m1_params.range.start_column + 1;

        // Shift the cell values in m2 to account for the inserted columns
        if let Some(ref mut cell_value) = m2_params.cell_value {
            shift_col_keys_for_insert(cell_value, insert_col, insert_count);
        }

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),  // Zero-copy for m1!
            m2_prime: MutationOutcome::Modified(MutationInfo {
                id: m2.id.clone(),
                params: serde_json::to_value(m2_params).unwrap_or_else(|_| m2.params.clone()),
            }),
            error: None,
        }
    })
}
