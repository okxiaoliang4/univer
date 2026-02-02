//! Transforms for Pivot Table mutations
//!
//! Pivot tables have complex conflict dimensions:
//! 1. Position shifts: source_range_info (IRange) and target_cell_info (row, col)
//! 2. Feature ID conflict: RemovePivotTable should remove Set mutations with same pivot_table_id
//! 3. Cross-sheet awareness: source_range and target_cell can be on different sheets

use crate::utils::shared_transforms as shared;
use crate::mutations::{
    InsertColMutation, InsertRowMutation, MoveColsMutation, MoveRangeMutation,
    MoveRowsMutation, RemoveColMutation, RemoveRowMutation, RemoveSheetMutation,
};
use crate::mutations::sheets_pivot_table::{
    AddPivotTableMutation, AddPivotTableMutationParams,
    RemovePivotTableMutation, RemovePivotTableMutationParams,
    SetPivotTableSourceRangeMutation, SetPivotTableSourceRangeMutationParams,
    SetPivotTableTargetCellMutation, SetPivotTableTargetCellMutationParams,
    SetPivotTableFieldsConfigMutation, SetPivotTableFieldsConfigMutationParams,
    SetPivotTableCalculatedDataMutation, SetPivotTableCalculatedDataMutationParams,
};
use crate::registry::{MutationId, TransformRegistry, TransformFnRef};
use crate::types::{MutationInfo, MutationOutcome, TransformResultRef};
use crate::utils::{ShiftOperation, ShiftResult, Shiftable, WorksheetParams};
use crate::utils::shiftable::{
    ShiftableByRowInsert, ShiftableByRowRemove, ShiftableByColInsert, ShiftableByColRemove,
};
use crate::utils::transform_helpers::{lww_transform, identity_transform};
use std::sync::Arc;

pub const ADD_PIVOT_TABLE_ID: MutationId = AddPivotTableMutation::ID;
pub const REMOVE_PIVOT_TABLE_ID: MutationId = RemovePivotTableMutation::ID;
pub const SET_SOURCE_RANGE_ID: MutationId = SetPivotTableSourceRangeMutation::ID;
pub const SET_TARGET_CELL_ID: MutationId = SetPivotTableTargetCellMutation::ID;
pub const SET_FIELDS_CONFIG_ID: MutationId = SetPivotTableFieldsConfigMutation::ID;
pub const SET_CALCULATED_DATA_ID: MutationId = SetPivotTableCalculatedDataMutation::ID;

// ============================================================================
// WorksheetParams implementations
// ============================================================================

impl WorksheetParams for AddPivotTableMutationParams {
    fn unit_id(&self) -> &str { &self.unit_id }
    fn sub_unit_id(&self) -> &str { &self.sub_unit_id }
}

impl WorksheetParams for RemovePivotTableMutationParams {
    fn unit_id(&self) -> &str { &self.unit_id }
    fn sub_unit_id(&self) -> &str { &self.sub_unit_id }
}

impl WorksheetParams for SetPivotTableSourceRangeMutationParams {
    fn unit_id(&self) -> &str { &self.unit_id }
    fn sub_unit_id(&self) -> &str { &self.sub_unit_id }
}

impl WorksheetParams for SetPivotTableTargetCellMutationParams {
    fn unit_id(&self) -> &str { &self.unit_id }
    fn sub_unit_id(&self) -> &str { &self.sub_unit_id }
}

impl WorksheetParams for SetPivotTableFieldsConfigMutationParams {
    fn unit_id(&self) -> &str { &self.unit_id }
    fn sub_unit_id(&self) -> &str { &self.sub_unit_id }
}

impl WorksheetParams for SetPivotTableCalculatedDataMutationParams {
    fn unit_id(&self) -> &str { &self.unit_id }
    fn sub_unit_id(&self) -> &str { &self.sub_unit_id }
}

// ============================================================================
// Shiftable for AddPivotTableMutationParams
// ============================================================================

