use crate::mutations::sheets::{InsertRowMutationParams, SetRangeValuesMutationParams};
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef};
use crate::utils::shift::shift_row_keys_for_insert;
use crate::utils::params::same_worksheet;
use std::sync::Arc;

pub const MUTATION_ID: MutationId = "sheet.mutation.insert-row";

/// Register all transforms for insert-row mutation
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transform (insert-row vs insert-row)
    registry.register_symmetric_ref(MUTATION_ID, create_self_transform());

    // Bidirectional: insert-row vs set-range-values
    registry.register_bidirectional_ref(
        MUTATION_ID,
        "sheet.mutation.set-range-values",
        create_transform_with_set_range_values(),
    );

    // Identity transforms with non-interfering mutations
    // These operations don't affect row positions or are at different levels
    registry.register_identity(MUTATION_ID, "sheet.mutation.set-range-theme");
    registry.register_identity(MUTATION_ID, "sheet.mutation.insert-sheet");
    registry.register_identity(MUTATION_ID, "sheet.mutation.set-workbook-name");
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

/// Create the symmetric transform for insert-row vs insert-row
fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        // Quick worksheet check BEFORE parsing (avoids clone for different worksheets)
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);  // Zero-copy!
        }

        let m1_params: InsertRowMutationParams = match serde_json::from_value(m1.params.clone()) {
            Ok(p) => p,
            Err(_) => return parse_error(m1, m2, "Failed to parse m1 params"),
        };

        let mut m2_params: InsertRowMutationParams = match serde_json::from_value(m2.params.clone()) {
            Ok(p) => p,
            Err(_) => return parse_error(m1, m2, "Failed to parse m2 params"),
        };

        // Double-check worksheet match (in case quick check returned None)
        if m1_params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
        {
            return identity(m1, m2);  // Zero-copy!
        }

        let m1_start = m1_params.range.start_row;
        let m1_count = m1_params.range.end_row - m1_params.range.start_row + 1;

        // Adjust m2's insert position based on m1's insert
        if m2_params.range.start_row >= m1_start {
            m2_params.range.start_row += m1_count;
            m2_params.range.end_row += m1_count;
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

/// Create the bidirectional transform for insert-row vs set-range-values
/// Note: Only the forward direction is implemented, reverse is auto-generated via swap
fn create_transform_with_set_range_values() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        // Quick worksheet check BEFORE parsing
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);  // Zero-copy!
        }

        let m1_params: InsertRowMutationParams = match serde_json::from_value(m1.params.clone()) {
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

        let insert_row = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - m1_params.range.start_row + 1;

        // Shift the cell values in m2 to account for the inserted rows
        if let Some(ref mut cell_value) = m2_params.cell_value {
            shift_row_keys_for_insert(cell_value, insert_row, insert_count);
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_insert_row_vs_insert_row_same_position() {
        let m1 = MutationInfo {
            id: MUTATION_ID.to_string(),
            params: json!({
                "unitId": "workbook1",
                "subUnitId": "sheet1",
                "range": {
                    "startRow": 5,
                    "startColumn": 0,
                    "endRow": 5,
                    "endColumn": 10
                }
            }),
        };

        let m2 = MutationInfo {
            id: MUTATION_ID.to_string(),
            params: json!({
                "unitId": "workbook1",
                "subUnitId": "sheet1",
                "range": {
                    "startRow": 5,
                    "startColumn": 0,
                    "endRow": 5,
                    "endColumn": 10
                }
            }),
        };

        let transform_fn = create_self_transform();
        let result = transform_fn(&m1, &m2);

        assert!(result.m1_prime.is_some());
        assert!(result.m2_prime.is_some());

        // m2's position should be shifted
        let m2_prime = result.m2_prime.unwrap();
        let m2_prime_params: InsertRowMutationParams =
            serde_json::from_value(m2_prime.params).unwrap();
        assert_eq!(m2_prime_params.range.start_row, 6);
    }

    #[test]
    fn test_insert_row_vs_set_range_values() {
        let m1 = MutationInfo {
            id: MUTATION_ID.to_string(),
            params: json!({
                "unitId": "workbook1",
                "subUnitId": "sheet1",
                "range": {
                    "startRow": 5,
                    "startColumn": 0,
                    "endRow": 5,
                    "endColumn": 10
                }
            }),
        };

        let m2 = MutationInfo {
            id: "sheet.mutation.set-range-values".to_string(),
            params: json!({
                "unitId": "workbook1",
                "subUnitId": "sheet1",
                "cellValue": {
                    "5": { "0": { "v": "test" } }
                }
            }),
        };

        let transform_fn = create_transform_with_set_range_values();
        let result = transform_fn(&m1, &m2);

        assert!(result.m1_prime.is_some());
        assert!(result.m2_prime.is_some());

        // Row 5 in cellValue should be shifted to row 6
        let m2_prime = result.m2_prime.unwrap();
        let m2_prime_params: SetRangeValuesMutationParams =
            serde_json::from_value(m2_prime.params).unwrap();
        assert!(m2_prime_params.cell_value.is_some());
        let cell_value = m2_prime_params.cell_value.unwrap();
        assert!(cell_value.data.contains_key("6"));
        assert!(!cell_value.data.contains_key("5"));
    }
}
