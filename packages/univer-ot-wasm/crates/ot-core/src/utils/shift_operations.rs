//! Unified shift operations for OT transformation
//!
//! This module provides a unified `ShiftOperation` enum and `Shiftable` trait
//! that replace multiple separate shift traits with a single, extensible abstraction.
//!
//! # Design
//!
//! Instead of:
//! ```ignore
//! trait ShiftableByRowInsert { fn shift_for_row_insert(&mut self, ...); }
//! trait ShiftableByRowRemove { fn shift_for_row_remove(&mut self, ...); }
//! trait ShiftableByColInsert { fn shift_for_col_insert(&mut self, ...); }
//! // etc.
//! ```
//!
//! We use:
//! ```ignore
//! enum ShiftOperation { InsertRows { start, count }, RemoveRows { start, end }, ... }
//! trait Shiftable { fn apply_shift(&mut self, op: &ShiftOperation) -> ShiftResult; }
//! ```

use crate::types::IRange;
use crate::utils::generic_params::*;
use crate::utils::shiftable::*;

// ============================================================================
// ShiftOperation - Unified shift operation enum
// ============================================================================

/// Unified shift operation type
///
/// This enum encapsulates all types of position-affecting operations
/// that can occur during OT transformation.
#[derive(Debug, Clone)]
pub enum ShiftOperation {
    // Row operations
    InsertRows {
        start: i32,
        count: i32,
    },
    RemoveRows {
        start: i32,
        end: i32,
    },
    MoveRows {
        from_start: i32,
        from_end: i32,
        to: i32,
    },

    // Column operations
    InsertCols {
        start: i32,
        count: i32,
    },
    RemoveCols {
        start: i32,
        end: i32,
    },
    MoveCols {
        from_start: i32,
        from_end: i32,
        to: i32,
    },

    // Range operations
    MoveRange {
        from: IRange,
        to: IRange,
    },

    // Sheet operations
    RemoveSheet {
        sub_unit_id: String,
    },
}

// ============================================================================
// ShiftResult - Result of applying a shift operation
// ============================================================================

/// Result of applying a shift operation to data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShiftResult {
    /// The data was modified by the shift operation
    Modified,
    /// The data should be removed (completely within deleted area)
    Removed,
    /// The data was not affected by the shift operation
    Unchanged,
}

// ============================================================================
// Shiftable Trait
// ============================================================================

/// Trait for types that can be shifted by OT operations
///
/// Implementors should handle all relevant ShiftOperation variants
/// and return the appropriate ShiftResult.
pub trait Shiftable: WorksheetParams {
    /// Apply a shift operation to this data
    ///
    /// Returns the result indicating whether the data was modified, removed, or unchanged.
    fn apply_shift(&mut self, op: &ShiftOperation) -> ShiftResult;
}

// ============================================================================
// Shiftable Implementations
// ============================================================================

impl Shiftable for GenericRangesParams {
    fn apply_shift(&mut self, op: &ShiftOperation) -> ShiftResult {
        match op {
            ShiftOperation::RemoveSheet { sub_unit_id } => {
                if &self.sub_unit_params.sub_unit_id == sub_unit_id {
                    ShiftResult::Removed
                } else {
                    ShiftResult::Unchanged
                }
            }
            // All position-based operations (Insert/Remove/Move for rows/cols/range)
            _ => {
                if self.ranges.is_empty() {
                    return ShiftResult::Unchanged;
                }
                if shift_ranges_vec(&mut self.ranges, op) {
                    ShiftResult::Modified
                } else {
                    ShiftResult::Removed
                }
            }
        }
    }
}