impl Shiftable for AddPivotTableMutationParams {
    fn apply_shift(&mut self, op: &ShiftOperation) -> ShiftResult {
        let source = &mut self.config.source_range_info;
        let target = &mut self.config.target_cell_info;

        match op {
            ShiftOperation::InsertRows { start, count } => {
                // Shift source range if on same sheet
                if source.sub_unit_id == self.sub_unit_id {
                    source.range.shift_for_row_insert(*start, *count);
                }
                // Shift target cell if on same sheet
                if target.sub_unit_id == self.sub_unit_id {
                    target.row += *count;
                }
                ShiftResult::Modified
            }
            ShiftOperation::RemoveRows { start, end } => {
                let mut source_valid = true;
                let mut target_valid = true;

                if source.sub_unit_id == self.sub_unit_id {
                    source_valid = source.range.shift_for_row_remove(*start, *end);
                }
                if target.sub_unit_id == self.sub_unit_id {
                    target_valid = target.row >= *start && target.row <= *end;
                }

                if !source_valid || !target_valid {
                    ShiftResult::Removed
                } else {
                    ShiftResult::Modified
                }
            }
            ShiftOperation::InsertCols { start, count } => {
                if source.sub_unit_id == self.sub_unit_id {
                    source.range.shift_for_col_insert(*start, *count);
                }
                if target.sub_unit_id == self.sub_unit_id {
                    target.col += *count;
                }
                ShiftResult::Modified
            }
            ShiftOperation::RemoveCols { start, end } => {
                let mut source_valid = true;
                let mut target_valid = true;

                if source.sub_unit_id == self.sub_unit_id {
                    source_valid = source.range.shift_for_col_remove(*start, *end);
                }
                if target.sub_unit_id == self.sub_unit_id {
                    target_valid = target.col >= *start && target.col <= *end;
                }

                if !source_valid || !target_valid {
                    ShiftResult::Removed
                } else {
                    ShiftResult::Modified
                }
            }
            ShiftOperation::MoveRows { from_start, from_end, to } => {
                if source.sub_unit_id == self.sub_unit_id {
                    shift_range_for_move_rows(&mut source.range, *from_start, *from_end, *to);
                }
                if target.sub_unit_id == self.sub_unit_id {
                    shift_row_col_for_move_rows(&mut target.row, *from_start, *from_end, *to);
                }
                ShiftResult::Modified
            }
            ShiftOperation::MoveCols { from_start, from_end, to } => {
                if source.sub_unit_id == self.sub_unit_id {
                    shift_range_for_move_cols(&mut source.range, *from_start, *from_end, *to);
                }
                if target.sub_unit_id == self.sub_unit_id {
                    shift_row_col_for_move_cols(&mut target.col, *from_start, *from_end, *to);
                }
                ShiftResult::Modified
            }
            ShiftOperation::MoveRange { from, to } => {
                // For MoveRange, we need to check if source/target overlap with the moved range
                // This is a simplified implementation
                ShiftResult::Unchanged
            }
            ShiftOperation::RemoveSheet { sub_unit_id } => {
                // If this mutation's sheet is removed, remove mutation
                if &self.sub_unit_id == sub_unit_id {
                    return ShiftResult::Removed;
                }
                // Also check if source or target sheet is removed
                if &source.sub_unit_id == sub_unit_id || &target.sub_unit_id == sub_unit_id {
                    return ShiftResult::Removed;
                }
                ShiftResult::Unchanged
            }
        }
    }
}

// ============================================================================
// Shiftable for SetPivotTableSourceRangeMutationParams
// ============================================================================

