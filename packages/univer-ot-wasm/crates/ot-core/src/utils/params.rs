//! Parameter utilities for zero-copy optimization
//!
//! This module provides utilities to avoid unnecessary clones when
//! checking mutation parameters. For large operations (e.g., batch cell updates),
//! avoiding clones can significantly reduce memory usage.

use serde_json::Value;

/// Quick check if two mutations target the same worksheet without full parsing
///
/// This is much faster than parsing the entire params struct when we just
/// need to check if the mutations are on different worksheets (identity transform).
///
/// # Returns
/// - `Some(true)` if same worksheet
/// - `Some(false)` if different worksheet
/// - `None` if cannot determine (missing fields)
#[inline]
pub fn same_worksheet(m1_params: &Value, m2_params: &Value) -> Option<bool> {
    let m1_unit = m1_params.get("unitId").and_then(|v| v.as_str())?;
    let m2_unit = m2_params.get("unitId").and_then(|v| v.as_str())?;

    if m1_unit != m2_unit {
        return Some(false);
    }

    let m1_sub = m1_params.get("subUnitId").and_then(|v| v.as_str())?;
    let m2_sub = m2_params.get("subUnitId").and_then(|v| v.as_str())?;

    Some(m1_sub == m2_sub)
}

/// Quick extraction of range start row without full parsing
#[inline]
pub fn get_range_start_row(params: &Value) -> Option<u32> {
    params.get("range")?
        .get("startRow")?
        .as_u64()
        .map(|v| v as u32)
}

/// Quick extraction of range end row without full parsing
#[inline]
pub fn get_range_end_row(params: &Value) -> Option<u32> {
    params.get("range")?
        .get("endRow")?
        .as_u64()
        .map(|v| v as u32)
}

/// Quick extraction of range start column without full parsing
#[inline]
pub fn get_range_start_col(params: &Value) -> Option<u32> {
    params.get("range")?
        .get("startColumn")?
        .as_u64()
        .map(|v| v as u32)
}

/// Quick extraction of range end column without full parsing
#[inline]
pub fn get_range_end_col(params: &Value) -> Option<u32> {
    params.get("range")?
        .get("endColumn")?
        .as_u64()
        .map(|v| v as u32)
}

/// Check if params contain valid range data
#[inline]
pub fn has_valid_range(params: &Value) -> bool {
    get_range_start_row(params).is_some() &&
    get_range_end_row(params).is_some() &&
    get_range_start_col(params).is_some() &&
    get_range_end_col(params).is_some()
}

/// Quick extraction of cellValue reference without cloning
#[inline]
pub fn get_cell_value_ref(params: &Value) -> Option<&Value> {
    params.get("cellValue")
}

/// Quick extraction of cellValue as mutable (requires mutable params)
#[inline]
pub fn get_cell_value_mut(params: &mut Value) -> Option<&mut Value> {
    params.get_mut("cellValue")
}

/// Get row range (start_row, end_row) without full parsing
#[inline]
pub fn get_row_range(params: &Value) -> Option<(u32, u32)> {
    Some((get_range_start_row(params)?, get_range_end_row(params)?))
}

/// Get column range (start_col, end_col) without full parsing
#[inline]
pub fn get_col_range(params: &Value) -> Option<(u32, u32)> {
    Some((get_range_start_col(params)?, get_range_end_col(params)?))
}

/// In-place modification of range row values
///
/// This modifies the JSON Value directly, avoiding serialization/deserialization.
/// delta can be positive (shift down) or negative (shift up).
#[inline]
pub fn modify_range_rows(params: &mut Value, delta: i32) {
    if let Some(range) = params.get_mut("range") {
        if let Some(start) = range.get_mut("startRow") {
            if let Some(n) = start.as_u64() {
                let new_val = (n as i64 + delta as i64).max(0) as u64;
                *start = Value::Number(new_val.into());
            }
        }
        if let Some(end) = range.get_mut("endRow") {
            if let Some(n) = end.as_u64() {
                let new_val = (n as i64 + delta as i64).max(0) as u64;
                *end = Value::Number(new_val.into());
            }
        }
    }
}

/// In-place modification of range column values
///
/// This modifies the JSON Value directly, avoiding serialization/deserialization.
/// delta can be positive (shift right) or negative (shift left).
#[inline]
pub fn modify_range_columns(params: &mut Value, delta: i32) {
    if let Some(range) = params.get_mut("range") {
        if let Some(start) = range.get_mut("startColumn") {
            if let Some(n) = start.as_u64() {
                let new_val = (n as i64 + delta as i64).max(0) as u64;
                *start = Value::Number(new_val.into());
            }
        }
        if let Some(end) = range.get_mut("endColumn") {
            if let Some(n) = end.as_u64() {
                let new_val = (n as i64 + delta as i64).max(0) as u64;
                *end = Value::Number(new_val.into());
            }
        }
    }
}

/// Set a specific range row value
#[inline]
pub fn set_range_start_row(params: &mut Value, value: u32) {
    if let Some(range) = params.get_mut("range") {
        if let Some(start) = range.get_mut("startRow") {
            *start = Value::Number((value as u64).into());
        }
    }
}

