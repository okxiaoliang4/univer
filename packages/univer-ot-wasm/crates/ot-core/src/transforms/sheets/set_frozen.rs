use crate::mutations::sheets::{
    SetFrozenMutation, SetFrozenMutationParams,
    InsertRowMutation, InsertColMutation,
    RemoveRowMutation, RemoveColMutation,
    MoveColsMutation, MoveRangeMutation, MoveRowsMutation, RemoveSheetMutation,
};
use crate::registry::{MutationId, TransformRegistry};
use crate::utils::{
    WorksheetParams, Shiftable, ShiftResult,
    shift_row_for_move_rows, shift_col_for_move_cols,
};
use crate::utils::shift_operations::ShiftOperation;
use crate::utils::shared_transforms as shared;
use crate::utils::transform_helpers::lww_transform;

pub const MUTATION_ID: MutationId = SetFrozenMutation::ID;

// ============================================================================
// WorksheetParams implementation
// ============================================================================

impl WorksheetParams for SetFrozenMutationParams {
    fn unit_id(&self) -> &str { &self.sub_unit_params.unit_id }
    fn sub_unit_id(&self) -> &str { &self.sub_unit_params.sub_unit_id }
}

// ============================================================================
// Shiftable implementation - shifts start_row and start_column
// ============================================================================

impl Shiftable for SetFrozenMutationParams {
    fn apply_shift(&mut self, op: &ShiftOperation) -> ShiftResult {
        match op {
            ShiftOperation::InsertRows { start, count } => {
                if self.start_row >= *start {
                    self.start_row += *count;
                    ShiftResult::Modified
                } else {
                    ShiftResult::Unchanged
                }
            }
            ShiftOperation::RemoveRows { start, end } => {
                if self.start_row >= *start && self.start_row <= *end {
                    // Frozen position is within removed range - reset to start of removal
                    self.start_row = *start;
                    ShiftResult::Modified
                } else if self.start_row > *end {
                    self.start_row -= end - start + 1;
                    ShiftResult::Modified
                } else {
                    ShiftResult::Unchanged
                }
            }
            ShiftOperation::InsertCols { start, count } => {
                if self.start_column >= *start {
                    self.start_column += *count;
                    ShiftResult::Modified
                } else {
                    ShiftResult::Unchanged
                }
            }
            ShiftOperation::RemoveCols { start, end } => {
                if self.start_column >= *start && self.start_column <= *end {
                    // Frozen position is within removed range - reset to start of removal
                    self.start_column = *start;
                    ShiftResult::Modified
                } else if self.start_column > *end {
                    self.start_column -= end - start + 1;
                    ShiftResult::Modified
                } else {
                    ShiftResult::Unchanged
                }
            }
            ShiftOperation::MoveRows { from_start, from_end, to } => {
                let old_row = self.start_row;
                shift_row_for_move_rows(&mut self.start_row, *from_start, *from_end, *to);
                if self.start_row != old_row {
                    ShiftResult::Modified
                } else {
                    ShiftResult::Unchanged
                }
            }
            ShiftOperation::MoveCols { from_start, from_end, to } => {
                let old_col = self.start_column;
                shift_col_for_move_cols(&mut self.start_column, *from_start, *from_end, *to);
                if self.start_column != old_col {
                    ShiftResult::Modified
                } else {
                    ShiftResult::Unchanged
                }
            }
            ShiftOperation::MoveRange { from, to } => {
                // Only shift if the frozen position is within the moved range
                if self.start_row >= from.start_row && self.start_row <= from.end_row
                    && self.start_column >= from.start_column && self.start_column <= from.end_column
                {
                    let row_offset = to.start_row - from.start_row;
                    let col_offset = to.start_column - from.start_column;
                    self.start_row += row_offset;
                    self.start_column += col_offset;
                    ShiftResult::Modified
                } else {
                    ShiftResult::Unchanged
                }
            }
            ShiftOperation::RemoveSheet { sub_unit_id } => {
                if &self.sub_unit_params.sub_unit_id == sub_unit_id {
                    ShiftResult::Removed
                } else {
                    ShiftResult::Unchanged
                }
            }
        }
    }
}

// ============================================================================
// Transform registration
// ============================================================================

/// Register transforms for SetFrozenMutation
///
/// Mutation ID: sheet.mutation.set-frozen
///
/// SetFrozenMutation sets the freeze panes configuration for a worksheet.
/// Transform strategy: Last-Write-Wins (LWW) for self-transforms.
/// Shift transforms adjust start_row/start_column when rows/columns change.
pub fn register_transforms(registry: &mut TransformRegistry) {
    // Self-transform: LWW
    registry.register_symmetric_ref(MUTATION_ID, lww_transform());

    // Shift transforms for frozen position
    registry.register_bidirectional_ref(InsertRowMutation::ID, MUTATION_ID, shared::insert_row_shift::<SetFrozenMutationParams>());
    registry.register_bidirectional_ref(InsertColMutation::ID, MUTATION_ID, shared::insert_col_shift::<SetFrozenMutationParams>());
    registry.register_bidirectional_ref(RemoveRowMutation::ID, MUTATION_ID, shared::remove_row_shift::<SetFrozenMutationParams>());
    registry.register_bidirectional_ref(RemoveColMutation::ID, MUTATION_ID, shared::remove_col_shift::<SetFrozenMutationParams>());
    registry.register_bidirectional_ref(MoveRowsMutation::ID, MUTATION_ID, shared::move_rows_shift::<SetFrozenMutationParams>());
    registry.register_bidirectional_ref(MoveColsMutation::ID, MUTATION_ID, shared::move_cols_shift::<SetFrozenMutationParams>());
    registry.register_bidirectional_ref(MoveRangeMutation::ID, MUTATION_ID, shared::move_range_shift::<SetFrozenMutationParams>());
    registry.register_bidirectional_ref(RemoveSheetMutation::ID, MUTATION_ID, shared::remove_sheet_shift::<SetFrozenMutationParams>());
}
