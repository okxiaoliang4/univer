//! Shiftable traits for unified OT conflict resolution
//!
//! These traits allow mutations with position data (ranges, row/col indices, cell values)
//! to be shifted uniformly when insert/remove operations occur.
//!
//! # Categories of shiftable data:
//! - `IRange` - single range with start_row, end_row, start_column, end_column
//! - `Vec<IRange>` - multiple ranges
//! - `ObjectMatrixPrimitiveType` - cell value HashMap<row, HashMap<col, ICellData>>
//! - `IObjectArrayPrimitiveType<T>` - row/col data HashMap<index, T>

use crate::types::{IRange, IObjectMatrixPrimitiveType};
use crate::utils::shift::{
    shift_range_rows_for_insert, shift_range_rows_for_remove,
    shift_range_cols_for_insert, shift_range_cols_for_remove,
    shift_row_keys_for_insert, shift_row_keys_for_remove,
    shift_col_keys_for_insert, shift_col_keys_for_remove,
    shift_array_keys_for_insert, shift_array_keys_for_remove,
};

// ============================================================================
// Shiftable Traits
// ============================================================================

/// Trait for types that can be shifted by row insertion
pub trait ShiftableByRowInsert {
    /// Apply row insertion shift
    /// - insert_start: the row index where insertion starts
    /// - insert_count: number of rows inserted
    fn shift_for_row_insert(&mut self, insert_start: i32, insert_count: i32);
}

/// Trait for types that can be shifted by row removal
pub trait ShiftableByRowRemove {
    /// Apply row removal shift
    /// - remove_start: the first row to be removed
    /// - remove_end: the last row to be removed (inclusive)
    /// - Returns false if the data should be completely removed
    fn shift_for_row_remove(&mut self, remove_start: i32, remove_end: i32) -> bool;
}

/// Trait for types that can be shifted by column insertion
pub trait ShiftableByColInsert {
    /// Apply column insertion shift
    /// - insert_start: the column index where insertion starts
    /// - insert_count: number of columns inserted
    fn shift_for_col_insert(&mut self, insert_start: i32, insert_count: i32);
}

/// Trait for types that can be shifted by column removal
pub trait ShiftableByColRemove {
    /// Apply column removal shift
    /// - remove_start: the first column to be removed
    /// - remove_end: the last column to be removed (inclusive)
    /// - Returns false if the data should be completely removed
    fn shift_for_col_remove(&mut self, remove_start: i32, remove_end: i32) -> bool;
}

// ============================================================================
// IRange Implementations
// ============================================================================

impl ShiftableByRowInsert for IRange {
    fn shift_for_row_insert(&mut self, insert_start: i32, insert_count: i32) {
        shift_range_rows_for_insert(self, insert_start, insert_count);
    }
}

impl ShiftableByRowRemove for IRange {
    fn shift_for_row_remove(&mut self, remove_start: i32, remove_end: i32) -> bool {
        shift_range_rows_for_remove(self, remove_start, remove_end)
    }
}

impl ShiftableByColInsert for IRange {
    fn shift_for_col_insert(&mut self, insert_start: i32, insert_count: i32) {
        shift_range_cols_for_insert(self, insert_start, insert_count);
    }
}

impl ShiftableByColRemove for IRange {
    fn shift_for_col_remove(&mut self, remove_start: i32, remove_end: i32) -> bool {
        shift_range_cols_for_remove(self, remove_start, remove_end)
    }
}

// ============================================================================
// Vec<IRange> Implementations
// ============================================================================

impl ShiftableByRowInsert for Vec<IRange> {
    fn shift_for_row_insert(&mut self, insert_start: i32, insert_count: i32) {
        for range in self.iter_mut() {
            range.shift_for_row_insert(insert_start, insert_count);
        }
    }
}

impl ShiftableByRowRemove for Vec<IRange> {
    fn shift_for_row_remove(&mut self, remove_start: i32, remove_end: i32) -> bool {
        self.retain_mut(|range| range.shift_for_row_remove(remove_start, remove_end));
        !self.is_empty()
    }
}

impl ShiftableByColInsert for Vec<IRange> {
    fn shift_for_col_insert(&mut self, insert_start: i32, insert_count: i32) {
        for range in self.iter_mut() {
            range.shift_for_col_insert(insert_start, insert_count);
        }
    }
}

impl ShiftableByColRemove for Vec<IRange> {
    fn shift_for_col_remove(&mut self, remove_start: i32, remove_end: i32) -> bool {
        self.retain_mut(|range| range.shift_for_col_remove(remove_start, remove_end));
        !self.is_empty()
    }
}

// ============================================================================
// ObjectMatrixPrimitiveType (Cell Value HashMap) Implementations
// ============================================================================

impl ShiftableByRowInsert for IObjectMatrixPrimitiveType {
    fn shift_for_row_insert(&mut self, insert_start: i32, insert_count: i32) {
        shift_row_keys_for_insert(self, insert_start, insert_count);
    }
}

impl ShiftableByRowRemove for IObjectMatrixPrimitiveType {
    fn shift_for_row_remove(&mut self, remove_start: i32, remove_end: i32) -> bool {
        shift_row_keys_for_remove(self, remove_start, remove_end);
        !self.is_empty()
    }
}

impl ShiftableByColInsert for IObjectMatrixPrimitiveType {
    fn shift_for_col_insert(&mut self, insert_start: i32, insert_count: i32) {
        shift_col_keys_for_insert(self, insert_start, insert_count);
    }
}