/// Set a specific range end row value
#[inline]
pub fn set_range_end_row(params: &mut Value, value: u32) {
    if let Some(range) = params.get_mut("range") {
        if let Some(end) = range.get_mut("endRow") {
            *end = Value::Number((value as u64).into());
        }
    }
}

/// Set a specific range start column value
#[inline]
pub fn set_range_start_col(params: &mut Value, value: u32) {
    if let Some(range) = params.get_mut("range") {
        if let Some(start) = range.get_mut("startColumn") {
            *start = Value::Number((value as u64).into());
        }
    }
}

/// Set a specific range end column value
#[inline]
pub fn set_range_end_col(params: &mut Value, value: u32) {
    if let Some(range) = params.get_mut("range") {
        if let Some(end) = range.get_mut("endColumn") {
            *end = Value::Number((value as u64).into());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_same_worksheet_same() {
        let m1 = json!({"unitId": "wb1", "subUnitId": "sheet1"});
        let m2 = json!({"unitId": "wb1", "subUnitId": "sheet1"});
        assert_eq!(same_worksheet(&m1, &m2), Some(true));
    }

    #[test]
    fn test_same_worksheet_different_workbook() {
        let m1 = json!({"unitId": "wb1", "subUnitId": "sheet1"});
        let m2 = json!({"unitId": "wb2", "subUnitId": "sheet1"});
        assert_eq!(same_worksheet(&m1, &m2), Some(false));
    }

    #[test]
    fn test_same_worksheet_different_sheet() {
        let m1 = json!({"unitId": "wb1", "subUnitId": "sheet1"});
        let m2 = json!({"unitId": "wb1", "subUnitId": "sheet2"});
        assert_eq!(same_worksheet(&m1, &m2), Some(false));
    }

    #[test]
    fn test_same_worksheet_missing_fields() {
        let m1 = json!({"unitId": "wb1"});
        let m2 = json!({"unitId": "wb1", "subUnitId": "sheet1"});
        assert_eq!(same_worksheet(&m1, &m2), None);
    }

    #[test]
    fn test_get_range_fields() {
        let params = json!({
            "range": {
                "startRow": 5,
                "startColumn": 3,
                "endRow": 10,
                "endColumn": 8
            }
        });
        assert_eq!(get_range_start_row(&params), Some(5));
        assert_eq!(get_range_end_row(&params), Some(10));
        assert_eq!(get_range_start_col(&params), Some(3));
        assert_eq!(get_range_end_col(&params), Some(8));
        assert!(has_valid_range(&params));
    }

    #[test]
    fn test_get_row_col_range() {
        let params = json!({
            "range": {
                "startRow": 5,
                "startColumn": 3,
                "endRow": 10,
                "endColumn": 8
            }
        });
        assert_eq!(get_row_range(&params), Some((5, 10)));
        assert_eq!(get_col_range(&params), Some((3, 8)));
    }

    #[test]
    fn test_modify_range_rows() {
        let mut params = json!({
            "range": {
                "startRow": 5,
                "startColumn": 3,
                "endRow": 10,
                "endColumn": 8
            }
        });
        modify_range_rows(&mut params, 3);
        assert_eq!(get_range_start_row(&params), Some(8));
        assert_eq!(get_range_end_row(&params), Some(13));
        // Columns should be unchanged
        assert_eq!(get_range_start_col(&params), Some(3));
        assert_eq!(get_range_end_col(&params), Some(8));
    }

    #[test]
    fn test_modify_range_rows_negative() {
        let mut params = json!({
            "range": {
                "startRow": 10,
                "startColumn": 3,
                "endRow": 15,
                "endColumn": 8
            }
        });
        modify_range_rows(&mut params, -3);
        assert_eq!(get_range_start_row(&params), Some(7));
        assert_eq!(get_range_end_row(&params), Some(12));
    }

    #[test]
    fn test_modify_range_columns() {
        let mut params = json!({
            "range": {
                "startRow": 5,
                "startColumn": 3,
                "endRow": 10,
                "endColumn": 8
            }
        });
        modify_range_columns(&mut params, 2);
        assert_eq!(get_range_start_col(&params), Some(5));
        assert_eq!(get_range_end_col(&params), Some(10));
        // Rows should be unchanged
        assert_eq!(get_range_start_row(&params), Some(5));
        assert_eq!(get_range_end_row(&params), Some(10));
    }

    #[test]
    fn test_set_range_values() {
        let mut params = json!({
            "range": {
                "startRow": 5,
                "startColumn": 3,
                "endRow": 10,
                "endColumn": 8
            }
        });
        set_range_start_row(&mut params, 0);
        set_range_end_row(&mut params, 100);
        set_range_start_col(&mut params, 1);
        set_range_end_col(&mut params, 50);

        assert_eq!(get_range_start_row(&params), Some(0));
        assert_eq!(get_range_end_row(&params), Some(100));
        assert_eq!(get_range_start_col(&params), Some(1));
        assert_eq!(get_range_end_col(&params), Some(50));
    }
}