impl Shiftable for GenericCellValueParams {
    fn apply_shift(&mut self, op: &ShiftOperation) -> ShiftResult {
        let cell_value = match &mut self.cell_value {
            Some(cv) => cv,
            None => return ShiftResult::Unchanged,
        };

        match op {
            ShiftOperation::InsertRows { start, count } => {
                cell_value.shift_for_row_insert(*start, *count);
                ShiftResult::Modified
            }

            ShiftOperation::RemoveRows { start, end } => {
                let retained = cell_value.shift_for_row_remove(*start, *end);
                if !retained {
                    ShiftResult::Removed
                } else {
                    ShiftResult::Modified
                }
            }

            ShiftOperation::InsertCols { start, count } => {
                cell_value.shift_for_col_insert(*start, *count);
                ShiftResult::Modified
            }

            ShiftOperation::RemoveCols { start, end } => {
                let retained = cell_value.shift_for_col_remove(*start, *end);
                if !retained {
                    ShiftResult::Removed
                } else {
                    ShiftResult::Modified
                }
            }

            ShiftOperation::MoveRows { from_start, from_end, to } => {
                shift_cell_value_for_row_move(cell_value, *from_start, *from_end, *to);
                ShiftResult::Modified
            }

            ShiftOperation::MoveCols { from_start, from_end, to } => {
                shift_cell_value_for_col_move(cell_value, *from_start, *from_end, *to);
                ShiftResult::Modified
            }

            ShiftOperation::MoveRange { from, to } => {
                shift_cell_value_for_range_move(cell_value, from, to);
                ShiftResult::Modified
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

impl Shiftable for GenericRowDataParams {
    fn apply_shift(&mut self, op: &ShiftOperation) -> ShiftResult {
        if self.row_data.is_empty() {
            return ShiftResult::Unchanged;
        }

        match op {
            ShiftOperation::InsertRows { start, count } => {
                shift_hashmap_keys_for_row_insert(&mut self.row_data, *start, *count);
                ShiftResult::Modified
            }

            ShiftOperation::RemoveRows { start, end } => {
                let retained = shift_hashmap_keys_for_row_remove(&mut self.row_data, *start, *end);
                if !retained {
                    ShiftResult::Removed
                } else {
                    ShiftResult::Modified
                }
            }

            ShiftOperation::MoveRows { from_start, from_end, to } => {
                shift_hashmap_keys_for_row_move(&mut self.row_data, *from_start, *from_end, *to);
                ShiftResult::Modified
            }

            ShiftOperation::RemoveSheet { sub_unit_id } => {
                if &self.sub_unit_params.sub_unit_id == sub_unit_id {
                    ShiftResult::Removed
                } else {
                    ShiftResult::Unchanged
                }
            }

            // Row data is not affected by column operations or MoveRange
            _ => ShiftResult::Unchanged,
        }
    }
}

impl Shiftable for GenericColDataParams {
    fn apply_shift(&mut self, op: &ShiftOperation) -> ShiftResult {
        if self.column_data.is_empty() {
            return ShiftResult::Unchanged;
        }

        match op {
            ShiftOperation::InsertCols { start, count } => {
                shift_hashmap_keys_for_col_insert(&mut self.column_data, *start, *count);
                ShiftResult::Modified
            }

            ShiftOperation::RemoveCols { start, end } => {
                let retained = shift_hashmap_keys_for_col_remove(&mut self.column_data, *start, *end);
                if !retained {
                    ShiftResult::Removed
                } else {
                    ShiftResult::Modified
                }
            }

            ShiftOperation::MoveCols { from_start, from_end, to } => {
                shift_hashmap_keys_for_col_move(&mut self.column_data, *from_start, *from_end, *to);
                ShiftResult::Modified
            }

            ShiftOperation::RemoveSheet { sub_unit_id } => {
                if &self.sub_unit_params.sub_unit_id == sub_unit_id {
                    ShiftResult::Removed
                } else {
                    ShiftResult::Unchanged
                }
            }

            // Column data is not affected by row operations or MoveRange
            _ => ShiftResult::Unchanged,
        }
    }
}

impl Shiftable for GenericRangeParams {
    fn apply_shift(&mut self, op: &ShiftOperation) -> ShiftResult {
        match op {
            ShiftOperation::InsertRows { start, count } => {
                self.range.shift_for_row_insert(*start, *count);
                ShiftResult::Modified
            }

            ShiftOperation::RemoveRows { start, end } => {
                if self.range.shift_for_row_remove(*start, *end) {
                    ShiftResult::Modified
                } else {
                    ShiftResult::Removed
                }
            }

            ShiftOperation::InsertCols { start, count } => {
                self.range.shift_for_col_insert(*start, *count);
                ShiftResult::Modified
            }

            ShiftOperation::RemoveCols { start, end } => {
                if self.range.shift_for_col_remove(*start, *end) {
                    ShiftResult::Modified
                } else {
                    ShiftResult::Removed
                }
            }

            ShiftOperation::MoveRows { from_start, from_end, to } => {
                shift_range_for_move_rows(&mut self.range, *from_start, *from_end, *to);
                ShiftResult::Modified
            }

            ShiftOperation::MoveCols { from_start, from_end, to } => {
                shift_range_for_move_cols(&mut self.range, *from_start, *from_end, *to);
                ShiftResult::Modified
            }

            ShiftOperation::MoveRange { from, to } => {
                shift_range_for_move_range(&mut self.range, from, to);
                ShiftResult::Modified
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

impl Shiftable for GenericRowColParams {
    fn apply_shift(&mut self, op: &ShiftOperation) -> ShiftResult {
      shift_row_col(&mut self.row, &mut self.col, op)
    }
}

impl Shiftable for GenericColParams {
    fn apply_shift(&mut self, op: &ShiftOperation) -> ShiftResult {
        match op {
            ShiftOperation::InsertCols { start, count } => {
                if self.col >= *start {
                    self.col += *count;
                    ShiftResult::Modified
                } else {
                    ShiftResult::Unchanged
                }
            }
            ShiftOperation::RemoveCols { start, end } => {
                if self.col >= *start && self.col <= *end {
                    ShiftResult::Removed
                } else if self.col > *end {
                    self.col -= end - start + 1;
                    ShiftResult::Modified
                } else {
                    ShiftResult::Unchanged
                }
            }
            ShiftOperation::MoveCols { from_start, from_end, to } => {
                let old_col = self.col;
                shift_col_for_move_cols(&mut self.col, *from_start, *from_end, *to);
                if self.col != old_col {
                    ShiftResult::Modified
                } else {
                    ShiftResult::Unchanged
                }
            }
            ShiftOperation::RemoveSheet { sub_unit_id } => {
                if &self.sub_unit_id == sub_unit_id {
                    ShiftResult::Removed
                } else {
                    ShiftResult::Unchanged
                }
            }
            // Row operations don't affect col-only params
            _ => ShiftResult::Unchanged,
        }
    }
}

impl Shiftable for GenericRowParams {
    fn apply_shift(&mut self, op: &ShiftOperation) -> ShiftResult {
        match op {
            ShiftOperation::InsertRows { start, count } => {
                if self.row >= *start {
                    self.row += *count;
                    ShiftResult::Modified
                } else {
                    ShiftResult::Unchanged
                }
            }
            ShiftOperation::RemoveRows { start, end } => {
                if self.row >= *start && self.row <= *end {
                    ShiftResult::Removed
                } else if self.row > *end {
                    self.row -= end - start + 1;
                    ShiftResult::Modified
                } else {
                    ShiftResult::Unchanged
                }
            }
            ShiftOperation::MoveRows { from_start, from_end, to } => {
                let old_row = self.row;
                shift_row_for_move_rows(&mut self.row, *from_start, *from_end, *to);
                if self.row != old_row {
                    ShiftResult::Modified
                } else {
                    ShiftResult::Unchanged
                }
            }
            ShiftOperation::RemoveSheet { sub_unit_id } => {
                if &self.sub_unit_id == sub_unit_id {
                    ShiftResult::Removed
                } else {
                    ShiftResult::Unchanged
                }
            }
            // Column operations don't affect row-only params
            _ => ShiftResult::Unchanged,
        }
    }
}

/// Helper: Apply shift operation to a Vec<IRange>, returns true if any ranges remain
pub fn shift_ranges_vec(ranges: &mut Vec<IRange>, op: &ShiftOperation) -> bool {
    match op {
        ShiftOperation::InsertRows { start, count } => {
            for range in ranges.iter_mut() {
                range.shift_for_row_insert(*start, *count);
            }
            true
        }
        ShiftOperation::RemoveRows { start, end } => {
            ranges.retain_mut(|range| range.shift_for_row_remove(*start, *end));
            !ranges.is_empty()
        }
        ShiftOperation::InsertCols { start, count } => {
            for range in ranges.iter_mut() {
                range.shift_for_col_insert(*start, *count);
            }
            true
        }
        ShiftOperation::RemoveCols { start, end } => {
            ranges.retain_mut(|range| range.shift_for_col_remove(*start, *end));
            !ranges.is_empty()
        }
        ShiftOperation::MoveRows { from_start, from_end, to } => {
            for range in ranges.iter_mut() {
                shift_range_for_move_rows(range, *from_start, *from_end, *to);
            }
            true
        }
        ShiftOperation::MoveCols { from_start, from_end, to } => {
            for range in ranges.iter_mut() {
                shift_range_for_move_cols(range, *from_start, *from_end, *to);
            }
            true
        }
        ShiftOperation::MoveRange { from, to } => {
            for range in ranges.iter_mut() {
                shift_range_for_move_range(range, from, to);
            }
            true
        }
        ShiftOperation::RemoveSheet { .. } => true, // Handled at mutation level
    }
}

/// Shift a single row index for row move operation
pub fn shift_row_for_move_rows(row: &mut i32, from_start: i32, from_end: i32, to: i32) {
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
pub fn shift_col_for_move_cols(col: &mut i32, from_start: i32, from_end: i32, to: i32) {
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
pub fn shift_range_for_move_rows(range: &mut IRange, from_start: i32, from_end: i32, to: i32) {
    shift_row_for_move_rows(&mut range.start_row, from_start, from_end, to);
    shift_row_for_move_rows(&mut range.end_row, from_start, from_end, to);
}

/// Shift a range for col move operation
pub fn shift_range_for_move_cols(range: &mut IRange, from_start: i32, from_end: i32, to: i32) {
    shift_col_for_move_cols(&mut range.start_column, from_start, from_end, to);
    shift_col_for_move_cols(&mut range.end_column, from_start, from_end, to);
}

/// Shift a range for move range operation
/// If the range overlaps with the moved range, it needs to be adjusted
pub fn shift_range_for_move_range(range: &mut IRange, from: &IRange, to: &IRange) {
    // Check if range is completely within the moved range
    if range.start_row >= from.start_row && range.end_row <= from.end_row
        && range.start_column >= from.start_column && range.end_column <= from.end_column
    {
        // Range moves with the source range
        let row_offset = to.start_row - from.start_row;
        let col_offset = to.start_column - from.start_column;
        range.start_row += row_offset;
        range.end_row += row_offset;
        range.start_column += col_offset;
        range.end_column += col_offset;
    }
    // For partial overlaps, we keep the range unchanged (simplified behavior)
    // More complex scenarios would require splitting or clipping ranges
}

/// Helper: Apply shift operation to a single row/col position
/// Returns ShiftResult indicating whether the position was modified, removed, or unchanged
pub fn shift_row_col(row: &mut i32, col: &mut i32, op: &ShiftOperation) -> ShiftResult {
    match op {
        ShiftOperation::InsertRows { start, count } => {
            if *row >= *start {
                *row += *count;
                ShiftResult::Modified
            } else {
                ShiftResult::Unchanged
            }
        }
        ShiftOperation::RemoveRows { start, end } => {
            if *row >= *start && *row <= *end {
                ShiftResult::Removed
            } else if *row > *end {
                *row -= end - start + 1;
                ShiftResult::Modified
            } else {
                ShiftResult::Unchanged
            }
        }
        ShiftOperation::InsertCols { start, count } => {
            if *col >= *start {
                *col += *count;
                ShiftResult::Modified
            } else {
                ShiftResult::Unchanged
            }
        }
        ShiftOperation::RemoveCols { start, end } => {
            if *col >= *start && *col <= *end {
                ShiftResult::Removed
            } else if *col > *end {
                *col -= end - start + 1;
                ShiftResult::Modified
            } else {
                ShiftResult::Unchanged
            }
        }
        ShiftOperation::MoveRows { from_start, from_end, to } => {
            let old_row = *row;
            shift_row_for_move_rows(row, *from_start, *from_end, *to);
            if *row != old_row {
                ShiftResult::Modified
            } else {
                ShiftResult::Unchanged
            }
        }
        ShiftOperation::MoveCols { from_start, from_end, to } => {
            let old_col = *col;
            shift_col_for_move_cols(col, *from_start, *from_end, *to);
            if *col != old_col {
                ShiftResult::Modified
            } else {
                ShiftResult::Unchanged
            }
        }
        ShiftOperation::MoveRange { from, to } => {
            // Check if position is within the moved range
            if *row >= from.start_row && *row <= from.end_row
                && *col >= from.start_column && *col <= from.end_column
            {
                let row_offset = to.start_row - from.start_row;
                let col_offset = to.start_column - from.start_column;
                *row += row_offset;
                *col += col_offset;
                ShiftResult::Modified
            } else {
                ShiftResult::Unchanged
            }
        }
        ShiftOperation::RemoveSheet { .. } => ShiftResult::Unchanged, // Handled at mutation level
    }
}

// ============================================================================
// Shiftable Implementations for HyperLink Mutations
// ============================================================================

use crate::mutations::sheets_hyper_link::{
    AddHyperLinkMutationParams, UpdateHyperLinkRefMutationParams, UpdateRichHyperLinkMutationParams,
};

impl WorksheetParams for AddHyperLinkMutationParams {
    fn unit_id(&self) -> &str { &self.unit_id }
    fn sub_unit_id(&self) -> &str { &self.sub_unit_id }
}

impl Shiftable for AddHyperLinkMutationParams {
    fn apply_shift(&mut self, op: &ShiftOperation) -> ShiftResult {
        match op {
            ShiftOperation::RemoveSheet { sub_unit_id } => {
                if &self.sub_unit_id == sub_unit_id {
                    return ShiftResult::Removed;
                }
                ShiftResult::Unchanged
            }
            _ => shift_row_col(&mut self.link.row, &mut self.link.column, op),
        }
    }
}

impl WorksheetParams for UpdateHyperLinkRefMutationParams {
    fn unit_id(&self) -> &str { &self.unit_id }
    fn sub_unit_id(&self) -> &str { &self.sub_unit_id }
}

impl Shiftable for UpdateHyperLinkRefMutationParams {
    fn apply_shift(&mut self, op: &ShiftOperation) -> ShiftResult {
        match op {
            ShiftOperation::RemoveSheet { sub_unit_id } => {
                if &self.sub_unit_id == sub_unit_id {
                    return ShiftResult::Removed;
                }
                ShiftResult::Unchanged
            }
            _ => shift_row_col(&mut self.row, &mut self.column, op),
        }
    }
}

impl WorksheetParams for UpdateRichHyperLinkMutationParams {
    fn unit_id(&self) -> &str { &self.unit_id }
    fn sub_unit_id(&self) -> &str { &self.sub_unit_id }
}

impl Shiftable for UpdateRichHyperLinkMutationParams {
    fn apply_shift(&mut self, op: &ShiftOperation) -> ShiftResult {
        match op {
            ShiftOperation::RemoveSheet { sub_unit_id } => {
                if &self.sub_unit_id == sub_unit_id {
                    return ShiftResult::Removed;
                }
                ShiftResult::Unchanged
            }
            _ => shift_row_col(&mut self.row, &mut self.col, op),
        }
    }
}

// ============================================================================
// Shiftable Implementations for Conditional Formatting Mutations
// ============================================================================

use crate::mutations::sheets_conditional_formatting::{
    AddConditionalRuleMutationParams, SetConditionalRuleMutationParams,
};

impl WorksheetParams for AddConditionalRuleMutationParams {
    fn unit_id(&self) -> &str { &self.unit_id }
    fn sub_unit_id(&self) -> &str { &self.sub_unit_id }
}

impl Shiftable for AddConditionalRuleMutationParams {
    fn apply_shift(&mut self, op: &ShiftOperation) -> ShiftResult {
        match op {
            ShiftOperation::RemoveSheet { sub_unit_id } => {
                if &self.sub_unit_id == sub_unit_id {
                    return ShiftResult::Removed;
                }
                ShiftResult::Unchanged
            }
            _ => {
                if shift_ranges_vec(&mut self.rule.ranges, op) {
                    ShiftResult::Modified
                } else {
                    ShiftResult::Removed
                }
            }
        }
    }
}

impl WorksheetParams for SetConditionalRuleMutationParams {
    fn unit_id(&self) -> &str { &self.unit_id }
    fn sub_unit_id(&self) -> &str { &self.sub_unit_id }
}

impl Shiftable for SetConditionalRuleMutationParams {
    fn apply_shift(&mut self, op: &ShiftOperation) -> ShiftResult {
        match op {
            ShiftOperation::RemoveSheet { sub_unit_id } => {
                if &self.sub_unit_id == sub_unit_id {
                    return ShiftResult::Removed;
                }
                ShiftResult::Unchanged
            }
            _ => {
                if shift_ranges_vec(&mut self.rule.ranges, op) {
                    ShiftResult::Modified
                } else {
                    ShiftResult::Removed
                }
            }
        }
    }
}

// ============================================================================
// Shiftable Implementations for Data Validation Mutations
// ============================================================================

use crate::mutations::data_validation::{
    AddDataValidationMutationParams, RuleOrRules, IDataValidationRule,
    UpdateDataValidationMutationParams, IUpdateRulePayload,
    IRange as DataValidationIRange,
};

/// Helper to shift a data validation IRange (has same structure as types::IRange)
fn shift_data_validation_range(range: &mut DataValidationIRange, op: &ShiftOperation) -> bool {
    match op {
        ShiftOperation::InsertRows { start, count } => {
            if range.end_row >= *start {
                if range.start_row >= *start {
                    range.start_row += *count;
                }
                range.end_row += *count;
            }
            true
        }
        ShiftOperation::RemoveRows { start, end } => {
            let remove_count = end - start + 1;
            if range.start_row >= *start && range.end_row <= *end {
                return false;
            }
            if range.start_row > *end {
                range.start_row -= remove_count;
                range.end_row -= remove_count;
            } else if range.end_row >= *start {
                if range.start_row < *start {
                    range.end_row = (*start - 1).max(range.start_row);
                } else {
                    range.start_row = *start;
                    range.end_row = *start;
                }
            }
            true
        }
        ShiftOperation::InsertCols { start, count } => {
            if range.end_column >= *start {
                if range.start_column >= *start {
                    range.start_column += *count;
                }
                range.end_column += *count;
            }
            true
        }
        ShiftOperation::RemoveCols { start, end } => {
            let remove_count = end - start + 1;
            if range.start_column >= *start && range.end_column <= *end {
                return false;
            }
            if range.start_column > *end {
                range.start_column -= remove_count;
                range.end_column -= remove_count;
            } else if range.end_column >= *start {
                if range.start_column < *start {
                    range.end_column = (*start - 1).max(range.start_column);
                } else {
                    range.start_column = *start;
                    range.end_column = *start;
                }
            }
            true
        }
        _ => true,
    }
}

/// Helper to shift ranges in a data validation rule
fn shift_data_validation_rule(rule: &mut IDataValidationRule, op: &ShiftOperation) -> bool {
    rule.ranges.retain_mut(|range| shift_data_validation_range(range, op));
    !rule.ranges.is_empty()
}

impl WorksheetParams for AddDataValidationMutationParams {
    fn unit_id(&self) -> &str { &self.unit_id }
    fn sub_unit_id(&self) -> &str { &self.sub_unit_id }
}

impl Shiftable for AddDataValidationMutationParams {
    fn apply_shift(&mut self, op: &ShiftOperation) -> ShiftResult {
        match op {
            ShiftOperation::RemoveSheet { sub_unit_id } => {
                if &self.sub_unit_id == sub_unit_id {
                    return ShiftResult::Removed;
                }
                ShiftResult::Unchanged
            }
            _ => {
                match &mut self.rule {
                    RuleOrRules::Single(rule) => {
                        if shift_data_validation_rule(rule, op) {
                            ShiftResult::Modified
                        } else {
                            ShiftResult::Removed
                        }
                    }
                    RuleOrRules::Multiple(rules) => {
                        // Remove rules whose ranges are all deleted
                        rules.retain_mut(|rule| shift_data_validation_rule(rule, op));
                        if rules.is_empty() {
                            ShiftResult::Removed
                        } else {
                            ShiftResult::Modified
                        }
                    }
                }
            }
        }
    }
}

impl WorksheetParams for UpdateDataValidationMutationParams {
    fn unit_id(&self) -> &str { &self.unit_id }
    fn sub_unit_id(&self) -> &str { &self.sub_unit_id }
}

impl Shiftable for UpdateDataValidationMutationParams {
    fn apply_shift(&mut self, op: &ShiftOperation) -> ShiftResult {
        match op {
            ShiftOperation::RemoveSheet { sub_unit_id } => {
                if &self.sub_unit_id == sub_unit_id {
                    return ShiftResult::Removed;
                }
                ShiftResult::Unchanged
            }
            _ => {
                // Only shift if the payload contains ranges
                match &mut self.payload {
                    IUpdateRulePayload::Range { payload } => {
                        payload.retain_mut(|range| shift_data_validation_range(range, op));
                        if payload.is_empty() {
                            ShiftResult::Removed
                        } else {
                            ShiftResult::Modified
                        }
                    }
                    IUpdateRulePayload::All { payload } => {
                        payload.ranges.retain_mut(|range| shift_data_validation_range(range, op));
                        if payload.ranges.is_empty() {
                            ShiftResult::Removed
                        } else {
                            ShiftResult::Modified
                        }
                    }
                    // Setting and Options payloads don't have position data
                    _ => ShiftResult::Unchanged,
                }
            }
        }
    }
}

// ============================================================================
// Shiftable Implementations for Sheet Table Mutations
// ============================================================================

use crate::mutations::sheets_table::{
    AddSheetTableParams, SetSheetTableMutationParams,
    types::ITableRange,
};

/// Shift an ITableRange (similar to IRange)
fn shift_table_range(range: &mut ITableRange, op: &ShiftOperation) -> bool {
    match op {
        ShiftOperation::InsertRows { start, count } => {
            if range.end_row >= *start {
                if range.start_row >= *start {
                    range.start_row += *count;
                }
                range.end_row += *count;
            }
            true
        }
        ShiftOperation::RemoveRows { start, end } => {
            let remove_count = end - start + 1;
            // Range completely within removed area
            if range.start_row >= *start && range.end_row <= *end {
                return false;
            }
            // Range after removed area
            if range.start_row > *end {
                range.start_row -= remove_count;
                range.end_row -= remove_count;
            }
            // Range overlaps with removed area
            else if range.end_row >= *start {
                if range.start_row < *start {
                    // Range starts before removal
                    range.end_row = (*start - 1).max(range.start_row);
                } else {
                    // Range starts within removal
                    range.start_row = *start;
                    range.end_row = *start;
                }
            }
            true
        }
        ShiftOperation::InsertCols { start, count } => {
            if range.end_column >= *start {
                if range.start_column >= *start {
                    range.start_column += *count;
                }
                range.end_column += *count;
            }
            true
        }
        ShiftOperation::RemoveCols { start, end } => {
            let remove_count = end - start + 1;
            // Range completely within removed area
            if range.start_column >= *start && range.end_column <= *end {
                return false;
            }
            // Range after removed area
            if range.start_column > *end {
                range.start_column -= remove_count;
                range.end_column -= remove_count;
            }
            // Range overlaps with removed area
            else if range.end_column >= *start {
                if range.start_column < *start {
                    range.end_column = (*start - 1).max(range.start_column);
                } else {
                    range.start_column = *start;
                    range.end_column = *start;
                }
            }
            true
        }
        _ => true,
    }
}

impl WorksheetParams for AddSheetTableParams {
    fn unit_id(&self) -> &str { &self.unit_id }
    fn sub_unit_id(&self) -> &str { &self.sub_unit_id }
}

impl Shiftable for AddSheetTableParams {
    fn apply_shift(&mut self, op: &ShiftOperation) -> ShiftResult {
        match op {
            ShiftOperation::RemoveSheet { sub_unit_id } => {
                if &self.sub_unit_id == sub_unit_id {
                    return ShiftResult::Removed;
                }
                ShiftResult::Unchanged
            }
            _ => {
                if shift_table_range(&mut self.range, op) {
                    ShiftResult::Modified
                } else {
                    ShiftResult::Removed
                }
            }
        }
    }
}

impl WorksheetParams for SetSheetTableMutationParams {
    fn unit_id(&self) -> &str { &self.unit_id }
    fn sub_unit_id(&self) -> &str { &self.sub_unit_id }
}

impl Shiftable for SetSheetTableMutationParams {
    fn apply_shift(&mut self, op: &ShiftOperation) -> ShiftResult {
        match op {
            ShiftOperation::RemoveSheet { sub_unit_id } => {
                if &self.sub_unit_id == sub_unit_id {
                    return ShiftResult::Removed;
                }
                ShiftResult::Unchanged
            }
            _ => {
                // Only shift if config has update_range
                if let Some(ref mut update_range) = self.config.update_range {
                    if shift_table_range(&mut update_range.new_range, op) {
                        ShiftResult::Modified
                    } else {
                        ShiftResult::Removed
                    }
                } else {
                    ShiftResult::Unchanged
                }
            }
        }
    }
}

// ============================================================================
// Shiftable Implementations for Thread Comment Mutations
// ============================================================================

use crate::mutations::thread_comment::{
    AddCommentMutationParams, UpdateCommentRefMutationParams,
};

/// Parse a cell reference string like "A1" to (row, col)
/// Returns None if parsing fails
fn parse_cell_ref(ref_str: &str) -> Option<(i32, i32)> {
    let ref_str = ref_str.trim();
    if ref_str.is_empty() {
        return None;
    }

    let mut col_str = String::new();
    let mut row_str = String::new();

    for ch in ref_str.chars() {
        if ch.is_ascii_alphabetic() {
            col_str.push(ch.to_ascii_uppercase());
        } else if ch.is_ascii_digit() {
            row_str.push(ch);
        } else {
            // Invalid character
            return None;
        }
    }

    if col_str.is_empty() || row_str.is_empty() {
        return None;
    }

    // Convert column letters to 0-based index (A=0, B=1, ..., Z=25, AA=26, ...)
    let mut col: i32 = 0;
    for ch in col_str.chars() {
        col = col * 26 + (ch as i32 - 'A' as i32 + 1);
    }
    col -= 1; // Convert to 0-based

    // Parse row number and convert to 0-based
    let row: i32 = row_str.parse().ok()?;
    let row = row - 1; // Convert to 0-based

    Some((row, col))
}

/// Convert (row, col) back to cell reference string
fn to_cell_ref(row: i32, col: i32) -> String {
    let mut col_str = String::new();
    let mut c = col + 1; // Convert to 1-based for calculation

    while c > 0 {
        let remainder = ((c - 1) % 26) as u8;
        col_str.insert(0, (b'A' + remainder) as char);
        c = (c - 1) / 26;
    }

    format!("{}{}", col_str, row + 1) // Row is 1-based in reference
}

/// Shift a cell reference string
fn shift_cell_ref(ref_str: &mut String, op: &ShiftOperation) -> ShiftResult {
    let (mut row, mut col) = match parse_cell_ref(ref_str) {
        Some(pos) => pos,
        None => return ShiftResult::Unchanged,
    };

    let result = shift_row_col(&mut row, &mut col, op);

    if result == ShiftResult::Modified {
        *ref_str = to_cell_ref(row, col);
    }

    result
}

impl WorksheetParams for AddCommentMutationParams {
    fn unit_id(&self) -> &str { &self.unit_id }
    fn sub_unit_id(&self) -> &str { &self.sub_unit_id }
}

impl Shiftable for AddCommentMutationParams {
    fn apply_shift(&mut self, op: &ShiftOperation) -> ShiftResult {
        match op {
            ShiftOperation::RemoveSheet { sub_unit_id } => {
                if &self.sub_unit_id == sub_unit_id {
                    return ShiftResult::Removed;
                }
                ShiftResult::Unchanged
            }
            _ => shift_cell_ref(&mut self.comment.ref_field, op),
        }
    }
}

impl WorksheetParams for UpdateCommentRefMutationParams {
    fn unit_id(&self) -> &str { &self.unit_id }
    fn sub_unit_id(&self) -> &str { &self.sub_unit_id }
}

impl Shiftable for UpdateCommentRefMutationParams {
    fn apply_shift(&mut self, op: &ShiftOperation) -> ShiftResult {
        match op {
            ShiftOperation::RemoveSheet { sub_unit_id } => {
                if &self.sub_unit_id == sub_unit_id {
                    return ShiftResult::Removed;
                }
                ShiftResult::Unchanged
            }
            _ => shift_cell_ref(&mut self.payload.ref_field, op),
        }
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SubUnitParams;
    use std::collections::HashMap;

    fn make_location() -> SubUnitParams {
        SubUnitParams {
            unit_id: "unit1".to_string(),
            sub_unit_id: "sheet1".to_string(),
        }
    }

    #[test]
    fn test_insert_rows_shifts_ranges() {
        let mut params = GenericRangesParams {
            sub_unit_params: make_location(),
            ranges: vec![
                IRange { start_row: 5, end_row: 10, start_column: 0, end_column: 5, range_type: None },
            ],
            other: serde_json::Map::new(),
        };

        let result = params.apply_shift(&ShiftOperation::InsertRows { start: 3, count: 2 });

        assert_eq!(result, ShiftResult::Modified);
        assert_eq!(params.ranges[0].start_row, 7);
        assert_eq!(params.ranges[0].end_row, 12);
    }

    #[test]
    fn test_insert_cols_shifts_ranges() {
        let mut params = GenericRangesParams {
            sub_unit_params: make_location(),
            ranges: vec![
                IRange { start_row: 0, end_row: 5, start_column: 5, end_column: 10, range_type: None },
            ],
            other: serde_json::Map::new(),
        };

        let result = params.apply_shift(&ShiftOperation::InsertCols { start: 3, count: 2 });

        assert_eq!(result, ShiftResult::Modified);
        assert_eq!(params.ranges[0].start_column, 7);
        assert_eq!(params.ranges[0].end_column, 12);
    }

    #[test]
    fn test_insert_rows_shifts_row_data() {
        let mut params = GenericRowDataParams {
            sub_unit_params: make_location(),
            row_data: {
                let mut map = HashMap::new();
                map.insert("5".to_string(), serde_json::json!({"h": 25}));
                map.insert("10".to_string(), serde_json::json!({"h": 30}));
                map
            },
            other: serde_json::Map::new(),
        };

        let result = params.apply_shift(&ShiftOperation::InsertRows { start: 3, count: 2 });

        assert_eq!(result, ShiftResult::Modified);
        assert!(params.row_data.contains_key("7"));  // 5 + 2
        assert!(params.row_data.contains_key("12")); // 10 + 2
        assert!(!params.row_data.contains_key("5"));
        assert!(!params.row_data.contains_key("10"));
    }

    #[test]
    fn test_insert_cols_shifts_col_data() {
        let mut params = GenericColDataParams {
            sub_unit_params: make_location(),
            column_data: {
                let mut map = HashMap::new();
                map.insert("5".to_string(), serde_json::json!({"w": 100}));
                map.insert("10".to_string(), serde_json::json!({"w": 150}));
                map
            },
            other: serde_json::Map::new(),
        };

        let result = params.apply_shift(&ShiftOperation::InsertCols { start: 3, count: 2 });

        assert_eq!(result, ShiftResult::Modified);
        assert!(params.column_data.contains_key("7"));  // 5 + 2
        assert!(params.column_data.contains_key("12")); // 10 + 2
    }

    #[test]
    fn test_remove_sheet_removes_matching() {
        let mut params = GenericRangesParams {
            sub_unit_params: make_location(),
            ranges: vec![IRange { start_row: 0, end_row: 5, start_column: 0, end_column: 5, range_type: None }],
            other: serde_json::Map::new(),
        };

        let result = params.apply_shift(&ShiftOperation::RemoveSheet {
            sub_unit_id: "sheet1".to_string(),
        });

        assert_eq!(result, ShiftResult::Removed);
    }

    #[test]
    fn test_remove_sheet_unchanged_different_sheet() {
        let mut params = GenericRangesParams {
            sub_unit_params: make_location(),
            ranges: vec![IRange { start_row: 0, end_row: 5, start_column: 0, end_column: 5, range_type: None }],
            other: serde_json::Map::new(),
        };

        let result = params.apply_shift(&ShiftOperation::RemoveSheet {
            sub_unit_id: "sheet2".to_string(),
        });

        assert_eq!(result, ShiftResult::Unchanged);
    }

    // ========================================================================
    // Remove Row Tests
    // ========================================================================

    #[test]
    fn test_remove_rows_shifts_ranges_after() {
        // Range after removed area should shift up
        let mut params = GenericRangesParams {
            sub_unit_params: make_location(),
            ranges: vec![
                IRange { start_row: 10, end_row: 15, start_column: 0, end_column: 5, range_type: None },
            ],
            other: serde_json::Map::new(),
        };

        // Remove rows 3-5 (3 rows)
        let result = params.apply_shift(&ShiftOperation::RemoveRows { start: 3, end: 5 });

        assert_eq!(result, ShiftResult::Modified);
        assert_eq!(params.ranges[0].start_row, 7);  // 10 - 3 = 7
        assert_eq!(params.ranges[0].end_row, 12);   // 15 - 3 = 12
    }

    #[test]
    fn test_remove_rows_removes_range_completely_inside() {
        // Range completely inside removed area should be removed
        let mut params = GenericRangesParams {
            sub_unit_params: make_location(),
            ranges: vec![
                IRange { start_row: 5, end_row: 8, start_column: 0, end_column: 5, range_type: None },
            ],
            other: serde_json::Map::new(),
        };

        // Remove rows 3-10
        let result = params.apply_shift(&ShiftOperation::RemoveRows { start: 3, end: 10 });

        assert_eq!(result, ShiftResult::Removed);
        assert!(params.ranges.is_empty());
    }

    #[test]
    fn test_remove_rows_range_before_unchanged() {
        // Range before removed area should be unchanged
        let mut params = GenericRangesParams {
            sub_unit_params: make_location(),
            ranges: vec![
                IRange { start_row: 0, end_row: 2, start_column: 0, end_column: 5, range_type: None },
            ],
            other: serde_json::Map::new(),
        };

        // Remove rows 5-10
        let result = params.apply_shift(&ShiftOperation::RemoveRows { start: 5, end: 10 });

        assert_eq!(result, ShiftResult::Modified);
        assert_eq!(params.ranges[0].start_row, 0);
        assert_eq!(params.ranges[0].end_row, 2);
    }

    #[test]
    fn test_remove_rows_shifts_row_data() {
        let mut params = GenericRowDataParams {
            sub_unit_params: make_location(),
            row_data: {
                let mut map = HashMap::new();
                map.insert("2".to_string(), serde_json::json!({"h": 20}));
                map.insert("5".to_string(), serde_json::json!({"h": 25})); // In removed range
                map.insert("10".to_string(), serde_json::json!({"h": 30}));
                map
            },
            other: serde_json::Map::new(),
        };

        // Remove rows 4-6 (3 rows)
        let result = params.apply_shift(&ShiftOperation::RemoveRows { start: 4, end: 6 });

        assert_eq!(result, ShiftResult::Modified);
        assert!(params.row_data.contains_key("2"));  // Before, unchanged
        assert!(!params.row_data.contains_key("5")); // Removed
        assert!(params.row_data.contains_key("7"));  // 10 - 3 = 7
        assert!(!params.row_data.contains_key("10"));
    }

    #[test]
    fn test_remove_rows_all_row_data_removed() {
        let mut params = GenericRowDataParams {
            sub_unit_params: make_location(),
            row_data: {
                let mut map = HashMap::new();
                map.insert("5".to_string(), serde_json::json!({"h": 25}));
                map.insert("6".to_string(), serde_json::json!({"h": 26}));
                map
            },
            other: serde_json::Map::new(),
        };

        // Remove rows 4-8, covers all data
        let result = params.apply_shift(&ShiftOperation::RemoveRows { start: 4, end: 8 });

        assert_eq!(result, ShiftResult::Removed);
        assert!(params.row_data.is_empty());
    }

    // ========================================================================
    // Remove Col Tests
    // ========================================================================

    #[test]
    fn test_remove_cols_shifts_ranges_after() {
        let mut params = GenericRangesParams {
            sub_unit_params: make_location(),
            ranges: vec![
                IRange { start_row: 0, end_row: 5, start_column: 10, end_column: 15, range_type: None },
            ],
            other: serde_json::Map::new(),
        };

        // Remove cols 3-5 (3 cols)
        let result = params.apply_shift(&ShiftOperation::RemoveCols { start: 3, end: 5 });

        assert_eq!(result, ShiftResult::Modified);
        assert_eq!(params.ranges[0].start_column, 7);  // 10 - 3 = 7
        assert_eq!(params.ranges[0].end_column, 12);   // 15 - 3 = 12
    }

    #[test]
    fn test_remove_cols_removes_range_completely_inside() {
        let mut params = GenericRangesParams {
            sub_unit_params: make_location(),
            ranges: vec![
                IRange { start_row: 0, end_row: 5, start_column: 5, end_column: 8, range_type: None },
            ],
            other: serde_json::Map::new(),
        };

        // Remove cols 3-10
        let result = params.apply_shift(&ShiftOperation::RemoveCols { start: 3, end: 10 });

        assert_eq!(result, ShiftResult::Removed);
        assert!(params.ranges.is_empty());
    }

    #[test]
    fn test_remove_cols_shifts_col_data() {
        let mut params = GenericColDataParams {
            sub_unit_params: make_location(),
            column_data: {
                let mut map = HashMap::new();
                map.insert("2".to_string(), serde_json::json!({"w": 80}));
                map.insert("5".to_string(), serde_json::json!({"w": 100})); // In removed range
                map.insert("10".to_string(), serde_json::json!({"w": 120}));
                map
            },
            other: serde_json::Map::new(),
        };

        // Remove cols 4-6 (3 cols)
        let result = params.apply_shift(&ShiftOperation::RemoveCols { start: 4, end: 6 });

        assert_eq!(result, ShiftResult::Modified);
        assert!(params.column_data.contains_key("2"));  // Before, unchanged
        assert!(!params.column_data.contains_key("5")); // Removed
        assert!(params.column_data.contains_key("7"));  // 10 - 3 = 7
        assert!(!params.column_data.contains_key("10"));
    }

    #[test]
    fn test_remove_cols_all_col_data_removed() {
        let mut params = GenericColDataParams {
            sub_unit_params: make_location(),
            column_data: {
                let mut map = HashMap::new();
                map.insert("5".to_string(), serde_json::json!({"w": 100}));
                map.insert("6".to_string(), serde_json::json!({"w": 110}));
                map
            },
            other: serde_json::Map::new(),
        };

        // Remove cols 4-8
        let result = params.apply_shift(&ShiftOperation::RemoveCols { start: 4, end: 8 });

        assert_eq!(result, ShiftResult::Removed);
        assert!(params.column_data.is_empty());
    }

    // ========================================================================
    // CellValue Remove Tests
    // ========================================================================

    fn make_cell_data() -> crate::types::ICellData {
        crate::types::ICellData {
            v: None, t: None, f: None, s: None, p: None,
            si: None, ref_field: None, xf: None, custom: None,
        }
    }

    #[test]
    fn test_remove_rows_shifts_cell_value() {
        use crate::types::IObjectMatrixPrimitiveType;

        let mut cell_value: IObjectMatrixPrimitiveType = std::collections::HashMap::new();
        cell_value.insert("2".to_string(), {
            let mut row = std::collections::HashMap::new();
            row.insert("0".to_string(), make_cell_data());
            row
        });
        cell_value.insert("10".to_string(), {
            let mut row = std::collections::HashMap::new();
            row.insert("0".to_string(), make_cell_data());
            row
        });

        let mut params = GenericCellValueParams {
            sub_unit_params: make_location(),
            cell_value: Some(cell_value),
            other: serde_json::Map::new(),
        };

        // Remove rows 5-7 (3 rows)
        let result = params.apply_shift(&ShiftOperation::RemoveRows { start: 5, end: 7 });

        assert_eq!(result, ShiftResult::Modified);
        let cv = params.cell_value.as_ref().unwrap();
        assert!(cv.contains_key("2"));  // Before, unchanged
        assert!(cv.contains_key("7"));  // 10 - 3 = 7
        assert!(!cv.contains_key("10"));
    }

    #[test]
    fn test_remove_cols_shifts_cell_value() {
        use crate::types::IObjectMatrixPrimitiveType;

        let mut cell_value: IObjectMatrixPrimitiveType = std::collections::HashMap::new();
        cell_value.insert("0".to_string(), {
            let mut row = std::collections::HashMap::new();
            row.insert("2".to_string(), make_cell_data());
            row.insert("10".to_string(), make_cell_data());
            row
        });

        let mut params = GenericCellValueParams {
            sub_unit_params: make_location(),
            cell_value: Some(cell_value),
            other: serde_json::Map::new(),
        };

        // Remove cols 5-7 (3 cols)
        let result = params.apply_shift(&ShiftOperation::RemoveCols { start: 5, end: 7 });

        assert_eq!(result, ShiftResult::Modified);
        let cv = params.cell_value.as_ref().unwrap();
        let row_data = cv.get("0").unwrap();
        assert!(row_data.contains_key("2"));  // Before, unchanged
        assert!(row_data.contains_key("7"));  // 10 - 3 = 7
        assert!(!row_data.contains_key("10"));
    }

    // ========================================================================
    // GenericRangeParams Tests (single range)
    // ========================================================================

    #[test]
    fn test_insert_rows_shifts_single_range() {
        let mut params = GenericRangeParams {
            sub_unit_params: make_location(),
            range: IRange { start_row: 5, end_row: 10, start_column: 0, end_column: 5, range_type: None },
            other: serde_json::Map::new(),
        };

        let result = params.apply_shift(&ShiftOperation::InsertRows { start: 3, count: 2 });

        assert_eq!(result, ShiftResult::Modified);
        assert_eq!(params.range.start_row, 7);
        assert_eq!(params.range.end_row, 12);
    }

    #[test]
    fn test_insert_cols_shifts_single_range() {
        let mut params = GenericRangeParams {
            sub_unit_params: make_location(),
            range: IRange { start_row: 0, end_row: 5, start_column: 5, end_column: 10, range_type: None },
            other: serde_json::Map::new(),
        };

        let result = params.apply_shift(&ShiftOperation::InsertCols { start: 3, count: 2 });

        assert_eq!(result, ShiftResult::Modified);
        assert_eq!(params.range.start_column, 7);
        assert_eq!(params.range.end_column, 12);
    }

    #[test]
    fn test_remove_rows_shifts_single_range() {
        let mut params = GenericRangeParams {
            sub_unit_params: make_location(),
            range: IRange { start_row: 10, end_row: 15, start_column: 0, end_column: 5, range_type: None },
            other: serde_json::Map::new(),
        };

        // Remove rows 3-5 (3 rows)
        let result = params.apply_shift(&ShiftOperation::RemoveRows { start: 3, end: 5 });

        assert_eq!(result, ShiftResult::Modified);
        assert_eq!(params.range.start_row, 7);  // 10 - 3 = 7
        assert_eq!(params.range.end_row, 12);   // 15 - 3 = 12
    }

    #[test]
    fn test_remove_rows_removes_single_range_completely_inside() {
        let mut params = GenericRangeParams {
            sub_unit_params: make_location(),
            range: IRange { start_row: 5, end_row: 8, start_column: 0, end_column: 5, range_type: None },
            other: serde_json::Map::new(),
        };

        // Remove rows 3-10 (covers entire range)
        let result = params.apply_shift(&ShiftOperation::RemoveRows { start: 3, end: 10 });

        assert_eq!(result, ShiftResult::Removed);
    }

    #[test]
    fn test_remove_cols_shifts_single_range() {
        let mut params = GenericRangeParams {
            sub_unit_params: make_location(),
            range: IRange { start_row: 0, end_row: 5, start_column: 10, end_column: 15, range_type: None },
            other: serde_json::Map::new(),
        };

        // Remove cols 3-5 (3 cols)
        let result = params.apply_shift(&ShiftOperation::RemoveCols { start: 3, end: 5 });

        assert_eq!(result, ShiftResult::Modified);
        assert_eq!(params.range.start_column, 7);  // 10 - 3 = 7
        assert_eq!(params.range.end_column, 12);   // 15 - 3 = 12
    }

    #[test]
    fn test_remove_cols_removes_single_range_completely_inside() {
        let mut params = GenericRangeParams {
            sub_unit_params: make_location(),
            range: IRange { start_row: 0, end_row: 5, start_column: 5, end_column: 8, range_type: None },
            other: serde_json::Map::new(),
        };

        // Remove cols 3-10 (covers entire range)
        let result = params.apply_shift(&ShiftOperation::RemoveCols { start: 3, end: 10 });

        assert_eq!(result, ShiftResult::Removed);
    }

    #[test]
    fn test_remove_sheet_removes_single_range() {
        let mut params = GenericRangeParams {
            sub_unit_params: make_location(),
            range: IRange { start_row: 0, end_row: 5, start_column: 0, end_column: 5, range_type: None },
            other: serde_json::Map::new(),
        };

        let result = params.apply_shift(&ShiftOperation::RemoveSheet {
            sub_unit_id: "sheet1".to_string(),
        });

        assert_eq!(result, ShiftResult::Removed);
    }

    #[test]
    fn test_remove_sheet_unchanged_different_sheet_single_range() {
        let mut params = GenericRangeParams {
            sub_unit_params: make_location(),
            range: IRange { start_row: 0, end_row: 5, start_column: 0, end_column: 5, range_type: None },
            other: serde_json::Map::new(),
        };

        let result = params.apply_shift(&ShiftOperation::RemoveSheet {
            sub_unit_id: "sheet2".to_string(),
        });

        assert_eq!(result, ShiftResult::Unchanged);
    }
}