impl ShiftableByColRemove for IObjectMatrixPrimitiveType {
    fn shift_for_col_remove(&mut self, remove_start: i32, remove_end: i32) -> bool {
        shift_col_keys_for_remove(self, remove_start, remove_end);
        !self.is_empty()
    }
}

// ============================================================================
// Helper functions for shifting HashMap<String, T> keys
// ============================================================================

/// Shift row keys in a HashMap<String, T> for row insert operation
/// Use this for row_data type HashMaps where keys are row indices
pub fn shift_hashmap_keys_for_row_insert<T>(
    data: &mut std::collections::HashMap<String, T>,
    insert_start: i32,
    insert_count: i32,
) {
    shift_array_keys_for_insert(data, insert_start, insert_count);
}

/// Shift row keys in a HashMap<String, T> for row remove operation
/// Use this for row_data type HashMaps where keys are row indices
/// Returns false if the HashMap becomes empty
pub fn shift_hashmap_keys_for_row_remove<T>(
    data: &mut std::collections::HashMap<String, T>,
    remove_start: i32,
    remove_end: i32,
) -> bool {
    shift_array_keys_for_remove(data, remove_start, remove_end);
    !data.is_empty()
}

/// Shift column keys in a HashMap<String, T> for column insert operation
/// Use this for col_data type HashMaps where keys are column indices
pub fn shift_hashmap_keys_for_col_insert<T>(
    data: &mut std::collections::HashMap<String, T>,
    insert_start: i32,
    insert_count: i32,
) {
    // Same logic as row shifting - keys are just indices
    shift_array_keys_for_insert(data, insert_start, insert_count);
}

/// Shift column keys in a HashMap<String, T> for column remove operation
/// Use this for col_data type HashMaps where keys are column indices
/// Returns false if the HashMap becomes empty
pub fn shift_hashmap_keys_for_col_remove<T>(
    data: &mut std::collections::HashMap<String, T>,
    remove_start: i32,
    remove_end: i32,
) -> bool {
    shift_array_keys_for_remove(data, remove_start, remove_end);
    !data.is_empty()
}

// NOTE: We don't implement Shiftable traits for IObjectArrayPrimitiveType<T>
// because it would conflict with IObjectMatrixPrimitiveType (which is
// HashMap<String, HashMap<String, ICellData>>). Instead, use the helper
// functions above directly when working with row_data/col_data HashMaps.

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ICellData;
    use std::collections::HashMap;

    #[test]
    fn test_range_shift_row_insert() {
        let mut range = IRange {
            start_row: 5,
            start_column: 0,
            end_row: 10,
            end_column: 5,
            range_type: None,
        };

        // Insert 3 rows at row 3 (before range)
        range.shift_for_row_insert(3, 3);
        assert_eq!(range.start_row, 8);
        assert_eq!(range.end_row, 13);
    }

    #[test]
    fn test_range_shift_row_remove() {
        let mut range = IRange {
            start_row: 5,
            start_column: 0,
            end_row: 10,
            end_column: 5,
            range_type: None,
        };

        // Remove rows 0-2 (before range)
        let retained = range.shift_for_row_remove(0, 2);
        assert!(retained);
        assert_eq!(range.start_row, 2);
        assert_eq!(range.end_row, 7);
    }

    #[test]
    fn test_vec_range_shift_row_insert() {
        let mut ranges = vec![
            IRange { start_row: 0, start_column: 0, end_row: 2, end_column: 5, range_type: None },
            IRange { start_row: 5, start_column: 0, end_row: 10, end_column: 5, range_type: None },
        ];

        // Insert 2 rows at row 3
        ranges.shift_for_row_insert(3, 2);

        assert_eq!(ranges[0].start_row, 0); // Before insert, unchanged
        assert_eq!(ranges[0].end_row, 2);
        assert_eq!(ranges[1].start_row, 7); // After insert, shifted by 2
        assert_eq!(ranges[1].end_row, 12);
    }

    #[test]
    fn test_vec_range_shift_row_remove_complete() {
        let mut ranges = vec![
            IRange { start_row: 0, start_column: 0, end_row: 2, end_column: 5, range_type: None },
            IRange { start_row: 5, start_column: 0, end_row: 10, end_column: 5, range_type: None },
        ];

        // Remove rows 0-2 (completely removes first range)
        let retained = ranges.shift_for_row_remove(0, 2);
        assert!(retained);
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start_row, 2); // Was 5, shifted down by 3
    }

    #[test]
    fn test_cell_value_shift_row_insert() {
        let mut cell_value: IObjectMatrixPrimitiveType = HashMap::new();
        let mut row_0 = HashMap::new();
        row_0.insert("0".to_string(), ICellData {
            v: None, t: None, f: None, s: None, p: None,
            si: None, ref_field: None, xf: None, custom: None,
        });
        cell_value.insert("5".to_string(), row_0);

        // Insert 3 rows at row 3
        cell_value.shift_for_row_insert(3, 3);

        assert!(!cell_value.contains_key("5"));
        assert!(cell_value.contains_key("8")); // 5 + 3 = 8
    }

    #[test]
    fn test_array_keys_shift_row_insert() {
        let mut row_data: HashMap<String, String> = HashMap::new();
        row_data.insert("5".to_string(), "data at row 5".to_string());
        row_data.insert("10".to_string(), "data at row 10".to_string());

        // Insert 2 rows at row 7
        shift_hashmap_keys_for_row_insert(&mut row_data, 7, 2);

        assert!(row_data.contains_key("5")); // Before insert, unchanged
        assert!(!row_data.contains_key("10")); // Was shifted
        assert!(row_data.contains_key("12")); // 10 + 2 = 12
    }
}