impl Shiftable for SetPivotTableSourceRangeMutationParams {
    fn apply_shift(&mut self, op: &ShiftOperation) -> ShiftResult {
        let source = &mut self.source_range_info;

        match op {
            ShiftOperation::InsertRows { start, count } => {
                if source.sub_unit_id == self.sub_unit_id {
                    source.range.shift_for_row_insert(*start, *count);
                    ShiftResult::Modified
                } else {
                    ShiftResult::Unchanged
                }
            }
            ShiftOperation::RemoveRows { start, end } => {
                if source.sub_unit_id == self.sub_unit_id {
                    if !source.range.shift_for_row_remove(*start, *end) {
                        return ShiftResult::Removed;
                    }
                    ShiftResult::Modified
                } else {
                    ShiftResult::Unchanged
                }
            }
            ShiftOperation::InsertCols { start, count } => {
                if source.sub_unit_id == self.sub_unit_id {
                    source.range.shift_for_col_insert(*start, *count);
                    ShiftResult::Modified
                } else {
                    ShiftResult::Unchanged
                }
            }
            ShiftOperation::RemoveCols { start, end } => {
                if source.sub_unit_id == self.sub_unit_id {
                    if !source.range.shift_for_col_remove(*start, *end) {
                        return ShiftResult::Removed;
                    }
                    ShiftResult::Modified
                } else {
                    ShiftResult::Unchanged
                }
            }
            ShiftOperation::MoveRows { from_start, from_end, to } => {
                if source.sub_unit_id == self.sub_unit_id {
                    shift_range_for_move_rows(&mut source.range, *from_start, *from_end, *to);
                    ShiftResult::Modified
                } else {
                    ShiftResult::Unchanged
                }
            }
            ShiftOperation::MoveCols { from_start, from_end, to } => {
                if source.sub_unit_id == self.sub_unit_id {
                    shift_range_for_move_cols(&mut source.range, *from_start, *from_end, *to);
                    ShiftResult::Modified
                } else {
                    ShiftResult::Unchanged
                }
            }
            ShiftOperation::MoveRange { .. } => ShiftResult::Unchanged,
            ShiftOperation::RemoveSheet { sub_unit_id } => {
                if &self.sub_unit_id == sub_unit_id || &source.sub_unit_id == sub_unit_id {
                    ShiftResult::Removed
                } else {
                    ShiftResult::Unchanged
                }
            }
        }
    }
}

// ============================================================================
// Shiftable for SetPivotTableTargetCellMutationParams
// ============================================================================

impl Shiftable for SetPivotTableTargetCellMutationParams {
    fn apply_shift(&mut self, op: &ShiftOperation) -> ShiftResult {
        let target = &mut self.target_cell_info;

        match op {
            ShiftOperation::InsertRows { start, count } => {
                if target.sub_unit_id == self.sub_unit_id {
                    ShiftResult::Modified
                } else {
                    ShiftResult::Unchanged
                }
            }
            ShiftOperation::RemoveRows { start, end } => {
                if target.sub_unit_id == self.sub_unit_id {
                    if !target.row >= *start && target.row <= *end {
                        return ShiftResult::Removed;
                    }
                    ShiftResult::Modified
                } else {
                    ShiftResult::Unchanged
                }
            }
            ShiftOperation::InsertCols { start, count } => {
                if target.sub_unit_id == self.sub_unit_id {
                    target.col += *count;
                    ShiftResult::Modified
                } else {
                    ShiftResult::Unchanged
                }
            }
            ShiftOperation::RemoveCols { start, end } => {
                if target.sub_unit_id == self.sub_unit_id {
                    if !target.col >= *start && target.col <= *end {
                        return ShiftResult::Removed;
                    }
                    ShiftResult::Modified
                } else {
                    ShiftResult::Unchanged
                }
            }
            ShiftOperation::MoveRows { from_start, from_end, to } => {
                if target.sub_unit_id == self.sub_unit_id {
                    shift_row_col_for_move_rows(&mut target.row, *from_start, *from_end, *to);
                    ShiftResult::Modified
                } else {
                    ShiftResult::Unchanged
                }
            }
            ShiftOperation::MoveCols { from_start, from_end, to } => {
                if target.sub_unit_id == self.sub_unit_id {
                    shift_row_col_for_move_cols(&mut target.col, *from_start, *from_end, *to);
                    ShiftResult::Modified
                } else {
                    ShiftResult::Unchanged
                }
            }
            ShiftOperation::MoveRange { .. } => ShiftResult::Unchanged,
            ShiftOperation::RemoveSheet { sub_unit_id } => {
                if &self.sub_unit_id == sub_unit_id || &target.sub_unit_id == sub_unit_id {
                    ShiftResult::Removed
                } else {
                    ShiftResult::Unchanged
                }
            }
        }
    }
}

// ============================================================================
// Shiftable for mutations without position fields (just RemoveSheet)
// ============================================================================

impl Shiftable for SetPivotTableFieldsConfigMutationParams {
    fn apply_shift(&mut self, op: &ShiftOperation) -> ShiftResult {
        match op {
            ShiftOperation::RemoveSheet { sub_unit_id } => {
                if &self.sub_unit_id == sub_unit_id {
                    ShiftResult::Removed
                } else {
                    ShiftResult::Unchanged
                }
            }
            _ => ShiftResult::Unchanged,
        }
    }
}

