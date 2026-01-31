# AddWorksheetMergeMutation Conflict Analysis

**Mutation ID**: `sheet.mutation.add-worksheet-merge`
**File**: `mutations/sheets/add_worksheet_merge_mutation.rs`
**Transform**: `transforms/sheets/merge.rs`
**Tests**: `tests/merge_tests.rs`

## Mutation Overview

**Purpose**: Merges cells in specified ranges, combining multiple cells into single merged cells.

**Affects**:
- [ ] Rows
- [ ] Columns
- [x] Cells/Ranges (primary - merges cell ranges)
- [ ] Worksheet properties
- [ ] Workbook properties
- [ ] Feature-specific

**Parameters**:
```rust
pub struct AddWorksheetMergeMutationParams {
    pub unit_id: String,           // Workbook ID
    pub sub_unit_id: String,       // Worksheet ID
    pub ranges: Vec<IRange>,       // Ranges to merge
}
```

**Data Structure**:
```json
{
  "unitId": "workbook1",
  "subUnitId": "sheet1",
  "ranges": [
    { "startRow": 0, "startColumn": 0, "endRow": 2, "endColumn": 2 },
    { "startRow": 5, "startColumn": 5, "endRow": 5, "endColumn": 7 }
  ]
}
```

## Conflict Dimensions Analysis

### Dimension 1: Unit ID (Workbook)
- **Conflicts when**: Same `unit_id`
- **No conflict when**: Different `unit_id` → Identity transform

### Dimension 2: Sub-Unit ID (Worksheet)
- **Conflicts when**: Same `unit_id` AND same `sub_unit_id`
- **No conflict when**: Different worksheets → Identity transform

### Dimension 3: Position (Row/Col/Range)
**Positions affected by this mutation**:
- Rows: Range rows (start_row to end_row)
- Columns: Range columns (start_column to end_column)
- Cells: All cells within each range become merged

**Conflict scenarios**:
1. **Overlapping merges**: Two merges overlap → Typically handled at application level
2. **Adjacent merges**: No conflict
3. **Separated merges**: No conflict

### Dimension 4: Feature Plugin ID
**Feature-specific IDs**: N/A

## Transform Coverage Matrix

**Status Legend**:
- ✅ Implemented and tested
- 🚧 Implementation in progress
- ⏳ Planned
- ❌ Not needed (identity)

### Core Sheets Mutations (53 total)

#### Structural Mutations (Insert/Remove)

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertRowMutation | ✅ | Shifting | Merge ranges shift with inserted rows |
| InsertColMutation | ✅ | Shifting | Merge ranges shift with inserted columns |
| RemoveRowsMutation | ✅ | Shifting + Removal | Merge ranges shrink/removed with deleted rows |
| RemoveColMutation | ✅ | Shifting + Removal | Merge ranges shrink/removed with deleted columns |
| MoveRowsMutation | ⏳ | Complex | Merges in source/target affected |
| MoveColumnsMutation | ⏳ | Complex | Merges in source/target affected |
| MoveRangeMutation | ⏳ | Complex | Merges in source/target affected |

#### Data Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetRangeValuesMutation | ❌ | Identity | Cell values independent of merge state |
| SetRowDataMutation | ❌ | Identity | Row metadata independent of merge |
| SetColDataMutation | ❌ | Identity | Column metadata independent of merge |
| ReorderRangeMutation | ⏳ | Complex | May affect merged cells |

#### Merge Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddWorksheetMergeMutation (self) | ✅ | Identity | Both merges apply independently |
| RemoveWorksheetMergeMutation | ⏳ | Conflict | Overlapping merge/unmerge |

#### Dimension Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetWorksheetRowHeightMutation | ❌ | Identity | Dimensions independent of merge |
| SetWorksheetColWidthMutation | ❌ | Identity | Dimensions independent of merge |
| SetWorksheetRowIsAutoHeightMutation | ❌ | Identity | Dimensions independent of merge |
| SetWorksheetRowAutoHeightMutation | ❌ | Identity | Dimensions independent of merge |
| MarkDirtyRowAutoHeightMutation | ❌ | Identity | Dimensions independent of merge |
| CancelMarkDirtyRowAutoHeightMutation | ❌ | Identity | Dimensions independent of merge |

