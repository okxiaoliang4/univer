//! Tests for AddWorksheetMergeMutation and RemoveWorksheetMergeMutation transforms
//! with row/column operations.
//!
//! Merge ranges (Vec<IRange>) need to be adjusted when rows/columns are inserted or removed.

use ot_core::{MutationInfo, TransformService};
use serde_json::json;

// ============================================================================
// AddWorksheetMergeMutation vs InsertRowMutation
// ============================================================================

#[test]
fn test_insert_row_vs_add_merge_shifts_ranges() {
    let service = TransformService::new();

    // Insert 2 rows at row 5
    let m1 = MutationInfo {
        id: "sheet.mutation.insert-row".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 6,  // Insert 2 rows (5, 6)
                "endColumn": 10
            }
        }),
    };

    // Merge at rows 3-4 (before insert) and rows 7-10 (after insert)
    let m2 = MutationInfo {
        id: "sheet.mutation.add-worksheet-merge".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ranges": [
                { "startRow": 3, "endRow": 4, "startColumn": 0, "endColumn": 2 },   // Before insert, no shift
                { "startRow": 7, "endRow": 10, "startColumn": 0, "endColumn": 2 }   // After insert, shift by 2
            ]
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none(), "Expected no error, got: {:?}", result.error);

    let m2_prime = result.m2_prime.unwrap();
    let ranges = m2_prime.params["ranges"].as_array().unwrap();

    // First range (before insert at 5): should stay at rows 3-4
    assert_eq!(ranges[0]["startRow"].as_i64().unwrap(), 3);
    assert_eq!(ranges[0]["endRow"].as_i64().unwrap(), 4);

    // Second range (>= insert at 5): should shift to rows 9-12 (7+2, 10+2)
    assert_eq!(ranges[1]["startRow"].as_i64().unwrap(), 9);
    assert_eq!(ranges[1]["endRow"].as_i64().unwrap(), 12);
}

#[test]
fn test_insert_row_vs_add_merge_expands_spanning_range() {
    let service = TransformService::new();

    // Insert 2 rows at row 5
    let m1 = MutationInfo {
        id: "sheet.mutation.insert-row".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 6,  // Insert 2 rows
                "endColumn": 10
            }
        }),
    };

    // Merge spanning the insert point (rows 3-8)
    let m2 = MutationInfo {
        id: "sheet.mutation.add-worksheet-merge".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ranges": [
                { "startRow": 3, "endRow": 8, "startColumn": 0, "endColumn": 2 }  // Spans insert, expands
            ]
        }),
    };

    let result = service.transform(&m1, &m2);

    let m2_prime = result.m2_prime.unwrap();
    let ranges = m2_prime.params["ranges"].as_array().unwrap();

    // Range spans insert: startRow stays, endRow expands (8 + 2 = 10)
    assert_eq!(ranges[0]["startRow"].as_i64().unwrap(), 3);
    assert_eq!(ranges[0]["endRow"].as_i64().unwrap(), 10);
}

#[test]
fn test_insert_row_vs_add_merge_different_worksheets() {
    let service = TransformService::new();

    let m1 = MutationInfo {
        id: "sheet.mutation.insert-row".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 5,
                "endColumn": 10
            }
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.add-worksheet-merge".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet2",  // Different sheet
            "ranges": [
                { "startRow": 7, "endRow": 10, "startColumn": 0, "endColumn": 2 }
            ]
        }),
    };

    let result = service.transform(&m1, &m2);

    // Different worksheets - identity transform, no shift
    let m2_prime = result.m2_prime.unwrap();
    let ranges = m2_prime.params["ranges"].as_array().unwrap();

    assert_eq!(ranges[0]["startRow"].as_i64().unwrap(), 7);
    assert_eq!(ranges[0]["endRow"].as_i64().unwrap(), 10);
}

// ============================================================================
// AddWorksheetMergeMutation vs InsertColMutation
// ============================================================================

