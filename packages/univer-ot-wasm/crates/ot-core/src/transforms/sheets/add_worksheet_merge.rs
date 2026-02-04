//! Transforms for AddWorksheetMergeMutation
//!
//! Merge ranges (Vec<IRange>) need to be adjusted when rows/columns are inserted, removed, or moved.
//! Uses GenericRangesParams for position shifting since the params structure matches the generic pattern.
//!
//! Self-transform uses range-level conflict resolution:
//! - When two merge operations have overlapping ranges, only the overlapping ranges from m1 are removed
//! - Non-overlapping ranges in m1 are preserved
//! - m2 keeps all its ranges (m2 wins for conflicts)

use crate::mutations::sheets::{
    AddWorksheetMergeMutation, AddWorksheetMergeMutationParams,
    InsertRowMutation, InsertColMutation,
    RemoveRowMutation, RemoveColMutation,
    MoveColsMutation, MoveRangeMutation, MoveRowsMutation, RemoveSheetMutation
};
use crate::registry::{TransformFnRef, TransformRegistry};
use crate::types::{IRange, MutationInfo, MutationOutcome, TransformResultRef};
use crate::utils::GenericRangesParams;
use crate::utils::shared_transforms::{insert_col_shift, insert_row_shift, remove_col_shift, remove_row_shift, move_cols_shift, move_rows_shift, move_range_shift, remove_sheet_shift};
use std::sync::Arc;

/// Check if two ranges overlap
/// Ranges overlap if they share at least one cell
fn ranges_overlap(r1: &IRange, r2: &IRange) -> bool {
    // No overlap if one range is completely above/below or left/right of the other
    !(r1.end_row < r2.start_row || r2.end_row < r1.start_row ||
      r1.end_column < r2.start_column || r2.end_column < r1.start_column)
}

/// Create a range-level conflict resolution transform for merge operations
///
/// Strategy:
/// - For each range in m1, check if it overlaps with any range in m2
/// - If overlapping: remove that range from m1 (m2 wins for that conflict)
/// - If not overlapping: keep the range in m1
/// - m2 keeps all its ranges unchanged
/// - If m1 has no ranges left after filtering, remove m1 entirely
fn merge_range_conflict_transform() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        // Parse both mutations' params
        let params1: Result<AddWorksheetMergeMutationParams, _> = serde_json::from_value(m1.params.clone());
        let params2: Result<AddWorksheetMergeMutationParams, _> = serde_json::from_value(m2.params.clone());

        match (params1, params2) {
            (Ok(p1), Ok(p2)) => {
                // Check if mutations are on the same worksheet
                if p1.unit_id != p2.unit_id || p1.sub_unit_id != p2.sub_unit_id {
                    // Different worksheets - no conflict
                    return TransformResultRef::identity(m1, m2);
                }

                // Store original count before consuming p1.ranges
                let original_count = p1.ranges.len();
                let unit_id = p1.unit_id.clone();
                let sub_unit_id = p1.sub_unit_id.clone();

                // Filter out ranges in m1 that overlap with any range in m2
                let non_overlapping_ranges: Vec<IRange> = p1.ranges
                    .into_iter()
                    .filter(|r1| !p2.ranges.iter().any(|r2| ranges_overlap(r1, r2)))
                    .collect();

                if non_overlapping_ranges.is_empty() {
                    // All ranges in m1 conflict with m2 - remove m1 entirely
                    TransformResultRef {
                        m1_prime: MutationOutcome::Removed,
                        m2_prime: MutationOutcome::Unchanged(m2),
                        error: None,
                    }
                } else if non_overlapping_ranges.len() == original_count {
                    // No ranges were removed - return identity
                    TransformResultRef::identity(m1, m2)
                } else {
                    // Some ranges were removed - create modified m1
                    let modified_params = AddWorksheetMergeMutationParams {
                        unit_id,
                        sub_unit_id,
                        ranges: non_overlapping_ranges,
                    };
                    let modified_m1 = MutationInfo {
                        id: m1.id.clone(),
                        params: serde_json::to_value(modified_params).unwrap(),
                    };
                    TransformResultRef {
                        m1_prime: MutationOutcome::Modified(modified_m1),
                        m2_prime: MutationOutcome::Unchanged(m2),
                        error: None,
                    }
                }
            }
            _ => {
                // Failed to parse - return identity (let both proceed)
                TransformResultRef::identity(m1, m2)
            }
        }
    })
}

pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transform: Range-level conflict resolution
    // When two merge operations have overlapping ranges, remove only the conflicting ranges from m1
    registry.register_symmetric_ref(AddWorksheetMergeMutation::ID, merge_range_conflict_transform());
    registry.register_bidirectional_ref(InsertRowMutation::ID, AddWorksheetMergeMutation::ID, insert_row_shift::<GenericRangesParams>());
    registry.register_bidirectional_ref(InsertColMutation::ID, AddWorksheetMergeMutation::ID, insert_col_shift::<GenericRangesParams>());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, AddWorksheetMergeMutation::ID, remove_row_shift::<GenericRangesParams>());
    registry.register_bidirectional_ref(RemoveColMutation::ID, AddWorksheetMergeMutation::ID, remove_col_shift::<GenericRangesParams>());
    registry.register_bidirectional_ref(MoveRowsMutation::ID, AddWorksheetMergeMutation::ID, move_rows_shift::<GenericRangesParams>());
    registry.register_bidirectional_ref(MoveColsMutation::ID, AddWorksheetMergeMutation::ID, move_cols_shift::<GenericRangesParams>());
    registry.register_bidirectional_ref(MoveRangeMutation::ID, AddWorksheetMergeMutation::ID, move_range_shift::<GenericRangesParams>());
    registry.register_bidirectional_ref(RemoveSheetMutation::ID, AddWorksheetMergeMutation::ID, remove_sheet_shift::<GenericRangesParams>());
}
