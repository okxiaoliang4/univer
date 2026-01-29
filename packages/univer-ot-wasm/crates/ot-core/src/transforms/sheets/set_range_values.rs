use crate::params::SetRangeValuesMutationParams;
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef};
use crate::utils::params::same_worksheet;
use std::sync::Arc;

pub const MUTATION_ID: MutationId = "sheet.mutation.set-range-values";

/// Register all transforms for set-range-values mutation
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transform (set-range-values vs set-range-values)
    registry.register_symmetric_ref(MUTATION_ID, create_self_transform());
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

/// Create the symmetric transform for set-range-values vs set-range-values
fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        // Quick worksheet check BEFORE parsing (avoids clone for different worksheets)
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);  // Zero-copy!
        }

        // Parse both parameters
        let m1_params: SetRangeValuesMutationParams = match serde_json::from_value(m1.params.clone()) {
            Ok(p) => p,
            Err(_) => return parse_error(m1, m2, "Failed to parse m1 params"),
        };

        let m2_params: SetRangeValuesMutationParams = match serde_json::from_value(m2.params.clone()) {
            Ok(p) => p,
            Err(_) => return parse_error(m1, m2, "Failed to parse m2 params"),
        };

        // Double-check worksheet match (in case quick check returned None)
        if m1_params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
        {
            return identity(m1, m2);  // Zero-copy!
        }

        // Implement Last-Write-Wins (LWW) conflict resolution
        // For conflicting cells, m2 wins (it's the "last write")
        let mut m1_prime_params = m1_params.clone();
        let m2_prime_params = m2_params.clone();

        if let (Some(m1_cells), Some(m2_cells)) = (&m1_params.cell_value, &m2_params.cell_value) {
            // Find conflicting cells
            let mut m1_prime_cells = m1_cells.data.clone();
            let m2_prime_cells = m2_cells.data.clone();

            // For each cell in m1, check if m2 also modifies it
            for (row_key, row_value) in &m1_cells.data {
                if let Some(m2_row) = m2_cells.data.get(row_key) {
                    if let (serde_json::Value::Object(m1_cols), serde_json::Value::Object(m2_cols)) =
                        (row_value, m2_row)
                    {
                        // Check for column conflicts
                        let mut m1_prime_row = m1_cols.clone();

                        for col_key in m1_cols.keys() {
                            if m2_cols.contains_key(col_key) {
                                // Conflict! LWW: m2 wins, so remove from m1_prime
                                m1_prime_row.remove(col_key);
                            }
                        }

                        // Update or remove the row in m1_prime
                        if m1_prime_row.is_empty() {
                            m1_prime_cells.remove(row_key);
                        } else {
                            m1_prime_cells.insert(
                                row_key.clone(),
                                serde_json::Value::Object(m1_prime_row),
                            );
                        }
                    }
                }
            }

            // Update m1_prime with filtered cells
            if m1_prime_cells.is_empty() {
                m1_prime_params.cell_value = None;
            } else {
                m1_prime_params.cell_value = Some(crate::types::ObjectMatrixPrimitiveType {
                    data: m1_prime_cells,
                });
            }

            // m2 keeps all its cells (LWW - it wins on conflicts)
            // Note: m2_prime_params already has a clone of m2_params, which includes the original cell_value
            let _ = m2_prime_cells; // m2_prime_cells is same as original, no need to reassign
        }

        TransformResultRef {
            m1_prime: MutationOutcome::Modified(MutationInfo {
                id: m1.id.clone(),
                params: serde_json::to_value(m1_prime_params).unwrap_or_else(|_| m1.params.clone()),
            }),
            m2_prime: MutationOutcome::Modified(MutationInfo {
                id: m2.id.clone(),
                params: serde_json::to_value(m2_prime_params).unwrap_or_else(|_| m2.params.clone()),
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
    fn test_different_worksheets() {
        let m1 = MutationInfo {
            id: MUTATION_ID.to_string(),
            params: json!({
                "unitId": "workbook1",
                "subUnitId": "sheet1",
                "cellValue": {}
            }),
        };

        let m2 = MutationInfo {
            id: MUTATION_ID.to_string(),
            params: json!({
                "unitId": "workbook1",
                "subUnitId": "sheet2",
                "cellValue": {}
            }),
        };

        let transform_fn = create_self_transform();
        let result = transform_fn(&m1, &m2);

        assert!(result.m1_prime.is_some());
        assert!(result.m2_prime.is_some());
        assert!(result.error.is_none());
    }

    #[test]
    fn test_same_worksheet() {
        let m1 = MutationInfo {
            id: MUTATION_ID.to_string(),
            params: json!({
                "unitId": "workbook1",
                "subUnitId": "sheet1",
                "cellValue": {}
            }),
        };

        let m2 = MutationInfo {
            id: MUTATION_ID.to_string(),
            params: json!({
                "unitId": "workbook1",
                "subUnitId": "sheet1",
                "cellValue": {}
            }),
        };

        let transform_fn = create_self_transform();
        let result = transform_fn(&m1, &m2);

        assert!(result.m1_prime.is_some());
        assert!(result.m2_prime.is_some());
        assert!(result.error.is_none());
    }
}