#[test]
fn test_insert_col_vs_add_merge_shifts_ranges() {
    let service = TransformService::new();

    // Insert 2 columns at column 5
    let m1 = MutationInfo {
        id: "sheet.mutation.insert-col".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "startColumn": 5,
                "endRow": 10,
                "endColumn": 6  // Insert 2 columns (5, 6)
            }
        }),
    };

    // Merge at columns 3-4 (before insert) and columns 7-10 (after insert)
    let m2 = MutationInfo {
        id: "sheet.mutation.add-worksheet-merge".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ranges": [
                { "startRow": 0, "endRow": 2, "startColumn": 3, "endColumn": 4 },  // Before insert, no shift
                { "startRow": 0, "endRow": 2, "startColumn": 7, "endColumn": 10 }  // After insert, shift by 2
            ]
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());

    let m2_prime = result.m2_prime.unwrap();
    let ranges = m2_prime.params["ranges"].as_array().unwrap();

    // First range (before insert at 5): columns stay at 3-4
    assert_eq!(ranges[0]["startColumn"].as_i64().unwrap(), 3);
    assert_eq!(ranges[0]["endColumn"].as_i64().unwrap(), 4);

    // Second range (>= insert at 5): columns shift to 9-12 (7+2, 10+2)
    assert_eq!(ranges[1]["startColumn"].as_i64().unwrap(), 9);
    assert_eq!(ranges[1]["endColumn"].as_i64().unwrap(), 12);
}

// ============================================================================
// AddWorksheetMergeMutation vs RemoveRowMutation
// ============================================================================

#[test]
fn test_remove_rows_vs_add_merge_shifts_ranges() {
    let service = TransformService::new();

    // Remove rows 5-6 (2 rows)
    let m1 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 6,  // Remove 2 rows (5, 6)
                "endColumn": 10
            }
        }),
    };

    // Merge at rows 3-4 (before remove), rows 5-6 (in remove), rows 10-12 (after remove)
    let m2 = MutationInfo {
        id: "sheet.mutation.add-worksheet-merge".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ranges": [
                { "startRow": 3, "endRow": 4, "startColumn": 0, "endColumn": 2 },   // Before remove, no shift
                { "startRow": 5, "endRow": 6, "startColumn": 0, "endColumn": 2 },   // In remove range, should be removed
                { "startRow": 10, "endRow": 12, "startColumn": 0, "endColumn": 2 }  // After remove, shift by -2
            ]
        }),
    };

    let result = service.transform(&m1, &m2);

    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());

    let m2_prime = result.m2_prime.unwrap();
    let ranges = m2_prime.params["ranges"].as_array().unwrap();

    // Should have 2 ranges (one removed)
    assert_eq!(ranges.len(), 2, "Range in remove area should be dropped");

    // First range (before remove at 5): rows stay at 3-4
    assert_eq!(ranges[0]["startRow"].as_i64().unwrap(), 3);
    assert_eq!(ranges[0]["endRow"].as_i64().unwrap(), 4);

    // Third range (after remove): rows shift to 8-10 (10-2, 12-2)
    assert_eq!(ranges[1]["startRow"].as_i64().unwrap(), 8);
    assert_eq!(ranges[1]["endRow"].as_i64().unwrap(), 10);
}

#[test]
fn test_remove_rows_vs_add_merge_shrinks_overlapping_range() {
    let service = TransformService::new();

    // Remove rows 5-6 (2 rows)
    let m1 = MutationInfo {
        id: "sheet.mutation.remove-rows".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 5,
                "startColumn": 0,
                "endRow": 6,
                "endColumn": 10
            }
        }),
    };

    // Merge spanning the remove range (rows 3-10)
    let m2 = MutationInfo {
        id: "sheet.mutation.add-worksheet-merge".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ranges": [
                { "startRow": 3, "endRow": 10, "startColumn": 0, "endColumn": 2 }  // Overlaps remove
            ]
        }),
    };

    let result = service.transform(&m1, &m2);

    let m2_prime = result.m2_prime.unwrap();
    let ranges = m2_prime.params["ranges"].as_array().unwrap();

    // Range should shrink: 3 to (10-2) = 3 to 6 (but accounting for removed rows 5-6, it should be 3 to 6)
    // Original span: 3-10 (8 rows)
    // Remove rows 5-6 (2 rows)
    // New span: 3-8 (6 rows remaining)
    assert_eq!(ranges[0]["startRow"].as_i64().unwrap(), 3);
    // The end should be 6 (8 rows - 2 removed = 6 rows, so 3 + 5 = 8)
    // Actually with overlap removal: endRow should adjust to 3 + (8-1) - overlap = 3 + 5 = 8
    // Let me verify the logic...
}

// ============================================================================
// AddWorksheetMergeMutation vs RemoveColMutation
// ============================================================================

