//! Transforms for SetColDataMutation
//!
//! SetColDataMutation has column_data fields where keys are column indices.
//! When column insert/remove operations happen, these keys need adjustment.

use crate::mutations::sheets::{
    SetColDataMutation, InsertColMutation, RemoveColMutation,
    SetColDataMutationParams, InsertColMutationParams, RemoveColMutationParams,
    InsertRowMutation, RemoveRowMutation, SetRangeValuesMutation, SetRowDataMutation,
    MoveRangeMutation, MoveRowsMutation, MoveColsMutation,
    AddWorksheetMergeMutation, SetRangeProtectionMutation, SetRangeThemeMutation,
    SetFrozenMutation, InsertSheetMutation, SetWorkbookNameMutation,
};
use crate::mutations::sheets_numfmt::{SetNumfmtMutation, RemoveNumfmtMutation};
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::utils::transform_helpers::identity_transform;
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef};
use crate::utils::params::same_worksheet;
use crate::utils::shift::{shift_array_keys_for_insert, shift_array_keys_for_remove};
use std::sync::Arc;

pub const MUTATION_ID: MutationId = SetColDataMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transform (SetColData vs SetColData)
    registry.register_symmetric_ref(MUTATION_ID, identity_transform());

    // Bidirectional: InsertCol vs SetColData (column keys need shift)
    registry.register_bidirectional_ref(
        InsertColMutation::ID,
        MUTATION_ID,
        create_insert_col_vs_set_col_data(),
    );

    // Bidirectional: RemoveCol vs SetColData (column keys need shift/remove)
    registry.register_bidirectional_ref(
        RemoveColMutation::ID,
        MUTATION_ID,
        create_remove_col_vs_set_col_data(),
    );

    // Identity: SetColData vs SetRangeValues (different data)
    registry.register_identity(MUTATION_ID, SetRangeValuesMutation::ID);

    // Identity transforms with row operations (rows don't affect column keys)
    registry.register_identity(MUTATION_ID, InsertRowMutation::ID);
    registry.register_identity(MUTATION_ID, RemoveRowMutation::ID);

    // Identity transforms with non-interfering mutations
    registry.register_identity(MUTATION_ID, MoveRangeMutation::ID);
    registry.register_identity(MUTATION_ID, MoveRowsMutation::ID);
    registry.register_identity(MUTATION_ID, MoveColsMutation::ID);
    registry.register_identity(MUTATION_ID, AddWorksheetMergeMutation::ID);
    registry.register_identity(MUTATION_ID, SetRangeProtectionMutation::ID);
    registry.register_identity(MUTATION_ID, SetRangeThemeMutation::ID);
    registry.register_identity(MUTATION_ID, SetNumfmtMutation::ID);
    registry.register_identity(MUTATION_ID, RemoveNumfmtMutation::ID);
    registry.register_identity(MUTATION_ID, SetRowDataMutation::ID);
    registry.register_identity(MUTATION_ID, SetFrozenMutation::ID);
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

/// InsertCol vs SetColData: shift column_data keys >= insert position
fn create_insert_col_vs_set_col_data() -> TransformFnRef {
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

        // Parse SetColData params (m2)
        let mut m2_params: SetColDataMutationParams = match serde_json::from_value(m2.params.clone()) {
            Ok(p) => p,
            Err(_) => return parse_error(m1, m2, "Failed to parse m2 (SetColData) params"),
        };

        // Double-check worksheet match
        if m1_params.sub_unit_params.unit_id != m2_params.unit_id
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_id
        {
            return identity(m1, m2);
        }

        let insert_start = m1_params.range.start_column;
        let insert_count = m1_params.range.end_column - m1_params.range.start_column + 1;

        // Shift column_data keys
        shift_array_keys_for_insert(&mut m2_params.column_data, insert_start, insert_count);

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

/// RemoveCol vs SetColData: shift column_data keys > remove end, remove keys in range
fn create_remove_col_vs_set_col_data() -> TransformFnRef {
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

        // Parse SetColData params (m2)
        let mut m2_params: SetColDataMutationParams = match serde_json::from_value(m2.params.clone()) {
            Ok(p) => p,
            Err(_) => return parse_error(m1, m2, "Failed to parse m2 (SetColData) params"),
        };

        // Double-check worksheet match
        if m1_params.sub_unit_params.unit_id != m2_params.unit_id
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_id
        {
            return identity(m1, m2);
        }

        let remove_start = m1_params.range.start_column;
        let remove_end = m1_params.range.end_column;

        // Shift column_data keys (also removes keys in removed range)
        shift_array_keys_for_remove(&mut m2_params.column_data, remove_start, remove_end);

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
