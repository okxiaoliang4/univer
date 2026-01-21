use crate::mutations::sheets::{
    SheetMoveRangeParams, SheetMoveRangeSide, SheetMutationRangeParams, SheetSetFrozenParams,
};
use crate::types::{MutationInfoInternal, ObjectMatrixPrimitiveType, Range};
use crate::{wasm_log_debug, wasm_log_warn};
use serde_json;

pub fn same_sub_unit(a: &SheetMutationRangeParams, b: &SheetMutationRangeParams) -> bool {
    a.sub_unit_params.unit_id == b.sub_unit_params.unit_id
        && a.sub_unit_params.sub_unit_id == b.sub_unit_params.sub_unit_id
}

pub fn shift_row_keys_for_insert(
    cell_value: &mut ObjectMatrixPrimitiveType,
    insert_start: u32,
    insert_count: u32,
) {
    let mut new_data = serde_json::Map::new();
    for (row_key, row_value) in cell_value.data.iter() {
        if let Ok(row_num) = row_key.parse::<u32>() {
            if row_num >= insert_start {
                new_data.insert((row_num + insert_count).to_string(), row_value.clone());
            } else {
                new_data.insert(row_key.clone(), row_value.clone());
            }
        } else {
            new_data.insert(row_key.clone(), row_value.clone());
        }
    }
    cell_value.data = new_data;
}

pub fn shift_row_keys_for_remove(
    cell_value: &mut ObjectMatrixPrimitiveType,
    remove_start: u32,
    remove_end: u32,
) {
    let remove_count = remove_end - remove_start + 1;
    let mut new_data = serde_json::Map::new();
    for (row_key, row_value) in cell_value.data.iter() {
        if let Ok(row_num) = row_key.parse::<u32>() {
            if row_num > remove_end {
                new_data.insert((row_num - remove_count).to_string(), row_value.clone());
            } else if row_num < remove_start {
                new_data.insert(row_key.clone(), row_value.clone());
            }
        } else {
            new_data.insert(row_key.clone(), row_value.clone());
        }
    }
    cell_value.data = new_data;
}

pub fn shift_col_keys_for_insert(
    cell_value: &mut ObjectMatrixPrimitiveType,
    insert_start: u32,
    insert_count: u32,
) {
    let mut new_data = serde_json::Map::new();
    for (row_key, row_value) in cell_value.data.iter() {
        if let serde_json::Value::Object(cols) = row_value {
            let mut new_row = serde_json::Map::new();
            for (col_key, col_value) in cols.iter() {
                if let Ok(col_num) = col_key.parse::<u32>() {
                    if col_num >= insert_start {
                        new_row.insert((col_num + insert_count).to_string(), col_value.clone());
                    } else {
                        new_row.insert(col_key.clone(), col_value.clone());
                    }
                } else {
                    new_row.insert(col_key.clone(), col_value.clone());
                }
            }
            if !new_row.is_empty() {
                new_data.insert(row_key.clone(), serde_json::Value::Object(new_row));
            }
        } else {
            new_data.insert(row_key.clone(), row_value.clone());
        }
    }
    cell_value.data = new_data;
}

pub fn shift_col_keys_for_remove(
    cell_value: &mut ObjectMatrixPrimitiveType,
    remove_start: u32,
    remove_end: u32,
) {
    let remove_count = remove_end - remove_start + 1;
    let mut new_data = serde_json::Map::new();
    for (row_key, row_value) in cell_value.data.iter() {
        if let serde_json::Value::Object(cols) = row_value {
            let mut new_row = serde_json::Map::new();
            for (col_key, col_value) in cols.iter() {
                if let Ok(col_num) = col_key.parse::<u32>() {
                    if col_num > remove_end {
                        new_row.insert((col_num - remove_count).to_string(), col_value.clone());
                    } else if col_num < remove_start {
                        new_row.insert(col_key.clone(), col_value.clone());
                    }
                } else {
                    new_row.insert(col_key.clone(), col_value.clone());
                }
            }
            if !new_row.is_empty() {
                new_data.insert(row_key.clone(), serde_json::Value::Object(new_row));
            }
        } else {
            new_data.insert(row_key.clone(), row_value.clone());
        }
    }
    cell_value.data = new_data;
}

pub fn shift_range_rows_for_insert(range: &mut Range, insert_start: u32, insert_count: u32) {
    if range.start_row >= insert_start {
        range.start_row += insert_count;
        range.end_row += insert_count;
    } else if range.end_row >= insert_start {
        range.end_row += insert_count;
    }
}