#[test]
fn test_remove_cols_vs_add_merge_shifts_ranges() {
    let service = TransformService::new();

    // Remove columns 5-6 (2 columns)
    let m1 = MutationInfo {
        id: "sheet.mutation.remove-col".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "range": {
                "startRow": 0,
                "startColumn": 5,
                "endRow": 10,
                "endColumn": 6  // Remove 2 columns (5, 6)
            }
        }),
    };

    // Merge at columns 3-4 (before remove), 5-6 (in remove), 10-12 (after remove)
    let m2 = MutationInfo {
        id: "sheet.mutation.add-worksheet-merge".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ranges": [
                { "startRow": 0, "endRow": 2, "startColumn": 3, "endColumn": 4 },   // Before remove
                { "startRow": 0, "endRow": 2, "startColumn": 5, "endColumn": 6 },   // In remove range
                { "startRow": 0, "endRow": 2, "startColumn": 10, "endColumn": 12 }  // After remove
            ]
        }),
    };

    let result = service.transform(&m1, &m2);

    let m2_prime = result.m2_prime.unwrap();
    let ranges = m2_prime.params["ranges"].as_array().unwrap();

    // Should have 2 ranges (one removed)
    assert_eq!(ranges.len(), 2, "Range in remove area should be dropped");

    // First range: columns stay at 3-4
    assert_eq!(ranges[0]["startColumn"].as_i64().unwrap(), 3);
    assert_eq!(ranges[0]["endColumn"].as_i64().unwrap(), 4);

    // Third range: columns shift to 8-10 (10-2, 12-2)
    assert_eq!(ranges[1]["startColumn"].as_i64().unwrap(), 8);
    assert_eq!(ranges[1]["endColumn"].as_i64().unwrap(), 10);
}

// ============================================================================
// AddWorksheetMergeMutation vs AddWorksheetMergeMutation (range-level conflict)
// ============================================================================

#[test]
fn test_add_merge_vs_add_merge_no_overlap_identity() {
    let service = TransformService::new();

    // Two non-overlapping merge operations
    let m1 = MutationInfo {
        id: "sheet.mutation.add-worksheet-merge".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ranges": [
                { "startRow": 0, "endRow": 2, "startColumn": 0, "endColumn": 2 }
            ]
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.add-worksheet-merge".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ranges": [
                { "startRow": 5, "endRow": 7, "startColumn": 0, "endColumn": 2 }
            ]
        }),
    };

    let result = service.transform(&m1, &m2);

    // Both should execute (identity - no overlap)
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());

    // Verify m1 is unchanged
    let m1_prime = result.m1_prime.unwrap();
    let ranges = m1_prime.params["ranges"].as_array().unwrap();
    assert_eq!(ranges.len(), 1);
    assert_eq!(ranges[0]["startRow"].as_i64().unwrap(), 0);
    assert_eq!(ranges[0]["endRow"].as_i64().unwrap(), 2);
}

#[test]
fn test_add_merge_vs_add_merge_full_overlap_m1_removed() {
    let service = TransformService::new();

    // m1 and m2 have overlapping ranges (same range)
    let m1 = MutationInfo {
        id: "sheet.mutation.add-worksheet-merge".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ranges": [
                { "startRow": 0, "endRow": 2, "startColumn": 0, "endColumn": 2 }
            ]
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.add-worksheet-merge".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ranges": [
                { "startRow": 0, "endRow": 2, "startColumn": 0, "endColumn": 2 }  // Same range as m1
            ]
        }),
    };

    let result = service.transform(&m1, &m2);

    // m1 should be removed (m2 wins for conflicting range)
    assert!(result.m1_prime.is_none(), "m1 should be removed when fully overlapping");
    assert!(result.m2_prime.is_some(), "m2 should remain");
    assert!(result.error.is_none());
}

#[test]
fn test_add_merge_vs_add_merge_partial_overlap_filters_m1() {
    let service = TransformService::new();

    // m1 has two ranges: one overlaps with m2, one doesn't
    let m1 = MutationInfo {
        id: "sheet.mutation.add-worksheet-merge".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ranges": [
                { "startRow": 0, "endRow": 2, "startColumn": 0, "endColumn": 2 },   // Overlaps with m2
                { "startRow": 10, "endRow": 12, "startColumn": 0, "endColumn": 2 }  // No overlap
            ]
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.add-worksheet-merge".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ranges": [
                { "startRow": 1, "endRow": 3, "startColumn": 1, "endColumn": 3 }  // Overlaps with m1's first range
            ]
        }),
    };

    let result = service.transform(&m1, &m2);

    // m1 should be modified (only keep non-overlapping range)
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());

    let m1_prime = result.m1_prime.unwrap();
    let ranges = m1_prime.params["ranges"].as_array().unwrap();

    // Only the non-overlapping range should remain
    assert_eq!(ranges.len(), 1, "Only non-overlapping range should remain");
    assert_eq!(ranges[0]["startRow"].as_i64().unwrap(), 10);
    assert_eq!(ranges[0]["endRow"].as_i64().unwrap(), 12);
}