#### Visibility Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetRowVisibleMutation | ❌ | Identity | Visibility independent of merge |
| SetRowHiddenMutation | ❌ | Identity | Visibility independent of merge |
| SetColVisibleMutation | ❌ | Identity | Visibility independent of merge |
| SetColHiddenMutation | ❌ | Identity | Visibility independent of merge |

#### Protection Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddRangeProtectionMutation | ❌ | Identity | Protection independent of merge |
| DeleteRangeProtectionMutation | ❌ | Identity | Protection independent of merge |
| SetRangeProtectionMutation | ❌ | Identity | Protection independent of merge |
| AddWorksheetProtectionMutation | ❌ | Identity | Worksheet-level, no position |
| DeleteWorksheetProtectionMutation | ❌ | Identity | Worksheet-level, no position |
| SetWorksheetProtectionMutation | ❌ | Identity | Worksheet-level, no position |
| SetWorksheetPermissionPointsMutation | ❌ | Identity | Worksheet-level, no position |

#### Theme/Style Mutations

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| AddRangeThemeMutation | ❌ | Identity | Theme independent of merge |
| RemoveRangeThemeMutation | ❌ | Identity | Theme independent of merge |
| SetRangeThemeMutation | ❌ | Identity | Theme independent of merge |
| RegisterWorksheetRangeThemeStyleMutation | ❌ | Identity | Unit-level, no position |
| UnregisterWorksheetRangeThemeStyleMutation | ❌ | Identity | Unit-level, no position |
| SetWorksheetRangeThemeStyleMutation | ❌ | Identity | Theme independent of merge |
| DeleteWorksheetRangeThemeStyleMutation | ❌ | Identity | Theme independent of merge |

#### Worksheet Configuration

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetFrozenMutation | ❌ | Identity | Frozen state independent of merge |
| SetGridlinesColorMutation | ❌ | Identity | Gridlines independent of merge |
| ToggleGridlinesMutation | ❌ | Identity | Gridlines independent of merge |
| SetTabColorMutation | ❌ | Identity | Tab color independent of merge |
| SetWorksheetHideMutation | ❌ | Identity | Visibility independent of merge |
| SetWorksheetNameMutation | ❌ | Identity | Name independent of merge |
| SetWorksheetOrderMutation | ❌ | Identity | Order independent of merge |
| SetWorksheetRightToLeftMutation | ❌ | Identity | RTL independent of merge |
| SetWorksheetDefaultStyleMutation | ❌ | Identity | Default style independent of merge |

#### Sheet Management

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| InsertSheetMutation | ❌ | Identity | Different worksheet |
| RemoveSheetMutation | ❌ | Identity | Different worksheet |
| CopyWorksheetEndMutation | ❌ | Identity | Different worksheet |

#### Worksheet Size

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetWorksheetColumnCountMutation | ❌ | Identity | Count independent of merge |
| SetWorksheetRowCountMutation | ❌ | Identity | Count independent of merge |

#### Workbook

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetWorkbookNameMutation | ❌ | Identity | Workbook level, no position |

#### Number Format

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| SetNumfmtMutation | ❌ | Identity | Format independent of merge |
| RemoveNumfmtMutation | ❌ | Identity | Format independent of merge |

#### Other

| Mutation | Status | Strategy | Notes |
|----------|--------|----------|-------|
| EmptyMutation | ❌ | Identity | No-op mutation |

### Feature Plugin Mutations

All feature plugin mutations are typically identity transforms with AddWorksheetMergeMutation, as merge state is independent of:
- Conditional formatting rules
- Data validation rules
- Filter configurations
- Hyperlinks
- Notes
- Pivot tables
- Tables
- Comments

