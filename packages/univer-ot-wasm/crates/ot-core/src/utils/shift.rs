use crate::types::{ObjectMatrixPrimitiveType, Range};

/// Shift row keys in cell_value for insert operation
/// Uses std::mem::take to avoid cloning - O(0) clones instead of O(n)
pub fn shift_row_keys_for_insert(
    cell_value: &mut ObjectMatrixPrimitiveType,
    insert_start: i32,
    insert_count: i32,
) {
    let mut new_data = std::collections::HashMap::with_capacity(cell_value.len());
    // Take ownership of old data, avoiding clone
    for (row_key, row_value) in std::mem::take(cell_value) {
        if let Ok(row_num) = row_key.parse::<i32>() {
            if row_num >= insert_start {
                new_data.insert((row_num + insert_count).to_string(), row_value);
            } else {
                new_data.insert(row_key, row_value);
            }
        } else {
            new_data.insert(row_key, row_value);
        }
    }
    *cell_value = new_data;
}

/// Shift row keys in cell_value for remove operation
/// Uses std::mem::take to avoid cloning - O(0) clones instead of O(n)
pub fn shift_row_keys_for_remove(
    cell_value: &mut ObjectMatrixPrimitiveType,
    remove_start: i32,
    remove_end: i32,
) {
    let remove_count = remove_end - remove_start + 1;
    let mut new_data = std::collections::HashMap::with_capacity(cell_value.len());
    // Take ownership of old data, avoiding clone
    for (row_key, row_value) in std::mem::take(cell_value) {
        if let Ok(row_num) = row_key.parse::<i32>() {
            if row_num > remove_end {
                new_data.insert((row_num - remove_count).to_string(), row_value);
            } else if row_num < remove_start {
                new_data.insert(row_key, row_value);
            }
            // Note: rows in [remove_start, remove_end] are dropped (not inserted)
        } else {
            new_data.insert(row_key, row_value);
        }
    }
    *cell_value = new_data;
}

/// Shift column keys in cell_value for insert operation
/// Uses std::mem::take to avoid cloning - O(0) clones instead of O(n*m)
pub fn shift_col_keys_for_insert(
    cell_value: &mut ObjectMatrixPrimitiveType,
    insert_start: i32,
    insert_count: i32,
) {
    let mut new_data = std::collections::HashMap::with_capacity(cell_value.len());
    // Take ownership of old data, avoiding clone
    for (row_key, cols) in std::mem::take(cell_value) {
        let mut new_row = std::collections::HashMap::with_capacity(cols.len());
        // Take ownership of column data
        for (col_key, col_value) in cols {
            if let Ok(col_num) = col_key.parse::<i32>() {
                if col_num >= insert_start {
                    new_row.insert((col_num + insert_count).to_string(), col_value);
                } else {
                    new_row.insert(col_key, col_value);
                }
            } else {
                new_row.insert(col_key, col_value);
            }
        }
        if !new_row.is_empty() {
            new_data.insert(row_key, new_row);
        }
    }
    *cell_value = new_data;
}

/// Shift column keys in cell_value for remove operation
/// Uses std::mem::take to avoid cloning - O(0) clones instead of O(n*m)
pub fn shift_col_keys_for_remove(
    cell_value: &mut ObjectMatrixPrimitiveType,
    remove_start: i32,
    remove_end: i32,
) {
    let remove_count = remove_end - remove_start + 1;
    let mut new_data = std::collections::HashMap::with_capacity(cell_value.len());
    // Take ownership of old data, avoiding clone
    for (row_key, cols) in std::mem::take(cell_value) {
        let mut new_row = std::collections::HashMap::with_capacity(cols.len());
        // Take ownership of column data
        for (col_key, col_value) in cols {
            if let Ok(col_num) = col_key.parse::<i32>() {
                if col_num > remove_end {
                    new_row.insert((col_num - remove_count).to_string(), col_value);
                } else if col_num < remove_start {
                    new_row.insert(col_key, col_value);
                }
                // Note: cols in [remove_start, remove_end] are dropped (not inserted)
            } else {
                new_row.insert(col_key, col_value);
            }
        }
        if !new_row.is_empty() {
            new_data.insert(row_key, new_row);
        }
    }
    *cell_value = new_data;
}

/// Shift range rows for insert operation
pub fn shift_range_rows_for_insert(range: &mut Range, insert_start: i32, insert_count: i32) {
    if range.start_row >= insert_start {
        range.start_row += insert_count;
        range.end_row += insert_count;
    } else if range.end_row >= insert_start {
        range.end_row += insert_count;
    }
}

/// Shift range rows for remove operation
/// Returns false if the range is completely removed
pub fn shift_range_rows_for_remove(range: &mut Range, remove_start: i32, remove_end: i32) -> bool {
    let remove_count = remove_end - remove_start + 1;
    if range.end_row < remove_start {
        return true;
    }
    if range.start_row > remove_end {
        range.start_row -= remove_count;
        range.end_row -= remove_count;
        return true;
    }

    let overlap_start = std::cmp::max(range.start_row, remove_start);
    let overlap_end = std::cmp::min(range.end_row, remove_end);
    let overlap_count = overlap_end - overlap_start + 1;
    let remaining = (range.end_row - range.start_row + 1).saturating_sub(overlap_count);
    if remaining == 0 {
        return false;
    }
    if remove_start <= range.start_row {
        range.start_row = range.start_row.saturating_sub(remove_count);
    }
    range.end_row = range.start_row + remaining - 1;
    true
}

/// Shift range columns for insert operation
pub fn shift_range_cols_for_insert(range: &mut Range, insert_start: i32, insert_count: i32) {
    if range.start_column >= insert_start {
        range.start_column += insert_count;
        range.end_column += insert_count;
    } else if range.end_column >= insert_start {
        range.end_column += insert_count;
    }
}

/// Shift range columns for remove operation
/// Returns false if the range is completely removed
pub fn shift_range_cols_for_remove(range: &mut Range, remove_start: i32, remove_end: i32) -> bool {
    let remove_count = remove_end - remove_start + 1;
    if range.end_column < remove_start {
        return true;
    }
    if range.start_column > remove_end {
        range.start_column -= remove_count;
        range.end_column -= remove_count;
        return true;
    }

    let overlap_start = std::cmp::max(range.start_column, remove_start);
    let overlap_end = std::cmp::min(range.end_column, remove_end);
    let overlap_count = overlap_end - overlap_start + 1;
    let remaining = (range.end_column - range.start_column + 1).saturating_sub(overlap_count);
    if remaining == 0 {
        return false;
    }
    if remove_start <= range.start_column {
        range.start_column = range.start_column.saturating_sub(remove_count);
    }
    range.end_column = range.start_column + remaining - 1;
    true
}
