use crate::mutations::sheets::{
    RemoveColMutation, SetRangeValuesMutation, InsertColMutation, InsertRowMutation,
    RemoveRowMutation, MoveRangeMutation, MoveColsMutation, AddWorksheetMergeMutation,
    SetRangeProtectionMutation, SetRangeThemeMutation, SetFrozenMutation, SetRowDataMutation,
    InsertSheetMutation, SetWorkbookNameMutation,
    InsertColMutationParams, RemoveColMutationParams, SetRangeValuesMutationParams,
};
use crate::mutations::sheets_numfmt::SetNumfmtMutation;
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef};
use crate::utils::shift::shift_col_keys_for_remove;
use crate::utils::params::same_worksheet;
use std::sync::Arc;

pub const MUTATION_ID: MutationId = RemoveColMutation::ID;

/// Register all transforms for remove-col mutation
pub fn register_transforms(registry: &mut TransformRegistry) {
    registry.register_symmetric_ref(MUTATION_ID, create_self_transform());
    registry.register_bidirectional_ref(
        MUTATION_ID,
        SetRangeValuesMutation::ID,
        create_transform_with_set_range_values(),
    );
    registry.register_bidirectional_ref(
        MUTATION_ID,
        InsertColMutation::ID,
        create_transform_with_insert_col(),
    );
    registry.register_identity(MUTATION_ID, InsertRowMutation::ID);
    registry.register_identity(MUTATION_ID, RemoveRowMutation::ID);

    // Identity transforms with non-interfering mutations
    registry.register_identity(MUTATION_ID, MoveRangeMutation::ID);
    registry.register_identity(MUTATION_ID, MoveColsMutation::ID);
    registry.register_identity(MUTATION_ID, AddWorksheetMergeMutation::ID);
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

fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        // Quick worksheet check BEFORE parsing (avoids clone for different worksheets)
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);  // Zero-copy!
        }

        let m1_params: RemoveColMutationParams = match serde_json::from_value(m1.params.clone()) {
            Ok(p) => p,
            Err(_) => return parse_error(m1, m2, "Failed to parse m1 params"),
        };

        let mut m2_params: RemoveColMutationParams = match serde_json::from_value(m2.params.clone()) {
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
        let m1_end = m1_params.range.end_column;
        let m1_count = m1_end - m1_start + 1;

        let m2_start = m2_params.range.start_column;
        let m2_end = m2_params.range.end_column;

        if m2_end < m1_start {
            return identity(m1, m2);  // Zero-copy!
        } else if m2_start > m1_end {
            m2_params.range.start_column -= m1_count;
            m2_params.range.end_column -= m1_count;
        } else {
            if m2_start >= m1_start && m2_end <= m1_end {
                return TransformResultRef {
                    m1_prime: MutationOutcome::Unchanged(m1),  // Zero-copy!
                    m2_prime: MutationOutcome::Removed,
                    error: None,
                };
            }

            if m2_start < m1_start && m2_end <= m1_end {
                m2_params.range.end_column = m1_start - 1;
            } else if m2_start >= m1_start && m2_end > m1_end {
                m2_params.range.start_column = m1_start;
                m2_params.range.end_column = m2_end - m1_count;
            } else {
                m2_params.range.end_column -= m1_count;
            }
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

fn create_transform_with_set_range_values() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        // Quick worksheet check BEFORE parsing
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);  // Zero-copy!
        }

        let m1_params: RemoveColMutationParams = match serde_json::from_value(m1.params.clone()) {
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

        let remove_start = m1_params.range.start_column;
        let remove_end = m1_params.range.end_column;

        if let Some(ref mut cell_value) = m2_params.cell_value {
            shift_col_keys_for_remove(cell_value, remove_start, remove_end);
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

fn create_transform_with_insert_col() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        // Quick worksheet check BEFORE parsing
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);  // Zero-copy!
        }

        let m1_params: RemoveColMutationParams = match serde_json::from_value(m1.params.clone()) {
            Ok(p) => p,
            Err(_) => return parse_error(m1, m2, "Failed to parse m1 params"),
        };

        let mut m2_params: InsertColMutationParams = match serde_json::from_value(m2.params.clone()) {
            Ok(p) => p,
            Err(_) => return parse_error(m1, m2, "Failed to parse m2 params"),
        };

        // Double-check worksheet match
        if m1_params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
        {
            return identity(m1, m2);  // Zero-copy!
        }

        let remove_start = m1_params.range.start_column;
        let remove_end = m1_params.range.end_column;
        let remove_count = remove_end - remove_start + 1;

        let insert_start = m2_params.range.start_column;

        if insert_start > remove_end {
            m2_params.range.start_column -= remove_count;
            m2_params.range.end_column -= remove_count;
        } else if insert_start >= remove_start {
            m2_params.range.start_column = remove_start;
            m2_params.range.end_column = remove_start + (m2_params.range.end_column - m2_params.range.start_column);
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
