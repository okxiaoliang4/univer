use crate::mutations::sheets::{InsertRowMutationParams, RemoveRowsMutationParams, SetRangeValuesMutationParams};
use crate::registry::{MutationId, TransformFnRef, TransformRegistry};
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef};
use crate::utils::shift::shift_row_keys_for_remove;
use crate::utils::params::same_worksheet;
use std::sync::Arc;

pub const MUTATION_ID: MutationId = "sheet.mutation.remove-rows";

/// Register all transforms for remove-rows mutation
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transform (remove-rows vs remove-rows)
    registry.register_symmetric_ref(MUTATION_ID, create_self_transform());

    // Bidirectional: remove-rows vs set-range-values
    registry.register_bidirectional_ref(
        MUTATION_ID,
        "sheet.mutation.set-range-values",
        create_transform_with_set_range_values(),
    );

    // Bidirectional: remove-rows vs insert-row
    registry.register_bidirectional_ref(
        MUTATION_ID,
        "sheet.mutation.insert-row",
        create_transform_with_insert_row(),
    );

    // Identity: remove-rows vs insert-col/remove-col (different dimensions)
    registry.register_identity(MUTATION_ID, "sheet.mutation.insert-col");
    registry.register_identity(MUTATION_ID, "sheet.mutation.remove-col");

    // Identity transforms with non-interfering mutations
    registry.register_identity(MUTATION_ID, "sheet.mutation.move-range");
    registry.register_identity(MUTATION_ID, "sheet.mutation.add-worksheet-merge");
    registry.register_identity(MUTATION_ID, "sheet.mutation.set-range-protection");
    registry.register_identity(MUTATION_ID, "sheet.mutation.set-range-theme");
    registry.register_identity(MUTATION_ID, "sheet.mutation.set.numfmt");
    registry.register_identity(MUTATION_ID, "sheet.mutation.set-frozen");
    registry.register_identity(MUTATION_ID, "sheet.mutation.set-row-data");
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

/// Create the symmetric transform for remove-rows vs remove-rows
fn create_self_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        // Quick worksheet check BEFORE parsing (avoids clone for different worksheets)
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);  // Zero-copy!
        }

        // Parse m1 params (only cloned when we actually need the data)
        let m1_params: RemoveRowsMutationParams = match serde_json::from_value(m1.params.clone()) {
            Ok(p) => p,
            Err(_) => return parse_error(m1, m2, "Failed to parse m1 params"),
        };

        let mut m2_params: RemoveRowsMutationParams = match serde_json::from_value(m2.params.clone()) {
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
        let m1_end = m1_params.range.end_row;
        let m1_count = m1_end - m1_start + 1;

        let m2_start = m2_params.range.start_row;
        let m2_end = m2_params.range.end_row;

        // Check for overlap
        if m2_end < m1_start {
            // m2 is completely before m1, no adjustment needed
            return identity(m1, m2);  // Zero-copy!
        } else if m2_start > m1_end {
            // m2 is completely after m1, shift down
            m2_params.range.start_row -= m1_count;
            m2_params.range.end_row -= m1_count;
        } else {
            // There's overlap - this is tricky
            // If m2 is completely contained in m1, m2 becomes noop
            if m2_start >= m1_start && m2_end <= m1_end {
                return TransformResultRef {
                    m1_prime: MutationOutcome::Unchanged(m1),  // Zero-copy!
                    m2_prime: MutationOutcome::Removed, // m2 is already removed by m1
                    error: None,
                };
            }

            // Partial overlap - adjust m2 to remove only what remains
            if m2_start < m1_start && m2_end <= m1_end {
                // m2 starts before, ends within m1
                m2_params.range.end_row = m1_start - 1;
            } else if m2_start >= m1_start && m2_end > m1_end {
                // m2 starts within, ends after m1
                m2_params.range.start_row = m1_start;
                m2_params.range.end_row = m2_end - m1_count;
            } else {
                // m2 completely contains m1
                m2_params.range.end_row -= m1_count;
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

/// Create the bidirectional transform for remove-rows vs set-range-values
fn create_transform_with_set_range_values() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        // Quick worksheet check BEFORE parsing
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);  // Zero-copy!
        }

        let m1_params: RemoveRowsMutationParams = match serde_json::from_value(m1.params.clone()) {
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

        let remove_start = m1_params.range.start_row;
        let remove_end = m1_params.range.end_row;

        // Shift the cell values in m2 to account for the removed rows
        if let Some(ref mut cell_value) = m2_params.cell_value {
            shift_row_keys_for_remove(cell_value, remove_start, remove_end);
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

/// Create the bidirectional transform for remove-rows vs insert-row
fn create_transform_with_insert_row() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        // Quick worksheet check BEFORE parsing
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);  // Zero-copy!
        }

        let m1_params: RemoveRowsMutationParams = match serde_json::from_value(m1.params.clone()) {
            Ok(p) => p,
            Err(_) => return parse_error(m1, m2, "Failed to parse m1 params"),
        };

        let mut m2_params: InsertRowMutationParams = match serde_json::from_value(m2.params.clone()) {
            Ok(p) => p,
            Err(_) => return parse_error(m1, m2, "Failed to parse m2 params"),
        };

        // Double-check worksheet match
        if m1_params.sub_unit_params.unit_id != m2_params.sub_unit_params.unit_id
            || m1_params.sub_unit_params.sub_unit_id != m2_params.sub_unit_params.sub_unit_id
        {
            return identity(m1, m2);  // Zero-copy!
        }

        let remove_start = m1_params.range.start_row;
        let remove_end = m1_params.range.end_row;
        let remove_count = remove_end - remove_start + 1;

        let insert_start = m2_params.range.start_row;

        // Adjust insert position based on removal
        if insert_start > remove_end {
            // Insert is after the removed range
            m2_params.range.start_row -= remove_count;
            m2_params.range.end_row -= remove_count;
        } else if insert_start >= remove_start {
            // Insert is within the removed range, move to start of removal
            m2_params.range.start_row = remove_start;
            m2_params.range.end_row = remove_start + (m2_params.range.end_row - m2_params.range.start_row);
        }
        // If insert_start < remove_start, no adjustment needed

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
