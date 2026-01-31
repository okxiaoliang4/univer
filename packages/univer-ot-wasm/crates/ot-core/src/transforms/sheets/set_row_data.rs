//! Transforms for SetRowDataMutation and SetColDataMutation
//!
//! These mutations have row_data/column_data fields where keys are indices.
//! When row/column insert/remove operations happen, these keys need adjustment.

use crate::mutations::sheets::{
    SetRowDataMutation, InsertRowMutation, SetRangeValuesMutation, InsertColMutation,
    RemoveRowMutation, RemoveColMutation, MoveRangeMutation, MoveRowsMutation, MoveColsMutation,
    AddWorksheetMergeMutation, SetRangeProtectionMutation, SetRangeThemeMutation,
    SetFrozenMutation, InsertSheetMutation, SetWorkbookNameMutation,
    SetRowDataMutationParams, InsertRowMutationParams, RemoveRowsMutationParams,
};
use crate::mutations::sheets_numfmt::SetNumfmtMutation;
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::utils::transform_helpers::identity_transform;
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef};
use crate::utils::params::same_worksheet;
use crate::utils::shift::{shift_array_keys_for_insert, shift_array_keys_for_remove};
use std::sync::Arc;

pub const MUTATION_ID: MutationId = SetRowDataMutation::ID;

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transform (SetRowData vs SetRowData)
    registry.register_symmetric_ref(MUTATION_ID, identity_transform());

    // Bidirectional: InsertRow vs SetRowData (row keys need shift)
    registry.register_bidirectional_ref(
        InsertRowMutation::ID,
        MUTATION_ID,
        create_insert_row_vs_set_row_data(),
    );

    // Bidirectional: RemoveRows vs SetRowData (row keys need shift/remove)
    registry.register_bidirectional_ref(
        RemoveRowMutation::ID,
        MUTATION_ID,
        create_remove_rows_vs_set_row_data(),
    );

    // Identity: SetRowData vs SetRangeValues (different data)
    registry.register_identity(MUTATION_ID, SetRangeValuesMutation::ID);

    // Identity transforms with column operations (columns don't affect row keys)
    registry.register_identity(MUTATION_ID, InsertColMutation::ID);
    registry.register_identity(MUTATION_ID, RemoveColMutation::ID);

    // Identity transforms with non-interfering mutations
    registry.register_identity(MUTATION_ID, MoveRangeMutation::ID);
    registry.register_identity(MUTATION_ID, MoveRowsMutation::ID);
    registry.register_identity(MUTATION_ID, MoveColsMutation::ID);
    registry.register_identity(MUTATION_ID, AddWorksheetMergeMutation::ID);
    registry.register_identity(MUTATION_ID, SetRangeProtectionMutation::ID);
    registry.register_identity(MUTATION_ID, SetRangeThemeMutation::ID);
    registry.register_identity(MUTATION_ID, SetNumfmtMutation::ID);
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

/// InsertRow vs SetRowData: shift row_data keys >= insert position
fn create_insert_row_vs_set_row_data() -> TransformFnRef {
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

        // Parse SetRowData params (m2)
        let mut m2_params: SetRowDataMutationParams = match serde_json::from_value(m2.params.clone()) {
            Ok(p) => p,
            Err(_) => return parse_error(m1, m2, "Failed to parse m2 (SetRowData) params"),
        };

        // Double-check worksheet match
        if m1_params.sub_unit_params.unit_id != m2_params.unit_id
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_id
        {
            return identity(m1, m2);
        }

        let insert_start = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - m1_params.range.start_row + 1;

        // Shift row_data keys
        shift_array_keys_for_insert(&mut m2_params.row_data, insert_start, insert_count);

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

/// RemoveRows vs SetRowData: shift row_data keys > remove end, remove keys in range
fn create_remove_rows_vs_set_row_data() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        // Quick worksheet check BEFORE parsing
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        // Parse RemoveRows params (m1)
        let m1_params: RemoveRowsMutationParams = match serde_json::from_value(m1.params.clone()) {
            Ok(p) => p,
            Err(_) => return parse_error(m1, m2, "Failed to parse m1 (RemoveRows) params"),
        };

        // Parse SetRowData params (m2)
        let mut m2_params: SetRowDataMutationParams = match serde_json::from_value(m2.params.clone()) {
            Ok(p) => p,
            Err(_) => return parse_error(m1, m2, "Failed to parse m2 (SetRowData) params"),
        };

        // Double-check worksheet match
        if m1_params.sub_unit_params.unit_id != m2_params.unit_id
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_id
        {
            return identity(m1, m2);
        }

        let remove_start = m1_params.range.start_row;
        let remove_end = m1_params.range.end_row;

        // Shift row_data keys (also removes keys in removed range)
        shift_array_keys_for_remove(&mut m2_params.row_data, remove_start, remove_end);

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