impl Shiftable for SetPivotTableCalculatedDataMutationParams {
    fn apply_shift(&mut self, op: &ShiftOperation) -> ShiftResult {
        match op {
            ShiftOperation::RemoveSheet { sub_unit_id } => {
                if &self.sub_unit_id == sub_unit_id {
                    ShiftResult::Removed
                } else {
                    ShiftResult::Unchanged
                }
            }
            _ => ShiftResult::Unchanged,
        }
    }
}

impl Shiftable for RemovePivotTableMutationParams {
    fn apply_shift(&mut self, op: &ShiftOperation) -> ShiftResult {
        match op {
            ShiftOperation::RemoveSheet { sub_unit_id } => {
                if &self.sub_unit_id == sub_unit_id {
                    ShiftResult::Removed
                } else {
                    ShiftResult::Unchanged
                }
            }
            _ => ShiftResult::Unchanged,
        }
    }
}

// ============================================================================
// Helper functions for move operations
// ============================================================================

/// Shift a single row index for row move operation
fn shift_row_col_for_move_rows(row: &mut i32, from_start: i32, from_end: i32, to: i32) {
    let count = from_end - from_start + 1;

    if *row >= from_start && *row <= from_end {
        // Row is in the moved range - it moves with the range
        let offset = *row - from_start;
        if to > from_end {
            // Moving down: new position is to - count + offset
            *row = to - count + offset;
        } else {
            // Moving up: new position is to + offset
            *row = to + offset;
        }
    } else if to > from_end {
        // Moving down
        if *row > from_end && *row < to {
            // Rows between source and target shift up
            *row -= count;
        }
    } else {
        // Moving up
        if *row >= to && *row < from_start {
            // Rows between target and source shift down
            *row += count;
        }
    }
}

/// Shift a single col index for col move operation
fn shift_row_col_for_move_cols(col: &mut i32, from_start: i32, from_end: i32, to: i32) {
    let count = from_end - from_start + 1;

    if *col >= from_start && *col <= from_end {
        // Col is in the moved range - it moves with the range
        let offset = *col - from_start;
        if to > from_end {
            *col = to - count + offset;
        } else {
            *col = to + offset;
        }
    } else if to > from_end {
        if *col > from_end && *col < to {
            *col -= count;
        }
    } else {
        if *col >= to && *col < from_start {
            *col += count;
        }
    }
}

/// Shift a range for row move operation
fn shift_range_for_move_rows(range: &mut crate::mutations::sheets::types::IRange, from_start: i32, from_end: i32, to: i32) {
    shift_row_col_for_move_rows(&mut range.start_row, from_start, from_end, to);
    shift_row_col_for_move_rows(&mut range.end_row, from_start, from_end, to);
}

/// Shift a range for col move operation
fn shift_range_for_move_cols(range: &mut crate::mutations::sheets::types::IRange, from_start: i32, from_end: i32, to: i32) {
    shift_row_col_for_move_cols(&mut range.start_column, from_start, from_end, to);
    shift_row_col_for_move_cols(&mut range.end_column, from_start, from_end, to);
}

// ============================================================================
// Feature ID conflict helper
// ============================================================================

/// Create a transform that removes m2 if RemovePivotTable removes the same pivot_table_id
fn remove_pivot_table_vs_set<T>() -> TransformFnRef
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + 'static,
{
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        // Parse RemovePivotTable params
        let m1_params: RemovePivotTableMutationParams = match serde_json::from_value(m1.params.clone()) {
            Ok(p) => p,
            Err(_) => return TransformResultRef::identity(m1, m2),
        };

        // Get pivot_table_id from m2
        let m2_pivot_id = m2.params.get("pivotTableId").and_then(|v| v.as_str());

        // If same pivot_table_id, m2 should be removed
        if m2_pivot_id == Some(&m1_params.pivot_table_id) {
            return TransformResultRef {
                m1_prime: MutationOutcome::Unchanged(m1),
                m2_prime: MutationOutcome::Removed,
                error: None,
            };
        }

        TransformResultRef::identity(m1, m2)
    })
}


// ============================================================================
// Register all transforms
// ============================================================================