## Detailed Conflict Resolution

### InsertRowMutation vs AddWorksheetMergeMutation

**Conflict Analysis**:
- **Dimension 3 (Position)**: Merge ranges need row shifting

**Resolution Strategy**: Shifting (range rows)

**Implementation**:
```rust
fn create_insert_row_vs_add_merge() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: InsertRowMutationParams = parse(m1)?;
        let mut m2_params: AddWorksheetMergeMutationParams = parse(m2)?;

        let insert_start = m1_params.range.start_row;
        let insert_count = m1_params.range.end_row - insert_start + 1;

        // Shift all merge ranges
        for range in &mut m2_params.ranges {
            shift_range_rows_for_insert(range, insert_start, insert_count);
        }

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(serialize(m2_params)),
            error: None,
        }
    })
}
```

**Test Cases**:
- [x] Different worksheets (identity)
- [x] Merge range below insert - shifts down
- [x] Merge range spanning insert - expands
- [x] Merge range above insert - no change

**Status**: ✅ Implemented

---

### RemoveRowsMutation vs AddWorksheetMergeMutation

**Conflict Analysis**:
- **Dimension 3 (Position)**: Merge ranges may overlap with removed rows

**Resolution Strategy**: Shifting + Removal

**Implementation**:
```rust
fn create_remove_row_vs_add_merge() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        if let Some(false) = same_worksheet(&m1.params, &m2.params) {
            return identity(m1, m2);
        }

        let m1_params: RemoveRowsMutationParams = parse(m1)?;
        let mut m2_params: AddWorksheetMergeMutationParams = parse(m2)?;

        let remove_start = m1_params.range.start_row;
        let remove_end = m1_params.range.end_row;

        // Shift/remove merge ranges, keeping only valid ones
        m2_params.ranges.retain_mut(|range| {
            shift_range_rows_for_remove(range, remove_start, remove_end)
        });

        TransformResultRef {
            m1_prime: MutationOutcome::Unchanged(m1),
            m2_prime: MutationOutcome::Modified(serialize(m2_params)),
            error: None,
        }
    })
}
```

**Test Cases**:
- [x] Merge completely in removed range - removed
- [x] Merge partially overlaps - shrinks
- [x] Merge below removed range - shifts up
- [x] Merge above removed range - no change

**Status**: ✅ Implemented

---

### AddWorksheetMergeMutation vs AddWorksheetMergeMutation (Self-Transform)

**Conflict Analysis**:
- **Dimension 3 (Position)**: Overlapping merges could conflict

**Resolution Strategy**: Identity (both apply)

**Rationale**: At the OT level, we allow both merges to apply. The application layer handles any conflicts (e.g., warning the user or merging the merged regions).

**Implementation**:
```rust
fn create_identity() -> TransformFnRef {
    Arc::new(|m1: &MutationInfo, m2: &MutationInfo| {
        TransformResultRef::identity(m1, m2)
    })
}
```

**Status**: ✅ Implemented

## Implementation Checklist

- [x] Self-transform registered (identity)
- [x] Transform with InsertRowMutation implemented
- [x] Transform with InsertColMutation implemented
- [x] Transform with RemoveRowsMutation implemented
- [x] Transform with RemoveColMutation implemented
- [ ] Transform with MoveRowsMutation implemented
- [ ] Transform with MoveColsMutation implemented
- [ ] Transform with MoveRangeMutation implemented
- [x] Tests written for implemented scenarios
- [x] Zero-copy optimizations applied
- [x] This document updated with implementation status

## Notes

- Merge mutations use `Vec<IRange>` to support multiple merge operations
- Use `shift_range_rows_for_insert/remove()` and `shift_range_cols_for_insert/remove()` utilities
- The `retain_mut()` pattern efficiently removes invalid ranges
- Self-transform is identity because overlapping merges are handled at application level

## Last Updated

**Date**: 2026-01-31
**By**: Claude Code
**Changes**: Initial document creation with comprehensive coverage matrix