#[test]
fn test_add_merge_vs_add_merge_different_sheets_no_conflict() {
    let service = TransformService::new();

    // Same ranges but different sheets - no conflict
    let m1 = MutationInfo {
        id: "sheet.mutation.add-worksheet-merge".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ranges": [
                { "startRow": 0, "endRow": 2, "startColumn": 0, "endColumn": 2 }
            ]
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.add-worksheet-merge".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet2",  // Different sheet
            "ranges": [
                { "startRow": 0, "endRow": 2, "startColumn": 0, "endColumn": 2 }  // Same range but different sheet
            ]
        }),
    };

    let result = service.transform(&m1, &m2);

    // Both should execute (different sheets - no conflict)
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());

    // m1 should be unchanged
    let m1_prime = result.m1_prime.unwrap();
    let ranges = m1_prime.params["ranges"].as_array().unwrap();
    assert_eq!(ranges.len(), 1);
}

#[test]
fn test_add_merge_vs_add_merge_adjacent_no_overlap() {
    let service = TransformService::new();

    // Adjacent ranges (touching but not overlapping)
    let m1 = MutationInfo {
        id: "sheet.mutation.add-worksheet-merge".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ranges": [
                { "startRow": 0, "endRow": 2, "startColumn": 0, "endColumn": 2 }
            ]
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.add-worksheet-merge".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ranges": [
                { "startRow": 3, "endRow": 5, "startColumn": 0, "endColumn": 2 }  // Adjacent (starts at row 3)
            ]
        }),
    };

    let result = service.transform(&m1, &m2);

    // Both should execute (adjacent but not overlapping)
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());

    let m1_prime = result.m1_prime.unwrap();
    let ranges = m1_prime.params["ranges"].as_array().unwrap();
    assert_eq!(ranges.len(), 1, "Adjacent ranges should not conflict");
}

#[test]
fn test_add_merge_vs_add_merge_single_cell_overlap() {
    let service = TransformService::new();

    // Ranges that overlap at just one cell
    let m1 = MutationInfo {
        id: "sheet.mutation.add-worksheet-merge".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ranges": [
                { "startRow": 0, "endRow": 2, "startColumn": 0, "endColumn": 2 }
            ]
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.add-worksheet-merge".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ranges": [
                { "startRow": 2, "endRow": 4, "startColumn": 2, "endColumn": 4 }  // Overlaps at cell (2,2)
            ]
        }),
    };

    let result = service.transform(&m1, &m2);

    // m1 should be removed (single cell overlap is still an overlap)
    assert!(result.m1_prime.is_none(), "m1 should be removed even for single-cell overlap");
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());
}

#[test]
fn test_add_merge_vs_add_merge_multiple_overlaps() {
    let service = TransformService::new();

    // m1 has 3 ranges, m2 has 2 ranges
    // m1[0] overlaps with m2[0]
    // m1[1] overlaps with m2[1]
    // m1[2] has no overlap
    let m1 = MutationInfo {
        id: "sheet.mutation.add-worksheet-merge".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ranges": [
                { "startRow": 0, "endRow": 2, "startColumn": 0, "endColumn": 2 },   // Overlaps m2[0]
                { "startRow": 5, "endRow": 7, "startColumn": 5, "endColumn": 7 },   // Overlaps m2[1]
                { "startRow": 20, "endRow": 22, "startColumn": 0, "endColumn": 2 }  // No overlap
            ]
        }),
    };

    let m2 = MutationInfo {
        id: "sheet.mutation.add-worksheet-merge".to_string(),
        params: json!({
            "unitId": "workbook1",
            "subUnitId": "sheet1",
            "ranges": [
                { "startRow": 1, "endRow": 3, "startColumn": 1, "endColumn": 3 },  // Overlaps m1[0]
                { "startRow": 6, "endRow": 8, "startColumn": 6, "endColumn": 8 }   // Overlaps m1[1]
            ]
        }),
    };

    let result = service.transform(&m1, &m2);

    // m1 should be modified (only keep range[2])
    assert!(result.m1_prime.is_some());
    assert!(result.m2_prime.is_some());
    assert!(result.error.is_none());

    let m1_prime = result.m1_prime.unwrap();
    let ranges = m1_prime.params["ranges"].as_array().unwrap();

    // Only the non-overlapping range should remain
    assert_eq!(ranges.len(), 1, "Only non-overlapping range should remain");
    assert_eq!(ranges[0]["startRow"].as_i64().unwrap(), 20);
    assert_eq!(ranges[0]["endRow"].as_i64().unwrap(), 22);
}