pub fn register_transforms(registry: &mut TransformRegistry) {
    // ========================================================================
    // Self-transforms
    // ========================================================================
    // Add and Remove use identity - concurrent adds/removes don't conflict
    registry.register_symmetric_ref(ADD_PIVOT_TABLE_ID, identity_transform());
    registry.register_symmetric_ref(REMOVE_PIVOT_TABLE_ID, identity_transform());
    registry.register_symmetric_ref(SET_SOURCE_RANGE_ID, lww_transform());
    registry.register_symmetric_ref(SET_TARGET_CELL_ID, lww_transform());
    registry.register_symmetric_ref(SET_FIELDS_CONFIG_ID, lww_transform());
    registry.register_symmetric_ref(SET_CALCULATED_DATA_ID, lww_transform());

    // ========================================================================
    // Feature ID conflicts: RemovePivotTable removes Set mutations
    // ========================================================================
    registry.register_bidirectional_ref(REMOVE_PIVOT_TABLE_ID, ADD_PIVOT_TABLE_ID, remove_pivot_table_vs_set::<AddPivotTableMutationParams>());
    registry.register_bidirectional_ref(REMOVE_PIVOT_TABLE_ID, SET_SOURCE_RANGE_ID, remove_pivot_table_vs_set::<SetPivotTableSourceRangeMutationParams>());
    registry.register_bidirectional_ref(REMOVE_PIVOT_TABLE_ID, SET_TARGET_CELL_ID, remove_pivot_table_vs_set::<SetPivotTableTargetCellMutationParams>());
    registry.register_bidirectional_ref(REMOVE_PIVOT_TABLE_ID, SET_FIELDS_CONFIG_ID, remove_pivot_table_vs_set::<SetPivotTableFieldsConfigMutationParams>());
    registry.register_bidirectional_ref(REMOVE_PIVOT_TABLE_ID, SET_CALCULATED_DATA_ID, remove_pivot_table_vs_set::<SetPivotTableCalculatedDataMutationParams>());

    // ========================================================================
    // Cross-mutation relationships (identity - they don't conflict positionally)
    // ========================================================================
    registry.register_bidirectional_ref(ADD_PIVOT_TABLE_ID, SET_SOURCE_RANGE_ID, identity_transform());
    registry.register_bidirectional_ref(ADD_PIVOT_TABLE_ID, SET_TARGET_CELL_ID, identity_transform());
    registry.register_bidirectional_ref(ADD_PIVOT_TABLE_ID, SET_FIELDS_CONFIG_ID, identity_transform());
    registry.register_bidirectional_ref(ADD_PIVOT_TABLE_ID, SET_CALCULATED_DATA_ID, identity_transform());
    registry.register_bidirectional_ref(SET_SOURCE_RANGE_ID, SET_TARGET_CELL_ID, identity_transform());
    registry.register_bidirectional_ref(SET_SOURCE_RANGE_ID, SET_FIELDS_CONFIG_ID, identity_transform());
    registry.register_bidirectional_ref(SET_SOURCE_RANGE_ID, SET_CALCULATED_DATA_ID, identity_transform());
    registry.register_bidirectional_ref(SET_TARGET_CELL_ID, SET_FIELDS_CONFIG_ID, identity_transform());
    registry.register_bidirectional_ref(SET_TARGET_CELL_ID, SET_CALCULATED_DATA_ID, identity_transform());
    registry.register_bidirectional_ref(SET_FIELDS_CONFIG_ID, SET_CALCULATED_DATA_ID, identity_transform());

    // ========================================================================
    // Position shift transforms for AddPivotTable
    // ========================================================================
    registry.register_bidirectional_ref(InsertRowMutation::ID, ADD_PIVOT_TABLE_ID, shared::insert_row_shift::<AddPivotTableMutationParams>());
    registry.register_bidirectional_ref(InsertColMutation::ID, ADD_PIVOT_TABLE_ID, shared::insert_col_shift::<AddPivotTableMutationParams>());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, ADD_PIVOT_TABLE_ID, shared::remove_row_shift::<AddPivotTableMutationParams>());
    registry.register_bidirectional_ref(RemoveColMutation::ID, ADD_PIVOT_TABLE_ID, shared::remove_col_shift::<AddPivotTableMutationParams>());
    registry.register_bidirectional_ref(MoveRowsMutation::ID, ADD_PIVOT_TABLE_ID, shared::move_rows_shift::<AddPivotTableMutationParams>());
    registry.register_bidirectional_ref(MoveColsMutation::ID, ADD_PIVOT_TABLE_ID, shared::move_cols_shift::<AddPivotTableMutationParams>());
    registry.register_bidirectional_ref(MoveRangeMutation::ID, ADD_PIVOT_TABLE_ID, shared::move_range_shift::<AddPivotTableMutationParams>());
    registry.register_bidirectional_ref(RemoveSheetMutation::ID, ADD_PIVOT_TABLE_ID, shared::remove_sheet_shift::<AddPivotTableMutationParams>());

    // ========================================================================
    // Position shift transforms for SetPivotTableSourceRange
    // ========================================================================
    registry.register_bidirectional_ref(InsertRowMutation::ID, SET_SOURCE_RANGE_ID, shared::insert_row_shift::<SetPivotTableSourceRangeMutationParams>());
    registry.register_bidirectional_ref(InsertColMutation::ID, SET_SOURCE_RANGE_ID, shared::insert_col_shift::<SetPivotTableSourceRangeMutationParams>());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, SET_SOURCE_RANGE_ID, shared::remove_row_shift::<SetPivotTableSourceRangeMutationParams>());
    registry.register_bidirectional_ref(RemoveColMutation::ID, SET_SOURCE_RANGE_ID, shared::remove_col_shift::<SetPivotTableSourceRangeMutationParams>());
    registry.register_bidirectional_ref(MoveRowsMutation::ID, SET_SOURCE_RANGE_ID, shared::move_rows_shift::<SetPivotTableSourceRangeMutationParams>());
    registry.register_bidirectional_ref(MoveColsMutation::ID, SET_SOURCE_RANGE_ID, shared::move_cols_shift::<SetPivotTableSourceRangeMutationParams>());
    registry.register_bidirectional_ref(MoveRangeMutation::ID, SET_SOURCE_RANGE_ID, shared::move_range_shift::<SetPivotTableSourceRangeMutationParams>());
    registry.register_bidirectional_ref(RemoveSheetMutation::ID, SET_SOURCE_RANGE_ID, shared::remove_sheet_shift::<SetPivotTableSourceRangeMutationParams>());

    // ========================================================================
    // Position shift transforms for SetPivotTableTargetCell
    // ========================================================================
    registry.register_bidirectional_ref(InsertRowMutation::ID, SET_TARGET_CELL_ID, shared::insert_row_shift::<SetPivotTableTargetCellMutationParams>());
    registry.register_bidirectional_ref(InsertColMutation::ID, SET_TARGET_CELL_ID, shared::insert_col_shift::<SetPivotTableTargetCellMutationParams>());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, SET_TARGET_CELL_ID, shared::remove_row_shift::<SetPivotTableTargetCellMutationParams>());
    registry.register_bidirectional_ref(RemoveColMutation::ID, SET_TARGET_CELL_ID, shared::remove_col_shift::<SetPivotTableTargetCellMutationParams>());
    registry.register_bidirectional_ref(MoveRowsMutation::ID, SET_TARGET_CELL_ID, shared::move_rows_shift::<SetPivotTableTargetCellMutationParams>());
    registry.register_bidirectional_ref(MoveColsMutation::ID, SET_TARGET_CELL_ID, shared::move_cols_shift::<SetPivotTableTargetCellMutationParams>());
    registry.register_bidirectional_ref(MoveRangeMutation::ID, SET_TARGET_CELL_ID, shared::move_range_shift::<SetPivotTableTargetCellMutationParams>());
    registry.register_bidirectional_ref(RemoveSheetMutation::ID, SET_TARGET_CELL_ID, shared::remove_sheet_shift::<SetPivotTableTargetCellMutationParams>());

    // ========================================================================
    // RemoveSheet transforms for mutations without position fields
    // ========================================================================
    registry.register_bidirectional_ref(RemoveSheetMutation::ID, SET_FIELDS_CONFIG_ID, shared::remove_sheet_shift::<SetPivotTableFieldsConfigMutationParams>());
    registry.register_bidirectional_ref(RemoveSheetMutation::ID, SET_CALCULATED_DATA_ID, shared::remove_sheet_shift::<SetPivotTableCalculatedDataMutationParams>());
    registry.register_bidirectional_ref(RemoveSheetMutation::ID, REMOVE_PIVOT_TABLE_ID, shared::remove_sheet_shift::<RemovePivotTableMutationParams>());
}