pub fn shift_range_rows_for_remove(range: &mut Range, remove_start: u32, remove_end: u32) -> bool {
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

pub fn shift_range_cols_for_insert(range: &mut Range, insert_start: u32, insert_count: u32) {
    if range.start_column >= insert_start {
        range.start_column += insert_count;
        range.end_column += insert_count;
    } else if range.end_column >= insert_start {
        range.end_column += insert_count;
    }
}

pub fn shift_range_cols_for_remove(range: &mut Range, remove_start: u32, remove_end: u32) -> bool {
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

pub fn shift_ranges_for_insert(
    ranges: &mut Vec<Range>,
    insert_row: Option<(u32, u32)>,
    insert_col: Option<(u32, u32)>,
) {
    for range in ranges.iter_mut() {
        if let Some((row_start, row_count)) = insert_row {
            shift_range_rows_for_insert(range, row_start, row_count);
        }
        if let Some((col_start, col_count)) = insert_col {
            shift_range_cols_for_insert(range, col_start, col_count);
        }
    }
}

pub fn shift_ranges_for_remove(
    ranges: &mut Vec<Range>,
    remove_row: Option<(u32, u32)>,
    remove_col: Option<(u32, u32)>,
) -> bool {
    ranges.retain_mut(|range| {
        let mut keep = true;
        if let Some((row_start, row_end)) = remove_row {
            keep = shift_range_rows_for_remove(range, row_start, row_end);
        }
        if keep {
            if let Some((col_start, col_end)) = remove_col {
                keep = shift_range_cols_for_remove(range, col_start, col_end);
            }
        }
        keep
    });
    !ranges.is_empty()
}

pub fn shift_ranges_value_for_insert(
    ranges_value: &mut serde_json::Value,
    insert_row: Option<(u32, u32)>,
    insert_col: Option<(u32, u32)>,
) {
    if let serde_json::Value::Array(ranges) = ranges_value {
        for range_value in ranges.iter_mut() {
            if let Ok(mut range) = serde_json::from_value::<Range>(range_value.clone()) {
                if let Some((row_start, row_count)) = insert_row {
                    shift_range_rows_for_insert(&mut range, row_start, row_count);
                }
                if let Some((col_start, col_count)) = insert_col {
                    shift_range_cols_for_insert(&mut range, col_start, col_count);
                }
                *range_value = serde_json::to_value(range).unwrap_or(range_value.clone());
            }
        }
    }
}

pub fn shift_ranges_value_for_remove(
    ranges_value: &mut serde_json::Value,
    remove_row: Option<(u32, u32)>,
    remove_col: Option<(u32, u32)>,
) -> bool {
    if let serde_json::Value::Array(ranges) = ranges_value {
        ranges.retain_mut(|range_value| {
            if let Ok(mut range) = serde_json::from_value::<Range>(range_value.clone()) {
                if let Some((row_start, row_end)) = remove_row {
                    if !shift_range_rows_for_remove(&mut range, row_start, row_end) {
                        return false;
                    }
                }
                if let Some((col_start, col_end)) = remove_col {
                    if !shift_range_cols_for_remove(&mut range, col_start, col_end) {
                        return false;
                    }
                }
                *range_value = serde_json::to_value(range).unwrap_or(range_value.clone());
                true
            } else {
                true
            }
        });
        return !ranges.is_empty();
    }
    true
}

pub fn shift_move_range_value_for_insert(
    params: &mut SheetMoveRangeParams,
    insert_row: Option<(u32, u32)>,
    insert_col: Option<(u32, u32)>,
) {
    if let Some((insert_start, insert_count)) = insert_row {
        shift_range_rows_for_insert(&mut params.from_range, insert_start, insert_count);
        shift_range_rows_for_insert(&mut params.to_range, insert_start, insert_count);
        shift_row_keys_for_insert(&mut params.from.value, insert_start, insert_count);
        shift_row_keys_for_insert(&mut params.to.value, insert_start, insert_count);
    }
    if let Some((insert_start, insert_count)) = insert_col {
        shift_range_cols_for_insert(&mut params.from_range, insert_start, insert_count);
        shift_range_cols_for_insert(&mut params.to_range, insert_start, insert_count);
        shift_col_keys_for_insert(&mut params.from.value, insert_start, insert_count);
        shift_col_keys_for_insert(&mut params.to.value, insert_start, insert_count);
    }
}

pub fn shift_move_range_value_for_remove(
    params: &mut SheetMoveRangeParams,
    remove_row: Option<(u32, u32)>,
    remove_col: Option<(u32, u32)>,
) -> bool {
    let mut valid = true;
    if let Some((remove_start, remove_end)) = remove_row {
        if !shift_range_rows_for_remove(&mut params.from_range, remove_start, remove_end) {
            valid = false;
        }
        if !shift_range_rows_for_remove(&mut params.to_range, remove_start, remove_end) {
            valid = false;
        }
        shift_row_keys_for_remove(&mut params.from.value, remove_start, remove_end);
        shift_row_keys_for_remove(&mut params.to.value, remove_start, remove_end);
    }
    if let Some((remove_start, remove_end)) = remove_col {
        if !shift_range_cols_for_remove(&mut params.from_range, remove_start, remove_end) {
            valid = false;
        }
        if !shift_range_cols_for_remove(&mut params.to_range, remove_start, remove_end) {
            valid = false;
        }
        shift_col_keys_for_remove(&mut params.from.value, remove_start, remove_end);
        shift_col_keys_for_remove(&mut params.to.value, remove_start, remove_end);
    }
    valid
}

pub fn shift_set_frozen_for_insert(
    params: &mut SheetSetFrozenParams,
    insert_row: Option<(u32, u32)>,
    insert_col: Option<(u32, u32)>,
) {
    if let Some((insert_start, insert_count)) = insert_row {
        if params.start_row >= insert_start {
            params.start_row += insert_count;
        }
    }
    if let Some((insert_start, insert_count)) = insert_col {
        if params.start_column >= insert_start {
            params.start_column += insert_count;
        }
    }
}

pub fn shift_set_frozen_for_remove(
    params: &mut SheetSetFrozenParams,
    remove_row: Option<(u32, u32)>,
    remove_col: Option<(u32, u32)>,
) {
    if let Some((remove_start, remove_end)) = remove_row {
        let remove_count = remove_end - remove_start + 1;
        if params.start_row > remove_end {
            params.start_row = params.start_row.saturating_sub(remove_count);
        } else if params.start_row >= remove_start {
            params.start_row = remove_start;
            params.y_split = params.y_split.saturating_sub(remove_count);
        }
    }
    if let Some((remove_start, remove_end)) = remove_col {
        let remove_count = remove_end - remove_start + 1;
        if params.start_column > remove_end {
            params.start_column = params.start_column.saturating_sub(remove_count);
        } else if params.start_column >= remove_start {
            params.start_column = remove_start;
            params.x_split = params.x_split.saturating_sub(remove_count);
        }
    }
}

pub fn parse_range_params(_id: &str, params: &serde_json::Value) -> SheetMutationRangeParams {
    serde_json::from_value(params.clone()).unwrap_or_else(|_| {
        wasm_log_warn!("invalid params");
        SheetMutationRangeParams {
            sub_unit_params: crate::types::SubUnitParams {
                unit_id: "".to_string(),
                sub_unit_id: "".to_string(),
            },
            range: Range {
                start_row: 0,
                start_column: 0,
                end_row: 0,
                end_column: 0,
            },
        }
    })
}

pub fn parse_move_range_params(params: &MutationInfoInternal) -> Option<SheetMoveRangeParams> {
    let parsed: SheetMoveRangeParams = serde_json::from_value(params.params.clone())
        .unwrap_or_else(|_| {
            wasm_log_warn!("{} invalid move-range params", params.id);
            SheetMoveRangeParams {
                unit_id: "".to_string(),
                from_range: Range {
                    start_row: 0,
                    start_column: 0,
                    end_row: 0,
                    end_column: 0,
                },
                to_range: Range {
                    start_row: 0,
                    start_column: 0,
                    end_row: 0,
                    end_column: 0,
                },
                from: SheetMoveRangeSide {
                    sub_unit_id: "".to_string(),
                    value: ObjectMatrixPrimitiveType {
                        data: serde_json::Map::new(),
                    },
                },
                to: SheetMoveRangeSide {
                    sub_unit_id: "".to_string(),
                    value: ObjectMatrixPrimitiveType {
                        data: serde_json::Map::new(),
                    },
                },
            }
        });
    if parsed.unit_id.is_empty() {
        None
    } else {
        Some(parsed)
    }
}

pub fn log_conflict(id: &str, details: &str) -> Option<String> {
    wasm_log_debug!("{} conflict {}", id, details);
    Some(format!("{} conflict {}", id, details))
}
